use crossterm::event::{KeyEvent, MouseEvent};

#[derive(Debug, Clone)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Tick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventResult {
    Consumed,
    Propagate,
}
