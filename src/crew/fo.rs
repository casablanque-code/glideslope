//! The First Officer: an autonomous worker the player delegates tasks to.
//!
//! Roadmap: "The FO is not a menu. The FO is another human working under
//! pressure," with characteristics (experience, fatigue, stress,
//! assertiveness) driving imperfect behavior (delayed execution,
//! readback mistakes, missed callouts, ...). Full CRM behavior --
//! mistakes, challenging unsafe decisions, staying silent -- is out of
//! scope for this issue. What lands here is the one characteristic that
//! actually has a call site today: experience scales how fast tasks get
//! done. Fatigue/stress/assertiveness stay undeclared rather than being
//! added as fields nothing reads yet -- they'll show up once there's a
//! reason (a checklist or comms system that behaves differently under
//! them) to model them.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FirstOfficer {
    /// 0.0 (brand new) ..= 1.0 (fully proficient). Scales task completion
    /// speed -- see `speed_multiplier`.
    pub experience: f64,
}

impl FirstOfficer {
    pub fn new(experience: f64) -> Self {
        Self { experience: experience.clamp(0.0, 1.0) }
    }

    /// A brand-new FO takes twice as long as a fully proficient one; a
    /// fully proficient FO completes tasks in exactly their listed
    /// `base_duration`. Linear, and not sourced from any real
    /// training-curve data -- picked to make experience visibly matter
    /// without dominating the numbers.
    pub fn speed_multiplier(&self) -> f64 {
        2.0 - self.experience
    }
}

impl Default for FirstOfficer {
    fn default() -> Self {
        // Reasonably experienced by default -- matches the "basic ATC"
        // MVP tone elsewhere: competent, not perfect, not a rookie.
        Self::new(0.7)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fully_experienced_fo_has_a_multiplier_of_one() {
        assert_eq!(FirstOfficer::new(1.0).speed_multiplier(), 1.0);
    }

    #[test]
    fn a_brand_new_fo_takes_twice_as_long() {
        assert_eq!(FirstOfficer::new(0.0).speed_multiplier(), 2.0);
    }

    #[test]
    fn experience_is_clamped_to_valid_range() {
        assert_eq!(FirstOfficer::new(1.5).experience, 1.0);
        assert_eq!(FirstOfficer::new(-0.5).experience, 0.0);
    }
}
