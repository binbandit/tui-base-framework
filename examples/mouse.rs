//! Mouse input: clicks, drags, and scroll wheel.
//!
//! Mouse capture is off by default (it breaks native text selection), so
//! this example opts in through `TerminalConfig`.
//!
//! Run with: `cargo run --example mouse`

use anyhow::Result;
use std::collections::HashSet;
use tui_base_framework::layout::{Constraint, Layout, Position};
use tui_base_framework::style::{Color, Style};
use tui_base_framework::widgets::{Block, Paragraph};
use tui_base_framework::{
    AppConfig, Component, Context, Event, EventResult, Frame, KeyCode, MouseButton, MouseEventKind,
    Rect, TerminalConfig, run_with_config,
};

const INKS: [Color; 4] = [Color::Cyan, Color::Yellow, Color::Green, Color::Magenta];

struct Paint {
    cells: HashSet<(u16, u16)>,
    ink: usize,
    canvas: Rect,
}

impl Component for Paint {
    type Message = ();

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let [canvas_area, help] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(area);

        let block = Block::bordered().title("Canvas");
        self.canvas = block.inner(canvas_area);
        frame.render_widget(block, canvas_area);

        let ink = Style::default().bg(INKS[self.ink]);
        for &(x, y) in &self.cells {
            if self.canvas.contains(Position::new(x, y)) {
                frame.buffer_mut().set_style(Rect::new(x, y, 1, 1), ink);
            }
        }

        frame.render_widget(
            Paragraph::new(" Click/drag: paint | Right-click: erase | Scroll: change color | c: clear | q: quit")
                .style(Style::default().fg(Color::DarkGray)),
            help,
        );
    }

    fn handle_event(&mut self, event: Event, context: &Context<Self::Message>) -> EventResult {
        match event {
            Event::Mouse(mouse) => {
                let cell = (mouse.column, mouse.row);
                match mouse.kind {
                    MouseEventKind::Down(MouseButton::Left)
                    | MouseEventKind::Drag(MouseButton::Left) => {
                        self.cells.insert(cell);
                        EventResult::Consumed
                    }
                    MouseEventKind::Down(MouseButton::Right)
                    | MouseEventKind::Drag(MouseButton::Right) => {
                        self.cells.remove(&cell);
                        EventResult::Consumed
                    }
                    MouseEventKind::ScrollUp => {
                        self.ink = (self.ink + 1) % INKS.len();
                        EventResult::Consumed
                    }
                    MouseEventKind::ScrollDown => {
                        self.ink = (self.ink + INKS.len() - 1) % INKS.len();
                        EventResult::Consumed
                    }
                    _ => EventResult::Propagate,
                }
            }
            Event::Key(_) if event.is_key(KeyCode::Char('c')) => {
                self.cells.clear();
                EventResult::Consumed
            }
            Event::Key(_) if event.is_key(KeyCode::Char('q')) || event.is_key(KeyCode::Esc) => {
                context.quit();
                EventResult::Consumed
            }
            _ => EventResult::Propagate,
        }
    }
}

fn main() -> Result<()> {
    let config = AppConfig {
        terminal: TerminalConfig {
            mouse_capture: true,
            ..TerminalConfig::default()
        },
        ..AppConfig::default()
    };

    run_with_config(
        Paint {
            cells: HashSet::new(),
            ink: 0,
            canvas: Rect::default(),
        },
        config,
    )
}
