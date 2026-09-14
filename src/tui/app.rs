//! The app loop: terminal lifecycle, event pump, and redraw scheduling.

use crate::tui::component::{Component, Context};
use crate::tui::event::Event;
use crate::tui::terminal::{TerminalConfig, TerminalGuard};
use anyhow::{Context as AnyhowContext, Result};
use crossterm::event;
use std::sync::{
    Arc, Mutex, MutexGuard,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::MissedTickBehavior;

type RuntimeEvent = Result<Event>;

/// Runs `component` until it quits, creating the Tokio runtime for you.
///
/// This is all a typical `main` needs:
///
/// ```ignore
/// fn main() -> anyhow::Result<()> {
///     tui_base_framework::run(MyApp::new())
/// }
/// ```
///
/// Components can still `tokio::spawn` background tasks — they run on the
/// runtime created here. If you need async setup before the UI starts, or
/// your own runtime configuration, use `#[tokio::main]` with [`App`] instead.
pub fn run<C: Component>(component: C) -> Result<()> {
    run_with_config(component, AppConfig::default())
}

/// Like [`run`], with a custom [`AppConfig`].
pub fn run_with_config<C: Component>(component: C, config: AppConfig) -> Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("build tokio runtime")?;

    runtime.block_on(async move { App::with_config(component, config)?.run().await })
}

/// Runtime tuning knobs. Start from [`AppConfig::default`] and override what
/// you need with struct-update syntax.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// How often [`Event::Tick`] fires. Lower it for smoother animation.
    pub tick_rate: Duration,
    /// How long the input thread blocks waiting for terminal input before
    /// checking for shutdown. Also bounds shutdown and inline redraw latency;
    /// keep this short. Inline apps also cap this at `tick_rate` so animation
    /// draws need not wait longer than one tick. Zero uses the default of 50 ms.
    pub input_poll_rate: Duration,
    /// Capacity of the event and message channels.
    pub channel_capacity: usize,
    /// Exit the app loop on Ctrl-C. The component sees the key press first:
    /// if it consumes the event (say, to ask for confirmation), the app keeps
    /// running. Disable to handle Ctrl-C entirely yourself.
    pub quit_on_ctrl_c: bool,
    /// Suspend to the shell on Ctrl-Z and take the terminal back on resume
    /// (`fg`). As with Ctrl-C, the component sees the key press first. Unix
    /// only — on Windows the key reaches the component like any other.
    pub suspend_on_ctrl_z: bool,
    /// Terminal features to enable (mouse capture, bracketed paste, ...).
    pub terminal: TerminalConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            tick_rate: Duration::from_millis(250),
            input_poll_rate: Duration::from_millis(50),
            channel_capacity: 256,
            quit_on_ctrl_c: true,
            suspend_on_ctrl_z: true,
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

/// Owns the terminal and drives a [`Component`].
///
/// Construction puts the terminal into raw mode and the configured viewport;
/// dropping the `App` (or panicking) restores it.
pub struct App<C>
where
    C: Component,
{
    terminal_guard: TerminalGuard,
    component: C,
    config: AppConfig,
    context: Context<C::Message>,
    message_rx: mpsc::Receiver<C::Message>,
    should_quit: bool,
    input_lock: Arc<InputGate>,
}

impl<C> App<C>
where
    C: Component,
{
    /// Creates an app with [`AppConfig::default`] and takes over the terminal.
    pub fn new(component: C) -> Result<Self> {
        Self::with_config(component, AppConfig::default())
    }

    /// Creates an app with a custom [`AppConfig`] and takes over the terminal.
    pub fn with_config(component: C, config: AppConfig) -> Result<Self> {
        let terminal_guard = TerminalGuard::with_config(config.terminal)?;
        let (message_tx, message_rx) = mpsc::channel(config.channel_capacity());

        Ok(Self {
            terminal_guard,
            component,
            config,
            context: Context::new(message_tx),
            message_rx,
            should_quit: false,
            input_lock: Arc::new(InputGate::default()),
        })
    }

    /// Returns a sender that delivers messages to the component from outside
    /// the app loop (for example, a task spawned before [`App::run`]).
    pub fn message_sender(&self) -> mpsc::Sender<C::Message> {
        self.context.sender()
    }

    /// Runs the app loop until the component quits, Ctrl-C is pressed (when
    /// enabled), [`Context::fail`] reports an error, or an input error occurs.
    ///
    /// Cancelling this future stops and joins its input thread. The terminal
    /// remains owned by `App` until it is dropped, so `run` can be called again.
    pub async fn run(&mut self) -> Result<()> {
        self.should_quit = false;
        self.context.reset();

        let (event_tx, mut event_rx) = mpsc::channel(self.config.channel_capacity());
        // The reader joins on drop, including when this future is cancelled.
        // A new run can never race an old reader for terminal input.
        let input = InputReader::spawn(
            event_tx,
            match self.config.terminal.viewport {
                crate::tui::terminal::Viewport::Inline(_) => {
                    self.config.input_poll_rate().min(self.config.tick_rate())
                }
                crate::tui::terminal::Viewport::Fullscreen => self.config.input_poll_rate(),
            },
            self.input_lock.clone(),
            read_terminal_event,
        )?;

        let result = self.render_loop(&mut event_rx).await;
        drop(input);

        result?;

        // An error reported through `Context::fail` (from a handler or a
        // background task) surfaces as the run's result.
        match self.context.take_error() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    async fn render_loop(&mut self, event_rx: &mut mpsc::Receiver<RuntimeEvent>) -> Result<()> {
        let context = self.context.clone();
        let mut needs_render = true;

        let tick_rate = self.config.tick_rate();
        let mut ticks =
            tokio::time::interval_at(tokio::time::Instant::now() + tick_rate, tick_rate);
        ticks.set_missed_tick_behavior(MissedTickBehavior::Skip);
        let mut last_tick = Instant::now();

        self.component.init(&context);

        loop {
            self.drain_queued_work(event_rx, &context, &mut needs_render)?;

            // Fatal initialization/update failures can leave state unsuitable
            // for rendering. Preserve the error instead of drawing that state.
            if context.quit_requested()
                && let Some(error) = context.take_error()
            {
                return Err(error);
            }

            if needs_render {
                self.draw()?;
                needs_render = false;
            }

            // Render the final state before exiting, particularly for inline
            // progress UIs whose final frame remains in the scrollback.
            if self.quit_pending(&context) {
                break;
            }

            // A continuously replenished queue must not monopolize a
            // single-threaded runtime and starve its background tasks.
            tokio::task::yield_now().await;
            tokio::select! {
                event = event_rx.recv() => {
                    match event {
                        Some(event) => self.handle_runtime_event(event, &context, &mut needs_render)?,
                        None => anyhow::bail!("terminal input thread stopped unexpectedly"),
                    }
                }
                _ = ticks.tick() => {
                    let now = Instant::now();
                    self.handle_event(Event::Tick(now - last_tick), &context, &mut needs_render)?;
                    last_tick = now;
                }
                message = self.message_rx.recv() => {
                    match message {
                        Some(message) => self.handle_message(message, &context, &mut needs_render),
                        None => break,
                    }
                }
                () = context.quit_notified() => {}
            }
        }

        Ok(())
    }

    /// Coalesces a bounded batch, alternating messages and input so neither
    /// a self-sending component nor a busy producer can starve the other.
    fn drain_queued_work(
        &mut self,
        event_rx: &mut mpsc::Receiver<RuntimeEvent>,
        context: &Context<C::Message>,
        needs_render: &mut bool,
    ) -> Result<()> {
        for _ in 0..self.config.channel_capacity().min(64) {
            if self.quit_pending(context) {
                break;
            }

            let message = self.message_rx.try_recv().ok();
            let event = event_rx.try_recv().ok();
            if message.is_none() && event.is_none() {
                break;
            }

            if let Some(message) = message {
                self.handle_message(message, context, needs_render);
            }
            if self.quit_pending(context) {
                break;
            }
            if let Some(event) = event {
                self.handle_runtime_event(event, context, needs_render)?;
            }
        }
        Ok(())
    }

    fn quit_pending(&mut self, context: &Context<C::Message>) -> bool {
        self.should_quit |= context.quit_requested();
        self.should_quit
    }

    fn handle_runtime_event(
        &mut self,
        event: RuntimeEvent,
        context: &Context<C::Message>,
        needs_render: &mut bool,
    ) -> Result<()> {
        self.handle_event(event?, context, needs_render)
    }

    fn handle_event(
        &mut self,
        event: Event,
        context: &Context<C::Message>,
        needs_render: &mut bool,
    ) -> Result<()> {
        let resized = matches!(event, Event::Resize(_, _));
        let ctrl_c = event.is_ctrl('c');
        #[cfg(unix)]
        let ctrl_z = event.is_ctrl('z');

        let result = self.component.handle_event(event, context);
        *needs_render |= resized || result.is_consumed();

        // The component gets first refusal on Ctrl-C and Ctrl-Z: consuming
        // the event overrides the default (e.g. to confirm before quitting).
        if result.is_consumed() {
            return Ok(());
        }

        if self.config.quit_on_ctrl_c && ctrl_c {
            self.should_quit = true;
            return Ok(());
        }

        #[cfg(unix)]
        if self.config.suspend_on_ctrl_z && ctrl_z {
            self.suspend()?;
            *needs_render = true;
        }

        Ok(())
    }

    /// Hands the terminal back to the shell and stops the process until it
    /// is resumed (e.g. `fg`), then takes the terminal over again.
    #[cfg(unix)]
    fn suspend(&mut self) -> Result<()> {
        // Inline resume queries stdin for the cursor position. The reader
        // must not consume that reply (or shell input while suspended).
        let _input = self.input_lock.pause();
        self.terminal_guard.suspend();

        // The whole process stops inside `raise` and continues from here
        // once the shell resumes it.
        signal_hook::low_level::raise(signal_hook::consts::SIGTSTP).context("raise SIGTSTP")?;

        self.terminal_guard.resume()
    }

    fn handle_message(
        &mut self,
        message: C::Message,
        context: &Context<C::Message>,
        needs_render: &mut bool,
    ) {
        self.component.update(message, context);
        *needs_render = true;
    }

    fn draw(&mut self) -> Result<()> {
        // Ratatui queries the cursor through stdin when an inline viewport
        // resizes. Fullscreen draws never need to wait for the input reader.
        let _input = matches!(
            self.config.terminal.viewport,
            crate::tui::terminal::Viewport::Inline(_)
        )
        .then(|| self.input_lock.pause());
        let Self {
            terminal_guard,
            component,
            ..
        } = self;

        terminal_guard
            .terminal()
            .draw(|frame| component.render(frame, frame.area()))
            .context("draw terminal frame")?;

        Ok(())
    }
}

/// A pause flag prevents the reader immediately reacquiring the mutex while
/// a draw or resume is waiting to query stdin. The mutex waits out any read
/// already in progress; the flag alone would leave that race open.
#[derive(Default)]
struct InputGate {
    paused: AtomicBool,
    lock: Mutex<()>,
}

impl InputGate {
    fn pause(&self) -> InputPause<'_> {
        self.paused.store(true, Ordering::Relaxed);
        InputPause {
            gate: self,
            _lock: self.lock.lock().unwrap_or_else(|e| e.into_inner()),
        }
    }
}

struct InputPause<'a> {
    gate: &'a InputGate,
    _lock: MutexGuard<'a, ()>,
}

impl Drop for InputPause<'_> {
    fn drop(&mut self) {
        self.gate.paused.store(false, Ordering::Relaxed);
    }
}

/// Owns the input thread so cancellation cannot detach it. Unlike a Tokio
/// blocking task, a started OS thread cannot be stopped with `abort()`.
struct InputReader {
    shutdown: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl InputReader {
    fn spawn(
        event_tx: mpsc::Sender<RuntimeEvent>,
        poll_rate: Duration,
        input_lock: Arc<InputGate>,
        mut read: impl FnMut(Duration) -> Result<Option<Event>> + Send + 'static,
    ) -> Result<Self> {
        let shutdown = Arc::new(AtomicBool::new(false));
        let stopped = shutdown.clone();
        let thread = std::thread::Builder::new()
            .name("terminal-input".into())
            .spawn(move || {
                while !stopped.load(Ordering::Relaxed) {
                    let next = {
                        if input_lock.paused.load(Ordering::Relaxed) {
                            std::thread::park_timeout(Duration::from_millis(1));
                            continue;
                        }
                        let _input = input_lock.lock.lock().unwrap_or_else(|e| e.into_inner());
                        if stopped.load(Ordering::Relaxed) {
                            break;
                        }
                        if input_lock.paused.load(Ordering::Relaxed) {
                            continue;
                        }
                        read(poll_rate)
                    };
                    let next = match next {
                        Ok(Some(event)) => Ok(event),
                        Ok(None) => continue,
                        Err(error) => Err(error),
                    };
                    let failed = next.is_err();
                    if !send_input(&event_tx, next, &stopped) || failed {
                        break;
                    }
                }
            })
            .context("start terminal input thread")?;
        Ok(Self {
            shutdown,
            thread: Some(thread),
        })
    }
}

impl Drop for InputReader {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            thread.thread().unpark();
            // Shutdown is bounded by poll_rate. Joining is necessary even on
            // cancellation: terminal restoration and the next run must happen
            // after the last read, not while it is still in flight.
            let _ = thread.join();
        }
    }
}

fn read_terminal_event(poll_rate: Duration) -> Result<Option<Event>> {
    if !event::poll(poll_rate).context("poll terminal events")? {
        return Ok(None);
    }
    let event = event::read().context("read terminal event")?;
    Ok((!event.is_key_release()).then(|| Event::from(event)))
}

fn send_input(
    sender: &mpsc::Sender<RuntimeEvent>,
    mut event: RuntimeEvent,
    shutdown: &AtomicBool,
) -> bool {
    // blocking_send cannot be cancelled while the queue is full. Keep the
    // event under backpressure, but let shutdown wake the thread promptly.
    while !shutdown.load(Ordering::Relaxed) {
        match sender.try_send(event) {
            Ok(()) => return true,
            Err(mpsc::error::TrySendError::Closed(_)) => return false,
            Err(mpsc::error::TrySendError::Full(pending)) => event = pending,
        }
        std::thread::park_timeout(Duration::from_millis(1));
    }
    false
}

fn non_zero_duration(value: Duration, fallback: Duration) -> Duration {
    if value.is_zero() { fallback } else { value }
}

#[cfg(test)]
mod tests {
    use super::{AppConfig, InputGate, InputReader, non_zero_duration};
    use crate::tui::Event;
    use crossterm::event::KeyCode;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc as sync_mpsc,
    };
    use std::time::Duration;
    use tokio::sync::mpsc;

    struct Stopped(Arc<AtomicBool>);

    impl Drop for Stopped {
        fn drop(&mut self) {
            self.0.store(true, Ordering::Relaxed);
        }
    }

    #[test]
    fn input_shutdown_does_not_wait_for_a_full_channel_to_drain() {
        let (sender, _receiver) = mpsc::channel(1);
        sender
            .try_send(Ok(Event::key_press(KeyCode::Char('a'))))
            .unwrap();
        let (reading, read_started) = sync_mpsc::channel();
        let stopped = Arc::new(AtomicBool::new(false));
        let marker = Stopped(stopped.clone());
        let reader = InputReader::spawn(
            sender,
            Duration::from_millis(1),
            Arc::new(InputGate::default()),
            move |_| {
                let _keep_alive = &marker;
                reading.send(()).unwrap();
                Ok(Some(Event::key_press(KeyCode::Char('b'))))
            },
        )
        .unwrap();
        read_started.recv_timeout(Duration::from_secs(1)).unwrap();

        let (finished, completion) = sync_mpsc::channel();
        std::thread::spawn(move || {
            drop(reader);
            finished.send(()).unwrap();
        });
        completion
            .recv_timeout(Duration::from_secs(1))
            .expect("reader shutdown must not wait for channel capacity");
        assert!(stopped.load(Ordering::Relaxed), "drop joins the reader");
    }

    #[tokio::test]
    async fn cancelling_a_run_joins_its_input_reader() {
        let (sender, mut receiver) = mpsc::channel(1);
        let stopped = Arc::new(AtomicBool::new(false));
        let marker = Stopped(stopped.clone());
        let (started, ready) = tokio::sync::oneshot::channel();
        let run = tokio::spawn(async move {
            let _reader = InputReader::spawn(
                sender,
                Duration::from_millis(1),
                Arc::new(InputGate::default()),
                move |poll_rate| {
                    let _keep_alive = &marker;
                    std::thread::sleep(poll_rate);
                    Ok(None)
                },
            )
            .unwrap();
            started.send(()).unwrap();
            std::future::pending::<()>().await;
        });
        ready.await.unwrap();
        run.abort();
        assert!(run.await.unwrap_err().is_cancelled());
        assert!(stopped.load(Ordering::Relaxed));
        assert!(
            receiver.recv().await.is_none(),
            "old sender is gone before another run starts"
        );
    }

    #[test]
    fn input_backpressure_preserves_order_and_reports_errors() {
        let (sender, mut receiver) = mpsc::channel(1);
        let mut keys = ['a', 'b', 'c'].into_iter();
        let reader = InputReader::spawn(
            sender,
            Duration::from_millis(1),
            Arc::new(InputGate::default()),
            move |_| match keys.next() {
                Some(key) => Ok(Some(Event::key_press(KeyCode::Char(key)))),
                None => anyhow::bail!("input disconnected"),
            },
        )
        .unwrap();
        for expected in ['a', 'b', 'c'] {
            assert!(
                receiver
                    .blocking_recv()
                    .unwrap()
                    .unwrap()
                    .is_key(KeyCode::Char(expected))
            );
        }
        assert_eq!(
            receiver.blocking_recv().unwrap().unwrap_err().to_string(),
            "input disconnected"
        );
        assert!(receiver.blocking_recv().is_none());
        drop(reader);
    }

    #[test]
    fn paused_input_cannot_read_and_can_still_shut_down() {
        let gate = Arc::new(InputGate::default());
        let paused = gate.pause();
        let (sender, _receiver) = mpsc::channel(1);
        let (reading, read_started) = sync_mpsc::channel();
        let reader =
            InputReader::spawn(sender, Duration::from_millis(1), gate.clone(), move |_| {
                reading.send(()).unwrap();
                Ok(None)
            })
            .unwrap();
        assert!(matches!(
            read_started.recv_timeout(Duration::from_millis(20)),
            Err(sync_mpsc::RecvTimeoutError::Timeout)
        ));
        drop(reader);
        assert!(matches!(
            read_started.try_recv(),
            Err(sync_mpsc::TryRecvError::Disconnected)
        ));
        drop(paused);
    }

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
