use crate::component::Component;
use crate::event::{Event, EventResult};
use crate::message::Message;
use ratatui::{Frame, layout::{Rect, Layout, Direction, Constraint}};
use ratatui::widgets::{Paragraph, Block, Borders, Gauge};
use ratatui::style::{Style, Color};
use crossterm::event::KeyCode;
use tokio::sync::mpsc;
use std::time::Instant;

/// Demonstrates progress bars and time-based updates
pub struct ProgressDemo {
    progress: u16,
    start_time: Instant,
    paused: bool,
    message_sender: Option<mpsc::Sender<Message>>,
}

impl ProgressDemo {
    pub fn new() -> Self {
        Self {
            progress: 0,
            start_time: Instant::now(),
            paused: false,
            message_sender: None,
        }
    }
    
    fn update_progress(&mut self) {
        if !self.paused {
            let elapsed = self.start_time.elapsed().as_secs();
            self.progress = ((elapsed % 10) * 10) as u16;
        }
    }
}

impl Component for ProgressDemo {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);
        
        // Title
        let title = Paragraph::new("Progress Bar Demo")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(title, chunks[0]);
        
        // Progress bar
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Progress"))
            .gauge_style(Style::default().fg(Color::Green))
            .percent(self.progress);
        frame.render_widget(gauge, chunks[1]);
        
        // Info
        let status = if self.paused { "PAUSED" } else { "RUNNING" };
        let info = Paragraph::new(format!(
            "Status: {}\nProgress: {}%\n\nThe progress bar automatically updates\nevery second (simulating a task).",
            status, self.progress
        ))
        .block(Block::default().borders(Borders::ALL).title("Info"))
        .style(Style::default().fg(Color::White));
        frame.render_widget(info, chunks[2]);
        
        // Controls
        let controls = Paragraph::new("Space to pause/resume | r to reset | q to quit")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(controls, chunks[3]);
    }
    
    fn handle_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Tick => {
                // Update progress on every tick
                self.update_progress();
                EventResult::Consumed
            }
            Event::Key(key) => {
                match key.code {
                    KeyCode::Char(' ') => {
                        self.paused = !self.paused;
                        EventResult::Consumed
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        self.start_time = Instant::now();
                        self.progress = 0;
                        self.paused = false;
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

impl Default for ProgressDemo {
    fn default() -> Self {
        Self::new()
    }
}
