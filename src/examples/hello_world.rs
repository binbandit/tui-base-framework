use crate::component::{Component, Context};
use crate::event::{Event, EventResult};
use crossterm::event::KeyCode;
use ratatui::layout::Alignment;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{Frame, layout::Rect};

/// The simplest possible TUI app - just displays text and quits on 'q'
pub struct HelloWorld;

impl HelloWorld {
    pub fn new() -> Self {
        Self
    }
}

impl Component for HelloWorld {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let text = "Hello, TUI World!\n\nPress 'q' to quit";
        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Hello World Example")
                    .style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
    }

    fn handle_event(&mut self, event: Event, context: &Context) -> EventResult {
        if let Event::Key(key) = event {
            if let KeyCode::Char('q') | KeyCode::Char('Q') = key.code {
                context.quit();
                return EventResult::Consumed;
            }
        }
        EventResult::Propagate
    }
}

impl Default for HelloWorld {
    fn default() -> Self {
        Self::new()
    }
}
