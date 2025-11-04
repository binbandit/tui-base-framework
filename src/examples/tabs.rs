use crate::component::Component;
use crate::event::{Event, EventResult};
use crate::message::Message;
use ratatui::{Frame, layout::{Rect, Layout, Direction, Constraint}};
use ratatui::widgets::{Paragraph, Block, Borders, Tabs};
use ratatui::style::{Style, Color, Modifier};
use ratatui::text::Span;
use crossterm::event::KeyCode;
use tokio::sync::mpsc;

/// Demonstrates tab navigation between different views
pub struct TabsDemo {
    selected_tab: usize,
    tab_titles: Vec<String>,
    message_sender: Option<mpsc::Sender<Message>>,
}

impl TabsDemo {
    pub fn new() -> Self {
        Self {
            selected_tab: 0,
            tab_titles: vec![
                "Home".to_string(),
                "Settings".to_string(),
                "About".to_string(),
            ],
            message_sender: None,
        }
    }
    
    fn render_content(&self, frame: &mut Frame, area: Rect) {
        let content = match self.selected_tab {
            0 => "Welcome to the Home tab!\n\nThis is where your main content would go.\n\nUse ← → or Tab to switch tabs.",
            1 => "Settings Tab\n\nConfigure your application here.\n\n• Option 1: Enabled\n• Option 2: Disabled\n• Option 3: Auto",
            2 => "About Tab\n\nTUI Base Framework\nVersion 0.1.0\n\nA minimal framework for building\nterminal user interfaces.",
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
        let titles: Vec<Span> = self.tab_titles
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
                    .add_modifier(Modifier::BOLD)
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
    
    fn handle_event(&mut self, event: Event) -> EventResult {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Left => {
                    if self.selected_tab > 0 {
                        self.selected_tab -= 1;
                    }
                    EventResult::Consumed
                }
                KeyCode::Right | KeyCode::Tab => {
                    if self.selected_tab < self.tab_titles.len() - 1 {
                        self.selected_tab += 1;
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

impl Default for TabsDemo {
    fn default() -> Self {
        Self::new()
    }
}
