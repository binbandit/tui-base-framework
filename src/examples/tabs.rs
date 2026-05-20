use crate::component::{Component, Context};
use crate::event::{Event, EventResult};
use crossterm::event::KeyCode;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Paragraph, Tabs};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

/// Demonstrates tab navigation between different views
pub struct TabsDemo {
    selected_tab: usize,
    tab_titles: Vec<String>,
}

impl TabsDemo {
    pub fn new() -> Self {
        Self::with_tabs(["Home", "Settings", "About"])
    }

    pub fn with_tabs<I, S>(tab_titles: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            selected_tab: 0,
            tab_titles: tab_titles.into_iter().map(Into::into).collect(),
        }
    }

    fn select_previous(&mut self) {
        self.selected_tab = self.selected_tab.saturating_sub(1);
    }

    fn select_next(&mut self) {
        if let Some(last_index) = self.tab_titles.len().checked_sub(1) {
            self.selected_tab = (self.selected_tab + 1).min(last_index);
        }
    }

    fn render_content(&self, frame: &mut Frame, area: Rect) {
        let content = match self.selected_tab {
            0 => {
                "Welcome to the Home tab!\n\nThis is where your main content would go.\n\nUse ← → or Tab to switch tabs."
            }
            1 => {
                "Settings Tab\n\nConfigure your application here.\n\n• Option 1: Enabled\n• Option 2: Disabled\n• Option 3: Auto"
            }
            2 => {
                "About Tab\n\nTUI Base Framework\nVersion 0.1.0\n\nA minimal framework for building\nterminal user interfaces."
            }
            _ => "Unknown tab",
        };

        let paragraph = Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::White));

        frame.render_widget(paragraph, area);
    }
}

impl Component for TabsDemo {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        // Render tabs
        let titles: Vec<Span> = self
            .tab_titles
            .iter()
            .map(|t| Span::raw(t.as_str()))
            .collect();

        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title("Tabs Demo"))
            .select(self.selected_tab)
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(tabs, chunks[0]);

        // Render content
        self.render_content(frame, chunks[1]);

        // Footer
        let footer = Paragraph::new("← → or Tab to switch | q to quit")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(footer, chunks[2]);
    }

    fn handle_event(&mut self, event: Event, context: &Context) -> EventResult {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Left => {
                    self.select_previous();
                    EventResult::Consumed
                }
                KeyCode::Right | KeyCode::Tab => {
                    self.select_next();
                    EventResult::Consumed
                }
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    context.quit();
                    EventResult::Consumed
                }
                _ => EventResult::Propagate,
            }
        } else {
            EventResult::Propagate
        }
    }
}

impl Default for TabsDemo {
    fn default() -> Self {
        Self::new()
    }
}
