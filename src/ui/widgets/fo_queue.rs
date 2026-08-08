//! First Officer task queue panel, matching the roadmap's mockup:
//!
//! ```text
//! [Executing]
//! Read QRH
//! 10 sec remaining
//! ------------------------
//! [Pending]
//! Readback
//! ```
//!
//! Real as of issue #7: shows whatever's actually in `crew::queue::TaskQueue`.
//! There's no way to interrupt the executing task from here yet (no
//! command for it -- see the note in `crew::queue`).

use crate::crew::queue::TaskQueue;
use crate::ui::theme;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn widget(queue: &TaskQueue) -> Paragraph<'static> {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border())
        .title(Span::styled("FO TASK QUEUE", theme::title()));

    let mut lines = Vec::new();

    match queue.executing() {
        Some(executing) => {
            lines.push(Line::styled("[Executing]", theme::title()));
            lines.push(Line::raw(executing.task.description.clone()));
            lines.push(Line::raw(format!("{} sec remaining", executing.remaining.as_secs())));
        }
        None => {
            lines.push(Line::styled("[Idle]", theme::placeholder_text()));
        }
    }

    let pending: Vec<&str> = queue.pending().map(|task| task.description.as_str()).collect();
    if !pending.is_empty() {
        lines.push(Line::raw("------------------------"));
        lines.push(Line::styled("[Pending]", theme::title()));
        for description in pending {
            lines.push(Line::raw(description.to_string()));
        }
    }

    Paragraph::new(lines).block(block)
}
