//! Background work with typed messages: spawn a Tokio task, report progress
//! back to the UI through `Context::sender`.
//!
//! This is the pattern for anything slow — network requests, file IO,
//! subprocesses — so the UI stays responsive while work happens elsewhere.
//!
//! Run with: `cargo run --example async_task`

use anyhow::Result;
use std::time::Duration;
use tui_base_framework::layout::{Constraint, Layout};
use tui_base_framework::style::{Color, Style};
use tui_base_framework::widgets::{Block, Gauge, Paragraph};
use tui_base_framework::{App, Component, Context, Event, EventResult, Frame, KeyCode, Rect};

/// Everything the background task can tell the UI.
enum Msg {
    Progress(u16),
    Done { records: u32 },
}

enum JobState {
    Idle,
    Running { progress: u16 },
    Finished { records: u32 },
}

struct Downloader {
    job: JobState,
}

impl Downloader {
    fn start_job(&mut self, context: &Context<Msg>) {
        self.job = JobState::Running { progress: 0 };

        // Move a sender into the task; every message lands in `update`.
        let sender = context.sender();
        tokio::spawn(async move {
            for percent in (0..=100).step_by(4) {
                tokio::time::sleep(Duration::from_millis(60)).await;
                if sender.send(Msg::Progress(percent)).await.is_err() {
                    return; // App shut down; stop working.
                }
            }
            let _ = sender.send(Msg::Done { records: 1337 }).await;
        });
    }
}

impl Component for Downloader {
    type Message = Msg;

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let [status_area, gauge_area, help_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .areas(area);

        let (status, progress) = match self.job {
            JobState::Idle => ("Idle - press 's' to start the download".to_string(), 0),
            JobState::Running { progress } => (format!("Downloading... {progress}%"), progress),
            JobState::Finished { records } => (format!("Done! Fetched {records} records."), 100),
        };

        frame.render_widget(
            Paragraph::new(status).block(Block::bordered().title("Async Task")),
            status_area,
        );

        frame.render_widget(
            Gauge::default()
                .block(Block::bordered())
                .gauge_style(Style::default().fg(Color::Green))
                .percent(progress),
            gauge_area,
        );

        frame.render_widget(
            Paragraph::new(
                "The download runs in a spawned Tokio task and reports back\n\
                through typed messages. Try mashing 's' or moving the window:\n\
                the UI never blocks.\n\n\
                s to start | q to quit",
            )
            .block(Block::bordered().title("How it works")),
            help_area,
        );
    }

    fn handle_event(&mut self, event: Event, context: &Context<Self::Message>) -> EventResult {
        let Event::Key(key) = event else {
            return EventResult::Propagate;
        };

        match key.code {
            KeyCode::Char('s') | KeyCode::Char('S') => {
                if !matches!(self.job, JobState::Running { .. }) {
                    self.start_job(context);
                }
                EventResult::Consumed
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                context.quit();
                EventResult::Consumed
            }
            _ => EventResult::Propagate,
        }
    }

    fn update(&mut self, message: Self::Message, _context: &Context<Self::Message>) {
        match message {
            Msg::Progress(percent) => self.job = JobState::Running { progress: percent },
            Msg::Done { records } => self.job = JobState::Finished { records },
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    App::new(Downloader {
        job: JobState::Idle,
    })?
    .run()
    .await
}
