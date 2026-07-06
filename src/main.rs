//! Your app starts here. Edit this file directly.
//! Or copy a self-contained program from `examples/` over it as a starting point.

use anyhow::Result;
use tui_base_framework::layout::Alignment;
use tui_base_framework::widgets::{Block, Paragraph};
use tui_base_framework::{Component, Context, Event, EventResult, Frame, KeyCode, Rect, run};

/// Rename this, grow its fields, and make it your app's state.
struct Starter;

impl Component for Starter {
    /// Define an enum here once your app needs messages from background
    /// tasks. See `examples/async_task.rs`.
    type Message = ();

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let text = "Welcome to your new TUI!\n\n\
            Edit src/main.rs to build your app.\n\n\
            Press q or Esc to quit";

        let widget = Paragraph::new(text)
            .block(Block::bordered().title(" tui-base-framework "))
            .alignment(Alignment::Center);

        frame.render_widget(widget, area);
    }

    fn handle_event(&mut self, event: Event, context: &Context<Self::Message>) -> EventResult {
        if event.is_key(KeyCode::Char('q')) || event.is_key(KeyCode::Esc) {
            context.quit();
            return EventResult::Consumed;
        }

        EventResult::Propagate
    }
}

fn main() -> Result<()> {
    run(Starter)
}
