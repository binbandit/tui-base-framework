# TUI Framework Cheat Sheet

## Imports

```rust
use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use tui_base_framework::{
    App, AppConfig, Component, Context, Event, EventResult, Frame, Message, Rect,
    TerminalConfig,
};
use tui_base_framework::layout::{Alignment, Constraint, Direction, Layout};
use tui_base_framework::style::{Color, Modifier, Style};
use tui_base_framework::widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Tabs};
```

## Component

```rust
struct MyComponent {
    value: i32,
}

impl Component for MyComponent {
    fn init(&mut self, _context: &Context) {}

    fn render(&self, frame: &mut Frame, area: Rect) {
        let widget = Paragraph::new(format!("Value: {}", self.value))
            .block(Block::default().borders(Borders::ALL));

        frame.render_widget(widget, area);
    }

    fn handle_event(&mut self, event: Event, context: &Context) -> EventResult {
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

    fn update(&mut self, _message: Message, _context: &Context) {}
}
```

## Main

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let mut app = App::new(MyComponent { value: 0 })?;
    app.run().await
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

Send:

```rust
let _ = context.try_send(Message::custom(AppMessage::Saved));
```

Receive:

```rust
fn update(&mut self, message: Message, _context: &Context) {
    if let Ok(message) = message.downcast::<AppMessage>() {
        match *message {
            AppMessage::Saved => {}
        }
    }
}
```

From a background task:

```rust
fn init(&mut self, context: &Context) {
    let sender = context.message_sender();

    tokio::spawn(async move {
        let _ = sender.send(Message::custom(AppMessage::Loaded)).await;
    });
}
```

## Layouts

### Header / Body / Footer

```rust
let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .split(area);
```

### Sidebar / Main

```rust
let chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Length(30), Constraint::Min(0)])
    .split(area);
```

## Widgets

### Paragraph

```rust
let widget = Paragraph::new("Hello")
    .block(Block::default().borders(Borders::ALL).title("Title"))
    .alignment(Alignment::Center)
    .style(Style::default().fg(Color::Cyan));
```

### List

```rust
let items = self.items.iter().map(|item| ListItem::new(item.as_str()));

let widget = List::new(items)
    .block(Block::default().borders(Borders::ALL).title("Items"))
    .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black));
```

### Gauge

```rust
let widget = Gauge::default()
    .block(Block::default().borders(Borders::ALL).title("Progress"))
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
- Use `Context::message_sender()` for background work instead of blocking in `handle_event`.
- Keep terminal input polling modest. The default is 50ms.

## Debugging

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

If the terminal is left in a bad state after a hard crash:

```bash
reset
```
