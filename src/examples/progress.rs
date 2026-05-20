use crate::component::{Component, Context};
use crate::event::{Event, EventResult};
use crossterm::event::KeyCode;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

/// Demonstrates progress bars and time-based updates
pub struct ProgressDemo {
    progress: u16,
    paused: bool,
}

impl ProgressDemo {
    pub fn new() -> Self {
        Self {
            progress: 0,
            paused: false,
        }
    }

    fn update_progress(&mut self) {
        if !self.paused {
            self.progress = (self.progress + 1) % 101;
        }
    }
}

impl Component for ProgressDemo {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        // Title
        let title = Paragraph::new("Progress Bar Demo")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(title, chunks[0]);

        // Progress bar
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Progress"))
            .gauge_style(Style::default().fg(Color::Green))
            .percent(self.progress);
        frame.render_widget(gauge, chunks[1]);

        // Info
        let status = if self.paused { "PAUSED" } else { "RUNNING" };
        let info = Paragraph::new(format!(
            "Status: {}\nProgress: {}%\n\nThe progress bar advances on each tick\n(simulating a task).",
            status, self.progress
        ))
        .block(Block::default().borders(Borders::ALL).title("Info"))
        .style(Style::default().fg(Color::White));
        frame.render_widget(info, chunks[2]);

        // Controls
        let controls = Paragraph::new("Space to pause/resume | r to reset | q to quit")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(controls, chunks[3]);
    }

    fn handle_event(&mut self, event: Event, context: &Context) -> EventResult {
        match event {
            Event::Tick => {
                self.update_progress();
                EventResult::Consumed
            }
            Event::Key(key) => match key.code {
                KeyCode::Char(' ') => {
                    self.paused = !self.paused;
                    EventResult::Consumed
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    self.progress = 0;
                    self.paused = false;
                    EventResult::Consumed
                }
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    context.quit();
                    EventResult::Consumed
                }
                _ => EventResult::Propagate,
            },
            _ => EventResult::Propagate,
        }
    }
}

impl Default for ProgressDemo {
    fn default() -> Self {
        Self::new()
    }
}
