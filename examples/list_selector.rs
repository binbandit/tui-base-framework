//! List navigation with Ratatui's stateful `List` widget.
//!
//! `Component::render` takes `&mut self`, so widget state like `ListState`
//! lives directly in your component — no interior mutability needed.
//!
//! Run with: `cargo run --example list_selector`

use anyhow::Result;
use tui_base_framework::style::{Color, Modifier, Style};
use tui_base_framework::widgets::{Block, List, ListItem, ListState};
use tui_base_framework::{App, Component, Context, Event, EventResult, Frame, KeyCode, Rect};

struct ListSelector {
    items: Vec<String>,
    state: ListState,
}

impl ListSelector {
    fn new<I, S>(items: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            items: items.into_iter().map(Into::into).collect(),
            state: ListState::default().with_selected(Some(0)),
        }
    }
}

impl Component for ListSelector {
    type Message = ();

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(String::as_str)
            .map(ListItem::new)
            .collect();

        let list = List::new(items)
            .block(Block::bordered().title("List Selector (↑/↓ to navigate, q to quit)"))
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("► ");

        frame.render_stateful_widget(list, area, &mut self.state);
    }

    fn handle_event(&mut self, event: Event, context: &Context<Self::Message>) -> EventResult {
        let Event::Key(key) = event else {
            return EventResult::Propagate;
        };

        match key.code {
            KeyCode::Up => {
                self.state.select_previous();
                EventResult::Consumed
            }
            KeyCode::Down => {
                self.state.select_next();
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
    let selector = ListSelector::new([
        "Rust",
        "Python",
        "JavaScript",
        "Go",
        "TypeScript",
        "C++",
        "Java",
    ]);

    App::new(selector)?.run().await
}
