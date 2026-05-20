pub mod app;
pub mod component;
pub mod event;
pub mod examples;
pub mod message;
pub mod terminal;

pub use app::{App, AppConfig};
pub use component::{Component, Context, MessageSender};
pub use event::{Event, EventResult};
pub use message::Message;
pub use terminal::{TerminalConfig, TerminalGuard, TerminalType};

pub use ratatui::style;
pub use ratatui::{
    layout,
    prelude::{Frame, Rect},
    widgets,
};
