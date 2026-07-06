//! A small, fast foundation for terminal UIs built on Ratatui, Crossterm, and
//! Tokio.
//!
//! Implement [`Component`] for your app state, hand it to [`run`], and the
//! framework takes care of the terminal lifecycle, input, ticks, typed
//! message passing, and event-driven redraws.
//!
//! ```no_run
//! use anyhow::Result;
//! use tui_base_framework::{Component, Context, Event, EventResult, Frame, KeyCode, Rect};
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
//!         if event.is_key(KeyCode::Char('q')) {
//!             context.quit();
//!             return EventResult::Consumed;
//!         }
//!         EventResult::Propagate
//!     }
//! }
//!
//! fn main() -> Result<()> {
//!     tui_base_framework::run(Hello)
//! }
//! ```
//!
//! The framework lives entirely in the [`tui`] module; this crate root just
//! re-exports it. Run `./setup.sh <name> --app-only` to fold it into a
//! binary-only project.

pub mod tui;

pub use tui::*;
