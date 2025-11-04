use crate::component::Component;
use crate::event::{Event, EventResult};
use crate::message::Message;
use ratatui::{Frame, layout::Rect};
use ratatui::widgets::Paragraph;
use ratatui::layout::Alignment;
use crossterm::event::{KeyCode, KeyModifiers};
use tokio::sync::mpsc;

pub struct Counter {
    count: i32,
    message_sender: Option<mpsc::Sender<Message>>,
}

impl Counter {
    pub fn new() -> Self {
        Self {
            count: 0,
            message_sender: None,
        }
    }
}

impl Component for Counter {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let text = format!("Count: {} (Press ↑/↓, q to quit)", self.count);
        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
    }
    
    fn handle_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Key(key) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    return EventResult::Propagate;
                }
                
                match key.code {
                    KeyCode::Up => {
                        self.count += 1;
                        EventResult::Consumed
                    }
                    KeyCode::Down => {
                        self.count -= 1;
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
            }
            _ => EventResult::Propagate,
        }
    }
    
    fn update(&mut self, _message: Message) {}
    
    fn set_message_sender(&mut self, sender: mpsc::Sender<Message>) {
        self.message_sender = Some(sender);
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}
