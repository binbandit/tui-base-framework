#!/usr/bin/env python3
"""Exercise the real app loop in a pseudo-terminal (Python stdlib, Unix only)."""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

if os.name != "posix":
    print("Runtime PTY checks require Unix; skipping.")
    raise SystemExit(0)

import errno
import fcntl
import pty
import select
import struct
import termios
import time

ROOT = Path(__file__).resolve().parents[1]
PROBE = r'''
use std::{io::Read, sync::Arc, sync::atomic::{AtomicUsize, Ordering}, time::Duration};
use tui_base_framework::{App, AppConfig, Component, Context, Event, EventResult, Frame,
                         KeyCode, Rect, TerminalConfig, Viewport, widgets::Paragraph};

struct Probe {
    mode: String,
    value: usize,
    starts: Arc<AtomicUsize>,
}

impl Component for Probe {
    type Message = ();

    fn init(&mut self, context: &Context<()>) {
        self.starts.fetch_add(1, Ordering::Relaxed);
        if matches!(self.mode.as_str(), "flood" | "ticks") {
            context.try_send(()).unwrap();
        } else if self.mode == "fatal" {
            context.fail(anyhow::anyhow!("expected initialization failure"));
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        assert_ne!(self.mode, "fatal", "fatal initialization must not render");
        eprintln!("DRAW {}", self.value);
        frame.render_widget(Paragraph::new(format!("progress {}%", self.value)), area);
    }

    fn update(&mut self, _: (), context: &Context<()>) {
        self.value += 1;
        if self.mode == "flood" {
            context.try_send(()).unwrap();
        } else {
            // The tick delivered after this busy period must include it.
            std::thread::sleep(Duration::from_millis(120));
        }
    }

    fn handle_event(&mut self, event: Event, context: &Context<()>) -> EventResult {
        if event.is_key(KeyCode::Char('q')) {
            context.quit();
            return EventResult::Consumed;
        }
        if let Event::Tick(elapsed) = event {
            if self.mode == "inline" {
                self.value += 25;
                if self.value == 100 {
                    context.quit();
                }
                return EventResult::Consumed;
            }
            if self.mode == "ticks" {
                eprintln!("ELAPSED {}", elapsed.as_secs_f64());
                context.quit();
                return EventResult::Consumed;
            }
        }
        EventResult::Propagate
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let mode = std::env::args().nth(1).unwrap();
    let starts = Arc::new(AtomicUsize::new(0));
    let config = AppConfig {
        tick_rate: Duration::from_millis(10),
        terminal: TerminalConfig {
            viewport: if mode == "inline" { Viewport::Inline(3) } else { Viewport::Fullscreen },
            ..TerminalConfig::default()
        },
        ..AppConfig::default()
    };
    let mut app = App::with_config(Probe { mode: mode.clone(), value: 0, starts: starts.clone() }, config)?;
    if mode == "cancel" {
        for _ in 0..5 {
            tokio::select! {
                result = app.run() => panic!("run ended before cancellation: {result:?}"),
                () = tokio::time::sleep(Duration::from_millis(20)) => {}
            }
        }
        assert_eq!(starts.load(Ordering::Relaxed), 5);
        eprintln!("CANCELLED");
    }
    let result = app.run().await;
    if mode == "fatal" {
        assert_eq!(result.unwrap_err().to_string(), "expected initialization failure");
    } else {
        result?;
    }
    drop(app);
    eprintln!("DONE");
    // Keep the PTY alive while the parent verifies restoration; macOS may
    // reject termios queries after the controlling process has exited.
    std::io::stdin().read_exact(&mut [0_u8])?;
    Ok(())
}
'''


def acquire_terminal():
    """Run in the child after stdin has been connected to the PTY slave."""
    os.setsid()
    fcntl.ioctl(0, termios.TIOCSCTTY, 0)


def check_case(binary, mode):
    master, slave = pty.openpty()
    original = termios.tcgetattr(slave)
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
    process = subprocess.Popen(
        [str(binary), mode], stdin=slave, stdout=slave, stderr=subprocess.PIPE,
        preexec_fn=acquire_terminal,
        env={**os.environ, "TERM": "xterm-256color"},
    )
    logs = bytearray()
    output_tail = b""
    sent_quit = False
    resized = False
    restored = False
    deadline = time.monotonic() + 10
    try:
        while process.poll() is None:
            if time.monotonic() > deadline:
                raise AssertionError(f"{mode}: app stopped making progress: {logs[-2000:]!r}")
            for descriptor in select.select([master, process.stderr], [], [], 0.1)[0]:
                try:
                    chunk = os.read(descriptor if isinstance(descriptor, int) else descriptor.fileno(), 65536)
                except OSError as error:
                    if error.errno == errno.EIO:
                        continue
                    raise
                if descriptor == master:
                    # An actual terminal replies to cursor-position queries.
                    output_tail += chunk
                    queries = output_tail.count(b"\x1b[6n")
                    if queries:
                        os.write(master, b"\x1b[1;1R" * queries)
                        output_tail = output_tail.rsplit(b"\x1b[6n", 1)[1]
                    output_tail = output_tail[-3:]
                else:
                    logs.extend(chunk)
            if b"DONE\n" in logs and not restored:
                assert termios.tcgetattr(slave) == original, f"{mode}: terminal settings were not restored"
                restored = True
                os.write(master, b"\n")
            if mode == "flood" and not sent_quit and logs.count(b"DRAW ") >= 3:
                os.write(master, b"q")
                sent_quit = True
            if mode == "cancel" and not sent_quit and b"CANCELLED\n" in logs:
                os.write(master, b"q")
                sent_quit = True
            if mode == "inline" and not resized and b"DRAW 0\n" in logs:
                fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 100, 0, 0))
                resized = True
        logs.extend(process.stderr.read())
        assert process.returncode == 0, f"{mode}: {logs.decode(errors='replace')}"
        assert b"DONE\n" in logs, f"{mode}: missing completion marker"
        assert restored, f"{mode}: restoration was not verified"
        if mode == "inline":
            assert b"DRAW 100\n" in logs, "inline: final completed state was not rendered"
            assert resized, "inline: resize path was not exercised"
        if mode == "ticks":
            elapsed = next(float(line.split()[1]) for line in logs.splitlines() if line.startswith(b"ELAPSED "))
            assert elapsed >= 0.1, f"ticks: delivered stale elapsed time {elapsed}"
        print(f"PASS {mode}")
    finally:
        if process.poll() is None:
            process.kill()
            process.wait()
        process.stderr.close()
        os.close(master)
        os.close(slave)


def main():
    subprocess.run(
        ["cargo", "fetch", "--quiet", "--locked", "--manifest-path", str(ROOT / "Cargo.toml")],
        check=True,
    )
    with tempfile.TemporaryDirectory(prefix="tui-runtime-test-") as directory:
        project = Path(directory)
        (project / "src").mkdir()
        (project / "src/main.rs").write_text(PROBE)
        root_path = json.dumps(str(ROOT))
        (project / "Cargo.toml").write_text(
            '[package]\nname = "runtime-probe"\nversion = "0.0.0"\nedition = "2024"\n'
            f"[dependencies]\ntui-base-framework = {{ path = {root_path} }}\n"
            'tokio = { version = "1", features = ["macros", "rt-multi-thread", "time"] }\n'
            'anyhow = "1"\n'
        )
        (project / "Cargo.lock").write_bytes((ROOT / "Cargo.lock").read_bytes())
        target = ROOT / "target/runtime-tests"
        subprocess.run(
            ["cargo", "build", "--quiet", "--offline", "--manifest-path", str(project / "Cargo.toml"),
             "--target-dir", str(target)], check=True,
        )
        for mode in ("flood", "ticks", "inline", "cancel", "fatal"):
            check_case(target / "debug/runtime-probe", mode)


if __name__ == "__main__":
    main()
