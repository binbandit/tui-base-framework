use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io::Stdout;
use anyhow::Result;

pub type TerminalType = Terminal<CrosstermBackend<Stdout>>;

pub struct TerminalGuard {
    terminal: TerminalType,
}

impl TerminalGuard {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        execute!(std::io::stdout(), EnterAlternateScreen)?;
        
        let backend = CrosstermBackend::new(std::io::stdout());
        let terminal = Terminal::new(backend)?;
        
        Ok(Self { terminal })
    }
    
    pub fn terminal(&mut self) -> &mut TerminalType {
        &mut self.terminal
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
    }
}
