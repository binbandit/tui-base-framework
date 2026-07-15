//! Multi-screen apps: a router, screens as components, reusable widgets.
//!
//! Three patterns, all plain Rust — the framework has no screen or widget
//! registry to learn:
//!
//! - **Screens are components.** Each screen implements [`Component`] with
//!   the app's message type and owns its own state. The root owns every
//!   screen and points `active` at one; rendering and events go to it.
//! - **Navigation is a message.** Screens announce what happened through the
//!   [`Context`]; the root's `update` moves data between screens and switches
//!   the active one. No screen knows any other screen exists.
//! - **Reusable widgets are plain structs.** `TextField` below never touches
//!   messages, so the same widget drops into any screen — or any app —
//!   regardless of its message type.
//!
//! Run with: `cargo run --example screens`

use anyhow::Result;
use tui_base_framework::layout::{Constraint, Layout, Position, Rect};
use tui_base_framework::style::{Color, Modifier, Style};
use tui_base_framework::text::Line;
use tui_base_framework::widgets::{Block, List, ListState, Paragraph};
use tui_base_framework::{Component, Context, Event, EventResult, Frame, KeyCode, run};

/// Everything that can happen across screens. Navigation is just data.
enum Msg {
    /// Open the editor for the item at this index.
    Edit(usize),
    /// Editor finished: write the result back and return home.
    Save {
        index: usize,
        title: String,
        notes: String,
    },
    /// Editor abandoned: return home unchanged.
    Cancel,
}

// ---------------------------------------------------------------------------
// Reusable widget: a single-line text field.
//
// Not a `Component` — it has no message type, so it works in any screen of
// any app. It follows the component contract in miniature: `handle_event`
// reports whether it consumed the event, `render` draws it.
// ---------------------------------------------------------------------------

struct TextField {
    label: &'static str,
    value: String,
    focused: bool,
}

impl TextField {
    fn new(label: &'static str) -> Self {
        Self {
            label,
            value: String::new(),
            focused: false,
        }
    }

    fn handle_event(&mut self, event: &Event) -> bool {
        if !self.focused {
            return false;
        }

        // `char` ignores Ctrl/Alt chords, so shortcuts pass through.
        if let Some(c) = event.char() {
            self.value.push(c);
            return true;
        }

        match event {
            Event::Paste(text) => {
                self.value.push_str(text);
                true
            }
            Event::Key(key) if key.code == KeyCode::Backspace => {
                self.value.pop();
                true
            }
            _ => false,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let border = if self.focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        frame.render_widget(
            Paragraph::new(self.value.as_str())
                .block(Block::bordered().title(self.label).border_style(border)),
            area,
        );

        // Only the focused field claims the real terminal cursor.
        if self.focused {
            let typed = Line::from(self.value.as_str()).width() as u16;
            frame.set_cursor_position(Position::new(
                area.x + 1 + typed.min(area.width.saturating_sub(3)),
                area.y + 1,
            ));
        }
    }
}

// ---------------------------------------------------------------------------
// Home screen: pick an item to edit.
// ---------------------------------------------------------------------------

struct Item {
    title: String,
    notes: String,
}

struct HomeScreen {
    items: Vec<Item>,
    state: ListState,
}

impl Component for HomeScreen {
    type Message = Msg;

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let [list_area, footer] = split_screen(area);

        let rows = self
            .items
            .iter()
            .map(|item| format!("{} — {}", item.title, item.notes));

        frame.render_stateful_widget(
            List::new(rows)
                .block(Block::bordered().title("Items"))
                .highlight_style(
                    Style::default()
                        .bg(Color::Cyan)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("► "),
            list_area,
            &mut self.state,
        );

        frame.render_widget(status_bar("↑/↓ select | Enter edit | q quit"), footer);
    }

    fn handle_event(&mut self, event: Event, context: &Context<Msg>) -> EventResult {
        if event.is_key(KeyCode::Up) {
            self.state.select_previous();
            return EventResult::Consumed;
        }

        if event.is_key(KeyCode::Down) {
            self.state.select_next();
            return EventResult::Consumed;
        }

        if event.is_key(KeyCode::Enter) {
            if let Some(index) = self.state.selected() {
                let _ = context.try_send(Msg::Edit(index));
            }
            return EventResult::Consumed;
        }

        if event.is_key(KeyCode::Char('q')) || event.is_key(KeyCode::Esc) {
            context.quit();
            return EventResult::Consumed;
        }

        EventResult::Propagate
    }
}

// ---------------------------------------------------------------------------
// Editor screen: two reusable text fields sharing focus.
// ---------------------------------------------------------------------------

struct EditorScreen {
    index: usize,
    title: TextField,
    notes: TextField,
}

impl EditorScreen {
    fn load(&mut self, index: usize, item: &Item) {
        self.index = index;
        self.title.value = item.title.clone();
        self.notes.value = item.notes.clone();
        self.title.focused = true;
        self.notes.focused = false;
    }

    fn toggle_focus(&mut self) {
        self.title.focused = !self.title.focused;
        self.notes.focused = !self.notes.focused;
    }
}

impl Component for EditorScreen {
    type Message = Msg;

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let [body, footer] = split_screen(area);
        let [title_area, notes_area, _] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .areas(body);

        self.title.render(frame, title_area);
        self.notes.render(frame, notes_area);

        frame.render_widget(
            status_bar("Tab switch field | Enter save | Esc cancel"),
            footer,
        );
    }

    fn handle_event(&mut self, event: Event, context: &Context<Msg>) -> EventResult {
        // The focused widget gets first refusal, exactly like child
        // components do in `examples/focus.rs`.
        if self.title.handle_event(&event) || self.notes.handle_event(&event) {
            return EventResult::Consumed;
        }

        if event.is_key(KeyCode::Tab) {
            self.toggle_focus();
            return EventResult::Consumed;
        }

        if event.is_key(KeyCode::Enter) {
            let _ = context.try_send(Msg::Save {
                index: self.index,
                title: self.title.value.clone(),
                notes: self.notes.value.clone(),
            });
            return EventResult::Consumed;
        }

        if event.is_key(KeyCode::Esc) {
            let _ = context.try_send(Msg::Cancel);
            return EventResult::Consumed;
        }

        EventResult::Propagate
    }
}

// ---------------------------------------------------------------------------
// Root: owns every screen, points at the active one, handles navigation.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum ScreenId {
    Home,
    Editor,
}

struct ScreensDemo {
    active: ScreenId,
    home: HomeScreen,
    editor: EditorScreen,
}

impl ScreensDemo {
    /// The whole router: every screen is interchangeable behind
    /// `dyn Component`. A `Vec` of boxed screens works the same way if you
    /// need a navigation stack instead of a fixed set.
    fn active_mut(&mut self) -> &mut dyn Component<Message = Msg> {
        match self.active {
            ScreenId::Home => &mut self.home,
            ScreenId::Editor => &mut self.editor,
        }
    }
}

impl Component for ScreensDemo {
    type Message = Msg;

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.active_mut().render(frame, area);
    }

    fn handle_event(&mut self, event: Event, context: &Context<Msg>) -> EventResult {
        self.active_mut().handle_event(event, context)
    }

    // Navigation lives here: screens announce what happened, the root moves
    // data between them and switches the active screen. Messages a screen
    // should handle itself can be forwarded to `self.active_mut().update(..)`.
    fn update(&mut self, message: Msg, _context: &Context<Msg>) {
        match message {
            Msg::Edit(index) => {
                if let Some(item) = self.home.items.get(index) {
                    self.editor.load(index, item);
                    self.active = ScreenId::Editor;
                }
            }
            Msg::Save {
                index,
                title,
                notes,
            } => {
                if let Some(item) = self.home.items.get_mut(index) {
                    item.title = title;
                    item.notes = notes;
                }
                self.active = ScreenId::Home;
            }
            Msg::Cancel => self.active = ScreenId::Home,
        }
    }
}

// Small shared helpers — reuse doesn't have to be a struct.

fn split_screen(area: Rect) -> [Rect; 2] {
    Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(area)
}

fn status_bar(hint: &str) -> Paragraph<'_> {
    Paragraph::new(format!(" {hint}")).style(Style::default().fg(Color::DarkGray))
}

fn main() -> Result<()> {
    let items = vec![
        Item {
            title: "Groceries".into(),
            notes: "milk, eggs, coffee".into(),
        },
        Item {
            title: "Errands".into(),
            notes: "post office before 5pm".into(),
        },
        Item {
            title: "Project".into(),
            notes: "write the README".into(),
        },
    ];

    run(ScreensDemo {
        active: ScreenId::Home,
        home: HomeScreen {
            items,
            state: ListState::default().with_selected(Some(0)),
        },
        editor: EditorScreen {
            index: 0,
            title: TextField::new("Title"),
            notes: TextField::new("Notes"),
        },
    })
}
