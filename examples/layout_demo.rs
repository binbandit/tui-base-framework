//! Layout composition: header/body/footer with a split body.
//!
//! Run with: `cargo run --example layout_demo`

use anyhow::Result;
use tui_base_framework::layout::{Alignment, Constraint, Layout};
use tui_base_framework::style::{Color, Style};
use tui_base_framework::widgets::{Block, Paragraph};
use tui_base_framework::{Component, Context, Event, EventResult, Frame, KeyCode, Rect, run};

struct LayoutDemo;

impl Component for LayoutDemo {
    type Message = ();

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let [header, body, footer] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .areas(area);

        let [left, right] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(body);

        frame.render_widget(
            Paragraph::new("Layout Demo - Multiple Panels")
                .block(Block::bordered())
                .style(Style::default().fg(Color::Cyan))
                .alignment(Alignment::Center),
            header,
        );

        frame.render_widget(
            Paragraph::new("Left Panel\n\nThis demonstrates\nhorizontal layout\nsplitting")
                .block(Block::bordered().title("Left"))
                .style(Style::default().fg(Color::Green)),
            left,
        );

        frame.render_widget(
            Paragraph::new("Right Panel\n\nYou can nest\nlayouts to create\ncomplex UIs")
                .block(Block::bordered().title("Right"))
                .style(Style::default().fg(Color::Yellow)),
            right,
        );

        frame.render_widget(
            Paragraph::new("Press 'q' to quit")
                .block(Block::bordered())
                .style(Style::default().fg(Color::Magenta))
                .alignment(Alignment::Center),
            footer,
        );
    }

    fn handle_event(&mut self, event: Event, context: &Context<Self::Message>) -> EventResult {
        if event.is_key(KeyCode::Char('q')) || event.is_key(KeyCode::Esc) {
            context.quit();
            return EventResult::Consumed;
        }

        EventResult::Propagate
    }
}

fn main() -> Result<()> {
    run(LayoutDemo)
}
