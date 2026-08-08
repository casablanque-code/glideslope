//! Top-level frame assembly. Deliberately takes a plain data struct
//! ([`ScreenState`]) rather than [`crate::app::app::App`] directly, so the
//! UI stays decoupled from app/terminal/event-loop concerns and can be
//! exercised without a real terminal.

use crate::aircraft::state::AircraftState;
use crate::ui::{layout, theme, widgets};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub struct ScreenState<'a> {
    pub tick_count: u64,
    pub aircraft: &'a AircraftState,
    pub log_entries: &'a [String],
    pub command_input: &'a str,
}

pub fn draw(frame: &mut Frame, state: &ScreenState) {
    let regions = layout::compute(frame.size());

    frame.render_widget(widgets::pfd::widget(state.aircraft), regions.pfd);
    frame.render_widget(engine_panel(state.aircraft), regions.engine);
    frame.render_widget(
        aircraft_status_panel(state.tick_count, state.aircraft),
        regions.aircraft_status,
    );

    frame.render_widget(atc_panel(), regions.atc);
    frame.render_widget(weather_panel(), regions.weather);
    frame.render_widget(widgets::fo_queue::widget(), regions.fo_queue);

    frame.render_widget(widgets::log::widget(state.log_entries.iter()), regions.log);
    frame.render_widget(widgets::command_line::widget(state.command_input), regions.command_line);
}

/// Only N1 is real -- fuel flow, EGT, N2 aren't modeled (see
/// `aircraft::engines`), so this doesn't show placeholder rows for them.
fn engine_panel(aircraft: &AircraftState) -> Paragraph<'static> {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border())
        .title(Span::styled("ENGINE", theme::title()));

    let line = Line::from(vec![
        Span::raw("N1    "),
        Span::raw(format!("{:>5.1}%", aircraft.engine.n1_percent)),
    ]);

    Paragraph::new(line).block(block)
}

fn atc_panel() -> Paragraph<'static> {
    widgets::placeholder("ATC", "awaiting ATC constraint generator (issue #TBD)")
}

fn weather_panel() -> Paragraph<'static> {
    widgets::placeholder("WEATHER", "awaiting weather model (issue #TBD)")
}

fn aircraft_status_panel(tick_count: u64, aircraft: &AircraftState) -> Paragraph<'static> {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border())
        .title(Span::styled("AIRCRAFT STATUS", theme::title()));

    let lines = vec![
        Line::from(vec![
            Span::raw("sim tick: "),
            Span::styled(tick_count.to_string(), theme::status_running()),
        ]),
        Line::from(vec![
            Span::raw("altitude: "),
            Span::raw(format!("{:.0} ft", aircraft.altitude_ft)),
        ]),
        Line::from(vec![
            Span::raw("heading:  "),
            Span::raw(format!("{:.0}", aircraft.heading_deg)),
        ]),
    ];

    Paragraph::new(lines).block(block)
}
