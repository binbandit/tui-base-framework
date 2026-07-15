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

Click **Use this template** on GitHub (or clone directly), then run one command:

```bash
git clone https://github.com/binbandit/tui-base-framework my-tui-app
cd my-tui-app
./setup.sh my-tui-app     # make the project yours (one command)
cargo run
```

`setup.sh` renames the crate everywhere, fills in your author info from git config, and cleans up the template metadata. Two flags cover the common paths:

```bash
./setup.sh my-tui-app                # keep the lib + examples (learning mode)
./setup.sh my-tui-app --app-only     # binary-only app: no lib.rs, no examples
```

`--app-only` is the right choice when you're building an application: it folds the framework into your binary as a plain `src/tui/` module — no library target, nothing published, just your app. The script verifies the result with `cargo check` and deletes itself when done. (`--no-examples`, `--fresh-git`, and `--yes` for non-interactive use are also available; run `./setup.sh --help`.)

On Windows, run `setup.sh` from Git Bash (it ships with Git for Windows) or WSL. The apps themselves build and run natively on Windows — CI checks every push there too.

`cargo run` launches a small starter app whose code lives in `src/main.rs`, ready to edit. Or start from an example — every example is a single self-contained file:

```bash
cargo run --example counter      # try it
cp examples/counter.rs src/main.rs   # make it yours
cargo run
```

## Minimal App

No async boilerplate — `run` creates the Tokio runtime for you:

```rust
use anyhow::Result;
use tui_base_framework::{run, Component, Context, Event, EventResult, Frame, KeyCode, Rect};
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

fn main() -> Result<()> {
    run(Counter { count: 0 })
}
```

Need async setup before the UI starts, or your own runtime? Use `#[tokio::main]` and drive `App` directly:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let data = load_config().await?;
    tui_base_framework::App::new(MyApp::new(data))?.run().await
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

For one-key bindings, `Event::is_key` collapses the match boilerplate, and
`Event::is_ctrl` covers the most common chord:

```rust
if event.is_key(KeyCode::Char('q')) || event.is_key(KeyCode::Esc) {
    context.quit();
    return EventResult::Consumed;
}

if event.is_ctrl('s') {
    self.save();
    return EventResult::Consumed;
}
```

For text input, `Event::char` returns the typed character and ignores
Ctrl/Alt chords, so an input field never swallows keyboard shortcuts:

```rust
if let Some(c) = event.char() {
    self.input.push(c);
    return EventResult::Consumed;
}
```

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
    Tick(Duration),
}
```

`Tick` fires every 250ms by default (change it through `AppConfig`) and carries the time elapsed since the previous tick. Scale animations by it and they stay wall-clock accurate at any tick rate — ticks are dropped rather than queued while the UI is busy, so the elapsed time can span more than one interval:

```rust
Event::Tick(elapsed) => {
    self.percent += elapsed.as_secs_f64() * FILL_RATE;
    EventResult::Consumed
}
```

### The Cursor

The terminal cursor is hidden by default. To show it — the natural thing for text input — set its position during `render`; it is visible on frames that set a position and hidden on frames that don't:

```rust
fn render(&mut self, frame: &mut Frame, area: Rect) {
    // ...draw the input field...
    frame.set_cursor_position(Position::new(area.x + 1 + typed_width, area.y + 1));
}
```

See `examples/text_input.rs` for a complete input field with a live cursor.

### Errors

Recoverable errors are ordinary data: send them as a message and render the failure. For fatal errors, `Context::fail` stops the app, restores the terminal, and returns the error from `run`:

```rust
let context = context.clone();
tokio::spawn(async move {
    if let Err(error) = sync_database().await {
        context.fail(error); // run() returns Err(error) after cleanup
    }
});
```

Components stay infallible by design — `handle_event` and `update` don't return `Result` — so the trait stays small and the common path stays clean.

## Configuration

Use `run_with_config` (or `App::with_config`) when you need to tune runtime behavior:

```rust
use std::time::Duration;
use tui_base_framework::{run_with_config, AppConfig, TerminalConfig, Viewport};

let config = AppConfig {
    tick_rate: Duration::from_millis(100),
    input_poll_rate: Duration::from_millis(25),
    channel_capacity: 512,
    quit_on_ctrl_c: true,
    suspend_on_ctrl_z: true,
    terminal: TerminalConfig {
        mouse_capture: true,
        bracketed_paste: true,
        focus_change: true,
        viewport: Viewport::Fullscreen,
    },
};

run_with_config(component, config)?;
```

Mouse capture and focus change are opt-in because they change normal terminal behavior. Bracketed paste is enabled by default so paste input arrives as a single `Event::Paste(String)`.

### Ctrl-C, Ctrl-Z, and Suspending

By default the app quits on Ctrl-C and suspends to the shell on Ctrl-Z (resuming cleanly on `fg` — Unix only; on Windows Ctrl-Z reaches the component like any other key). Your component always sees the key press first: consume it to override the default, e.g. to show a "really quit?" confirmation on Ctrl-C. Set `quit_on_ctrl_c: false` / `suspend_on_ctrl_z: false` to take over entirely.

The same primitives are public: if you own the terminal directly through `TerminalGuard` (instead of `run`), `suspend()` / `resume()` hand the terminal to a subprocess (`$EDITOR`, a pager) and take it back with a full repaint.

### Inline Apps

`Viewport::Inline(height)` draws the UI in `height` rows of the normal scrollback instead of taking over the screen — the right shape for progress displays and prompt-style tools. Output printed before the app ran stays visible, and the final frame stays in the scrollback after exit, with the shell prompt continuing below it.

```rust
terminal: TerminalConfig {
    viewport: Viewport::Inline(6),
    ..TerminalConfig::default()
},
```

Inline setup locates the viewport by querying the cursor position, so it needs a real interactive terminal (not a pipe or CI).

## Examples

Each example is one self-contained file you can read top to bottom and copy over `src/main.rs`.

| Example | What it shows |
| --- | --- |
| `hello_world` | Basic rendering and quit handling |
| `counter` | Mutable state and keyboard input |
| `text_input` | Character input with a real terminal cursor, paste handling |
| `list_selector` | Stateful `List` widget with `ListState` navigation |
| `layout_demo` | Nested Ratatui layouts |
| `tabs` | View switching |
| `progress` | Tick-driven animation scaled by elapsed time |
| `inline` | Inline viewport: a progress bar living in the scrollback |
| `async_task` | Background Tokio task reporting progress via typed messages |
| `focus` | Composing components: parent routes events to the focused child |
| `screens` | Multi-screen navigation: a router, screens as components, reusable widgets |
| `mouse` | Mouse capture: click, drag, and scroll handling |

```bash
cargo run --example async_task
```

## Growing to Multiple Screens

Real apps outgrow one component. The framework adds no screen manager or widget registry for this — two plain-Rust patterns cover it, demonstrated end to end in `examples/screens.rs`:

**Screens are components.** Each screen implements `Component` with the app's message type and owns its own state. The root owns every screen and routes `render`/`handle_event` to the active one — that router is a match statement:

```rust
fn active_mut(&mut self) -> &mut dyn Component<Message = Msg> {
    match self.active {
        ScreenId::Home => &mut self.home,
        ScreenId::Editor => &mut self.editor,
    }
}
```

Navigation is just a message: a screen sends `Msg::OpenSettings` through its `Context`, and the root's `update` switches the active screen and moves data between screens. No screen knows any other screen exists, so screens stay independently testable. (A `Vec<Box<dyn Component<Message = Msg>>>` gives you a navigation stack the same way.)

**Reusable widgets are plain structs**, not `Component`s. A widget that never touches messages — it takes `&Event`, reports whether it consumed it, and draws itself — plugs into any screen of any app, regardless of message type:

```rust
impl TextField {
    fn handle_event(&mut self, event: &Event) -> bool { /* consumed? */ }
    fn render(&self, frame: &mut Frame, area: Rect) { /* draw */ }
}
```

Keep `Component` for things that live on the app's message bus (screens, panes with async work); keep leaf widgets message-free and share them everywhere. `examples/focus.rs` shows the middle ground — child components composed inside one screen with focus routing.

## Template Structure

```text
tui-base-framework/
├── src/
│   ├── tui/             # The framework (self-contained)
│   │   ├── mod.rs       #   Re-exports: everything apps import
│   │   ├── app.rs       #   App loop, config, run() helpers
│   │   ├── component.rs #   Component trait and Context
│   │   ├── event.rs     #   Framework event type
│   │   └── terminal.rs  #   TerminalGuard, terminal config, panic hook
│   ├── lib.rs           # Thin re-export of src/tui/
│   └── main.rs          # Your app starts here
├── examples/            # Self-contained runnable examples
├── setup.sh             # One-command project setup
├── CHEATSHEET.md        # Copy-paste reference for common tasks
├── AGENTS.md            # Repo guide for AI coding agents (CLAUDE.md imports it)
└── Cargo.toml
```

The framework lives entirely under `src/tui/` and your app only imports from it — that clean boundary is what lets `setup.sh --app-only` fold it into a binary-only project mechanically. When you are done learning from the examples, delete the `examples/` directory — nothing else references it.

## Customizing

1. Run `./setup.sh <your-app-name>` (see Quick Start) — it handles Cargo.toml metadata and renames for you.
2. Pick the example closest to your app and copy it to `src/main.rs` (or just edit the starter that's already there).
3. Replace the example state and rendering with your domain.
4. Keep the app loop until you have a reason to own lower-level terminal details — the `src/tui/` folder is yours to modify too.

## Working With AI Coding Agents

The repo ships an `AGENTS.md` (with a `CLAUDE.md` that imports it) so tools like Claude Code, Cursor, and Codex understand the architecture, invariants, and test patterns without exploring first. `setup.sh` trims it down to the app-facing guide, so your generated project keeps working agent docs too.

## Testing Your Components

Components are plain structs, so they test without a terminal. `Context::test()` gives you a context plus the receiving end of its message channel; `Event::key_press` fabricates input; ratatui's `TestBackend` (re-exported) checks what actually renders:

```rust
#[test]
fn q_quits() {
    let (context, _messages) = Context::test();
    let mut app = MyApp::new();

    let result = app.handle_event(Event::key_press(KeyCode::Char('q')), &context);

    assert_eq!(result, EventResult::Consumed);
    assert!(context.quit_requested());
}

#[test]
fn renders_title() {
    let mut terminal = Terminal::new(TestBackend::new(60, 12)).unwrap();
    let mut app = MyApp::new();

    terminal.draw(|frame| app.render(frame, frame.area())).unwrap();

    let screen: String = terminal.backend().buffer().content()
        .iter().map(|cell| cell.symbol()).collect();
    assert!(screen.contains("My App"));
}
```

The starter `src/main.rs` ships with working tests in this style — `cargo test` passes from the first minute, and new components can copy the pattern.

## Performance Defaults

The runtime is built to be efficient by default:

- Event-driven rendering: the app redraws after handled events/messages instead of repainting every frame, and coalesces bursts of input into a single redraw.
- Blocking terminal input is isolated in a blocking task, so it does not park Tokio worker threads.
- Messages are statically typed — no boxing or runtime downcasts on the message path.
- The app is generic over your component type, avoiding heap allocation and dynamic dispatch unless you box a component yourself.
- Stale animation ticks are dropped when the UI is busy, so background ticks do not build up into delayed redraws — and `Event::Tick` carries the real elapsed time, so animations stay accurate across drops.
- Key release events are filtered before they reach components, avoiding double-handling on terminals that emit enhanced keyboard events.
- Tokio and Ratatui are built with only the features this template needs.
- Release builds use thin LTO, one codegen unit, and stripped symbols.

## Releasing Your App

Tag a version and CI ships it:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The release workflow builds optimized binaries for Linux (x86_64, arm64), macOS (Apple Silicon, Intel), and Windows, and attaches them to a GitHub release with generated notes. The binary name is read from `Cargo.toml`, so it works unchanged after `setup.sh` renames your project. Release builds use thin LTO, a single codegen unit, and stripped symbols — the starter app comes out under 1 MB.

Dependabot is configured to open weekly grouped PRs for Cargo dependencies and GitHub Actions, so the project stays current after you fork off.

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
- `signal-hook` 0.3 (Unix only) to raise SIGTSTP for Ctrl-Z suspend without `unsafe`

The minimum supported Rust version is declared as `rust-version` in `Cargo.toml` (currently **1.94**, edition 2024); CI reads it from there and checks it on every push. `Cargo.lock` is tracked because this is an application template. New projects get reproducible example builds immediately, then can update dependencies on their own cadence (`cargo update`).

## License

Licensed under either MIT or Apache-2.0, at your option. See `LICENSE-MIT` and `LICENSE-APACHE`.
