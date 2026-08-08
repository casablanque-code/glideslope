//! Command input line.
//!
//! Only captures and echoes raw text for now -- there is no grammar to
//! validate against until issue #4 (command parser) lands, so this
//! deliberately does not pretend to interpret what's typed.

use crate::ui::theme;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn widget(input: &str) -> Paragraph<'_> {
    let block = Block::default().borders(Borders::ALL).border_style(theme::border());

    let line = Line::from(vec![
        Span::styled("Command > ", theme::command_prompt()),
        Span::raw(input),
        Span::styled("_", theme::command_prompt()), // cursor
    ]);

    Paragraph::new(line).block(block)
}
