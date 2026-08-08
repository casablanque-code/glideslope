//! Primary Flight Display panel.
//!
//! Stubbed until issue #6 (aircraft state) exists -- there is no attitude,
//! airspeed, or altitude to show yet. This will likely compose
//! [`super::altimeter`] and [`super::vsi`] once real data exists.

use ratatui::widgets::Paragraph;

pub fn widget() -> Paragraph<'static> {
    super::placeholder("PFD", "awaiting aircraft state (issue #6)")
}
