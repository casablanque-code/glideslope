//! Altimeter tape. Stubbed until issue #6 (aircraft state) provides a
//! real altitude to render.

use ratatui::widgets::Paragraph;

#[allow(dead_code)] // not composed into pfd::widget() yet -- pfd itself is still a placeholder
pub fn widget() -> Paragraph<'static> {
    super::placeholder("ALT", "awaiting aircraft state (issue #6)")
}
