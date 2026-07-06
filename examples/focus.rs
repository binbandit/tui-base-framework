//! Composing multiple components with focus routing — the pattern for any
//! app bigger than one screen.
//!
//! Children are ordinary [`Component`] structs owned by a parent. The parent
//! routes events to the focused child first; whatever the child propagates,
//! the parent can handle itself (Tab to move focus, q to quit). Nothing in
//! the framework needs to know about focus — it's plain composition.
//!
//! Run with: `cargo run --example focus`

use anyhow::Result;
use tui_base_framework::layout::{Constraint, Layout};
use tui_base_framework::style::{Color, Modifier, Style};
use tui_base_framework::widgets::{Block, List, ListItem, ListState, Paragraph, Wrap};
use tui_base_framework::{Component, Context, Event, EventResult, Frame, KeyCode, Rect, run};

const TOPICS: [(&str, &str); 4] = [
    (
        "Components",
        "A component is a plain struct that owns state, renders widgets, and \
         handles events. Children are just fields on their parent.",
    ),
    (
        "Focus",
        "The parent tracks which child is focused and offers events to that \
         child first. A child returns Propagate for keys it doesn't handle, \
         letting the parent act on them instead.",
    ),
    (
        "Messages",
        "All components in an app share the app's message type, so a parent \
         can pass its Context straight to children. Use messages for anything \
         async; plain method calls are fine for parent-child sync.",
    ),
    (
        "Growing",
        "Start with one component. When a piece of state and its keys feel \
         separate, move them into a child struct. The trait stays the same at \
         every level.",
    ),
];

fn pane_block(title: &str, focused: bool) -> Block<'_> {
    let block = Block::bordered().title(title);
    if focused {
        block.border_style(Style::default().fg(Color::Yellow))
    } else {
        block
    }
}

/// Left pane: picks a topic. Handles up/down, propagates everything else.
struct Sidebar {
    state: ListState,
    focused: bool,
}

impl Sidebar {
    fn selected(&self) -> usize {
        self.state.selected().unwrap_or(0)
    }
}

impl Component for Sidebar {
    type Message = ();

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = TOPICS
            .iter()
            .map(|(title, _)| ListItem::new(*title))
            .collect();

        let list = List::new(items)
            .block(pane_block("Topics", self.focused))
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("► ");

        frame.render_stateful_widget(list, area, &mut self.state);
    }

    fn handle_event(&mut self, event: Event, _context: &Context<Self::Message>) -> EventResult {
        if event.is_key(KeyCode::Up) {
            self.state.select_previous();
            EventResult::Consumed
        } else if event.is_key(KeyCode::Down) {
            self.state.select_next();
            EventResult::Consumed
        } else {
            EventResult::Propagate
        }
    }
}

/// Right pane: shows the selected topic. Handles its own scrolling.
struct Content {
    topic: usize,
    scroll: u16,
    focused: bool,
}

impl Content {
    fn show(&mut self, topic: usize) {
        if self.topic != topic {
            self.topic = topic;
            self.scroll = 0;
        }
    }
}

impl Component for Content {
    type Message = ();

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let (title, body) = TOPICS[self.topic];

        let paragraph = Paragraph::new(body)
            .block(pane_block(title, self.focused))
            .wrap(Wrap { trim: true })
            .scroll((self.scroll, 0));

        frame.render_widget(paragraph, area);
    }

    fn handle_event(&mut self, event: Event, _context: &Context<Self::Message>) -> EventResult {
        if event.is_key(KeyCode::Up) {
            self.scroll = self.scroll.saturating_sub(1);
            EventResult::Consumed
        } else if event.is_key(KeyCode::Down) {
            self.scroll = self.scroll.saturating_add(1);
            EventResult::Consumed
        } else {
            EventResult::Propagate
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Focus {
    Sidebar,
    Content,
}

/// The parent: owns both children, routes events, and syncs shared state.
struct FocusDemo {
    sidebar: Sidebar,
    content: Content,
    focus: Focus,
}

impl Component for FocusDemo {
    type Message = ();

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let [body, footer] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(area);
        let [left, right] =
            Layout::horizontal([Constraint::Length(24), Constraint::Min(0)]).areas(body);

        // Sync children with parent-owned state before drawing them.
        self.sidebar.focused = self.focus == Focus::Sidebar;
        self.content.focused = self.focus == Focus::Content;
        self.content.show(self.sidebar.selected());

        self.sidebar.render(frame, left);
        self.content.render(frame, right);

        frame.render_widget(
            Paragraph::new(" Tab: switch pane | ↑/↓: navigate or scroll | q: quit")
                .style(Style::default().fg(Color::DarkGray)),
            footer,
        );
    }

    fn handle_event(&mut self, event: Event, context: &Context<Self::Message>) -> EventResult {
        // The focused child gets first refusal...
        let child_result = match self.focus {
            Focus::Sidebar => self.sidebar.handle_event(event.clone(), context),
            Focus::Content => self.content.handle_event(event.clone(), context),
        };
        if child_result.is_consumed() {
            return EventResult::Consumed;
        }

        // ...and the parent handles whatever propagated back up.
        if event.is_key(KeyCode::Tab) {
            self.focus = match self.focus {
                Focus::Sidebar => Focus::Content,
                Focus::Content => Focus::Sidebar,
            };
            return EventResult::Consumed;
        }

        if event.is_key(KeyCode::Char('q')) || event.is_key(KeyCode::Esc) {
            context.quit();
            return EventResult::Consumed;
        }

        EventResult::Propagate
    }
}

fn main() -> Result<()> {
    run(FocusDemo {
        sidebar: Sidebar {
            state: ListState::default().with_selected(Some(0)),
            focused: true,
        },
        content: Content {
            topic: 0,
            scroll: 0,
            focused: false,
        },
        focus: Focus::Sidebar,
    })
}
