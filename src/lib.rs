pub mod event;
pub mod message;
pub mod component;
pub mod terminal;
pub mod app;
pub mod examples;

pub use component::Component;
pub use event::{Event, EventResult};
pub use message::Message;
pub use terminal::TerminalGuard;
pub use app::App;

pub use ratatui::prelude::{Frame, Rect};
pub use ratatui::widgets;
pub use ratatui::layout;
pub use ratatui::style;
