//! Control inputs the aircraft responds to.
//!
//! Nothing sets these to anything but their default yet -- there's no
//! command parser (issue #4) or FO wiring (issue #7) to change them.
//! [`state::AircraftState::integrate`] already reads them, though, so
//! that path is real and testable ahead of something actually driving it.

/// Target values the aircraft's simple first-order response chases.
/// Defaults describe stable, trimmed level cruise: hold current pitch,
/// wings level, hold current thrust.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ControlInputs {
    pub pitch_target_deg: f64,
    pub bank_target_deg: f64,
    pub thrust_target_percent: f64,
}

impl ControlInputs {
    /// Cruise trim: level pitch, wings level, cruise thrust setting.
    /// This is the only source of truth until commands can override it.
    pub const CRUISE_TRIM: ControlInputs =
        ControlInputs { pitch_target_deg: 2.5, bank_target_deg: 0.0, thrust_target_percent: 85.0 };
}

impl Default for ControlInputs {
    fn default() -> Self {
        Self::CRUISE_TRIM
    }
}
