//! The smallest possible app: render some text, quit on `q`.
//!
//! Run with: `cargo run --example hello_world`

use anyhow::Result;
use tui_base_framework::layout::Alignment;
use tui_base_framework::style::{Color, Style};
use tui_base_framework::widgets::{Block, Paragraph};
use tui_base_framework::{App, Component, Context, Event, EventResult, Frame, KeyCode, Rect};

struct HelloWorld;

impl Component for HelloWorld {
    type Message = ();

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let widget = Paragraph::new("Hello, TUI World!\n\nPress 'q' to quit")
            .block(
                Block::bordered()
                    .title("Hello World")
                    .style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Center);

        frame.render_widget(widget, area);
    }

    fn handle_event(&mut self, event: Event, context: &Context<Self::Message>) -> EventResult {
        if let Event::Key(key) = event
            && matches!(key.code, KeyCode::Char('q') | KeyCode::Char('Q'))
        {
            context.quit();
            return EventResult::Consumed;
        }

        EventResult::Propagate
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    App::new(HelloWorld)?.run().await
}
