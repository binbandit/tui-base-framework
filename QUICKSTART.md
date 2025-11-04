# Quick Start Guide

Get up and running with this TUI template in 5 minutes.

> **💡 Fastest way**: Copy an example to `src/main.rs` and customize it!
> 
> ```bash
> cp examples/counter.rs src/main.rs
> cargo run
> ```

## Get the Template

```bash
git clone https://github.com/binbandit/tui-base-framework my-tui-app
cd my-tui-app
```

The template already has all dependencies configured in `Cargo.toml`:

```toml
[dependencies]
ratatui = "0.29"
crossterm = "0.29"
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
```

## Your First App (3 Steps)

Create `src/main.rs` with the following code (or copy from an example):

### Step 1: Create a Component

```rust
use tui_base_framework::{Component, Event, EventResult, Message, Frame, Rect};
use ratatui::widgets::{Paragraph, Block, Borders};
use crossterm::event::KeyCode;
use tokio::sync::mpsc;

struct MyApp {
    message_sender: Option<mpsc::Sender<Message>>,
}

impl MyApp {
    fn new() -> Self {
        Self { message_sender: None }
    }
}
```

### Step 2: Implement the Component Trait

```rust
impl Component for MyApp {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let text = Paragraph::new("Hello! Press 'q' to quit")
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(text, area);
    }
    
    fn handle_event(&mut self, event: Event) -> EventResult {
        if let Event::Key(key) = event {
            if let KeyCode::Char('q') = key.code {
                if let Some(sender) = &self.message_sender {
                    let _ = sender.try_send(Message::Quit);
                }
                return EventResult::Consumed;
            }
        }
        EventResult::Propagate
    }
    
    fn update(&mut self, _message: Message) {}
    
    fn set_message_sender(&mut self, sender: mpsc::Sender<Message>) {
        self.message_sender = Some(sender);
    }
}
```

### Step 3: Run It

```rust
use tui_base_framework::App;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let my_app = MyApp::new();
    let mut app = App::new(Box::new(my_app))?;
    app.run().await?;
    Ok(())
}
```

That's it! Run with `cargo run`.

## Common Tasks

### Add State

```rust
struct MyApp {
    count: i32,  // Add your state fields
    message_sender: Option<mpsc::Sender<Message>>,
}
```

### Handle Keyboard Input

```rust
fn handle_event(&mut self, event: Event) -> EventResult {
    if let Event::Key(key) = event {
        match key.code {
            KeyCode::Up => {
                self.count += 1;
                EventResult::Consumed
            }
            KeyCode::Down => {
                self.count -= 1;
                EventResult::Consumed
            }
            KeyCode::Char('q') => {
                // Send quit message
                EventResult::Consumed
            }
            _ => EventResult::Propagate,
        }
    } else {
        EventResult::Propagate
    }
}
```

### Create Layouts

```rust
use ratatui::layout::{Layout, Direction, Constraint};

fn render(&self, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Header
            Constraint::Min(0),      // Body
            Constraint::Length(3),   // Footer
        ])
        .split(area);
    
    // Render widgets in each chunk
    frame.render_widget(header_widget, chunks[0]);
    frame.render_widget(body_widget, chunks[1]);
    frame.render_widget(footer_widget, chunks[2]);
}
```

### Add Colors and Styles

```rust
use ratatui::style::{Style, Color, Modifier};

let styled_text = Paragraph::new("Styled text")
    .style(Style::default()
        .fg(Color::Cyan)
        .bg(Color::Black)
        .add_modifier(Modifier::BOLD));
```

### Use Different Widgets

```rust
use ratatui::widgets::{
    Paragraph, List, ListItem, Table, Row, 
    Gauge, Block, Borders, Tabs
};

// Paragraph - simple text
let para = Paragraph::new("Text here");

// List - multiple items
let items = vec![
    ListItem::new("Item 1"),
    ListItem::new("Item 2"),
];
let list = List::new(items);

// Gauge - progress bar
let gauge = Gauge::default()
    .percent(50)
    .label("50%");
```

## Examples to Learn From

The template includes 7 complete examples. Run them to see different patterns:

```bash
# Beginner
cargo run --example hello_world    # Basics
cargo run --example counter        # State management
cargo run --example text_input     # User input

# Intermediate
cargo run --example list_selector  # Navigation
cargo run --example layout_demo    # Complex layouts
cargo run --example tabs           # View switching

# Advanced
cargo run --example progress       # Animations
```

**Pro tip**: Copy an example to `src/main.rs` as your starting point:

```bash
cp examples/counter.rs src/main.rs
cargo run
```

## Key Concepts

### Component Trait

Every app implements three methods:

1. **`render()`** - Draw your UI
2. **`handle_event()`** - Respond to input
3. **`update()`** - Handle messages (optional)

### Event Flow

```
User Input → Event → handle_event() → Update State → Render
```

### EventResult

- `EventResult::Consumed` - Stop event propagation
- `EventResult::Propagate` - Pass event to next handler

### Messages

Send messages to communicate between components or trigger actions:

```rust
// Quit the app
sender.try_send(Message::Quit);

// Custom messages
sender.try_send(Message::Custom(Box::new(MyMessage::DoSomething)));
```

## Cheat Sheet

### Essential Imports

```rust
use tui_base_framework::{
    Component, Event, EventResult, Message, 
    Frame, Rect, App
};
use ratatui::{
    widgets::{Paragraph, Block, Borders},
    layout::{Layout, Direction, Constraint, Alignment},
    style::{Style, Color, Modifier},
};
use crossterm::event::{KeyCode, KeyModifiers};
use tokio::sync::mpsc;
use anyhow::Result;
```

### Component Template

```rust
struct MyComponent {
    // Your state fields
    message_sender: Option<mpsc::Sender<Message>>,
}

impl Component for MyComponent {
    fn render(&self, frame: &mut Frame, area: Rect) {
        // Draw UI
    }
    
    fn handle_event(&mut self, event: Event) -> EventResult {
        // Handle input
        EventResult::Propagate
    }
    
    fn update(&mut self, _message: Message) {
        // Handle messages
    }
    
    fn set_message_sender(&mut self, sender: mpsc::Sender<Message>) {
        self.message_sender = Some(sender);
    }
}
```

### Main Function Template

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let component = MyComponent::new();
    let mut app = App::new(Box::new(component))?;
    app.run().await?;
    Ok(())
}
```

## Customizing the Template

### 1. Update Project Info

Edit `Cargo.toml`:

```toml
[package]
name = "my-awesome-tui"
authors = ["Your Name <you@example.com>"]
description = "My awesome TUI application"
```

### 2. Pick Your Starting Point

```bash
# Copy an example to src/main.rs
cp examples/counter.rs src/main.rs
cargo run
```

### 3. Clean Up When Ready

```bash
# Remove examples after learning
rm -rf examples/ src/examples/ src/examples.rs

# Keep CHEATSHEET.md as reference
```

### Core Framework Files (Keep These)

- `src/app.rs` - Application lifecycle
- `src/component.rs` - Component trait
- `src/event.rs` - Event types
- `src/message.rs` - Message types
- `src/terminal.rs` - Terminal management

Modify or remove `src/lib.rs` based on your needs.

## Common Patterns

### Quit on 'q' or Escape

```rust
match key.code {
    KeyCode::Char('q') | KeyCode::Esc => {
        if let Some(sender) = &self.message_sender {
            let _ = sender.try_send(Message::Quit);
        }
        EventResult::Consumed
    }
    _ => EventResult::Propagate,
}
```

### Toggle Boolean State

```rust
KeyCode::Char(' ') => {
    self.paused = !self.paused;
    EventResult::Consumed
}
```

### Navigate List

```rust
KeyCode::Up => {
    if self.selected > 0 {
        self.selected -= 1;
    }
    EventResult::Consumed
}
KeyCode::Down => {
    if self.selected < self.items.len() - 1 {
        self.selected += 1;
    }
    EventResult::Consumed
}
```

### Center Text

```rust
use ratatui::layout::Alignment;

Paragraph::new("Centered")
    .alignment(Alignment::Center)
```

### Add Borders

```rust
use ratatui::widgets::{Block, Borders};

Paragraph::new("Text")
    .block(Block::default()
        .borders(Borders::ALL)
        .title("Title"))
```

Happy coding! 🎉
