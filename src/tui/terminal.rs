//! Terminal setup and RAII cleanup.

use anyhow::{Context, Result};
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
use std::sync::Once;

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
            return Err(error).context("enter terminal");
        }

        let terminal = match Self::build_terminal(config) {
            Ok(terminal) => terminal,
            Err(error) => {
                restore_terminal();
                return Err(error);
            }
        };

        Ok(Self { terminal, config })
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
    /// forces a full repaint on the next draw.
    pub fn resume(&mut self) -> Result<()> {
        enable_raw_mode().context("re-enable terminal raw mode")?;
        Self::enter_terminal(&mut io::stdout(), self.config).context("re-enter terminal")?;

        // Rebuild rather than reuse the ratatui terminal: this re-anchors an
        // inline viewport at the current cursor position (whatever ran while
        // suspended has scrolled the screen) and starts from empty buffers so
        // the next draw repaints everything.
        self.terminal = Self::build_terminal(self.config)?;
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
        let inline = matches!(self.config.viewport, Viewport::Inline(_));

        if inline {
            let area = self.terminal.get_frame().area();
            let _ = execute!(io::stdout(), MoveTo(0, area.bottom().saturating_sub(1)));
        }

        restore_terminal();

        if inline {
            println!();
        }
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        self.hand_back_terminal();
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
