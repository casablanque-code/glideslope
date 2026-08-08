//! First Officer task queue panel. Stubbed until issue #7 (FO task queue)
//! exists -- there are no tasks to show yet.

use ratatui::widgets::Paragraph;

pub fn widget() -> Paragraph<'static> {
    super::placeholder("FO TASK QUEUE", "awaiting FO task queue (issue #7)")
}
