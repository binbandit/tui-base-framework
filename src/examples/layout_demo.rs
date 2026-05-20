use crate::component::{Component, Context};
use crate::event::{Event, EventResult};
use crossterm::event::KeyCode;
use ratatui::layout::Alignment;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

/// Demonstrates layout composition with multiple panels
pub struct LayoutDemo;

impl LayoutDemo {
    pub fn new() -> Self {
        Self
    }
}

impl Component for LayoutDemo {
    fn render(&self, frame: &mut Frame, area: Rect) {
        // Split into header, body, footer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        // Header
        let header = Paragraph::new("Layout Demo - Multiple Panels")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center);
        frame.render_widget(header, chunks[0]);

        // Split body into left and right
        let body_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        // Left panel
        let left = Paragraph::new("Left Panel\n\nThis demonstrates\nhorizontal layout\nsplitting")
            .block(Block::default().borders(Borders::ALL).title("Left"))
            .style(Style::default().fg(Color::Green));
        frame.render_widget(left, body_chunks[0]);

        // Right panel
        let right = Paragraph::new("Right Panel\n\nYou can nest\nlayouts to create\ncomplex UIs")
            .block(Block::default().borders(Borders::ALL).title("Right"))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(right, body_chunks[1]);

        // Footer
        let footer = Paragraph::new("Press 'q' to quit")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Magenta))
            .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[2]);
    }

    fn handle_event(&mut self, event: Event, context: &Context) -> EventResult {
        if let Event::Key(key) = event {
            if let KeyCode::Char('q') | KeyCode::Char('Q') = key.code {
                context.quit();
                return EventResult::Consumed;
            }
        }
        EventResult::Propagate
    }
}

impl Default for LayoutDemo {
    fn default() -> Self {
        Self::new()
    }
}
