//! Control inputs the aircraft responds to.
//!
//! Player commands (issue #4) can override these; nothing from ATC or
//! the FO writes to them yet (that's #12 / #7's remaining scope).
//! [`state::AircraftState::integrate`] reads them every tick.

/// Target values the aircraft's simple first-order response chases.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ControlInputs {
    pub pitch_target_deg: f64,
    pub bank_target_deg: f64,
    pub thrust_target_percent: f64,
}

impl ControlInputs {
    /// Stable, trimmed level cruise: hold current pitch, wings level,
    /// cruise thrust setting.
    pub const CRUISE_TRIM: ControlInputs =
        ControlInputs { pitch_target_deg: 2.5, bank_target_deg: 0.0, thrust_target_percent: 85.0 };

    /// Parked at the gate: engines off, wings level, no pitch command.
    /// Nothing should move a gate-started aircraft on its own -- see
    /// `AircraftState::at_gate`, which is constructed to already be in
    /// equilibrium with this, the same way `AircraftState::cruise`
    /// matches `CRUISE_TRIM`.
    pub const GATE_TRIM: ControlInputs =
        ControlInputs { pitch_target_deg: 0.0, bank_target_deg: 0.0, thrust_target_percent: 0.0 };
}

impl Default for ControlInputs {
    fn default() -> Self {
        Self::CRUISE_TRIM
    }
}
