//! The framework event type delivered to [`Component::handle_event`].
//!
//! [`Component::handle_event`]: crate::tui::Component::handle_event

use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use std::time::Duration;

/// A terminal event or timer tick.
#[derive(Debug, Clone)]
pub enum Event {
    /// The terminal window gained focus (requires `TerminalConfig::focus_change`).
    FocusGained,
    /// The terminal window lost focus (requires `TerminalConfig::focus_change`).
    FocusLost,
    /// A key press. Key releases are filtered out by the runtime.
    Key(KeyEvent),
    /// A mouse event (requires `TerminalConfig::mouse_capture`).
    Mouse(MouseEvent),
    /// Pasted text, delivered whole (requires `TerminalConfig::bracketed_paste`).
    Paste(String),
    /// The terminal was resized to (width, height).
    Resize(u16, u16),
    /// Fires every `AppConfig::tick_rate`, carrying the time actually elapsed
    /// since the previous tick. Scale animations by it — ticks are dropped
    /// (not queued) while the UI is busy, so the elapsed time can span more
    /// than one tick interval.
    Tick(Duration),
}

impl Event {
    /// Creates a key-press event with no modifiers. Mainly useful for
    /// feeding events to components in unit tests.
    pub fn key_press(code: KeyCode) -> Self {
        Self::Key(KeyEvent::from(code))
    }

    /// Returns the key event if this is a key press.
    pub const fn key(&self) -> Option<&KeyEvent> {
        match self {
            Self::Key(key) => Some(key),
            _ => None,
        }
    }

    /// Returns `true` if this event is a press of `code`, ignoring modifiers.
    ///
    /// Handy for one-key bindings:
    ///
    /// ```ignore
    /// if event.is_key(KeyCode::Char('q')) || event.is_key(KeyCode::Esc) {
    ///     context.quit();
    ///     return EventResult::Consumed;
    /// }
    /// ```
    pub fn is_key(&self, code: KeyCode) -> bool {
        matches!(self, Self::Key(key) if key.code == code)
    }

    /// Returns `true` if this event is a press of `code` with exactly the
    /// given modifiers, e.g. Ctrl+S:
    ///
    /// ```ignore
    /// if event.is_key_with(KeyCode::Char('s'), KeyModifiers::CONTROL) {
    ///     self.save();
    ///     return EventResult::Consumed;
    /// }
    /// ```
    pub fn is_key_with(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        matches!(self, Self::Key(key) if key.code == code && key.modifiers == modifiers)
    }

    /// Returns `true` if this event is a press of Ctrl+`c` — shorthand for
    /// the most common chord:
    ///
    /// ```ignore
    /// if event.is_ctrl('s') {
    ///     self.save();
    ///     return EventResult::Consumed;
    /// }
    /// ```
    pub fn is_ctrl(&self, c: char) -> bool {
        self.is_key_with(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    /// Returns the character this key press would type, if any.
    ///
    /// `None` for non-key events and for chords (Ctrl/Alt/Super/Meta), so
    /// text input built on this never swallows keyboard shortcuts. Shifted
    /// characters are included — they arrive already uppercased/symbolized.
    ///
    /// ```ignore
    /// if let Some(c) = event.char() {
    ///     self.input.push(c);
    ///     return EventResult::Consumed;
    /// }
    /// ```
    pub fn char(&self) -> Option<char> {
        let chord = KeyModifiers::CONTROL
            | KeyModifiers::ALT
            | KeyModifiers::SUPER
            | KeyModifiers::META
            | KeyModifiers::HYPER;

        match self {
            Self::Key(key) if !key.modifiers.intersects(chord) => match key.code {
                KeyCode::Char(c) => Some(c),
                _ => None,
            },
            _ => None,
        }
    }
}

impl From<CrosstermEvent> for Event {
    fn from(event: CrosstermEvent) -> Self {
        match event {
            CrosstermEvent::FocusGained => Self::FocusGained,
            CrosstermEvent::FocusLost => Self::FocusLost,
            CrosstermEvent::Key(key) => Self::Key(key),
            CrosstermEvent::Mouse(mouse) => Self::Mouse(mouse),
            CrosstermEvent::Paste(text) => Self::Paste(text),
            CrosstermEvent::Resize(width, height) => Self::Resize(width, height),
        }
    }
}

/// Tells the runtime whether an event changed state (and the UI should
/// redraw) or was ignored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventResult {
    /// The event changed state; schedule a redraw.
    Consumed,
    /// The event was ignored; no redraw needed.
    Propagate,
}

impl EventResult {
    /// Returns `true` for [`EventResult::Consumed`].
    pub const fn is_consumed(self) -> bool {
        matches!(self, Self::Consumed)
    }
}

#[cfg(test)]
mod tests {
    use super::Event;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::time::Duration;

    fn chord(c: char, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent::new(KeyCode::Char(c), modifiers))
    }

    #[test]
    fn is_key_matches_code_regardless_of_state() {
        let event = Event::key_press(KeyCode::Char('q'));

        assert!(event.is_key(KeyCode::Char('q')));
        assert!(!event.is_key(KeyCode::Esc));
        assert!(!Event::Tick(Duration::ZERO).is_key(KeyCode::Char('q')));
    }

    #[test]
    fn is_key_with_requires_exact_modifiers() {
        let plain = Event::key_press(KeyCode::Char('s'));

        assert!(plain.is_key_with(KeyCode::Char('s'), KeyModifiers::NONE));
        assert!(!plain.is_key_with(KeyCode::Char('s'), KeyModifiers::CONTROL));
    }

    #[test]
    fn is_ctrl_matches_only_the_control_chord() {
        assert!(chord('c', KeyModifiers::CONTROL).is_ctrl('c'));
        assert!(!Event::key_press(KeyCode::Char('c')).is_ctrl('c'));
        assert!(!chord('c', KeyModifiers::CONTROL | KeyModifiers::ALT).is_ctrl('c'));
    }

    #[test]
    fn char_returns_typed_characters_but_not_chords() {
        assert_eq!(Event::key_press(KeyCode::Char('a')).char(), Some('a'));
        assert_eq!(chord('A', KeyModifiers::SHIFT).char(), Some('A'));
        assert_eq!(chord('c', KeyModifiers::CONTROL).char(), None);
        assert_eq!(chord('x', KeyModifiers::ALT).char(), None);
        assert_eq!(Event::key_press(KeyCode::Backspace).char(), None);
        assert_eq!(Event::Paste("hi".into()).char(), None);
    }
}
