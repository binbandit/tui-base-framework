use crate::component::{Component, Context};
use crate::event::{Event, EventResult};
use crossterm::event::KeyCode;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::{Frame, layout::Rect};

/// A list selector demonstrating navigation and selection
pub struct ListSelector {
    items: Vec<String>,
    selected: usize,
}

impl ListSelector {
    pub fn new() -> Self {
        Self::with_items([
            "Rust",
            "Python",
            "JavaScript",
            "Go",
            "TypeScript",
            "C++",
            "Java",
        ])
    }

    pub fn with_items<I, S>(items: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            items: items.into_iter().map(Into::into).collect(),
            selected: 0,
        }
    }

    fn select_previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    fn select_next(&mut self) {
        if let Some(last_index) = self.items.len().checked_sub(1) {
            self.selected = (self.selected + 1).min(last_index);
        }
    }
}

impl Component for ListSelector {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == self.selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let prefix = if i == self.selected { "► " } else { "  " };
                ListItem::new(format!("{}{}", prefix, item)).style(style)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title("List Selector (↑/↓ to navigate, q to quit)"),
        );

        frame.render_widget(list, area);
    }

    fn handle_event(&mut self, event: Event, context: &Context) -> EventResult {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Up => {
                    self.select_previous();
                    EventResult::Consumed
                }
                KeyCode::Down => {
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

impl Default for ListSelector {
    fn default() -> Self {
        Self::new()
    }
}
