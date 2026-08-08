//! Top-level frame assembly. Deliberately takes a plain data struct
//! ([`ScreenState`]) rather than [`crate::app::app::App`] directly, so the
//! UI stays decoupled from app/terminal/event-loop concerns and can be
//! exercised without a real terminal.

use crate::ui::{layout, theme, widgets};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub struct ScreenState<'a> {
    pub tick_count: u64,
    pub log_entries: &'a [String],
    pub command_input: &'a str,
}

pub fn draw(frame: &mut Frame, state: &ScreenState) {
    let regions = layout::compute(frame.size());

    frame.render_widget(widgets::pfd::widget(), regions.pfd);
    frame.render_widget(engine_panel(), regions.engine);
    frame.render_widget(aircraft_status_panel(state.tick_count), regions.aircraft_status);

    frame.render_widget(atc_panel(), regions.atc);
    frame.render_widget(weather_panel(), regions.weather);
    frame.render_widget(widgets::fo_queue::widget(), regions.fo_queue);

    frame.render_widget(widgets::log::widget(state.log_entries.iter()), regions.log);
    frame.render_widget(widgets::command_line::widget(state.command_input), regions.command_line);
}

fn engine_panel() -> Paragraph<'static> {
    widgets::placeholder("ENGINE", "awaiting aircraft state (issue #6)")
}

fn atc_panel() -> Paragraph<'static> {
    widgets::placeholder("ATC", "awaiting ATC constraint generator (issue #TBD)")
}

fn weather_panel() -> Paragraph<'static> {
    widgets::placeholder("WEATHER", "awaiting weather model (issue #TBD)")
}

/// The one "instrument" panel that's honest to show today: the sim clock
/// itself is real, unlike the aircraft state the other panels would need.
fn aircraft_status_panel(tick_count: u64) -> Paragraph<'static> {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border())
        .title(Span::styled("AIRCRAFT STATUS", theme::title()));

    let line = Line::from(vec![
        Span::raw("sim tick: "),
        Span::styled(tick_count.to_string(), theme::status_running()),
    ]);

    Paragraph::new(line).block(block)
}
