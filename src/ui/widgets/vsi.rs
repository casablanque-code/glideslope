//! Vertical speed readout line, composed into [`super::pfd`] rather than
//! shown as its own panel -- see the note in `altimeter.rs`.

use ratatui::text::{Line, Span};

pub fn line(vertical_speed_fpm: f64) -> Line<'static> {
    Line::from(vec![Span::raw("V/S   "), Span::raw(format!("{vertical_speed_fpm:>+6.0} fpm"))])
}
