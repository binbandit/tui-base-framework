//! Terminal setup and RAII cleanup.

use anyhow::{Context, Result, ensure};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{
        DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
        EnableFocusChange, EnableMouseCapture,
    },
    execute,
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use ratatui::{Terminal, TerminalOptions, backend::CrosstermBackend};
use std::io::{self, Stdout};
use std::sync::{Mutex, MutexGuard, Once};

/// The concrete Ratatui terminal type used by this template.
pub type TerminalType = Terminal<CrosstermBackend<Stdout>>;

/// Where the UI draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Viewport {
    /// Take over the whole terminal on the alternate screen (the default).
    /// The previous terminal contents reappear when the app exits.
    #[default]
    Fullscreen,
    /// Draw in `height` rows of the normal scrollback at the cursor position,
    /// like a progress display. No alternate screen: output printed before
    /// the app ran stays visible, and the UI's final frame stays in the
    /// scrollback after exit.
    ///
    /// Setup locates the viewport by querying the cursor position through
    /// stdin, so inline apps need a real interactive terminal (not a pipe).
    Inline(u16),
}

/// Optional terminal features. Mouse capture and focus change are off by
/// default because they alter normal terminal behavior (for example, mouse
/// capture breaks native text selection).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalConfig {
    /// Receive [`Event::Mouse`](crate::tui::Event::Mouse) events.
    pub mouse_capture: bool,
    /// Receive pasted text as a single [`Event::Paste`](crate::tui::Event::Paste)
    /// instead of a burst of key events.
    pub bracketed_paste: bool,
    /// Receive [`Event::FocusGained`](crate::tui::Event::FocusGained) and
    /// [`Event::FocusLost`](crate::tui::Event::FocusLost) events.
    pub focus_change: bool,
    /// Draw fullscreen (default) or inline in the scrollback.
    pub viewport: Viewport,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            mouse_capture: false,
            bracketed_paste: true,
            focus_change: false,
            viewport: Viewport::Fullscreen,
        }
    }
}

/// Puts the terminal into raw mode (and, for fullscreen apps, the alternate
/// screen) on construction and restores it on drop — including during
/// unwinding, and via a panic hook so panic messages print to a sane terminal
/// instead of the alternate screen.
pub struct TerminalGuard {
    terminal: TerminalType,
    config: TerminalConfig,
    _ownership: TerminalOwnership,
}

impl TerminalGuard {
    /// Takes over the terminal with [`TerminalConfig::default`].
    pub fn new() -> Result<Self> {
        Self::with_config(TerminalConfig::default())
    }

    /// Takes over the terminal with the given feature set.
    ///
    /// Returns an error if another guard is alive or an inline height is zero.
    pub fn with_config(config: TerminalConfig) -> Result<Self> {
        ensure!(
            config.viewport != Viewport::Inline(0),
            "inline viewport height must be positive"
        );
        let ownership = TerminalOwnership::acquire()?;
        install_panic_hook();
        let terminal = Self::activate(config)?;

        Ok(Self {
            terminal,
            config,
            _ownership: ownership,
        })
    }

    fn activate(config: TerminalConfig) -> Result<TerminalType> {
        // Record the configuration before the first side effect so errors and
        // panics both roll back even a partially completed setup.
        terminal_state().active = Some(config);
        let result = (|| {
            enable_raw_mode().context("enable terminal raw mode")?;
            Self::enter_terminal(io::stdout(), config).context("enter terminal")?;
            Self::build_terminal(config)
        })();
        if result.is_err() {
            restore_terminal();
        }
        result
    }

    /// Access the underlying Ratatui terminal.
    pub fn terminal(&mut self) -> &mut TerminalType {
        &mut self.terminal
    }

    /// Temporarily hands the terminal back to the shell: raw mode off, main
    /// screen restored, cursor visible. The guard stays alive; call
    /// [`TerminalGuard::resume`] to take the terminal over again.
    ///
    /// This is the primitive behind Ctrl-Z suspend, and equally useful for
    /// running a subprocess that needs the terminal (`$EDITOR`, a pager, a
    /// shell) in the middle of a session.
    pub fn suspend(&mut self) {
        self.hand_back_terminal();
    }

    /// Takes the terminal over again after [`TerminalGuard::suspend`] and
    /// forces a full repaint on the next draw. Repeated calls while active do
    /// nothing. On failure the terminal remains handed back to the shell.
    ///
    /// Pause other terminal input readers first: inline setup queries stdin.
    pub fn resume(&mut self) -> Result<()> {
        if terminal_state().active.is_some() {
            return Ok(());
        }

        // Rebuild to re-anchor inline viewports after shell output and start
        // with empty buffers for a full repaint. Activation rolls back on error.
        self.terminal = Self::activate(self.config)?;
        Ok(())
    }

    fn build_terminal(config: TerminalConfig) -> Result<TerminalType> {
        let viewport = match config.viewport {
            Viewport::Fullscreen => ratatui::Viewport::Fullscreen,
            Viewport::Inline(height) => ratatui::Viewport::Inline(height),
        };

        Terminal::with_options(
            CrosstermBackend::new(io::stdout()),
            TerminalOptions { viewport },
        )
        .context("create ratatui terminal")
    }

    fn enter_terminal(mut stdout: impl io::Write, config: TerminalConfig) -> io::Result<()> {
        match config.viewport {
            // Clear with a plain escape code rather than `Terminal::clear`,
            // which round-trips a cursor-position query through stdin and
            // hangs when the app runs without a responding terminal (CI,
            // pipes, tests).
            Viewport::Fullscreen => {
                execute!(stdout, EnterAlternateScreen, Clear(ClearType::All), Hide)?;
            }
            // Inline draws into the normal scrollback: no alternate screen,
            // no whole-screen clear. The cursor stays hidden between draws.
            Viewport::Inline(_) => execute!(stdout, Hide)?,
        }

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

    /// Restores the terminal for the shell. In inline mode the UI stays in
    /// the scrollback, so first park the cursor on the viewport's last line
    /// and finish with a newline — the next prompt starts below the UI
    /// instead of overwriting it.
    fn hand_back_terminal(&mut self) {
        let Some(config) = terminal_state().active.take() else {
            return;
        };

        // Keep Ratatui's cursor tracking in sync so replacing the terminal on
        // resume does not show the cursor again when the old instance drops.
        let _ = self.terminal.show_cursor();
        let mut stdout = io::stdout();
        if matches!(config.viewport, Viewport::Inline(_)) {
            let area = self.terminal.get_frame().area();
            let _ = execute!(stdout, MoveTo(0, area.bottom().saturating_sub(1)));
            // Explicit CRLF works in raw mode and avoids println!'s panic on
            // broken output while this method is already unwinding.
            let _ = io::Write::write_all(&mut stdout, b"\r\n");
        }
        leave_terminal(stdout, config);
        let _ = disable_raw_mode();
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        self.hand_back_terminal();
    }
}

// The terminal and panic hook are process-wide. A second live guard would
// otherwise restore the first one's terminal when it fails or drops.
struct TerminalState {
    owned: bool,
    active: Option<TerminalConfig>,
}

fn terminal_state() -> MutexGuard<'static, TerminalState> {
    static STATE: Mutex<TerminalState> = Mutex::new(TerminalState {
        owned: false,
        active: None,
    });
    STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

struct TerminalOwnership;

impl TerminalOwnership {
    fn acquire() -> Result<Self> {
        let mut state = terminal_state();
        ensure!(
            !state.owned,
            "another TerminalGuard already owns the terminal"
        );
        state.owned = true;
        Ok(Self)
    }
}

impl Drop for TerminalOwnership {
    fn drop(&mut self) {
        restore_terminal();
        terminal_state().owned = false;
    }
}

/// Claims cleanup exactly once, including when the panic hook runs before Drop.
fn restore_terminal() {
    let config = terminal_state().active.take();
    if let Some(config) = config {
        leave_terminal(io::stdout(), config);
        let _ = disable_raw_mode();
    }
}

fn leave_terminal(mut stdout: impl io::Write, config: TerminalConfig) {
    // Try every restoration step even if an earlier write fails. Only undo
    // features this guard enabled; inline mode never entered an alternate screen.
    let _ = execute!(stdout, Show);
    if config.focus_change {
        let _ = execute!(stdout, DisableFocusChange);
    }
    if config.bracketed_paste {
        let _ = execute!(stdout, DisableBracketedPaste);
    }
    if config.mouse_capture {
        let _ = execute!(stdout, DisableMouseCapture);
    }
    if matches!(config.viewport, Viewport::Fullscreen) {
        let _ = execute!(stdout, LeaveAlternateScreen);
    }
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

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use super::leave_terminal;
    use super::{TerminalConfig, TerminalGuard, TerminalOwnership, Viewport};
    #[cfg(unix)]
    use std::io::{self, Write};

    #[test]
    fn ownership_is_exclusive_until_the_guard_is_dropped() {
        let ownership = TerminalOwnership::acquire().unwrap();
        assert!(TerminalOwnership::acquire().is_err());
        drop(ownership);
        assert!(TerminalOwnership::acquire().is_ok());
    }

    #[test]
    fn zero_height_is_rejected_before_touching_the_terminal() {
        let result = TerminalGuard::with_config(TerminalConfig {
            viewport: Viewport::Inline(0),
            ..TerminalConfig::default()
        });
        assert!(result.is_err_and(|error| error.to_string().contains("height must be positive")));
    }

    #[cfg(unix)]
    #[test]
    fn inline_cleanup_preserves_screen_and_disabled_features() {
        let mut output = Vec::new();
        leave_terminal(
            &mut output,
            TerminalConfig {
                viewport: Viewport::Inline(3),
                bracketed_paste: false,
                ..TerminalConfig::default()
            },
        );
        assert_eq!(output, b"\x1b[?25h");
    }

    #[cfg(unix)]
    #[test]
    fn fullscreen_cleanup_restores_enabled_features() {
        let mut output = Vec::new();
        leave_terminal(
            &mut output,
            TerminalConfig {
                focus_change: true,
                mouse_capture: true,
                ..TerminalConfig::default()
            },
        );
        let output = String::from_utf8(output).unwrap();
        for sequence in [
            "\x1b[?25h",
            "\x1b[?1004l",
            "\x1b[?2004l",
            "\x1b[?1000l",
            "\x1b[?1049l",
        ] {
            assert!(output.contains(sequence), "missing {sequence:?}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn cleanup_continues_after_a_write_failure() {
        #[derive(Default)]
        struct FailFirstWrite {
            failed: bool,
            output: Vec<u8>,
        }
        impl Write for FailFirstWrite {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                if !self.failed {
                    self.failed = true;
                    return Err(io::Error::other("first write failed"));
                }
                self.output.write(bytes)
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let mut output = FailFirstWrite::default();
        leave_terminal(&mut output, TerminalConfig::default());
        let output = String::from_utf8(output.output).unwrap();
        assert!(output.contains("\x1b[?2004l"));
        assert!(output.contains("\x1b[?1049l"));
    }
}
