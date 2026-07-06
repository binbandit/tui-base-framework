//! The TUI framework: app loop, component trait, events, and terminal
//! lifecycle. Everything an app needs is re-exported from this module.
//!
//! This folder is deliberately self-contained — your app code lives outside
//! it and only imports from here, so `./setup.sh --app-only` can fold it into
//! a binary-only project unchanged.

pub mod app;
pub mod component;
pub mod event;
pub mod terminal;

pub use app::{App, AppConfig, run, run_with_config};
pub use component::{Component, Context};
pub use event::{Event, EventResult};
pub use terminal::{TerminalConfig, TerminalGuard, TerminalType};

// Input types every component needs, so app code can import from one place.
pub use crossterm::event::{
    KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};

// Ratatui building blocks used by nearly every `render` implementation.
// `Terminal` and `backend` are re-exported so component tests can render
// into `Terminal<TestBackend>` without importing ratatui directly.
pub use ratatui::{
    Terminal, backend, layout,
    prelude::{Frame, Rect},
    style, text, widgets,
};
