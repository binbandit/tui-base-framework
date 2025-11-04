use crate::component::Component;
use crate::event::{Event, EventResult};
use crate::message::Message;
use ratatui::{Frame, layout::Rect};
use ratatui::widgets::{List, ListItem, Block, Borders};
use ratatui::style::{Style, Color, Modifier};
use crossterm::event::KeyCode;
use tokio::sync::mpsc;

/// A list selector demonstrating navigation and selection
pub struct ListSelector {
    items: Vec<String>,
    selected: usize,
    message_sender: Option<mpsc::Sender<Message>>,
}

impl ListSelector {
    pub fn new() -> Self {
        Self {
            items: vec![
                "Rust".to_string(),
                "Python".to_string(),
                "JavaScript".to_string(),
                "Go".to_string(),
                "TypeScript".to_string(),
                "C++".to_string(),
                "Java".to_string(),
            ],
            selected: 0,
            message_sender: None,
        }
    }
}

impl Component for ListSelector {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.items
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
        
        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("List Selector (↑/↓ to navigate, q to quit)"));
        
        frame.render_widget(list, area);
    }
    
    fn handle_event(&mut self, event: Event) -> EventResult {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Up => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    }
                    EventResult::Consumed
                }
                KeyCode::Down => {
                    if self.selected < self.items.len() - 1 {
                        self.selected += 1;
                    }
                    EventResult::Consumed
                }
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    if let Some(sender) = &self.message_sender {
                        let _ = sender.try_send(Message::Quit);
                    }
                    EventResult::Consumed
                }
                _ => EventResult::Propagate,
            }
        } else {
            EventResult::Propagate
        }
    }
    
    fn update(&mut self, _message: Message) {}
    
    fn set_message_sender(&mut self, sender: mpsc::Sender<Message>) {
        self.message_sender = Some(sender);
    }
}

impl Default for ListSelector {
    fn default() -> Self {
        Self::new()
    }
}
