use crate::component::Component;
use crate::event::{Event, EventResult};
use crate::message::Message;
use ratatui::{Frame, layout::Rect};
use ratatui::widgets::{Paragraph, Block, Borders};
use ratatui::style::{Style, Color, Modifier};
use crossterm::event::{KeyCode, KeyModifiers};
use tokio::sync::mpsc;

/// A simple text input component demonstrating character input handling
pub struct TextInput {
    input: String,
    message_sender: Option<mpsc::Sender<Message>>,
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            message_sender: None,
        }
    }
}

impl Component for TextInput {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let text = format!(
            "Type something: {}_\n\n\
            Backspace to delete\n\
            Enter to clear\n\
            Press 'q' to quit",
            self.input
        );
        
        let paragraph = Paragraph::new(text)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Text Input Example")
                .style(Style::default().fg(Color::Green)))
            .style(Style::default().add_modifier(Modifier::BOLD));
        
        frame.render_widget(paragraph, area);
    }
    
    fn handle_event(&mut self, event: Event) -> EventResult {
        if let Event::Key(key) = event {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                return EventResult::Propagate;
            }
            
            match key.code {
                KeyCode::Char(c) if c == 'q' || c == 'Q' => {
                    if let Some(sender) = &self.message_sender {
                        let _ = sender.try_send(Message::Quit);
                    }
                    EventResult::Consumed
                }
                KeyCode::Char(c) => {
                    self.input.push(c);
                    EventResult::Consumed
                }
                KeyCode::Backspace => {
                    self.input.pop();
                    EventResult::Consumed
                }
                KeyCode::Enter => {
                    self.input.clear();
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

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}
