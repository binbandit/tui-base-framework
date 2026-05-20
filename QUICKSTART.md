# Quick Start

Get a TUI running in a few minutes.

## Run the App

```bash
cargo run
```

The default binary runs the `hello_world` component. Replace `src/main.rs` when you are ready to make the project yours.

## Run an Example

```bash
cargo run --example counter
```

Available examples:

```bash
cargo run --example hello_world
cargo run --example counter
cargo run --example text_input
cargo run --example list_selector
cargo run --example layout_demo
cargo run --example tabs
cargo run --example progress
```

## Start Your App

```bash
cp examples/counter.rs src/main.rs
cargo run
```

Then update `Cargo.toml` with your package name, authors, description, repository, and license.

## Dependencies

The template uses current, slim defaults:

```toml
[dependencies]
ratatui = { version = "0.30", default-features = false, features = ["crossterm", "layout-cache", "underline-color"] }
crossterm = "0.29"
tokio = { version = "1.52", default-features = false, features = ["macros", "rt-multi-thread", "sync", "time"] }
anyhow = "1.0"
```

Release builds are tuned with thin LTO, one codegen unit, and stripped symbols. `Cargo.lock` is tracked so copied projects start from reproducible dependency versions.

## Component Template

```rust
use crossterm::event::KeyCode;
use tui_base_framework::{
    Component, Context, Event, EventResult, Frame, Message, Rect,
};
use tui_base_framework::widgets::{Block, Borders, Paragraph};

struct MyApp {
    count: i32,
}

impl MyApp {
    fn new() -> Self {
        Self { count: 0 }
    }
}

impl Component for MyApp {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let widget = Paragraph::new(format!("Count: {}", self.count))
            .block(Block::default().borders(Borders::ALL).title("My App"));

        frame.render_widget(widget, area);
    }

    fn handle_event(&mut self, event: Event, context: &Context) -> EventResult {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Up => {
                    self.count += 1;
                    EventResult::Consumed
                }
                KeyCode::Down => {
                    self.count -= 1;
                    EventResult::Consumed
                }
                KeyCode::Char('q') => {
                    context.quit();
                    EventResult::Consumed
                }
                _ => EventResult::Propagate,
            },
            _ => EventResult::Propagate,
        }
    }

    fn update(&mut self, message: Message, _context: &Context) {
        if let Ok(count) = message.downcast::<i32>() {
            self.count = *count;
        }
    }
}
```

## Main Function

```rust
use anyhow::Result;
use tui_base_framework::App;

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = App::new(MyApp::new())?;
    app.run().await
}
```

`App::new` is generic, so components are not boxed by default. If you need dynamic dispatch, `Box<dyn Component>` still works.

## Common Tasks

### Quit

```rust
KeyCode::Char('q') | KeyCode::Esc => {
    context.quit();
    EventResult::Consumed
}
```

Ctrl-C also quits by default through `AppConfig::quit_on_ctrl_c`.

### Send a Custom Message

```rust
let _ = context.try_send(Message::custom(AppMessage::Saved));
```

Handle it in `update`:

```rust
fn update(&mut self, message: Message, _context: &Context) {
    if let Ok(message) = message.downcast::<AppMessage>() {
        // use *message
    }
}
```

### Background Task

```rust
fn init(&mut self, context: &Context) {
    let sender = context.message_sender();

    tokio::spawn(async move {
        // do work...
        let _ = sender.send(Message::custom(AppMessage::Loaded)).await;
    });
}
```

### Configure the Runtime

```rust
use std::time::Duration;
use tui_base_framework::{App, AppConfig, TerminalConfig};

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

let mut app = App::with_config(MyApp::new(), config)?;
```

### Layout

```rust
use tui_base_framework::layout::{Constraint, Direction, Layout};

let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .split(area);
```

### Text Input and Paste

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
        _ => EventResult::Propagate,
    },
    _ => EventResult::Propagate,
}
```

## Clean Up

When you are ready to turn the template into your app:

```bash
rm -rf examples/ src/examples/ src/examples.rs
```

Then remove `pub mod examples;` from `src/lib.rs`.
