//! Character input with a real terminal cursor, backspace, enter, and
//! bracketed paste.
//!
//! Two things to note:
//! - `Event::char` returns the typed character and ignores Ctrl/Alt chords,
//!   so text input never swallows keyboard shortcuts.
//! - `frame.set_cursor_position` places the real terminal cursor where the
//!   next character will go; the runtime shows it on frames that set a
//!   position and hides it otherwise.
//!
//! Run with: `cargo run --example text_input`

use anyhow::Result;
use tui_base_framework::layout::{Constraint, Layout, Position};
use tui_base_framework::style::{Color, Style};
use tui_base_framework::text::Line;
use tui_base_framework::widgets::{Block, Paragraph};
use tui_base_framework::{Component, Context, Event, EventResult, Frame, KeyCode, Rect, run};

struct TextInput {
    input: String,
}

impl Component for TextInput {
    type Message = ();

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let [input_area, help] =
            Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(area);

        frame.render_widget(
            Paragraph::new(self.input.as_str()).block(
                Block::bordered()
                    .title("Text Input")
                    .style(Style::default().fg(Color::Green)),
            ),
            input_area,
        );

        // Park the cursor after the last typed character (measured in display
        // width, not bytes), clamped inside the block's borders.
        let typed = Line::from(self.input.as_str()).width() as u16;
        frame.set_cursor_position(Position::new(
            input_area.x + 1 + typed.min(input_area.width.saturating_sub(3)),
            input_area.y + 1,
        ));

        frame.render_widget(
            Paragraph::new(
                "Type anything (even 'q') and the cursor follows\n\
                Paste arrives as one event\n\
                Backspace to delete, Enter to clear\n\
                Esc to quit",
            ),
            help,
        );
    }

    fn handle_event(&mut self, event: Event, context: &Context<Self::Message>) -> EventResult {
        if event.is_key(KeyCode::Esc) {
            context.quit();
            return EventResult::Consumed;
        }

        // `char` is None for Ctrl/Alt chords, so shortcuts stay shortcuts.
        if let Some(c) = event.char() {
            self.input.push(c);
            return EventResult::Consumed;
        }

        match event {
            // Bracketed paste is on by default, so pasted text arrives whole.
            Event::Paste(text) => {
                self.input.push_str(&text);
                EventResult::Consumed
            }
            Event::Key(key) => match key.code {
                KeyCode::Backspace => {
                    self.input.pop();
                    EventResult::Consumed
                }
                KeyCode::Enter => {
                    self.input.clear();
                    EventResult::Consumed
                }
                _ => EventResult::Propagate,
            },
            _ => EventResult::Propagate,
        }
    }
}

fn main() -> Result<()> {
    run(TextInput {
        input: String::new(),
    })
}
