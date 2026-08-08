//! Pure layout math: turns a terminal `Rect` into the named regions the
//! roadmap's ASCII mockup describes. No rendering happens here — this
//! only computes rectangles, which keeps it trivially testable without a
//! terminal backend.
//!
//! ```text
//! +-----------------------------------------------------------+
//! | PFD             ENGINE             AIRCRAFT STATUS         |
//! +-----------------------------------------------------------+
//! | ATC             WEATHER            FO TASK QUEUE           |
//! +-----------------------------------------------------------+
//! | LOG / WARNINGS                                        |
//! +-----------------------------------------------------------+
//! | Command >                                                |
//! +-----------------------------------------------------------+
//! ```

use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct AppLayout {
    pub pfd: Rect,
    pub engine: Rect,
    pub aircraft_status: Rect,
    pub atc: Rect,
    pub weather: Rect,
    pub fo_queue: Rect,
    pub log: Rect,
    pub command_line: Rect,
}

pub fn compute(area: Rect) -> AppLayout {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // instrument row
            Constraint::Length(8), // ATC/weather/FO row
            Constraint::Min(3),    // log, takes remaining space
            Constraint::Length(3), // command line
        ])
        .split(area);

    let instrument_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(1, 3), Constraint::Ratio(1, 3)])
        .split(rows[0]);

    let middle_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(1, 3), Constraint::Ratio(1, 3)])
        .split(rows[1]);

    AppLayout {
        pfd: instrument_row[0],
        engine: instrument_row[1],
        aircraft_status: instrument_row[2],
        atc: middle_row[0],
        weather: middle_row[1],
        fo_queue: middle_row[2],
        log: rows[2],
        command_line: rows[3],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regions_tile_the_full_area_without_gaps_or_overlap() {
        let area = Rect::new(0, 0, 120, 40);
        let layout = compute(area);

        // Every region's total area should sum to the parent area -- if
        // rows/columns overlapped or left gaps this would drift.
        let total = layout.pfd.area()
            + layout.engine.area()
            + layout.aircraft_status.area()
            + layout.atc.area()
            + layout.weather.area()
            + layout.fo_queue.area()
            + layout.log.area()
            + layout.command_line.area();
        assert_eq!(total, area.area());
    }

    #[test]
    fn command_line_is_pinned_to_the_bottom() {
        let area = Rect::new(0, 0, 120, 40);
        let layout = compute(area);
        assert_eq!(layout.command_line.y + layout.command_line.height, area.height);
    }
}
