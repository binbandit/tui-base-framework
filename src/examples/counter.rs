use crate::component::{Component, Context};
use crate::event::{Event, EventResult};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Alignment;
use ratatui::widgets::Paragraph;
use ratatui::{Frame, layout::Rect};

pub struct Counter {
    count: i32,
}

impl Counter {
    pub fn new() -> Self {
        Self { count: 0 }
    }
}

impl Component for Counter {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let text = format!("Count: {} (Press ↑/↓, q to quit)", self.count);
        let paragraph = Paragraph::new(text).alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
    }

    fn handle_event(&mut self, event: Event, context: &Context) -> EventResult {
        match event {
            Event::Key(key) => {
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
            _ => EventResult::Propagate,
        }
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}
