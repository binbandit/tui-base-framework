# Examples

Runnable examples for the template. Each binary wires one reusable component from `src/examples/` into `App`.

## Run

```bash
cargo run --example hello_world
cargo run --example counter
cargo run --example text_input
cargo run --example list_selector
cargo run --example layout_demo
cargo run --example tabs
cargo run --example progress
```

All examples support `q` to quit. Ctrl-C also exits through the framework default.

## Reference

| Example | What it shows | Controls |
| --- | --- | --- |
| `hello_world` | Basic render and quit handling | `q` |
| `counter` | State updates from keyboard events | Up/down, `q` |
| `text_input` | Character input, backspace, enter, paste | Type, Backspace, Enter, `q` |
| `list_selector` | Bounded list navigation and selected styling | Up/down, `q` |
| `layout_demo` | Header/body/footer and nested layout splits | `q` |
| `tabs` | View switching with Ratatui tabs | Left/right, Tab, `q` |
| `progress` | Tick-driven progress updates | Space, `r`, `q` |

## Learning Path

1. Start with `hello_world` to see the minimum component shape.
2. Move to `counter` for state and keyboard input.
3. Use `text_input` for text editing and paste handling.
4. Study `list_selector` for safe indexed navigation.
5. Use `layout_demo` when you need multiple panels.
6. Use `tabs` for view switching.
7. Use `progress` for tick-based updates.

## Use an Example as Your App

```bash
cp examples/counter.rs src/main.rs
cargo run
```

After copying an example, replace the imported example component with your own component and keep the `App::new(component)` wiring.

## Clean Up

When you no longer need the examples:

```bash
rm -rf examples/ src/examples/ src/examples.rs
```

Then remove `pub mod examples;` from `src/lib.rs`.
