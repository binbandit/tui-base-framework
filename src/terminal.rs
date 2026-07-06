//! Terminal setup and RAII cleanup.

use anyhow::{Context, Result};
use crossterm::{
    cursor::{Hide, Show},
    event::{
        DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
        EnableFocusChange, EnableMouseCapture,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, Stdout};
use std::sync::Once;

/// The concrete Ratatui terminal type used by this template.
pub type TerminalType = Terminal<CrosstermBackend<Stdout>>;

/// Optional terminal features. Mouse capture and focus change are off by
/// default because they alter normal terminal behavior (for example, mouse
/// capture breaks native text selection).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalConfig {
    /// Receive [`Event::Mouse`](crate::Event::Mouse) events.
    pub mouse_capture: bool,
    /// Receive pasted text as a single [`Event::Paste`](crate::Event::Paste)
    /// instead of a burst of key events.
    pub bracketed_paste: bool,
    /// Receive [`Event::FocusGained`](crate::Event::FocusGained) and
    /// [`Event::FocusLost`](crate::Event::FocusLost) events.
    pub focus_change: bool,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            mouse_capture: false,
            bracketed_paste: true,
            focus_change: false,
        }
    }
}

/// Puts the terminal into raw mode + alternate screen on construction and
/// restores it on drop — including during unwinding, and via a panic hook so
/// panic messages print to a sane terminal instead of the alternate screen.
pub struct TerminalGuard {
    terminal: TerminalType,
}

impl TerminalGuard {
    /// Takes over the terminal with [`TerminalConfig::default`].
    pub fn new() -> Result<Self> {
        Self::with_config(TerminalConfig::default())
    }

    /// Takes over the terminal with the given feature set.
    pub fn with_config(config: TerminalConfig) -> Result<Self> {
        install_panic_hook();

        enable_raw_mode().context("enable terminal raw mode")?;

        let mut stdout = io::stdout();
        if let Err(error) = Self::enter_terminal(&mut stdout, config) {
            restore_terminal();
            return Err(error).context("enter terminal alternate screen");
        }

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => {
                restore_terminal();
                return Err(error).context("create ratatui terminal");
            }
        };

        if let Err(error) = terminal.clear() {
            restore_terminal();
            return Err(error).context("clear terminal");
        }

        Ok(Self { terminal })
    }

    /// Access the underlying Ratatui terminal.
    pub fn terminal(&mut self) -> &mut TerminalType {
        &mut self.terminal
    }

    fn enter_terminal(mut stdout: impl io::Write, config: TerminalConfig) -> io::Result<()> {
        execute!(stdout, EnterAlternateScreen, Hide)?;

        if config.mouse_capture {
            execute!(stdout, EnableMouseCapture)?;
        }

        if config.bracketed_paste {
            execute!(stdout, EnableBracketedPaste)?;
        }

        if config.focus_change {
            execute!(stdout, EnableFocusChange)?;
        }

        Ok(())
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore_terminal();
    }
}

/// Undoes everything [`TerminalGuard`] set up. Safe to call more than once;
/// terminals ignore the disable sequences when the feature is not active.
fn restore_terminal() {
    let mut stdout = io::stdout();
    let _ = execute!(
        stdout,
        Show,
        DisableFocusChange,
        DisableBracketedPaste,
        DisableMouseCapture,
        LeaveAlternateScreen
    );
    let _ = disable_raw_mode();
}

/// Restores the terminal before the default panic handler prints, so the
/// message and backtrace are readable instead of being swallowed by the
/// alternate screen or mangled by raw mode.
fn install_panic_hook() {
    static HOOK: Once = Once::new();

    HOOK.call_once(|| {
        let original = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            restore_terminal();
            original(info);
        }));
    });
}
