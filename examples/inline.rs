//! An inline app: the UI lives in a few rows of the normal scrollback
//! instead of taking over the screen — the shape of a download bar, a test
//! runner, or any tool that should leave its output behind.
//!
//! Output printed before the app starts stays visible above the UI, and the
//! final frame stays in the scrollback after exit, with the shell prompt
//! continuing below it. Inline setup queries the cursor position, so this
//! needs a real interactive terminal (not a pipe).
//!
//! Run with: `cargo run --example inline`

use anyhow::Result;
use std::time::Duration;
use tui_base_framework::layout::{Constraint, Layout};
use tui_base_framework::style::{Color, Style};
use tui_base_framework::widgets::{Block, Gauge, Paragraph};
use tui_base_framework::{
    AppConfig, Component, Context, Event, EventResult, Frame, KeyCode, Rect, TerminalConfig,
    Viewport, run_with_config,
};

/// How fast the bar fills, in percent per second.
const FILL_RATE: f64 = 40.0;

struct InlineDemo {
    percent: f64,
}

impl Component for InlineDemo {
    type Message = ();

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let [bar, help] =
            Layout::vertical([Constraint::Length(3), Constraint::Length(1)]).areas(area);

        frame.render_widget(
            Gauge::default()
                .block(Block::bordered().title("Downloading"))
                .gauge_style(Style::default().fg(Color::Green))
                .percent(self.percent as u16),
            bar,
        );

        frame.render_widget(
            Paragraph::new(" The final frame stays in your scrollback | q to cancel")
                .style(Style::default().fg(Color::DarkGray)),
            help,
        );
    }

    fn handle_event(&mut self, event: Event, context: &Context<Self::Message>) -> EventResult {
        match event {
            Event::Tick(elapsed) => {
                self.percent = (self.percent + elapsed.as_secs_f64() * FILL_RATE).min(100.0);
                if self.percent >= 100.0 {
                    context.quit();
                }
                EventResult::Consumed
            }
            _ if event.is_key(KeyCode::Char('q')) || event.is_key(KeyCode::Esc) => {
                context.quit();
                EventResult::Consumed
            }
            _ => EventResult::Propagate,
        }
    }
}

fn main() -> Result<()> {
    // Anything printed before the app starts stays visible above the UI.
    println!("$ fetching release notes...");
    println!("$ downloading v2.0.1");

    let config = AppConfig {
        tick_rate: Duration::from_millis(50),
        terminal: TerminalConfig {
            viewport: Viewport::Inline(4),
            ..TerminalConfig::default()
        },
        ..AppConfig::default()
    };

    run_with_config(InlineDemo { percent: 0.0 }, config)?;

    // ...and the shell (or your program) continues below the final frame.
    println!("done.");
    Ok(())
}
