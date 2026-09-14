//! List navigation with Ratatui's stateful `List` widget.
//!
//! `Component::render` takes `&mut self`, so widget state like `ListState`
//! lives directly in your component — no interior mutability needed.
//!
//! Run with: `cargo run --example list_selector`

use anyhow::Result;
use tui_base_framework::style::{Color, Modifier, Style};
use tui_base_framework::widgets::{Block, List, ListItem, ListState};
use tui_base_framework::{Component, Context, Event, EventResult, Frame, KeyCode, Rect, run};

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
        let items: Vec<String> = items.into_iter().map(Into::into).collect();
        let selected = (!items.is_empty()).then_some(0);
        Self {
            items,
            state: ListState::default().with_selected(selected),
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
                self.state
                    .select(self.state.selected().map(|index| index.saturating_sub(1)));
                EventResult::Consumed
            }
            KeyCode::Down => {
                self.state
                    .select(self.items.len().checked_sub(1).map(|last| {
                        self.state
                            .selected()
                            .unwrap_or(0)
                            .saturating_add(1)
                            .min(last)
                    }));
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

fn main() -> Result<()> {
    run(ListSelector::new([
        "Rust",
        "Python",
        "JavaScript",
        "Go",
        "TypeScript",
        "C++",
        "Java",
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigation_handles_empty_lists_and_batched_input() {
        let (context, _messages) = Context::test();
        let mut empty = ListSelector::new(std::iter::empty::<String>());
        let mut list = ListSelector::new(["one", "two"]);
        for _ in 0..100 {
            empty.handle_event(Event::key_press(KeyCode::Down), &context);
            list.handle_event(Event::key_press(KeyCode::Down), &context);
        }
        assert_eq!(empty.state.selected(), None);
        assert_eq!(list.state.selected(), Some(1));
        for _ in 0..100 {
            list.handle_event(Event::key_press(KeyCode::Up), &context);
        }
        assert_eq!(list.state.selected(), Some(0));
    }
}
