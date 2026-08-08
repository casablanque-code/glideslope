pub mod altimeter;
pub mod command_line;
pub mod fo_queue;
pub mod log;
pub mod pfd;
pub mod vsi;

use crate::ui::theme;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

/// Shared shape for panels that don't have real data to show yet.
/// Renders a bordered, titled block with a note on what issue will fill
/// it in -- so the shell is honest about what's live versus stubbed,
/// instead of showing invented numbers.
pub fn placeholder(title: &str, note: &str) -> Paragraph<'static> {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border())
        .title(Span::styled(title.to_string(), theme::title()));

    Paragraph::new(Line::styled(note.to_string(), theme::placeholder_text())).block(block)
}
