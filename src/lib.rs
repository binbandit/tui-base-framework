//! A small, fast foundation for terminal UIs built on Ratatui, Crossterm, and
//! Tokio.
//!
//! Implement [`Component`] for your app state, hand it to [`App`], and the
//! runtime takes care of the terminal lifecycle, input, ticks, typed message
//! passing, and event-driven redraws.
//!
//! ```no_run
//! use anyhow::Result;
//! use tui_base_framework::{App, Component, Context, Event, EventResult, Frame, KeyCode, Rect};
//! use tui_base_framework::widgets::Paragraph;
//!
//! struct Hello;
//!
//! impl Component for Hello {
//!     type Message = ();
//!
//!     fn render(&mut self, frame: &mut Frame, area: Rect) {
//!         frame.render_widget(Paragraph::new("Hello! Press q to quit."), area);
//!     }
//!
//!     fn handle_event(&mut self, event: Event, context: &Context<()>) -> EventResult {
//!         if let Event::Key(key) = event
//!             && key.code == KeyCode::Char('q')
//!         {
//!             context.quit();
//!             return EventResult::Consumed;
//!         }
//!         EventResult::Propagate
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     App::new(Hello)?.run().await
//! }
//! ```

pub mod app;
pub mod component;
pub mod event;
pub mod terminal;

pub use app::{App, AppConfig};
pub use component::{Component, Context};
pub use event::{Event, EventResult};
pub use terminal::{TerminalConfig, TerminalGuard, TerminalType};

// Input types every component needs, so app code can import from one crate.
pub use crossterm::event::{
    KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};

// Ratatui building blocks used by nearly every `render` implementation.
pub use ratatui::{
    layout,
    prelude::{Frame, Rect},
    style, text, widgets,
};
