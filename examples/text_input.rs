//! Character input, backspace, enter, and bracketed paste.
//!
//! Run with: `cargo run --example text_input`

use anyhow::Result;
use tui_base_framework::style::{Color, Style};
use tui_base_framework::widgets::{Block, Paragraph};
use tui_base_framework::{
    App, Component, Context, Event, EventResult, Frame, KeyCode, KeyModifiers, Rect,
};

struct TextInput {
    input: String,
}

impl Component for TextInput {
    type Message = ();

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let text = format!(
            "Type something: {}_\n\n\
            Backspace to delete\n\
            Enter to clear\n\
            Press 'q' to quit",
            self.input
        );

        let widget = Paragraph::new(text).block(
            Block::bordered()
                .title("Text Input")
                .style(Style::default().fg(Color::Green)),
        );

        frame.render_widget(widget, area);
    }

    fn handle_event(&mut self, event: Event, context: &Context<Self::Message>) -> EventResult {
        match event {
            // Bracketed paste is on by default, so pasted text arrives whole.
            Event::Paste(text) => {
                self.input.push_str(&text);
                EventResult::Consumed
            }
            Event::Key(key) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    return EventResult::Propagate;
                }

                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        context.quit();
                        EventResult::Consumed
                    }
                    KeyCode::Char(c) => {
                        self.input.push(c);
                        EventResult::Consumed
                    }
                    KeyCode::Backspace => {
                        self.input.pop();
                        EventResult::Consumed
                    }
                    KeyCode::Enter => {
                        self.input.clear();
                        EventResult::Consumed
                    }
                    _ => EventResult::Propagate,
                }
            }
            _ => EventResult::Propagate,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    App::new(TextInput {
        input: String::new(),
    })?
    .run()
    .await
}
