# TUI Base Framework

> **📦 This is a template repository** - Click "Use this template" to create your own TUI app!

A minimal, extensible template for building terminal user interfaces in Rust. Built with Ratatui and Tokio, this template provides a component-based architecture that feels natural and requires minimal boilerplate.

**How to use**: Clone it, explore the examples, pick a starting point, customize it, and build your own TUI app!

```
Clone → Run Examples → Copy to src/main.rs → Customize → Ship! 🚀
```

## Why This Template?

Most TUI frameworks force you into complex abstractions or leave you wiring everything manually. This template finds the sweet spot: a simple Component trait, async event handling, and automatic terminal management. Start with a solid foundation and customize it for your needs.

## What Can You Build?

The framework includes 7 complete examples demonstrating different patterns:

- 📝 **Hello World** - Your first TUI app (30 lines)
- 🔢 **Counter** - Interactive state management (50 lines)
- ⌨️ **Text Input** - User input handling (60 lines)
- 📋 **List Selector** - Navigation and selection (70 lines)
- 🎨 **Layout Demo** - Complex multi-panel UIs (80 lines)
- 📑 **Tabs** - View switching and organization (100 lines)
- 📊 **Progress Bar** - Animations and time-based updates (100 lines)

Each example is fully documented, runnable, and teaches specific patterns you'll use in real apps.

## Quick Start

### Get Started in 60 Seconds

```bash
# 1. Use template on GitHub or clone
git clone https://github.com/binbandit/tui-base-framework my-tui-app
cd my-tui-app

# 2. Run an example
cargo run --example counter

# 3. Copy it as your starting point
cp examples/counter.rs src/main.rs

# 4. Run your app
cargo run

# 5. Start customizing!
```

### Explore Examples

```bash
cargo run --example hello_world   # Simplest app (30 lines)
cargo run --example counter       # State management (50 lines)
cargo run --example text_input    # User input (60 lines)
cargo run --example list_selector # Navigation (70 lines)
cargo run --example layout_demo   # Complex layouts (80 lines)
cargo run --example tabs          # View switching (100 lines)
cargo run --example progress      # Animations (100 lines)
```

### Customize

1. Update `Cargo.toml` with your project name
2. Modify `src/main.rs` for your needs
3. Remove examples when ready: `rm -rf examples/ src/examples/ src/examples.rs`

## Documentation

- **[QUICKSTART.md](QUICKSTART.md)** - Code patterns and quick reference
- **[CHEATSHEET.md](CHEATSHEET.md)** - Quick lookup for common patterns
- **[examples/](examples/)** - 7 complete examples with inline documentation

**Learning path**: Run examples → Pick one → Copy to `src/main.rs` → Customize → Build

## Core Concepts

Simple, clean architecture: implement `Component` trait, handle events, render UI. That's it.

### Components

Everything in your app is a component. Implement the `Component` trait on any struct:

```rust
use tui_base_framework::{Component, Event, EventResult, Message, Frame, Rect};

struct MyComponent {
    state: i32,
}

impl Component for MyComponent {
    fn render(&self, frame: &mut Frame, area: Rect) {
        // Draw your UI here using Ratatui widgets
    }
    
    fn handle_event(&mut self, event: Event) -> EventResult {
        // Respond to keyboard/mouse input
        EventResult::Consumed  // or EventResult::Propagate
    }
    
    fn update(&mut self, message: Message) {
        // Handle messages from other components
    }
}
```

### Events

The framework captures terminal events and routes them to your components:

```rust
pub enum Event {
    Key(KeyEvent),      // Keyboard input
    Mouse(MouseEvent),  // Mouse events
    Resize(u16, u16),   // Terminal resize
    Tick,               // Fires every 250ms for animations
}
```

Components return `EventResult::Consumed` to stop propagation or `EventResult::Propagate` to pass the event along.

The `Tick` event fires automatically 4 times per second, perfect for progress bars, animations, and time-based updates.

### Messages

Components communicate through typed messages:

```rust
pub enum Message {
    Quit,                           // Framework message to exit
    Custom(Box<dyn Any + Send>),    // Your custom messages
}
```

Send messages from event handlers, receive them in `update()`.

## Building Your First App

Let's build a simple counter app step by step. You can use this as your `src/main.rs`.

### Step 1: Create Your Component

```rust
use tui_base_framework::{Component, Event, EventResult, Message, Frame, Rect};
use tui_base_framework::widgets::{Paragraph, Block, Borders};
use tui_base_framework::layout::Alignment;
use crossterm::event::{KeyCode, KeyEvent};

struct Counter {
    count: i32,
}

impl Counter {
    fn new() -> Self {
        Self { count: 0 }
    }
}
```

### Step 2: Implement Rendering

```rust
impl Component for Counter {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let text = format!("Count: {} (Press ↑/↓, q to quit)", self.count);
        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Counter"))
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
    }
    
    // ... other methods
}
```

### Step 3: Handle Input

```rust
impl Component for Counter {
    // ... render method
    
    fn handle_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Key(KeyEvent { code, .. }) => match code {
                KeyCode::Up => {
                    self.count += 1;
                    EventResult::Consumed
                }
                KeyCode::Down => {
                    self.count -= 1;
                    EventResult::Consumed
                }
                KeyCode::Char('q') => {
                    // Quit handled by framework
                    EventResult::Propagate
                }
                _ => EventResult::Propagate,
            },
            _ => EventResult::Propagate,
        }
    }
    
    fn update(&mut self, _message: Message) {
        // Handle messages if needed
    }
}
```

### Step 4: Run Your App

```rust
use tui_base_framework::App;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let counter = Counter::new();
    let mut app = App::new(Box::new(counter))?;
    app.run().await?;
    Ok(())
}
```

That's it! Your app handles rendering, input, and terminal cleanup automatically.

## Philosophy

This framework is built on three principles:

### Purposeful Simplicity

Every abstraction must earn its place. No complex state management, no elaborate component trees, no hidden magic. Just a trait, some events, and a run loop.

### Beautiful Code

Code should inspire. Clean implementations, minimal boilerplate, obvious behavior. When you read framework code, you should think "I could have written this."

### Extensibility

The framework provides the foundation. You build the features. Components compose naturally, events flow predictably, and nothing stops you from adding your own patterns.

## Template Structure

```
tui-base-framework/
├── src/
│   ├── app.rs           # ✅ Keep - Application lifecycle
│   ├── component.rs     # ✅ Keep - Component trait
│   ├── event.rs         # ✅ Keep - Event types
│   ├── message.rs       # ✅ Keep - Message types
│   ├── terminal.rs      # ✅ Keep - Terminal management
│   ├── lib.rs           # ⚙️  Optional - Remove if binary-only
│   ├── examples.rs      # ❌ Remove - Example module
│   └── examples/        # ❌ Remove - Example implementations
├── examples/            # ❌ Remove - After learning
│   ├── hello_world.rs
│   ├── counter.rs
│   └── ... (7 total)
├── README.md            # 📝 Replace with your own
├── QUICKSTART.md        # 📚 Keep as reference
├── CHEATSHEET.md        # 📚 Keep as reference
└── Cargo.toml           # ⚙️  Update with your info
```

**After setup**: Remove `examples/`, `src/examples/`, and `src/examples.rs`. Update README and Cargo.toml.

## Examples

The `examples/` directory contains working applications that demonstrate different features and complexity levels. Each example is fully documented and ready to run.

### Beginner Examples

**1. Hello World** (`cargo run --example hello_world`)
- The simplest possible TUI app
- Shows basic rendering and quit handling
- Perfect starting point for newcomers
- ~30 lines of code

**2. Counter** (`cargo run --example counter`)
- Interactive counter with keyboard input
- Demonstrates state management
- Shows event handling patterns
- ~50 lines of code

**3. Text Input** (`cargo run --example text_input`)
- Simple text input field
- Character input and backspace handling
- Good introduction to user input
- ~60 lines of code

### Intermediate Examples

**4. List Selector** (`cargo run --example list_selector`)
- Navigable list with selection highlighting
- Arrow key navigation
- Visual feedback for selected items
- ~70 lines of code

**5. Layout Demo** (`cargo run --example layout_demo`)
- Multiple panels with nested layouts
- Header, body (split left/right), and footer
- Shows how to compose complex UIs
- ~80 lines of code

**6. Tabs** (`cargo run --example tabs`)
- Tab navigation between different views
- Dynamic content based on selected tab
- Demonstrates view switching patterns
- ~100 lines of code

### Advanced Examples

**7. Progress Bar** (`cargo run --example progress`)
- Animated progress bar
- Time-based updates
- Pause/resume functionality
- Shows how to handle continuous updates
- ~100 lines of code

### Running Examples

Run any example with:

```bash
cargo run --example <name>
```

For example:
```bash
cargo run --example hello_world
cargo run --example counter
cargo run --example tabs
```

All examples support quitting with 'q' and include helpful on-screen instructions.

### Learning Path

If you're new to TUI development, we recommend following this path:

1. **Start with `hello_world`** - Understand the basic structure
2. **Move to `counter`** - Learn event handling and state
3. **Try `text_input`** - See how to capture user input
4. **Explore `list_selector`** - Navigation and selection patterns
5. **Study `layout_demo`** - Complex UI composition
6. **Check out `tabs`** - View switching and organization
7. **Finish with `progress`** - Time-based updates and animations

Each example builds on concepts from the previous ones, gradually introducing new patterns and techniques.

### Example Comparison

| Example | Lines | Concepts | Best For |
|---------|-------|----------|----------|
| hello_world | ~30 | Rendering, quit handling | First-time TUI developers |
| counter | ~50 | State, keyboard input | Learning state management |
| text_input | ~60 | Character input, string manipulation | Building forms |
| list_selector | ~70 | Navigation, selection, styling | Menus and lists |
| layout_demo | ~80 | Layouts, panels, composition | Complex UIs |
| tabs | ~100 | View switching, dynamic content | Multi-view apps |
| progress | ~100 | Time-based updates, animations | Loading screens, dashboards |

## Advanced Usage

### Composing Components

Components can contain other components:

```rust
struct App {
    header: Header,
    body: Body,
    footer: Footer,
}

impl Component for App {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);
        
        self.header.render(frame, chunks[0]);
        self.body.render(frame, chunks[1]);
        self.footer.render(frame, chunks[2]);
    }
    
    fn handle_event(&mut self, event: Event) -> EventResult {
        // Route events to child components
        if self.body.handle_event(event) == EventResult::Consumed {
            return EventResult::Consumed;
        }
        EventResult::Propagate
    }
}
```

### Custom Messages

Define your own message types:

```rust
enum AppMessage {
    UpdateCount(i32),
    SwitchView(ViewType),
}

// In your component:
fn update(&mut self, message: Message) {
    if let Message::Custom(boxed) = message {
        if let Some(app_msg) = boxed.downcast_ref::<AppMessage>() {
            match app_msg {
                AppMessage::UpdateCount(n) => self.count = *n,
                AppMessage::SwitchView(view) => self.current_view = *view,
            }
        }
    }
}
```

## Dependencies

This template uses:

- `ratatui` - Terminal rendering
- `crossterm` - Cross-platform terminal control
- `tokio` - Async runtime
- `anyhow` - Error handling

Feel free to add or remove dependencies based on your needs!

## Customizing for Your Project

1. **Update Cargo.toml**: Change the package name, version, authors, and description
2. **Choose a starting point**: Pick an example that's closest to your needs
3. **Copy to main.rs**: `cp examples/your_choice.rs src/main.rs`
4. **Start customizing**: Modify the code to fit your requirements
5. **Remove unused code**: Delete examples and framework code you don't need
6. **Add your features**: Build on top of the foundation

## License

This template is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

**Your project built from this template can use any license you choose.**

## Troubleshooting

### Terminal doesn't restore after crash

The framework automatically restores the terminal on exit, but if your app panics, you might need to reset it:

```bash
reset
```

### Colors don't show up

Make sure your terminal supports colors. Most modern terminals do, but you can test with:

```bash
echo $TERM
```

### App doesn't respond to input

Check that you're returning `EventResult::Consumed` for events you handle. If you always return `Propagate`, events might not be processed correctly.

### Layout looks wrong

Terminal size matters! Test your app at different sizes:

```bash
# Resize your terminal window and see how the app adapts
```

## Contributing

Found a bug or have an improvement for the template? Contributions are welcome! Please keep the philosophy in mind: purposeful simplicity, beautiful code, extensibility.

## Showcase

Built something cool with this template? Open an issue to add your project here!

## Contributing

Improvements to the template are welcome! Keep it simple, well-documented, and beginner-friendly.
