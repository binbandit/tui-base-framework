use crate::event::{Event, EventResult};
use crate::message::Message;
use ratatui::{Frame, layout::Rect};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::mpsc;

pub type MessageSender = mpsc::Sender<Message>;

#[derive(Clone)]
pub struct Context {
    message_sender: MessageSender,
    quit_requested: Arc<AtomicBool>,
}

impl Context {
    pub(crate) fn new(message_sender: MessageSender, quit_requested: Arc<AtomicBool>) -> Self {
        Self {
            message_sender,
            quit_requested,
        }
    }

    pub fn message_sender(&self) -> MessageSender {
        self.message_sender.clone()
    }

    pub fn try_send(&self, message: Message) -> Result<(), mpsc::error::TrySendError<Message>> {
        self.message_sender.try_send(message)
    }

    pub fn quit(&self) {
        self.quit_requested.store(true, Ordering::Relaxed);
        let _ = self.try_send(Message::Quit);
    }

    pub(crate) fn quit_requested(&self) -> bool {
        self.quit_requested.load(Ordering::Relaxed)
    }
}

pub trait Component: Send {
    fn init(&mut self, _context: &Context) {}

    fn render(&self, frame: &mut Frame, area: Rect);

    fn handle_event(&mut self, _event: Event, _context: &Context) -> EventResult {
        EventResult::Propagate
    }

    fn update(&mut self, _message: Message, _context: &Context) {}
}

impl<T> Component for Box<T>
where
    T: Component + ?Sized,
{
    fn init(&mut self, context: &Context) {
        (**self).init(context);
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        (**self).render(frame, area);
    }

    fn handle_event(&mut self, event: Event, context: &Context) -> EventResult {
        (**self).handle_event(event, context)
    }

    fn update(&mut self, message: Message, context: &Context) {
        (**self).update(message, context);
    }
}
