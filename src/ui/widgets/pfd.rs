//! Primary Flight Display panel.
//!
//! Real numbers as of issue #6, but still a plain text readout, not an
//! actual attitude/speed-tape rendering -- that's a later UI polish pass.
//! The pitch->vertical-speed coupling behind these numbers is a simplified
//! placeholder (see `aircraft::state`), not real performance modeling.

use crate::aircraft::state::AircraftState;
use crate::ui::theme;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn widget(aircraft: &AircraftState) -> Paragraph<'static> {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border())
        .title(Span::styled("PFD", theme::title()));

    let lines = vec![
        Line::from(vec![Span::raw("HDG   "), Span::raw(format!("{:>3.0}", aircraft.heading_deg))]),
        Line::from(vec![
            Span::raw("IAS   "),
            Span::raw(format!("{:>3.0} kt", aircraft.indicated_airspeed_kt)),
        ]),
        super::altimeter::line(aircraft.altitude_ft),
        super::vsi::line(aircraft.vertical_speed_fpm),
        Line::from(vec![Span::raw("PITCH "), Span::raw(format!("{:>+5.1}", aircraft.pitch_deg))]),
        Line::from(vec![Span::raw("BANK  "), Span::raw(format!("{:>+5.1}", aircraft.bank_deg))]),
    ];

    Paragraph::new(lines).block(block)
}
