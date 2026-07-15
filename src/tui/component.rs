//! The [`Component`] trait and the [`Context`] handle components use to talk
//! back to the app loop.

use crate::tui::event::{Event, EventResult};
use ratatui::{Frame, layout::Rect};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::{Notify, mpsc};

/// Handle a component uses to talk back to the app loop.
///
/// A `Context` is cheap to clone and safe to move into background tasks. Use
/// [`Context::sender`] to report results back to the UI from async work,
/// [`Context::quit`] to stop the app, and [`Context::fail`] to stop it with
/// an error.
///
/// `M` is the component's [`Component::Message`] type.
pub struct Context<M> {
    sender: mpsc::Sender<M>,
    quit_requested: Arc<AtomicBool>,
    quit_notify: Arc<Notify>,
    error: Arc<Mutex<Option<anyhow::Error>>>,
}

// Manual impl: `Context<M>` is clonable regardless of whether `M` is.
impl<M> Clone for Context<M> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            quit_requested: Arc::clone(&self.quit_requested),
            quit_notify: Arc::clone(&self.quit_notify),
            error: Arc::clone(&self.error),
        }
    }
}

impl<M> Context<M> {
    pub(crate) fn new(sender: mpsc::Sender<M>) -> Self {
        Self {
            sender,
            quit_requested: Arc::new(AtomicBool::new(false)),
            quit_notify: Arc::new(Notify::new()),
            error: Arc::new(Mutex::new(None)),
        }
    }

    /// Returns a clone of the message sender.
    ///
    /// Move it into a background task and `send(message).await` to deliver
    /// results to [`Component::update`].
    pub fn sender(&self) -> mpsc::Sender<M> {
        self.sender.clone()
    }

    /// Sends a message to [`Component::update`] without waiting.
    ///
    /// Fails if the message channel is full. From async code prefer
    /// `context.sender()` and `send(message).await`, which waits for capacity
    /// instead of dropping the message.
    pub fn try_send(&self, message: M) -> Result<(), mpsc::error::TrySendError<M>> {
        self.sender.try_send(message)
    }

    /// Asks the app loop to exit.
    ///
    /// Safe to call from event handlers, `update`, or background tasks. The
    /// request latches: it cannot be lost, even under load.
    pub fn quit(&self) {
        self.quit_requested.store(true, Ordering::Relaxed);
        self.quit_notify.notify_one();
    }

    /// Returns `true` once [`Context::quit`] has been called.
    ///
    /// The app loop checks this internally; in unit tests, use it to assert
    /// that an event handler requested exit.
    pub fn quit_requested(&self) -> bool {
        self.quit_requested.load(Ordering::Relaxed)
    }

    /// Reports a fatal error and quits: the terminal is restored and the
    /// error is returned from [`run`](crate::tui::run) / `App::run`.
    ///
    /// Safe to call from event handlers, `update`, or background tasks. The
    /// first error wins; later ones are dropped. For errors the app can
    /// recover from, prefer sending a message and rendering the failure
    /// instead.
    pub fn fail(&self, error: impl Into<anyhow::Error>) {
        // A poisoned lock only means another thread panicked mid-`fail`;
        // the slot is still valid, so keep going rather than lose the error.
        let mut slot = self
            .error
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        slot.get_or_insert_with(|| error.into());
        drop(slot);

        self.quit();
    }

    /// Removes and returns the error stored by [`Context::fail`], if any.
    ///
    /// The app loop takes it internally to return from `run`; in unit tests,
    /// use it to assert that a handler reported failure.
    pub fn take_error(&self) -> Option<anyhow::Error> {
        self.error
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
    }

    /// Creates a context for unit-testing components without a terminal,
    /// plus the receiving end of its message channel.
    ///
    /// ```
    /// use tui_base_framework::Context;
    ///
    /// let (context, mut messages) = Context::<String>::test();
    ///
    /// context.try_send("saved".to_string()).unwrap();
    /// context.quit();
    ///
    /// assert_eq!(messages.try_recv().unwrap(), "saved");
    /// assert!(context.quit_requested());
    /// ```
    pub fn test() -> (Self, mpsc::Receiver<M>) {
        let (sender, receiver) = mpsc::channel(64);
        (Self::new(sender), receiver)
    }

    /// Clears quit and error state so `App::run` can be called again.
    pub(crate) fn reset(&self) {
        self.quit_requested.store(false, Ordering::Relaxed);
        self.take_error();
    }

    /// Resolves once [`Context::quit`] has been called.
    pub(crate) async fn quit_notified(&self) {
        self.quit_notify.notified().await;
    }
}

/// A unit of UI: state, rendering, and input handling.
///
/// This is the only trait you implement. The [`App`](crate::tui::App) loop calls
/// [`render`](Component::render) whenever the UI needs a repaint,
/// [`handle_event`](Component::handle_event) for every terminal event, and
/// [`update`](Component::update) for every message sent through the
/// [`Context`].
pub trait Component: Send {
    /// The message type delivered to [`Component::update`].
    ///
    /// Define an enum with one variant per thing that can happen
    /// asynchronously in your app. Use `()` if your component does not use
    /// messages.
    type Message: Send + 'static;

    /// Called once before the first render. A good place to spawn startup
    /// tasks with [`Context::sender`].
    fn init(&mut self, _context: &Context<Self::Message>) {}

    /// Draws the component into `area`.
    ///
    /// Takes `&mut self` so stateful widgets (`ListState`, `TableState`,
    /// scroll offsets) work without interior mutability. Avoid doing real
    /// work here; mutate state in `handle_event`/`update` instead.
    ///
    /// To show the real terminal cursor (for text input), call
    /// `frame.set_cursor_position(..)`: the cursor is visible on frames that
    /// set a position and hidden on frames that don't. See
    /// `examples/text_input.rs`.
    fn render(&mut self, frame: &mut Frame, area: Rect);

    /// Reacts to a terminal event or tick.
    ///
    /// Return [`EventResult::Consumed`] when the event changed state and the
    /// UI should redraw; return [`EventResult::Propagate`] when you ignored
    /// it.
    fn handle_event(&mut self, _event: Event, _context: &Context<Self::Message>) -> EventResult {
        EventResult::Propagate
    }

    /// Reacts to a message sent via [`Context::sender`] or
    /// [`Context::try_send`]. Always triggers a redraw.
    fn update(&mut self, _message: Self::Message, _context: &Context<Self::Message>) {}
}

impl<T> Component for Box<T>
where
    T: Component + ?Sized,
{
    type Message = T::Message;

    fn init(&mut self, context: &Context<Self::Message>) {
        (**self).init(context);
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        (**self).render(frame, area);
    }

    fn handle_event(&mut self, event: Event, context: &Context<Self::Message>) -> EventResult {
        (**self).handle_event(event, context)
    }

    fn update(&mut self, message: Self::Message, context: &Context<Self::Message>) {
        (**self).update(message, context);
    }
}

#[cfg(test)]
mod tests {
    use super::Context;
    use tokio::sync::mpsc;

    #[test]
    fn quit_latches_even_when_message_channel_is_full() {
        let (sender, _receiver) = mpsc::channel(1);
        sender.try_send("queued").expect("queue first message");

        let context = Context::new(sender);
        context.quit();

        assert!(context.quit_requested());
    }

    #[tokio::test]
    async fn quit_wakes_a_waiting_loop() {
        let (sender, _receiver) = mpsc::channel::<()>(1);
        let context = Context::new(sender);

        context.quit();

        // A stored notification must wake the next waiter immediately.
        context.quit_notified().await;
        assert!(context.quit_requested());
    }

    #[test]
    fn messages_are_delivered_typed() {
        let (sender, mut receiver) = mpsc::channel(4);
        let context = Context::new(sender);

        context.try_send(42_u32).expect("send message");

        assert_eq!(receiver.try_recv(), Ok(42));
    }

    #[test]
    fn fail_stores_the_first_error_and_quits() {
        let (context, _messages) = Context::<()>::test();

        context.fail(std::io::Error::other("disk on fire"));
        context.fail(std::io::Error::other("second failure"));

        assert!(context.quit_requested());
        let error = context.take_error().expect("error stored");
        assert_eq!(error.to_string(), "disk on fire");
        assert!(context.take_error().is_none(), "take_error consumes");
    }
}
