# TUI Base Framework

A fast, small Rust template for building terminal user interfaces with Ratatui, Crossterm, and Tokio.

This repository is meant to be copied, trimmed, and shipped. It gives you a working app loop, terminal cleanup, typed events, type-safe message passing, and a set of runnable examples without forcing a large architecture on your project.

## Why This Template?

Most TUI starters either leave you wiring every terminal detail yourself or introduce a framework that is bigger than the app you wanted to build. This template keeps the interface small:

- Implement `Component`
- Render Ratatui widgets
- Handle input events
- Send typed messages through `Context`
- Let `App` manage terminal lifecycle, ticks, and redraws

## Quick Start

```bash
git clone https://github.com/binbandit/tui-base-framework my-tui-app
cd my-tui-app
cargo run
```

`cargo run` launches a small starter app whose code lives in `src/main.rs`, ready to edit. Or start from an example — every example is a single self-contained file:

```bash
cargo run --example counter      # try it
cp examples/counter.rs src/main.rs   # make it yours
cargo run
```

## Minimal App

```rust
use anyhow::Result;
use tui_base_framework::{App, Component, Context, Event, EventResult, Frame, KeyCode, Rect};
use tui_base_framework::widgets::{Block, Paragraph};

struct Counter {
    count: i64,
}

impl Component for Counter {
    type Message = ();

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let widget = Paragraph::new(format!("Count: {} | q to quit", self.count))
            .block(Block::bordered().title("Counter"));

        frame.render_widget(widget, area);
    }

    fn handle_event(&mut self, event: Event, context: &Context<Self::Message>) -> EventResult {
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
}

#[tokio::main]
async fn main() -> Result<()> {
    App::new(Counter { count: 0 })?.run().await
}
```

## Core Concepts

### Components

A component owns state, renders UI, and reacts to events and messages.

```rust
pub trait Component: Send {
    type Message: Send + 'static;

    fn init(&mut self, _context: &Context<Self::Message>) {}
    fn render(&mut self, frame: &mut Frame, area: Rect);
    fn handle_event(&mut self, _event: Event, _context: &Context<Self::Message>) -> EventResult {
        EventResult::Propagate
    }
    fn update(&mut self, _message: Self::Message, _context: &Context<Self::Message>) {}
}
```

Return `EventResult::Consumed` when an event changed state and should trigger a redraw. Return `EventResult::Propagate` when you ignored it.

`render` takes `&mut self`, so stateful Ratatui widgets (`ListState`, `TableState`, scroll offsets) live directly in your component — see `examples/list_selector.rs`.

### Messages

Messages are fully typed: declare an enum with one variant per thing that can happen asynchronously, and set it as your component's `Message` type. No downcasting, no boxing.

```rust
enum Msg {
    Progress(u16),
    Done { records: u32 },
}

impl Component for MyApp {
    type Message = Msg;

    fn update(&mut self, message: Msg, _context: &Context<Msg>) {
        match message {
            Msg::Progress(pct) => self.progress = pct,
            Msg::Done { records } => self.finish(records),
        }
    }
    // ...
}
```

Components that don't need messages use `type Message = ();`.

### Context

`Context` is how components talk back to the app loop:

```rust
context.quit();                        // ask the loop to exit
let _ = context.try_send(Msg::Saved);  // deliver a message to update()
let sender = context.sender();         // clone a sender for background tasks
```

### Background Work

Spawn a Tokio task, move a sender into it, and report back with typed messages. The UI never blocks:

```rust
fn handle_event(&mut self, event: Event, context: &Context<Msg>) -> EventResult {
    // ...on some key press:
    let sender = context.sender();
    tokio::spawn(async move {
        let records = fetch_data().await;
        let _ = sender.send(Msg::Done { records }).await;
    });
    EventResult::Consumed
}
```

See `examples/async_task.rs` for a complete, runnable version of this pattern.

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

Mouse capture and focus change are opt-in because they change normal terminal behavior. Bracketed paste is enabled by default so paste input arrives as a single `Event::Paste(String)`.

## Examples

Each example is one self-contained file you can read top to bottom and copy over `src/main.rs`.

| Example | What it shows |
| --- | --- |
| `hello_world` | Basic rendering and quit handling |
| `counter` | Mutable state and keyboard input |
| `text_input` | Character input, backspace, enter, paste |
| `list_selector` | Stateful `List` widget with `ListState` navigation |
| `layout_demo` | Nested Ratatui layouts |
| `tabs` | View switching |
| `progress` | Tick-driven updates and a custom tick rate |
| `async_task` | Background Tokio task reporting progress via typed messages |

```bash
cargo run --example async_task
```

## Template Structure

```text
tui-base-framework/
├── src/
│   ├── app.rs           # App loop, config, event pump
│   ├── component.rs     # Component trait and Context
│   ├── event.rs         # Framework event type
│   ├── terminal.rs      # TerminalGuard, terminal config, panic hook
│   ├── lib.rs           # Public exports
│   └── main.rs          # Your app starts here
├── examples/            # Self-contained runnable examples
├── CHEATSHEET.md        # Copy-paste reference for common tasks
└── Cargo.toml
```

When you are done learning from the examples, delete the `examples/` directory — nothing else references it.

## Customizing

1. Update `Cargo.toml` with your project name, authors, description, repository, and license.
2. Pick the example closest to your app and copy it to `src/main.rs` (or just edit the starter that's already there).
3. Replace the example state and rendering with your domain.
4. Keep the app loop until you have a reason to own lower-level terminal details.

## Performance Defaults

The runtime is built to be efficient by default:

- Event-driven rendering: the app redraws after handled events/messages instead of repainting every frame, and coalesces bursts of input into a single redraw.
- Blocking terminal input is isolated in a blocking task, so it does not park Tokio worker threads.
- Messages are statically typed — no boxing or runtime downcasts on the message path.
- The app is generic over your component type, avoiding heap allocation and dynamic dispatch unless you box a component yourself.
- Stale animation ticks are dropped when the UI is busy, so background ticks do not build up into delayed redraws.
- Key release events are filtered before they reach components, avoiding double-handling on terminals that emit enhanced keyboard events.
- Tokio and Ratatui are built with only the features this template needs.
- Release builds use thin LTO, one codegen unit, and stripped symbols.

## Troubleshooting

**Panics print normally.** A panic hook restores the terminal before the panic message is printed, so you get a readable message and backtrace instead of a mangled alternate screen. Terminal cleanup is also RAII-based: cursor, raw mode, alternate screen, paste, focus, and mouse state are restored when `App` drops.

If your terminal is somehow left in a bad state after a hard kill (e.g. `kill -9`), run `reset`.

If mouse events do not arrive, enable `TerminalConfig { mouse_capture: true, ..Default::default() }`.

If the UI does not redraw after input, make sure the component returns `EventResult::Consumed` for events that mutate state.

## Dependencies

- `ratatui` 0.30 for terminal UI rendering
- `crossterm` 0.29 for terminal input/control
- `tokio` 1.x with minimal runtime features
- `anyhow` 1.0 for ergonomic error handling

`Cargo.lock` is tracked because this is an application template. New projects get reproducible example builds immediately, then can update dependencies on their own cadence.

## License

Licensed under either MIT or Apache-2.0, at your option. See `LICENSE-MIT` and `LICENSE-APACHE`.
