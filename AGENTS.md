# AGENTS.md

Guidance for AI coding agents (and new contributors) working in this repository.

<!-- template-only:start -->
## What this repository is

This is a **template repository**, not a published library. People clone it, run
`./setup.sh <name>`, and ship their own app from it. That shapes every change you make:

- The audience is someone reading this code for the first time. Prefer clarity over
  cleverness; keep doc comments accurate.
- `setup.sh` mechanically rewrites the project (rename, strip, fold). Several
  invariants below exist only so that machinery keeps working. Breaking them breaks
  every future clone, not this repo's CI alone.

## Template invariants — do not break these

1. **`src/tui/` is self-contained.** Framework code under `src/tui/` must only import
   from `std`, dependencies, and `crate::tui::...` — never from `main.rs`, `lib.rs`, or
   anything outside the module. This is what lets `setup.sh --app-only` move it into a
   binary crate unchanged.
2. **App code imports only through the crate root.** `src/main.rs` and `examples/*.rs`
   import `tui_base_framework::...` (never `tui_base_framework::tui::...`), so the
   `--app-only` rewrite to `crate::tui::...` stays a pure string substitution.
3. **`setup.sh` renames by literal string replacement.** The strings
   `tui-base-framework` and `tui_base_framework` are find-and-replaced across
   `src/`, `examples/`, `Cargo.toml`, `Cargo.lock`, and `*.md`. Don't split, wrap, or
   re-spell them in ways that would dodge the substitution.
4. **`setup.sh` must run on both GNU and BSD userlands.** macOS ships BSD `sed`,
   `awk`, and `find`. No GNU-only flags or extensions (e.g. `sed 0,/re/` addresses or
   `\n` in `s///` replacements — a real bug we shipped once). CI runs the script on
   both `ubuntu-latest` and `macos-latest`, and shellchecks it.
5. **`src/main.rs` must keep at least one top-level `use ` line.** `--app-only`
   inserts `mod tui;` immediately before the first line matching `^use `.
6. **Marker blocks are load-bearing.** The `# --- template setup ---` comment block in
   `Cargo.toml` and the `<!-- template-only:start/end -->` blocks in this file are
   deleted by `setup.sh`. Keep template-specific prose inside them; keep everything a
   generated app still needs outside them.
7. **Examples are single self-contained files.** Each `examples/*.rs` contains a
   component plus `main` and works when copied over `src/main.rs`. Adding one means
   updating the tables in `examples/README.md` and `README.md`.
8. **The MSRV lives in `Cargo.toml`.** `rust-version` is the source of truth and the
   CI `msrv` job reads it from there. When bumping it, update the mention in
   `README.md` (Dependencies section) — nothing else hardcodes it.

## Checks to run before committing

CI (`.github/workflows/ci.yml`) runs, in order:

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo test --all-targets
cargo test --doc
cargo clippy --all-targets --all-features -- -D warnings
cargo doc --no-deps
shellcheck setup.sh
```

Plus an MSRV check and a matrix that actually executes `./setup.sh` in both modes on
Linux and macOS. If you change `setup.sh`, test both modes yourself in a scratch copy
of the repo (the script deletes itself and mutates the tree — never run it in place):

```bash
cp -r . /tmp/scratch && cd /tmp/scratch
./setup.sh my-test-app --yes            # rename mode
# ...and again in a fresh copy:
./setup.sh my-test-app --app-only --yes # binary-only mode
cargo test --all-targets
```

## Documentation to keep in sync

A change rarely touches just code. Check whether it also affects: `README.md`
(concepts, tables, dependency/MSRV notes), `CHEATSHEET.md` (copy-paste snippets — they
must compile against the template as-is), `examples/README.md` (example tables and
controls), and the doc comments in `src/`.
<!-- template-only:end -->

## Project layout

```text
src/tui/             The framework (self-contained module)
  mod.rs             Re-exports: everything apps import
  app.rs             App loop, AppConfig, run() / run_with_config()
  component.rs       Component trait and Context
  event.rs           Event and EventResult types
  terminal.rs        TerminalGuard (raw mode, alt screen, panic hook)
src/lib.rs           Thin re-export of src/tui/ (absent in --app-only projects)
src/main.rs          The application
examples/            Self-contained runnable examples (optional)
```

## Architecture in one paragraph

An app is one `Component`: it owns state, draws in `render`, reacts to terminal input
in `handle_event`, and reacts to typed messages in `update`. The `App` loop (in
`src/tui/app.rs`) owns the terminal, pumps crossterm input from a blocking task, fires
`Event::Tick` on an interval, and delivers messages sent through `Context`. Rendering
is event-driven: a redraw happens only when `handle_event` returns
`EventResult::Consumed`, a message arrives, or the terminal resizes — and queued
events are drained first so input bursts coalesce into one redraw. Background work is
plain `tokio::spawn`: move `context.sender()` into the task and `send` a message back;
never block inside `handle_event` or `render`.

Key behaviors to remember when editing or debugging:

- `Context::quit()` latches and wakes the loop; it can't be lost even when channels
  are full.
- Key **release** events are filtered out before components see them; `Event::Key` is
  always a press.
- Stale `Tick` events are dropped when the UI is busy rather than queued.
- `TerminalGuard` restores the terminal on drop **and** via a panic hook, so panics
  print readably. Don't add early exits that bypass it (e.g. `std::process::exit`).
- Mouse capture and focus events are opt-in via `TerminalConfig`; bracketed paste is
  on by default.

## Testing patterns

Components are plain structs and test without a terminal. The three building blocks,
all re-exported from the crate root:

```rust
let (context, mut messages) = Context::test();          // context + message receiver
let event = Event::key_press(KeyCode::Char('q'));       // fabricate input
let mut terminal = Terminal::new(TestBackend::new(60, 12)).unwrap(); // render checks
```

Assert on `context.quit_requested()`, on `messages.try_recv()`, and on the
`TestBackend` buffer contents. `src/main.rs` ships working tests in this style — copy
them for new components. Async `update`-path behavior can be tested with
`#[tokio::test]`.

## Style

- Rust 2024 edition; `unsafe_code = "forbid"` is enforced via `[lints.rust]`.
- Clippy runs with `-D warnings`; fix lints rather than `#[allow]`-ing them unless
  there's a documented reason.
- Write doc comments for anything `pub` — `cargo doc --no-deps` must stay warning-free.
- Match the existing comment style: explain *why* (constraints, invariants), not what
  the next line does.
