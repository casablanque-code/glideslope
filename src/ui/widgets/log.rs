//! Scrolling event/warning log. The only instrument-row-adjacent widget
//! that's fully real right now: it renders whatever the event bus has
//! actually published, oldest first, most recent at the bottom.

use crate::ui::theme;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn widget<'a>(entries: impl Iterator<Item = &'a String>) -> Paragraph<'a> {
    let lines: Vec<Line> =
        entries.map(|entry| Line::from(Span::styled(entry.as_str(), theme::log_text()))).collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border())
        .title(Span::styled("LOG / WARNINGS", theme::title()));

    Paragraph::new(lines).block(block)
}
