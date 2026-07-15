//! Tick-driven animation with a progress gauge.
//!
//! `Event::Tick` fires at `AppConfig::tick_rate` (250ms by default; this
//! example speeds it up for a smoother animation) and carries the time
//! elapsed since the previous tick. Scaling movement by that duration keeps
//! animation speed independent of the tick rate — try changing `tick_rate`
//! below and the bar still fills at the same pace.
//!
//! Run with: `cargo run --example progress`

use anyhow::Result;
use std::time::Duration;
use tui_base_framework::layout::{Constraint, Layout};
use tui_base_framework::style::{Color, Style};
use tui_base_framework::widgets::{Block, Gauge, Paragraph};
use tui_base_framework::{
    AppConfig, Component, Context, Event, EventResult, Frame, KeyCode, Rect, run_with_config,
};

/// How fast the bar fills, in percent per second.
const FILL_RATE: f64 = 25.0;

struct ProgressDemo {
    percent: f64,
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
                .percent(self.percent as u16),
            bar,
        );

        let status = if self.paused { "PAUSED" } else { "RUNNING" };
        frame.render_widget(
            Paragraph::new(format!(
                "Status: {}\nProgress: {:.0}%\n\nThe bar advances by the time elapsed between\nticks ({FILL_RATE}%/s), independent of tick rate.",
                status, self.percent
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
            Event::Tick(elapsed) if !self.paused => {
                self.percent = (self.percent + elapsed.as_secs_f64() * FILL_RATE) % 100.0;
                EventResult::Consumed
            }
            Event::Key(key) => match key.code {
                KeyCode::Char(' ') => {
                    self.paused = !self.paused;
                    EventResult::Consumed
                }
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    self.percent = 0.0;
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

fn main() -> Result<()> {
    let config = AppConfig {
        tick_rate: Duration::from_millis(50),
        ..AppConfig::default()
    };

    let component = ProgressDemo {
        percent: 0.0,
        paused: false,
    };

    run_with_config(component, config)
}
