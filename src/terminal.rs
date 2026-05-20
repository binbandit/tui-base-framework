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

pub type TerminalType = Terminal<CrosstermBackend<Stdout>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalConfig {
    pub mouse_capture: bool,
    pub bracketed_paste: bool,
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

pub struct TerminalGuard {
    terminal: TerminalType,
}

impl TerminalGuard {
    pub fn new() -> Result<Self> {
        Self::with_config(TerminalConfig::default())
    }

    pub fn with_config(config: TerminalConfig) -> Result<Self> {
        enable_raw_mode().context("enable terminal raw mode")?;

        let mut stdout = io::stdout();
        if let Err(error) = Self::enter_terminal(&mut stdout, config) {
            Self::restore_terminal();
            return Err(error).context("enter terminal alternate screen");
        }

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => {
                Self::restore_terminal();
                return Err(error).context("create ratatui terminal");
            }
        };

        if let Err(error) = terminal.clear() {
            Self::restore_terminal();
            return Err(error).context("clear terminal");
        }

        Ok(Self { terminal })
    }

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
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(
            self.terminal.backend_mut(),
            Show,
            DisableFocusChange,
            DisableBracketedPaste,
            DisableMouseCapture,
            LeaveAlternateScreen
        );
        let _ = disable_raw_mode();
    }
}
