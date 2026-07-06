# TUI Framework Cheat Sheet

Copy-paste reference for common tasks. Everything here compiles against the template as-is.

## Imports

Input types (`KeyCode`, `KeyModifiers`, ...) and Ratatui modules (`layout`, `style`, `text`, `widgets`) are re-exported, so one crate covers most needs:

```rust
use anyhow::Result;
use tui_base_framework::{
    App, AppConfig, Component, Context, Event, EventResult, Frame, KeyCode, KeyModifiers,
    MouseButton, MouseEventKind, Rect, TerminalConfig,
};
use tui_base_framework::layout::{Alignment, Constraint, Layout};
use tui_base_framework::style::{Color, Modifier, Style};
use tui_base_framework::widgets::{Block, Gauge, List, ListItem, ListState, Paragraph, Tabs};
```

## Component

```rust
struct MyComponent {
    value: i32,
}

impl Component for MyComponent {
    type Message = (); // or your message enum

    fn init(&mut self, _context: &Context<Self::Message>) {}

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let widget = Paragraph::new(format!("Value: {}", self.value))
            .block(Block::bordered());

        frame.render_widget(widget, area);
    }

    fn handle_event(&mut self, event: Event, context: &Context<Self::Message>) -> EventResult {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => {
                    context.quit();
                    EventResult::Consumed
                }
                _ => EventResult::Propagate,
            },
            _ => EventResult::Propagate,
        }
    }

    fn update(&mut self, _message: Self::Message, _context: &Context<Self::Message>) {}
}
```

## Main

```rust
#[tokio::main]
async fn main() -> Result<()> {
    App::new(MyComponent { value: 0 })?.run().await
}
```

## Configure App

```rust
use std::time::Duration;

let config = AppConfig {
    tick_rate: Duration::from_millis(100),
    input_poll_rate: Duration::from_millis(25),
    channel_capacity: 512,
    quit_on_ctrl_c: true,
    terminal: TerminalConfig {
        mouse_capture: true,
        bracketed_paste: true,
        focus_change: false,
    },
};

let mut app = App::with_config(component, config)?;
```

## Events

```rust
match event {
    Event::FocusGained => {}
    Event::FocusLost => {}
    Event::Key(key) => {}
    Event::Mouse(mouse) => {}
    Event::Paste(text) => {}
    Event::Resize(width, height) => {}
    Event::Tick => {}
}
```

Return `EventResult::Consumed` when state changed. Return `EventResult::Propagate` for ignored events.

## Keyboard

```rust
match key.code {
    KeyCode::Char('q') | KeyCode::Esc => {
        context.quit();
        EventResult::Consumed
    }
    KeyCode::Up => {
        self.selected = self.selected.saturating_sub(1);
        EventResult::Consumed
    }
    KeyCode::Down => {
        if let Some(last) = self.items.len().checked_sub(1) {
            self.selected = (self.selected + 1).min(last);
        }
        EventResult::Consumed
    }
    KeyCode::Enter => EventResult::Consumed,
    _ => EventResult::Propagate,
}
```

## Modifiers

```rust
if key.modifiers.contains(KeyModifiers::CONTROL) {
    // Ctrl is pressed
}

if key.modifiers.contains(KeyModifiers::ALT) {
    // Alt is pressed
}

if key.modifiers.contains(KeyModifiers::SHIFT) {
    // Shift is pressed
}
```

## Mouse

Enable mouse capture first:

```rust
let config = AppConfig {
    terminal: TerminalConfig {
        mouse_capture: true,
        ..TerminalConfig::default()
    },
    ..AppConfig::default()
};
```

Handle mouse events:

```rust
if let Event::Mouse(mouse) = event {
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // mouse.column, mouse.row
            EventResult::Consumed
        }
        MouseEventKind::ScrollUp => EventResult::Consumed,
        MouseEventKind::ScrollDown => EventResult::Consumed,
        _ => EventResult::Propagate,
    }
} else {
    EventResult::Propagate
}
```

## Messages

Declare a message enum and set it as your component's `Message` type:

```rust
enum AppMessage {
    Saved,
    Loaded(Vec<String>),
}

impl Component for MyApp {
    type Message = AppMessage;
    // ...
}
```

Send:

```rust
let _ = context.try_send(AppMessage::Saved);
```

Receive — messages arrive fully typed, no downcasting:

```rust
fn update(&mut self, message: AppMessage, _context: &Context<AppMessage>) {
    match message {
        AppMessage::Saved => {}
        AppMessage::Loaded(items) => self.items = items,
    }
}
```

From a background task (see `examples/async_task.rs` for a full program):

```rust
fn init(&mut self, context: &Context<AppMessage>) {
    let sender = context.sender();

    tokio::spawn(async move {
        let items = load_items().await;
        let _ = sender.send(AppMessage::Loaded(items)).await;
    });
}
```

## Layouts

### Header / Body / Footer

```rust
let [header, body, footer] = Layout::vertical([
    Constraint::Length(3),
    Constraint::Min(0),
    Constraint::Length(3),
])
.areas(area);
```

### Sidebar / Main

```rust
let [sidebar, main] =
    Layout::horizontal([Constraint::Length(30), Constraint::Min(0)]).areas(area);
```

## Widgets

### Paragraph

```rust
let widget = Paragraph::new("Hello")
    .block(Block::bordered().title("Title"))
    .alignment(Alignment::Center)
    .style(Style::default().fg(Color::Cyan));
```

### List (stateful)

`render` takes `&mut self`, so keep a `ListState` in your component:

```rust
// In your struct: state: ListState
let items = self.items.iter().map(|item| ListItem::new(item.as_str()));

let widget = List::new(items)
    .block(Block::bordered().title("Items"))
    .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black))
    .highlight_symbol("► ");

frame.render_stateful_widget(widget, area, &mut self.state);
```

Navigate with `self.state.select_next()` / `self.state.select_previous()`.

### Gauge

```rust
let widget = Gauge::default()
    .block(Block::bordered().title("Progress"))
    .percent(self.progress)
    .gauge_style(Style::default().fg(Color::Green));
```

### Tabs

```rust
let widget = Tabs::new(["Home", "Settings", "About"])
    .select(self.selected_tab)
    .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
```

## Text Input

```rust
match event {
    Event::Paste(text) => {
        self.input.push_str(&text);
        EventResult::Consumed
    }
    Event::Key(key) => match key.code {
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
    },
    _ => EventResult::Propagate,
}
```

## Performance

- Keep `render` deterministic and cheap.
- Precompute expensive strings/data in `update` or event handlers.
- Return `Consumed` only when state changed or the UI should redraw.
- Tune `tick_rate` for animation. Slower ticks mean less redraw pressure.
- Use `Context::sender()` and a spawned task for background work instead of blocking in `handle_event`.
- Keep terminal input polling modest. The default is 50ms.

## Debugging

Panics are safe: a panic hook restores the terminal before the message prints, so you get a readable backtrace (`RUST_BACKTRACE=1 cargo run`).

The terminal is captured while the app runs, so write debug output to a file:

```rust
use std::fs::OpenOptions;
use std::io::Write;

let mut file = OpenOptions::new()
    .create(true)
    .append(true)
    .open("debug.log")?;

writeln!(file, "state = {:?}", self.state)?;
```

If the terminal is left in a bad state after a hard kill:

```bash
reset
```
