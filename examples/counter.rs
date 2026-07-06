//! Mutable state driven by keyboard events.
//!
//! Run with: `cargo run --example counter`

use anyhow::Result;
use tui_base_framework::layout::Alignment;
use tui_base_framework::widgets::Paragraph;
use tui_base_framework::{
    App, Component, Context, Event, EventResult, Frame, KeyCode, KeyModifiers, Rect,
};

struct Counter {
    count: i64,
}

impl Component for Counter {
    type Message = ();

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let text = format!("Count: {} (Press ↑/↓, q to quit)", self.count);
        frame.render_widget(Paragraph::new(text).alignment(Alignment::Center), area);
    }

    fn handle_event(&mut self, event: Event, context: &Context<Self::Message>) -> EventResult {
        let Event::Key(key) = event else {
            return EventResult::Propagate;
        };

        // Leave shortcuts like Ctrl-C to the framework default.
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return EventResult::Propagate;
        }

        match key.code {
            KeyCode::Up => {
                self.count += 1;
                EventResult::Consumed
            }
            KeyCode::Down => {
                self.count -= 1;
                EventResult::Consumed
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                context.quit();
                EventResult::Consumed
            }
            _ => EventResult::Propagate,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    App::new(Counter { count: 0 })?.run().await
}
