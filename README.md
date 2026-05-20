# TUI Base Framework

A fast, small Rust template for building terminal user interfaces with Ratatui, Crossterm, and Tokio.

This repository is meant to be copied, trimmed, and shipped. It gives you a working app loop, terminal cleanup, typed events, message passing, and a set of runnable examples without forcing a large architecture on your project.

## Why This Template?

Most TUI starters either leave you wiring every terminal detail yourself or introduce a framework that is bigger than the app you wanted to build. This template keeps the interface small:

- Implement `Component`
- Render Ratatui widgets
- Handle input events
- Send messages through `Context`
- Let `App` manage terminal lifecycle, ticks, and redraws

## Performance Defaults

The runtime is built to be efficient by default:

- Event-driven rendering: the app redraws after handled events/messages instead of repainting every frame.
- Blocking terminal input is isolated in a blocking task, so it does not park Tokio worker threads.
- Tokio uses only the features this template needs: macros, runtime, sync, and time.
- Ratatui 0.30 is used with the Crossterm backend and layout cache, without unrelated optional widgets/macros.
- The app is generic over your component type, avoiding a heap allocation and dynamic dispatch unless you choose to box a component yourself.
- Terminal cleanup is RAII-based and restores cursor, raw mode, alternate screen, paste, focus, and mouse state on drop.

## Quick Start

```bash
git clone https://github.com/binbandit/tui-base-framework my-tui-app
cd my-tui-app
cargo run --example counter
```

Start from an example:

```bash
cp examples/counter.rs src/main.rs
cargo run
```

Run every example:

```bash
cargo run --example hello_world
cargo run --example counter
cargo run --example text_input
cargo run --example list_selector
cargo run --example layout_demo
cargo run --example tabs
cargo run --example progress
```

## Minimal App

```rust
use anyhow::Result;
use crossterm::event::KeyCode;
use tui_base_framework::{
    App, Component, Context, Event, EventResult, Frame, Message, Rect,
};
use tui_base_framework::widgets::{Block, Borders, Paragraph};

struct Counter {
    count: i32,
}

impl Component for Counter {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let widget = Paragraph::new(format!("Count: {} | q to quit", self.count))
            .block(Block::default().borders(Borders::ALL).title("Counter"));

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
        if let Ok(value) = message.downcast::<i32>() {
            self.count = *value;
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = App::new(Counter { count: 0 })?;
    app.run().await
}
```

## Core Concepts

### Components

A component owns state, renders UI, and reacts to events/messages.

```rust
pub trait Component: Send {
    fn init(&mut self, _context: &Context) {}
    fn render(&self, frame: &mut Frame, area: Rect);
    fn handle_event(&mut self, _event: Event, _context: &Context) -> EventResult {
        EventResult::Propagate
    }
    fn update(&mut self, _message: Message, _context: &Context) {}
}
```

Return `EventResult::Consumed` when an event changed state and should trigger a redraw. Return `EventResult::Propagate` when you ignored it.

### Context

`Context` is how components talk back to the app loop:

```rust
context.quit();
let _ = context.try_send(Message::custom(MyMessage::Saved));
let sender = context.message_sender();
```

Use `message_sender()` when a background task needs to report back to the UI.

### Events

The runtime maps Crossterm events into framework events:

```rust
pub enum Event {
    FocusGained,
    FocusLost,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Paste(String),
    Resize(u16, u16),
    Tick,
}
```

`Tick` fires every 250ms by default. Change it through `AppConfig` when you need smoother animation or less frequent polling.

### Messages

Messages are for cross-component or background-task communication:

```rust
let _ = context.try_send(Message::custom(AppMessage::Refresh));

fn update(&mut self, message: Message, _context: &Context) {
    if let Ok(message) = message.downcast::<AppMessage>() {
        // handle your typed message
    }
}
```

The built-in `Message::Quit` exits the app loop.

## Configuration

Use `App::with_config` when you need to tune runtime behavior:

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
        focus_change: true,
    },
};

let mut app = App::with_config(component, config)?;
```

Mouse capture and focus change are opt-in because they change normal terminal behavior. Bracketed paste is enabled by default so paste input can arrive as `Event::Paste(String)`.

## Examples

| Example | What it shows |
| --- | --- |
| `hello_world` | Basic rendering and quit handling |
| `counter` | Mutable state and keyboard input |
| `text_input` | Character input, backspace, enter, paste |
| `list_selector` | Bounded list navigation and selected styling |
| `layout_demo` | Nested Ratatui layouts |
| `tabs` | View switching |
| `progress` | Tick-driven updates |

## Template Structure

```text
tui-base-framework/
├── src/
│   ├── app.rs           # App loop, config, event pump
│   ├── component.rs     # Component trait and Context
│   ├── event.rs         # Framework event type
│   ├── message.rs       # Quit/custom messages
│   ├── terminal.rs      # TerminalGuard and terminal config
│   ├── lib.rs           # Public exports
│   ├── examples.rs      # Example module exports
│   └── examples/        # Reusable example components
├── examples/            # Runnable binaries
├── QUICKSTART.md
├── CHEATSHEET.md
└── Cargo.toml
```

When you are done learning from the examples, remove `examples/`, `src/examples/`, and `src/examples.rs`, then update `src/lib.rs` if you no longer want to export examples.

## Customizing

1. Update `Cargo.toml` with your project name, authors, description, repository, and license.
2. Pick the example closest to your app.
3. Copy it to `src/main.rs`.
4. Replace example state and rendering with your domain.
5. Keep the app loop until you have a reason to own lower-level terminal details.

## Troubleshooting

If your terminal does not restore after a hard crash, run:

```bash
reset
```

If mouse events do not arrive, enable `TerminalConfig { mouse_capture: true, ..Default::default() }`.

If the UI does not redraw after input, make sure the component returns `EventResult::Consumed` for events that mutate state.

## Dependencies

- `ratatui` 0.30 for terminal UI rendering
- `crossterm` 0.29 for terminal input/control
- `tokio` 1.52 with minimal runtime features
- `anyhow` 1.0 for ergonomic error handling

## License

Licensed under either MIT or Apache-2.0, at your option.
