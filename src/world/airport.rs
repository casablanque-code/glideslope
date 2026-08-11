//! A single fictional airport, matching the roadmap's MVP scope
//! ("one airport, one runway"). No taxiway network, no navigation
//! database -- see DESIGN.md's taxi-abstraction decision, which applies
//! here too: this exists to (a) give the aircraft a believable starting
//! point on the ground, and (b) give later issues (ILS #9, ATC #12) a
//! runway to reference. It is not a full-fidelity airport model.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Runway {
    /// Magnetic heading, degrees. Also identifies the runway --
    /// see `designator()`.
    pub heading_deg: f64,
    pub length_ft: f64,
    /// Elevation of the runway threshold, feet MSL.
    pub elevation_ft: f64,
}

impl Runway {
    /// Conventional runway designator derived from heading (e.g. 093deg
    /// -> "09"), the way real runways are numbered and labeled on the
    /// pavement -- nearest 10 degrees, wrapped into 01..=36.
    pub fn designator(&self) -> String {
        let rounded = (self.heading_deg / 10.0).round() as i64;
        let number = rounded.rem_euclid(36);
        let number = if number == 0 { 36 } else { number };
        format!("{number:02}")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Airport {
    pub name: String,
    /// ICAO-style identifier. Uses the Z-prefix ICAO reserves for
    /// unlisted/fictional aerodromes -- not a placeholder that could be
    /// mistaken for a real airport's code.
    pub icao: String,
    pub runway: Runway,
}

impl Airport {
    /// The MVP's one fictional airport. No licensing/real-world
    /// concerns (see ROADMAP.md: "no licensing... freedom of interface
    /// design"), and no second runway -- matches "one airport, one
    /// runway" exactly.
    pub fn fictional() -> Self {
        Self {
            name: "Glideslope Regional".to_string(),
            icao: "ZZZG".to_string(),
            runway: Runway { heading_deg: 90.0, length_ft: 8_000.0, elevation_ft: 620.0 },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn designator_rounds_to_nearest_ten_degrees() {
        let runway = Runway { heading_deg: 93.0, length_ft: 1.0, elevation_ft: 0.0 };
        assert_eq!(runway.designator(), "09");
    }

    #[test]
    fn designator_wraps_zero_to_36() {
        let runway = Runway { heading_deg: 2.0, length_ft: 1.0, elevation_ft: 0.0 };
        assert_eq!(runway.designator(), "36");
    }

    #[test]
    fn designator_handles_360_as_36() {
        let runway = Runway { heading_deg: 360.0, length_ft: 1.0, elevation_ft: 0.0 };
        assert_eq!(runway.designator(), "36");
    }

    #[test]
    fn fictional_airport_has_a_single_runway_matching_mvp_scope() {
        let airport = Airport::fictional();
        assert_eq!(airport.runway.designator(), "09");
        assert!(airport.runway.length_ft > 0.0);
    }
}
