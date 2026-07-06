//! Tick-driven animation with a progress gauge.
//!
//! `Event::Tick` fires at `AppConfig::tick_rate` (250ms by default); this
//! example speeds it up for a smoother animation.
//!
//! Run with: `cargo run --example progress`

use anyhow::Result;
use std::time::Duration;
use tui_base_framework::layout::{Constraint, Layout};
use tui_base_framework::style::{Color, Style};
use tui_base_framework::widgets::{Block, Gauge, Paragraph};
use tui_base_framework::{
    App, AppConfig, Component, Context, Event, EventResult, Frame, KeyCode, Rect,
};

struct ProgressDemo {
    progress: u16,
    paused: bool,
}

impl Component for ProgressDemo {
    type Message = ();

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let [title, bar, info, controls] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .areas(area);

        frame.render_widget(
            Paragraph::new("Progress Bar Demo")
                .block(Block::bordered())
                .style(Style::default().fg(Color::Cyan)),
            title,
        );

        frame.render_widget(
            Gauge::default()
                .block(Block::bordered().title("Progress"))
                .gauge_style(Style::default().fg(Color::Green))
                .percent(self.progress),
            bar,
        );

        let status = if self.paused { "PAUSED" } else { "RUNNING" };
        frame.render_widget(
            Paragraph::new(format!(
                "Status: {}\nProgress: {}%\n\nThe progress bar advances on each tick\n(simulating a task).",
                status, self.progress
            ))
            .block(Block::bordered().title("Info")),
            info,
        );

        frame.render_widget(
            Paragraph::new("Space to pause/resume | r to reset | q to quit")
                .block(Block::bordered())
                .style(Style::default().fg(Color::Yellow)),
            controls,
        );
    }

    fn handle_event(&mut self, event: Event, context: &Context<Self::Message>) -> EventResult {
        match event {
            Event::Tick if !self.paused => {
                self.progress = (self.progress + 1) % 101;
                EventResult::Consumed
            }
            Event::Key(key) => match key.code {
                KeyCode::Char(' ') => {
                    self.paused = !self.paused;
                    EventResult::Consumed
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    self.progress = 0;
                    self.paused = false;
                    EventResult::Consumed
                }
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    context.quit();
                    EventResult::Consumed
                }
                _ => EventResult::Propagate,
            },
            _ => EventResult::Propagate,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig {
        tick_rate: Duration::from_millis(50),
        ..AppConfig::default()
    };

    let component = ProgressDemo {
        progress: 0,
        paused: false,
    };

    App::with_config(component, config)?.run().await
}
