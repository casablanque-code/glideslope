//! Altitude readout line, composed into [`super::pfd`] rather than shown
//! as its own panel -- the layout only allocates one region for the PFD,
//! matching the roadmap's mockup.

use ratatui::text::{Line, Span};

pub fn line(altitude_ft: f64) -> Line<'static> {
    Line::from(vec![Span::raw("ALT   "), Span::raw(format!("{altitude_ft:>6.0} ft"))])
}
