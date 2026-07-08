# Examples

Runnable examples for the template. Each one is a single self-contained file — component plus `main` — so you can read it top to bottom and copy it straight over `src/main.rs`.

## Run

The template has a default binary too:

```bash
cargo run
```

Run a specific example:

```bash
cargo run --example hello_world
cargo run --example counter
cargo run --example text_input
cargo run --example list_selector
cargo run --example layout_demo
cargo run --example tabs
cargo run --example progress
cargo run --example async_task
cargo run --example focus
cargo run --example mouse
```

All examples support `q` to quit. Ctrl-C also exits through the framework default.

## Reference

| Example | What it shows | Controls |
| --- | --- | --- |
| `hello_world` | Basic render and quit handling | `q` |
| `counter` | State updates from keyboard events | Up/down, `q` |
| `text_input` | Character input, backspace, enter, paste | Type, Backspace, Enter, `q` |
| `list_selector` | Stateful `List` widget with `ListState` | Up/down, `q` |
| `layout_demo` | Header/body/footer and nested layout splits | `q` |
| `tabs` | View switching with Ratatui tabs | Left/right, Tab, `q` |
| `progress` | Tick-driven updates and a custom tick rate | Space, `r`, `q` |
| `async_task` | Background Tokio task + typed messages | `s`, `q` |
| `focus` | Multi-component composition and focus routing | Tab, up/down, `q` |
| `mouse` | Mouse capture: click, drag, scroll | Mouse, `c`, `q` |

## Learning Path

1. Start with `hello_world` to see the minimum component shape.
2. Move to `counter` for state and keyboard input.
3. Use `text_input` for text editing and paste handling.
4. Study `list_selector` for stateful widgets (`ListState`).
5. Use `layout_demo` when you need multiple panels.
6. Use `tabs` for view switching.
7. Use `progress` for tick-based updates.
8. Move to `async_task` for background work and typed messages — the reason Tokio is here.
9. Finish with `focus` to see how components compose once your app outgrows one screen; `mouse` if you need pointer input.

## Use an Example as Your App

```bash
cp examples/counter.rs src/main.rs
cargo run
```

Examples are self-contained, so the copy works as-is. Rename the struct and start replacing state and rendering with your own.

## Clean Up

When you no longer need the examples:

```bash
rm -rf examples/
```

Nothing else references the directory — Cargo discovers examples automatically, so there is no configuration to update.
