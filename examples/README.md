# Examples

7 complete examples demonstrating TUI patterns. Run them, study them, copy one to `src/main.rs` as your starting point.

## Quick Reference

| Example | Lines | What It Shows | Run |
|---------|-------|---------------|-----|
| **hello_world** | 30 | Basic rendering, quit handling | `cargo run --example hello_world` |
| **counter** | 50 | State management, keyboard input | `cargo run --example counter` |
| **text_input** | 60 | Character input, string manipulation | `cargo run --example text_input` |
| **list_selector** | 70 | Navigation, selection, styling | `cargo run --example list_selector` |
| **layout_demo** | 80 | Complex layouts, multiple panels | `cargo run --example layout_demo` |
| **tabs** | 100 | View switching, dynamic content | `cargo run --example tabs` |
| **progress** | 100 | Time-based updates, animations | `cargo run --example progress` |

## Detailed Descriptions

### hello_world.rs
**The simplest possible TUI app**

Shows:
- Basic component structure
- Simple rendering with Paragraph widget
- Quit handling with 'q' key
- Minimal boilerplate

Perfect for: First-time TUI developers

```bash
cargo run --example hello_world
```

---

### counter.rs
**Interactive counter with state**

Shows:
- Mutable state management
- Keyboard input (up/down arrows)
- State updates triggering re-renders
- Event pattern matching

Perfect for: Learning state management

```bash
cargo run --example counter
```

Controls:
- ↑ : Increment
- ↓ : Decrement
- q : Quit

---

### text_input.rs
**Simple text input field**

Shows:
- Character input handling
- String building and manipulation
- Backspace and Enter key handling
- Input validation patterns

Perfect for: Building forms and input fields

```bash
cargo run --example text_input
```

Controls:
- Type to add characters
- Backspace to delete
- Enter to clear
- q : Quit

---

### list_selector.rs
**Navigable list with selection**

Shows:
- List rendering with multiple items
- Selection state tracking
- Conditional styling (selected vs unselected)
- Bounds checking for navigation
- Visual feedback with highlighting

Perfect for: Menus, file browsers, selection UIs

```bash
cargo run --example list_selector
```

Controls:
- ↑/↓ : Navigate
- q : Quit

---

### layout_demo.rs
**Complex multi-panel layout**

Shows:
- Vertical and horizontal layouts
- Nested layout composition
- Fixed and flexible sizing
- Header/body/footer pattern
- Multiple panels with borders

Perfect for: Complex UIs, dashboards

```bash
cargo run --example layout_demo
```

Controls:
- q : Quit

---

### tabs.rs
**Tab navigation between views**

Shows:
- Tab widget usage
- View switching logic
- Dynamic content based on selection
- Multiple navigation methods
- Tab highlighting

Perfect for: Multi-view applications

```bash
cargo run --example tabs
```

Controls:
- ← / → : Switch tabs
- Tab : Next tab
- q : Quit

---

### progress.rs
**Animated progress bar**

Shows:
- Gauge widget for progress display
- Time-based state updates
- Pause/resume functionality
- Continuous animation
- State control (paused flag)

Perfect for: Loading screens, task progress, dashboards

```bash
cargo run --example progress
```

Controls:
- Space : Pause/Resume
- r : Reset
- q : Quit

---

## Learning Path

### For Beginners
1. Start with `hello_world` - understand the basics
2. Move to `counter` - learn state management
3. Try `text_input` - handle user input

### For Intermediate Developers
4. Explore `list_selector` - navigation patterns
5. Study `layout_demo` - complex UIs
6. Check out `tabs` - view switching

### For Advanced Developers
7. Master `progress` - animations and time-based updates

## Common Patterns Demonstrated

### State Management
- **counter.rs**: Simple integer state
- **text_input.rs**: String state
- **list_selector.rs**: Collection and selection state
- **tabs.rs**: View state
- **progress.rs**: Time-based state

### Event Handling
- **hello_world.rs**: Basic quit handling
- **counter.rs**: Arrow key navigation
- **text_input.rs**: Character input
- **list_selector.rs**: List navigation
- **tabs.rs**: Multiple navigation methods
- **progress.rs**: Toggle and reset actions

### Layout Techniques
- **hello_world.rs**: Single widget
- **counter.rs**: Centered content
- **layout_demo.rs**: Multi-panel with nesting
- **tabs.rs**: Header/body/footer
- **progress.rs**: Vertical stack

### Styling
- **hello_world.rs**: Basic colors and borders
- **list_selector.rs**: Conditional styling
- **tabs.rs**: Highlight styles
- **progress.rs**: Mode-based styling

## Using Examples as Starting Points

After exploring these examples, pick one as your starting point:

```bash
# Copy an example to src/main.rs
cp examples/counter.rs src/main.rs

# Run your app
cargo run
```

Then you can:

1. **Customize**: Modify the copied example for your needs
2. **Mix and Match**: Combine patterns from different examples
3. **Extend**: Add features incrementally
4. **Clean Up**: Remove the examples/ directory when done

### Example Combinations

**File Browser**:
- List navigation from `list_selector.rs`
- Multi-panel layout from `layout_demo.rs`
- Status bar from `progress.rs`

**Settings UI**:
- Tab navigation from `tabs.rs`
- Text input from `text_input.rs`
- List selection from `list_selector.rs`

**Task Manager**:
- List from `list_selector.rs`
- Progress bars from `progress.rs`
- Layout from `layout_demo.rs`

## Tips

1. **Start Simple**: Begin with the simplest example that has what you need
2. **Read the Code**: All examples are heavily commented
3. **Experiment**: Modify examples to see what happens
4. **Build Incrementally**: Add features one at a time
5. **Test Often**: Run your app frequently to catch issues early

## Next Steps

1. **Pick an example** closest to what you want to build
2. **Copy it**: `cp examples/counter.rs src/main.rs`
3. **Run it**: `cargo run`
4. **Customize** for your needs
5. **Clean up** when ready: `rm -rf examples/ src/examples/`

## Resources

- [QUICKSTART.md](../QUICKSTART.md) - Code patterns
- [CHEATSHEET.md](../CHEATSHEET.md) - Quick reference
- [Ratatui Docs](https://ratatui.rs/) - Widget documentation

Happy coding! 🚀
