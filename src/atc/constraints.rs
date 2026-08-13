//! The vocabulary of things ATC can grant.
//!
//! Roadmap: "ATC is a constraint generator." Its examples mix two
//! different kinds of thing -- operational clearances ("continue
//! approach") and numeric restrictions ("Cross FIX above 4000", "Reduce
//! speed 180"). Only the clearance kind is real here: numeric
//! restrictions need navigation (fixes) that doesn't exist yet, so
//! they're left undeclared rather than added as vocabulary nothing can
//! honor or validate against.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClearanceType {
    Pushback,
    Taxi,
    Takeoff,
    Descend,
    Approach,
    Landing,
}

impl ClearanceType {
    pub const ALL: [ClearanceType; 6] = [
        ClearanceType::Pushback,
        ClearanceType::Taxi,
        ClearanceType::Takeoff,
        ClearanceType::Descend,
        ClearanceType::Approach,
        ClearanceType::Landing,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            ClearanceType::Pushback => "PUSHBACK",
            ClearanceType::Taxi => "TAXI",
            ClearanceType::Takeoff => "TAKEOFF",
            ClearanceType::Descend => "DESCEND",
            ClearanceType::Approach => "APPROACH",
            ClearanceType::Landing => "LANDING",
        }
    }

    pub fn lookup(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|clearance| clearance.name().eq_ignore_ascii_case(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_is_case_insensitive() {
        assert_eq!(ClearanceType::lookup("taxi"), Some(ClearanceType::Taxi));
        assert_eq!(ClearanceType::lookup("TaKeOfF"), Some(ClearanceType::Takeoff));
    }

    #[test]
    fn lookup_returns_none_for_unknown_clearance() {
        assert_eq!(ClearanceType::lookup("HOLD"), None);
    }

    #[test]
    fn every_clearance_round_trips_through_its_own_name() {
        for clearance in ClearanceType::ALL {
            assert_eq!(ClearanceType::lookup(clearance.name()), Some(clearance));
        }
    }
}
