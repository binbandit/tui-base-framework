use ratatui::{Frame, layout::Rect};
use crate::event::{Event, EventResult};
use crate::message::Message;
use tokio::sync::mpsc;

pub trait Component: Send {
    fn render(&self, frame: &mut Frame, area: Rect);
    
    fn handle_event(&mut self, _event: Event) -> EventResult {
        EventResult::Propagate
    }
    
    fn update(&mut self, _message: Message) {}
    
    fn set_message_sender(&mut self, _sender: mpsc::Sender<Message>) {}
}
