//! The framework event type delivered to [`Component::handle_event`].
//!
//! [`Component::handle_event`]: crate::tui::Component::handle_event

use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers, MouseEvent};

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
    /// Fires every `AppConfig::tick_rate`; drive animations from this.
    Tick,
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
    use crossterm::event::{KeyCode, KeyModifiers};

    #[test]
    fn is_key_matches_code_regardless_of_state() {
        let event = Event::key_press(KeyCode::Char('q'));

        assert!(event.is_key(KeyCode::Char('q')));
        assert!(!event.is_key(KeyCode::Esc));
        assert!(!Event::Tick.is_key(KeyCode::Char('q')));
    }

    #[test]
    fn is_key_with_requires_exact_modifiers() {
        let plain = Event::key_press(KeyCode::Char('s'));

        assert!(plain.is_key_with(KeyCode::Char('s'), KeyModifiers::NONE));
        assert!(!plain.is_key_with(KeyCode::Char('s'), KeyModifiers::CONTROL));
    }
}
