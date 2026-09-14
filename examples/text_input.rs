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
use tui_base_framework::text::{Line, Span};
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

        let block = Block::bordered()
            .title("Text Input")
            .style(Style::default().fg(Color::Green));
        let inner = block.inner(input_area);
        let (visible, column) = visible_input(&self.input, inner.width);
        frame.render_widget(Paragraph::new(visible).block(block), input_area);

        // Keep the insertion point inside the field, including on tiny terminals.
        if !inner.is_empty() {
            frame.set_cursor_position(Position::new(inner.x + column, inner.y));
        }

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
                self.input.extend(text.chars().filter(|c| !c.is_control()));
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

// Render a suffix rather than a u16 scroll offset: pastes can be much wider
// than 65,535 columns. Keep graphemes intact and reserve a cell for the cursor.
fn visible_input(value: &str, width: u16) -> (&str, u16) {
    let line = Line::from(value);
    let mut remaining: usize = line
        .styled_graphemes(Style::default())
        .map(|grapheme| Span::raw(grapheme.symbol).width())
        .sum();
    let available = usize::from(width.saturating_sub(1));
    let mut start = 0;
    for grapheme in line.styled_graphemes(Style::default()) {
        if remaining <= available {
            break;
        }
        remaining -= Span::raw(grapheme.symbol).width();
        start += grapheme.symbol.len();
    }
    let visible = &value[start..];
    (visible, Line::from(visible).width().min(available) as u16)
}

fn main() -> Result<()> {
    run(TextInput {
        input: String::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tui_base_framework::{Terminal, backend::TestBackend};

    #[test]
    fn long_input_keeps_the_end_and_graphemes_visible() {
        let value = format!("{}界e\u{301}END", "a".repeat(70_000));
        let (visible, cursor) = visible_input(&value, 8);
        assert_eq!(visible, "a界e\u{301}END");
        assert_eq!(cursor, 7);
    }

    #[test]
    fn paste_stays_on_one_line_and_preserves_unicode() {
        let (context, _messages) = Context::test();
        let mut input = TextInput {
            input: String::new(),
        };
        input.handle_event(Event::Paste("hello\r\n界\t!".into()), &context);
        assert_eq!(input.input, "hello界!");
        assert_eq!(
            input.handle_event(
                Event::Key(tui_base_framework::KeyEvent::new(
                    KeyCode::Char('c'),
                    tui_base_framework::KeyModifiers::CONTROL,
                )),
                &context
            ),
            EventResult::Propagate
        );
    }

    #[test]
    fn cursor_stays_in_visible_field_and_long_input_scrolls() {
        let mut input = TextInput {
            input: "abcdefghijk".into(),
        };
        for (width, height) in [(0, 0), (1, 1), (2, 2), (8, 3)] {
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            terminal
                .draw(|frame| input.render(frame, frame.area()))
                .unwrap();
            let backend = terminal.backend();
            if backend.cursor_visible() {
                let Position { x, y } = backend.cursor_position();
                assert!(x < width && y < height);
            } else {
                assert!(width < 3 || height < 3);
            }
            if width == 8 {
                let screen: String = terminal
                    .backend()
                    .buffer()
                    .content()
                    .iter()
                    .map(|cell| cell.symbol())
                    .collect();
                assert!(screen.contains("ghijk"));
            }
        }
    }
}
