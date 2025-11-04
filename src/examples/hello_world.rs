use crate::component::Component;
use crate::event::{Event, EventResult};
use crate::message::Message;
use ratatui::{Frame, layout::Rect};
use ratatui::widgets::{Paragraph, Block, Borders};
use ratatui::layout::Alignment;
use ratatui::style::{Style, Color};
use crossterm::event::KeyCode;
use tokio::sync::mpsc;

/// The simplest possible TUI app - just displays text and quits on 'q'
pub struct HelloWorld {
    message_sender: Option<mpsc::Sender<Message>>,
}

impl HelloWorld {
    pub fn new() -> Self {
        Self {
            message_sender: None,
        }
    }
}

impl Component for HelloWorld {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let text = "Hello, TUI World!\n\nPress 'q' to quit";
        let paragraph = Paragraph::new(text)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Hello World Example")
                .style(Style::default().fg(Color::Cyan)))
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
    }
    
    fn handle_event(&mut self, event: Event) -> EventResult {
        if let Event::Key(key) = event {
            if let KeyCode::Char('q') | KeyCode::Char('Q') = key.code {
                if let Some(sender) = &self.message_sender {
                    let _ = sender.try_send(Message::Quit);
                }
                return EventResult::Consumed;
            }
        }
        EventResult::Propagate
    }
    
    fn update(&mut self, _message: Message) {}
    
    fn set_message_sender(&mut self, sender: mpsc::Sender<Message>) {
        self.message_sender = Some(sender);
    }
}

impl Default for HelloWorld {
    fn default() -> Self {
        Self::new()
    }
}
