use crate::component::{Component, Context};
use crate::event::{Event, EventResult};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{Frame, layout::Rect};

/// A simple text input component demonstrating character input handling
pub struct TextInput {
    input: String,
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            input: String::new(),
        }
    }
}

impl Component for TextInput {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let text = format!(
            "Type something: {}_\n\n\
            Backspace to delete\n\
            Enter to clear\n\
            Press 'q' to quit",
            self.input
        );

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Text Input Example")
                    .style(Style::default().fg(Color::Green)),
            )
            .style(Style::default().add_modifier(Modifier::BOLD));

        frame.render_widget(paragraph, area);
    }

    fn handle_event(&mut self, event: Event, context: &Context) -> EventResult {
        match event {
            Event::Paste(text) => {
                self.input.push_str(&text);
                EventResult::Consumed
            }
            Event::Key(key) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    return EventResult::Propagate;
                }

                match key.code {
                    KeyCode::Char(c) if c == 'q' || c == 'Q' => {
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

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}
