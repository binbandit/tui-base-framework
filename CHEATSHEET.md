# TUI Framework Cheat Sheet

Quick reference for common patterns and code snippets.

## Component Template

```rust
use tui_base_framework::{Component, Event, EventResult, Message, Frame, Rect};
use ratatui::widgets::{Paragraph, Block, Borders};
use crossterm::event::KeyCode;
use tokio::sync::mpsc;

struct MyComponent {
    // Your state
    message_sender: Option<mpsc::Sender<Message>>,
}

impl Component for MyComponent {
    fn render(&self, frame: &mut Frame, area: Rect) {
        // Draw UI
    }
    
    fn handle_event(&mut self, event: Event) -> EventResult {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('q') => {
                    if let Some(sender) = &self.message_sender {
                        let _ = sender.try_send(Message::Quit);
                    }
                    EventResult::Consumed
                }
                _ => EventResult::Propagate,
            }
        } else {
            EventResult::Propagate
        }
    }
    
    fn update(&mut self, _message: Message) {}
    
    fn set_message_sender(&mut self, sender: mpsc::Sender<Message>) {
        self.message_sender = Some(sender);
    }
}
```

## Common Imports

```rust
// Framework
use tui_base_framework::{Component, Event, EventResult, Message, Frame, Rect, App};

// Widgets
use ratatui::widgets::{
    Paragraph, Block, Borders, List, ListItem, 
    Table, Row, Gauge, Tabs
};

// Layout
use ratatui::layout::{Layout, Direction, Constraint, Alignment};

// Styling
use ratatui::style::{Style, Color, Modifier};

// Input
use crossterm::event::{KeyCode, KeyModifiers, MouseEvent};

// Async
use tokio::sync::mpsc;

// Error handling
use anyhow::Result;
```

## Layouts

### Vertical Split (Header/Body/Footer)

```rust
let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(3),    // Fixed height
        Constraint::Min(0),       // Flexible
        Constraint::Length(3),    // Fixed height
    ])
    .split(area);
```

### Horizontal Split (Sidebar/Main)

```rust
let chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Percentage(20),  // 20% width
        Constraint::Percentage(80),  // 80% width
    ])
    .split(area);
```

### Grid Layout

```rust
let rows = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ])
    .split(area);

let top_cols = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ])
    .split(rows[0]);

let bottom_cols = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ])
    .split(rows[1]);
```

## Widgets

### Paragraph

```rust
let para = Paragraph::new("Text content")
    .block(Block::default()
        .borders(Borders::ALL)
        .title("Title"))
    .style(Style::default().fg(Color::White))
    .alignment(Alignment::Center);

frame.render_widget(para, area);
```

### List

```rust
let items: Vec<ListItem> = vec![
    ListItem::new("Item 1"),
    ListItem::new("Item 2"),
    ListItem::new("Item 3"),
];

let list = List::new(items)
    .block(Block::default().borders(Borders::ALL).title("List"))
    .style(Style::default().fg(Color::White))
    .highlight_style(Style::default().bg(Color::Cyan));

frame.render_widget(list, area);
```

### Gauge (Progress Bar)

```rust
let gauge = Gauge::default()
    .block(Block::default().borders(Borders::ALL).title("Progress"))
    .gauge_style(Style::default().fg(Color::Green))
    .percent(50)
    .label("50%");

frame.render_widget(gauge, area);
```

### Tabs

```rust
use ratatui::text::Span;

let titles = vec![
    Span::raw("Tab 1"),
    Span::raw("Tab 2"),
    Span::raw("Tab 3"),
];

let tabs = Tabs::new(titles)
    .block(Block::default().borders(Borders::ALL))
    .select(selected_index)
    .highlight_style(Style::default().fg(Color::Yellow));

frame.render_widget(tabs, area);
```

## Styling

### Colors

```rust
Style::default().fg(Color::Red)
Style::default().bg(Color::Blue)
Style::default().fg(Color::Rgb(255, 128, 0))  // Custom RGB
```

### Modifiers

```rust
Style::default()
    .add_modifier(Modifier::BOLD)
    .add_modifier(Modifier::ITALIC)
    .add_modifier(Modifier::UNDERLINED)
```

### Combined

```rust
Style::default()
    .fg(Color::Yellow)
    .bg(Color::Black)
    .add_modifier(Modifier::BOLD)
```

## Event Handling

### Keyboard

```rust
fn handle_event(&mut self, event: Event) -> EventResult {
    match event {
        Event::Key(key) => {
            match key.code {
                KeyCode::Char(c) => {
                    // Handle character
                    EventResult::Consumed
                }
                KeyCode::Up => {
                    // Handle up arrow
                    EventResult::Consumed
                }
                KeyCode::Down => {
                    // Handle down arrow
                    EventResult::Consumed
                }
                KeyCode::Enter => {
                    // Handle enter
                    EventResult::Consumed
                }
                KeyCode::Backspace => {
                    // Handle backspace
                    EventResult::Consumed
                }
                KeyCode::Esc => {
                    // Handle escape
                    EventResult::Consumed
                }
                KeyCode::Tab => {
                    // Handle tab
                    EventResult::Consumed
                }
                _ => EventResult::Propagate,
            }
        }
        _ => EventResult::Propagate,
    }
}
```

### Tick Events (for animations)

```rust
fn handle_event(&mut self, event: Event) -> EventResult {
    match event {
        Event::Tick => {
            // Fires every 250ms - perfect for animations
            self.update_animation();
            EventResult::Consumed
        }
        Event::Key(key) => {
            // Handle keyboard
            EventResult::Consumed
        }
        _ => EventResult::Propagate,
    }
}
```

### Modifiers

```rust
if key.modifiers.contains(KeyModifiers::CONTROL) {
    // Ctrl is pressed
}

if key.modifiers.contains(KeyModifiers::SHIFT) {
    // Shift is pressed
}

if key.modifiers.contains(KeyModifiers::ALT) {
    // Alt is pressed
}
```

### Mouse

```rust
if let Event::Mouse(mouse) = event {
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // Left click at (mouse.column, mouse.row)
        }
        MouseEventKind::ScrollUp => {
            // Scroll up
        }
        MouseEventKind::ScrollDown => {
            // Scroll down
        }
        _ => {}
    }
}
```

## Common Patterns

### List Navigation

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

### Text Input

```rust
KeyCode::Char(c) => {
    self.input.push(c);
    EventResult::Consumed
}
KeyCode::Backspace => {
    self.input.pop();
    EventResult::Consumed
}
KeyCode::Enter => {
    // Process input
    self.input.clear();
    EventResult::Consumed
}
```

### Toggle State

```rust
KeyCode::Char(' ') => {
    self.enabled = !self.enabled;
    EventResult::Consumed
}
```

### Modal/Mode Switching

```rust
enum Mode {
    Normal,
    Insert,
    Command,
}

fn handle_event(&mut self, event: Event) -> EventResult {
    match self.mode {
        Mode::Normal => self.handle_normal_mode(event),
        Mode::Insert => self.handle_insert_mode(event),
        Mode::Command => self.handle_command_mode(event),
    }
}
```

### Conditional Styling

```rust
let style = if is_selected {
    Style::default().fg(Color::Black).bg(Color::Cyan)
} else if is_disabled {
    Style::default().fg(Color::DarkGray)
} else {
    Style::default().fg(Color::White)
};
```

## Main Function

```rust
use tui_base_framework::App;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let component = MyComponent::new();
    let mut app = App::new(Box::new(component))?;
    app.run().await?;
    Ok(())
}
```

## Debugging Tips

### Print to File (Terminal is captured)

```rust
use std::fs::OpenOptions;
use std::io::Write;

let mut file = OpenOptions::new()
    .create(true)
    .append(true)
    .open("debug.log")?;

writeln!(file, "Debug: {:?}", value)?;
```

### Panic Handler

```rust
use std::panic;

fn main() {
    panic::set_hook(Box::new(|panic_info| {
        // Log panic to file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("panic.log")
            .unwrap();
        writeln!(file, "Panic: {:?}", panic_info).unwrap();
    }));
    
    // Run app...
}
```

## Performance Tips

1. **Avoid allocations in render()** - Pre-compute strings in update()
2. **Use references** - Don't clone unnecessarily
3. **Batch updates** - Update state once, render once
4. **Limit redraws** - Only redraw when state changes

## Common Mistakes

### ❌ Don't: Always return Consumed

```rust
fn handle_event(&mut self, event: Event) -> EventResult {
    // Handle event...
    EventResult::Consumed  // Blocks all events!
}
```

### ✅ Do: Return Propagate for unhandled events

```rust
fn handle_event(&mut self, event: Event) -> EventResult {
    if let Event::Key(key) = event {
        match key.code {
            KeyCode::Char('q') => EventResult::Consumed,
            _ => EventResult::Propagate,  // Let others handle it
        }
    } else {
        EventResult::Propagate
    }
}
```

### ❌ Don't: Forget to check bounds

```rust
self.selected += 1;  // Can go out of bounds!
```

### ✅ Do: Always check bounds

```rust
if self.selected < self.items.len() - 1 {
    self.selected += 1;
}
```

### ❌ Don't: Panic in render()

```rust
fn render(&self, frame: &mut Frame, area: Rect) {
    let item = &self.items[self.selected];  // Panics if empty!
}
```

### ✅ Do: Handle edge cases

```rust
fn render(&self, frame: &mut Frame, area: Rect) {
    if let Some(item) = self.items.get(self.selected) {
        // Render item
    }
}
```

## Quick Reference

| Task | Code |
|------|------|
| Quit app | `sender.try_send(Message::Quit)` |
| Center text | `.alignment(Alignment::Center)` |
| Add border | `.block(Block::default().borders(Borders::ALL))` |
| Bold text | `.add_modifier(Modifier::BOLD)` |
| Set color | `.fg(Color::Cyan)` |
| Fixed height | `Constraint::Length(3)` |
| Flexible size | `Constraint::Min(0)` |
| Percentage | `Constraint::Percentage(50)` |
| Consume event | `EventResult::Consumed` |
| Pass event | `EventResult::Propagate` |

## Resources

- [Ratatui Docs](https://ratatui.rs/)
- [Crossterm Docs](https://docs.rs/crossterm/)
- [Examples](./examples/)
- [Tutorial](./TUTORIAL.md)

---

Keep this handy while coding! 📋
