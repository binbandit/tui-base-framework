use crate::component::{Component, Context, MessageSender};
use crate::terminal::{TerminalConfig, TerminalGuard};
use crate::{Event, Message};
use anyhow::{Context as AnyhowContext, Result};
use crossterm::event::{self, KeyCode, KeyModifiers};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::MissedTickBehavior;

type RuntimeEvent = Result<Event>;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub tick_rate: Duration,
    pub input_poll_rate: Duration,
    pub channel_capacity: usize,
    pub quit_on_ctrl_c: bool,
    pub terminal: TerminalConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            tick_rate: Duration::from_millis(250),
            input_poll_rate: Duration::from_millis(50),
            channel_capacity: 256,
            quit_on_ctrl_c: true,
            terminal: TerminalConfig::default(),
        }
    }
}

impl AppConfig {
    fn channel_capacity(&self) -> usize {
        self.channel_capacity.max(1)
    }

    fn tick_rate(&self) -> Duration {
        non_zero_duration(self.tick_rate, Duration::from_millis(250))
    }

    fn input_poll_rate(&self) -> Duration {
        non_zero_duration(self.input_poll_rate, Duration::from_millis(50))
    }
}

pub struct App<C>
where
    C: Component,
{
    terminal_guard: TerminalGuard,
    component: C,
    config: AppConfig,
    message_tx: MessageSender,
    message_rx: mpsc::Receiver<Message>,
    should_quit: bool,
}

impl<C> App<C>
where
    C: Component,
{
    pub fn new(component: C) -> Result<Self> {
        Self::with_config(component, AppConfig::default())
    }

    pub fn with_config(component: C, config: AppConfig) -> Result<Self> {
        let terminal_guard = TerminalGuard::with_config(config.terminal)?;
        let (message_tx, message_rx) = mpsc::channel(config.channel_capacity());

        Ok(Self {
            terminal_guard,
            component,
            config,
            message_tx,
            message_rx,
            should_quit: false,
        })
    }

    pub fn message_sender(&self) -> MessageSender {
        self.message_tx.clone()
    }

    pub async fn run(&mut self) -> Result<()> {
        self.should_quit = false;

        let (event_tx, mut event_rx) = mpsc::channel(self.config.channel_capacity());
        let shutdown = Arc::new(AtomicBool::new(false));

        let input_handle = spawn_input_loop(
            event_tx.clone(),
            self.config.input_poll_rate(),
            shutdown.clone(),
        );
        let tick_handle = tokio::spawn(tick_loop(event_tx, self.config.tick_rate()));

        let result = self.render_loop(&mut event_rx).await;

        shutdown.store(true, Ordering::Relaxed);
        input_handle.abort();
        tick_handle.abort();

        result
    }

    async fn render_loop(&mut self, event_rx: &mut mpsc::Receiver<RuntimeEvent>) -> Result<()> {
        let context = Context::new(self.message_tx.clone(), Arc::new(AtomicBool::new(false)));
        let mut needs_render = true;

        self.component.init(&context);
        self.sync_context_state(&context);

        loop {
            self.drain_queued_work(event_rx, &context, &mut needs_render)?;

            if self.should_quit {
                break;
            }

            if needs_render {
                self.draw()?;
                needs_render = false;
            }

            tokio::select! {
                event = event_rx.recv() => {
                    match event {
                        Some(event) => self.handle_runtime_event(event, &context, &mut needs_render)?,
                        None => break,
                    }
                }
                message = self.message_rx.recv() => {
                    match message {
                        Some(message) => self.handle_message(message, &context, &mut needs_render),
                        None => break,
                    }
                }
            }
        }

        Ok(())
    }

    fn drain_queued_work(
        &mut self,
        event_rx: &mut mpsc::Receiver<RuntimeEvent>,
        context: &Context,
        needs_render: &mut bool,
    ) -> Result<()> {
        loop {
            let mut made_progress = false;

            while let Ok(message) = self.message_rx.try_recv() {
                made_progress = true;
                self.handle_message(message, context, needs_render);

                if self.should_quit {
                    return Ok(());
                }
            }

            while let Ok(event) = event_rx.try_recv() {
                made_progress = true;
                self.handle_runtime_event(event, context, needs_render)?;

                if self.should_quit {
                    return Ok(());
                }
            }

            if !made_progress {
                return Ok(());
            }
        }
    }

    fn handle_runtime_event(
        &mut self,
        event: RuntimeEvent,
        context: &Context,
        needs_render: &mut bool,
    ) -> Result<()> {
        self.handle_event(event?, context, needs_render);
        Ok(())
    }

    fn handle_event(&mut self, event: Event, context: &Context, needs_render: &mut bool) {
        if self.config.quit_on_ctrl_c && is_ctrl_c(&event) {
            self.should_quit = true;
            return;
        }

        let redraw_for_terminal_change = matches!(event, Event::Resize(_, _));
        let result = self.component.handle_event(event, context);
        self.sync_context_state(context);

        *needs_render |= redraw_for_terminal_change || result.is_consumed();
    }

    fn handle_message(&mut self, message: Message, context: &Context, needs_render: &mut bool) {
        match message {
            Message::Quit => {
                self.should_quit = true;
            }
            message => {
                self.component.update(message, context);
                self.sync_context_state(context);
                *needs_render = true;
            }
        }
    }

    fn sync_context_state(&mut self, context: &Context) {
        self.should_quit |= context.quit_requested();
    }

    fn draw(&mut self) -> Result<()> {
        let component = &self.component;
        let terminal = self.terminal_guard.terminal();

        terminal
            .draw(|frame| {
                component.render(frame, frame.area());
            })
            .context("draw terminal frame")?;

        Ok(())
    }
}

fn spawn_input_loop(
    event_tx: mpsc::Sender<RuntimeEvent>,
    input_poll_rate: Duration,
    shutdown: Arc<AtomicBool>,
) -> JoinHandle<()> {
    tokio::task::spawn_blocking(move || {
        while !shutdown.load(Ordering::Relaxed) {
            match event::poll(input_poll_rate) {
                Ok(true) => match event::read() {
                    Ok(event) if event.is_key_release() => {}
                    Ok(event) => {
                        if event_tx.blocking_send(Ok(Event::from(event))).is_err() {
                            break;
                        }
                    }
                    Err(error) => {
                        let _ = event_tx.blocking_send(Err(error).context("read terminal event"));
                        break;
                    }
                },
                Ok(false) => {}
                Err(error) => {
                    let _ = event_tx.blocking_send(Err(error).context("poll terminal events"));
                    break;
                }
            }
        }
    })
}

async fn tick_loop(event_tx: mpsc::Sender<RuntimeEvent>, tick_rate: Duration) {
    let mut interval = tokio::time::interval(tick_rate);
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    interval.tick().await;

    loop {
        interval.tick().await;

        match event_tx.try_send(Ok(Event::Tick)) {
            Ok(()) | Err(mpsc::error::TrySendError::Full(_)) => {}
            Err(mpsc::error::TrySendError::Closed(_)) => break,
        }
    }
}

fn is_ctrl_c(event: &Event) -> bool {
    matches!(
        event,
        Event::Key(key)
            if key.code == KeyCode::Char('c')
                && key.modifiers.contains(KeyModifiers::CONTROL)
    )
}

fn non_zero_duration(value: Duration, fallback: Duration) -> Duration {
    if value.is_zero() { fallback } else { value }
}

#[cfg(test)]
mod tests {
    use super::{AppConfig, non_zero_duration};
    use std::time::Duration;

    #[test]
    fn app_config_never_uses_a_zero_sized_channel() {
        let config = AppConfig {
            channel_capacity: 0,
            ..AppConfig::default()
        };

        assert_eq!(config.channel_capacity(), 1);
    }

    #[test]
    fn zero_duration_uses_fallback() {
        assert_eq!(
            non_zero_duration(Duration::ZERO, Duration::from_millis(50)),
            Duration::from_millis(50)
        );
    }
}
