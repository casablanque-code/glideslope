//! Top-level frame assembly. Deliberately takes a plain data struct
//! ([`ScreenState`]) rather than [`crate::app::app::App`] directly, so the
//! UI stays decoupled from app/terminal/event-loop concerns and can be
//! exercised without a real terminal.

use crate::aircraft::state::AircraftState;
use crate::atc::controller::Controller as AtcController;
use crate::core::time::TICKS_PER_SECOND;
use crate::crew::queue::TaskQueue;
use crate::sim::phase::FlightPhase;
use crate::ui::{layout, theme, widgets};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub struct ScreenState<'a> {
    pub tick_count: u64,
    pub aircraft: &'a AircraftState,
    pub phase: FlightPhase,
    pub atc: &'a AtcController,
    pub fo_queue: &'a TaskQueue,
    pub log_entries: &'a [String],
    pub command_input: &'a str,
}

pub fn draw(frame: &mut Frame, state: &ScreenState) {
    let regions = layout::compute(frame.size());

    frame.render_widget(widgets::pfd::widget(state.aircraft), regions.pfd);
    frame.render_widget(engine_panel(state.aircraft), regions.engine);
    frame.render_widget(
        aircraft_status_panel(state.tick_count, state.phase),
        regions.aircraft_status,
    );

    frame.render_widget(atc_panel(state.atc), regions.atc);
    frame.render_widget(weather_panel(), regions.weather);
    frame.render_widget(widgets::fo_queue::widget(state.fo_queue), regions.fo_queue);

    // The log region has a 1-cell border on top and bottom; only that
    // many text rows are actually visible. Paragraph doesn't auto-scroll
    // to the bottom on its own, so without this, once entries exceed the
    // visible height, the *newest* entries -- the ones at the end of the
    // vec -- are the ones that get clipped, which is backwards for a
    // live log.
    let visible_rows = regions.log.height.saturating_sub(2) as usize;
    let start = state.log_entries.len().saturating_sub(visible_rows);
    frame.render_widget(widgets::log::widget(state.log_entries[start..].iter()), regions.log);
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

/// Compact view -- just the clearances that have actually been granted,
/// since the full ALL-clearances-with-status listing (the ATC pseudo-
/// command's job) would overflow this panel's height quickly.
fn atc_panel(atc: &AtcController) -> Paragraph<'static> {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border())
        .title(Span::styled("ATC", theme::title()));

    let granted: Vec<&str> = crate::atc::constraints::ClearanceType::ALL
        .into_iter()
        .filter(|clearance| atc.is_granted(*clearance))
        .map(|clearance| clearance.name())
        .collect();

    let lines: Vec<Line> = if granted.is_empty() {
        vec![Line::styled("no clearances granted", theme::placeholder_text())]
    } else {
        granted.into_iter().map(|name| Line::raw(format!("cleared: {name}"))).collect()
    };

    Paragraph::new(lines).block(block)
}

fn weather_panel() -> Paragraph<'static> {
    widgets::placeholder("WEATHER", "awaiting weather model (issue #TBD)")
}

/// This panel is for aircraft configuration/systems summary (gear,
/// flaps, autobrake, ...) -- none of which exist yet (checklists #8,
/// failures #10). Flight/attitude data already lives on the PFD, so this
/// doesn't duplicate it; elapsed time is shown as clock time rather than
/// a raw tick count (an implementation detail with no business in the
/// UI), and the current flight phase (#13) has a real, live source now.
fn aircraft_status_panel(tick_count: u64, phase: FlightPhase) -> Paragraph<'static> {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border())
        .title(Span::styled("AIRCRAFT STATUS", theme::title()));

    let lines = vec![
        Line::from(vec![
            Span::raw("phase:   "),
            Span::styled(phase.name(), theme::status_running()),
        ]),
        Line::from(vec![
            Span::raw("elapsed: "),
            Span::styled(format_elapsed(tick_count), theme::status_running()),
        ]),
        Line::styled(
            "configuration: awaiting checklists (#8) / failures (#10)",
            theme::placeholder_text(),
        ),
    ];

    Paragraph::new(lines).block(block)
}

fn format_elapsed(tick_count: u64) -> String {
    let total_seconds = tick_count / TICKS_PER_SECOND as u64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn rendered_text(width: u16, height: u16, log_entries: &[String]) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        let aircraft = AircraftState::cruise();
        let fo_queue = TaskQueue::new();
        let atc = AtcController::new();
        let state = ScreenState {
            tick_count: 0,
            aircraft: &aircraft,
            phase: FlightPhase::Ground,
            atc: &atc,
            fo_queue: &fo_queue,
            log_entries,
            command_input: "",
        };

        terminal.draw(|frame| draw(frame, &state)).unwrap();

        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<Vec<_>>()
            .join("")
    }

    #[test]
    fn format_elapsed_renders_hh_mm_ss() {
        assert_eq!(format_elapsed(0), "00:00:00");
        assert_eq!(format_elapsed(TICKS_PER_SECOND as u64 * 90), "00:01:30");
        assert_eq!(format_elapsed(TICKS_PER_SECOND as u64 * 3661), "01:01:01");
    }

    #[test]
    fn log_shows_the_most_recent_entries_when_overflowing_the_visible_area() {
        // Small terminal -> the log region only fits a couple of rows.
        // With more entries than that, the tail (newest) must still be
        // visible -- the old bug clipped the newest entries instead.
        let entries: Vec<String> = (0..50).map(|i| format!("entry-{i}")).collect();
        let text = rendered_text(80, 24, &entries);

        assert!(text.contains("entry-49"), "newest entry should be visible");
        assert!(!text.contains("entry-0 "), "oldest entry should have scrolled off");
    }

    #[test]
    fn log_shows_everything_when_it_fits() {
        let entries: Vec<String> = vec!["only entry".to_string()];
        let text = rendered_text(80, 24, &entries);
        assert!(text.contains("only entry"));
    }
}
