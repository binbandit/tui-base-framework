use crate::{Component, Message, Event};
use crate::terminal::TerminalGuard;
use tokio::sync::mpsc;
use std::time::Duration;
use crossterm::event::{self, Event as CrosstermEvent};
use anyhow::Result;

pub struct App {
    terminal_guard: TerminalGuard,
    component: Box<dyn Component>,
    message_tx: mpsc::Sender<Message>,
    message_rx: mpsc::Receiver<Message>,
    event_rx: mpsc::Receiver<Event>,
    should_quit: bool,
}

impl App {
    pub fn new(mut component: Box<dyn Component>) -> Result<Self> {
        let terminal_guard = TerminalGuard::new()?;
        let (message_tx, message_rx) = mpsc::channel(100);
        let (_event_tx, event_rx) = mpsc::channel(100);
        
        component.set_message_sender(message_tx.clone());
        
        Ok(Self {
            terminal_guard,
            component,
            message_tx,
            message_rx,
            event_rx,
            should_quit: false,
        })
    }
    
    pub async fn run(&mut self) -> Result<()> {
        // Create event channel for this run
        let (event_tx, mut event_rx) = mpsc::channel(100);
        
        // Swap in the new receiver
        std::mem::swap(&mut self.event_rx, &mut event_rx);
        
        // Spawn input loop in background task
        let input_handle = tokio::spawn(async move {
            let _ = Self::input_loop(event_tx).await;
        });
        
        // Run render loop (blocks until quit)
        let render_result = self.render_loop().await;
        
        // Abort the input task - it's safe since we're exiting
        input_handle.abort();
        
        render_result
    }
    
    async fn render_loop(&mut self) -> Result<()> {
        let mut interval = tokio::time::interval(Duration::from_millis(16));
        
        loop {
            interval.tick().await;
            
            while let Ok(event) = self.event_rx.try_recv() {
                self.component.handle_event(event);
            }
            
            while let Ok(message) = self.message_rx.try_recv() {
                match message {
                    Message::Quit => {
                        self.should_quit = true;
                    }
                    _ => {
                        self.component.update(message);
                    }
                }
            }
            
            if self.should_quit {
                break;
            }
            
            let terminal = self.terminal_guard.terminal();
            terminal.draw(|frame| {
                let area = frame.area();
                self.component.render(frame, area);
            })?;
        }
        
        Ok(())
    }
    
    async fn input_loop(event_tx: mpsc::Sender<Event>) -> Result<()> {
        let mut tick_interval = tokio::time::interval(Duration::from_millis(250));
        
        loop {
            tokio::select! {
                _ = tick_interval.tick() => {
                    if event_tx.send(Event::Tick).await.is_err() {
                        // Channel closed, exit loop
                        break;
                    }
                }
                result = async {
                    if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                        if let Ok(crossterm_event) = event::read() {
                            let event = match crossterm_event {
                                CrosstermEvent::Key(key) => Some(Event::Key(key)),
                                CrosstermEvent::Mouse(mouse) => Some(Event::Mouse(mouse)),
                                CrosstermEvent::Resize(width, height) => Some(Event::Resize(width, height)),
                                _ => None,
                            };
                            
                            if let Some(event) = event {
                                return event_tx.send(event).await.is_err();
                            }
                        }
                    }
                    false
                } => {
                    if result {
                        // Channel closed, exit loop
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}
