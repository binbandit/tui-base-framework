//! View switching with Ratatui tabs.
//!
//! Run with: `cargo run --example tabs`

use anyhow::Result;
use tui_base_framework::layout::{Constraint, Layout};
use tui_base_framework::style::{Color, Modifier, Style};
use tui_base_framework::widgets::{Block, Paragraph, Tabs};
use tui_base_framework::{App, Component, Context, Event, EventResult, Frame, KeyCode, Rect};

const TABS: [&str; 3] = ["Home", "Settings", "About"];

const CONTENT: [&str; 3] = [
    "Welcome to the Home tab!\n\nThis is where your main content would go.\n\nUse ← → or Tab to switch tabs.",
    "Settings Tab\n\nConfigure your application here.\n\n• Option 1: Enabled\n• Option 2: Disabled\n• Option 3: Auto",
    "About Tab\n\nTUI Base Framework\n\nA minimal template for building\nterminal user interfaces.",
];

struct TabsDemo {
    selected: usize,
}

impl Component for TabsDemo {
    type Message = ();

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let [tab_bar, body, footer] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .areas(area);

        let tabs = Tabs::new(TABS)
            .block(Block::bordered().title("Tabs Demo"))
            .select(self.selected)
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
        frame.render_widget(tabs, tab_bar);

        frame.render_widget(
            Paragraph::new(CONTENT[self.selected]).block(Block::bordered()),
            body,
        );

        frame.render_widget(
            Paragraph::new("← → or Tab to switch | q to quit")
                .block(Block::bordered())
                .style(Style::default().fg(Color::Cyan)),
            footer,
        );
    }

    fn handle_event(&mut self, event: Event, context: &Context<Self::Message>) -> EventResult {
        let Event::Key(key) = event else {
            return EventResult::Propagate;
        };

        match key.code {
            KeyCode::Left => {
                self.selected = self.selected.saturating_sub(1);
                EventResult::Consumed
            }
            KeyCode::Right | KeyCode::Tab => {
                self.selected = (self.selected + 1).min(TABS.len() - 1);
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
    App::new(TabsDemo { selected: 0 })?.run().await
}
