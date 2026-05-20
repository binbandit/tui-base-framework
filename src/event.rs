use crossterm::event::{Event as CrosstermEvent, KeyEvent, MouseEvent};

#[derive(Debug, Clone)]
pub enum Event {
    FocusGained,
    FocusLost,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Paste(String),
    Resize(u16, u16),
    Tick,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventResult {
    Consumed,
    Propagate,
}

impl EventResult {
    pub const fn is_consumed(self) -> bool {
        matches!(self, Self::Consumed)
    }
}
