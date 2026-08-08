//! Interactive checklists.
//!
//! Roadmap: "Incorrect configuration has consequences" -- but gear,
//! flaps, spoilers, and autobrake don't exist as real aircraft systems
//! yet, so this issue only provides the checklist *mechanic* (an ordered
//! list of items the player steps through, either themselves or by
//! delegating to the FO). Nothing validates real system state yet
//! because there's no real system state to validate against -- that's
//! future work once those systems exist.

pub mod landing;

#[derive(Debug, Clone, PartialEq)]
pub struct ChecklistItem {
    pub name: String,
    pub complete: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Checklist {
    pub name: String,
    pub items: Vec<ChecklistItem>,
}

impl Checklist {
    pub fn new(name: impl Into<String>, item_names: &[&str]) -> Self {
        Self {
            name: name.into(),
            items: item_names
                .iter()
                .map(|n| ChecklistItem { name: (*n).to_string(), complete: false })
                .collect(),
        }
    }

    /// Index of the first not-yet-complete item, if any remain.
    pub fn next_pending(&self) -> Option<usize> {
        self.items.iter().position(|item| !item.complete)
    }

    /// Mark the next pending item complete directly -- the player doing
    /// it themselves rather than delegating to the FO, so it takes
    /// effect immediately with no queue/timing involved. Returns the
    /// item's name, or `None` if the checklist is already fully complete.
    pub fn check_next(&mut self) -> Option<String> {
        let idx = self.next_pending()?;
        self.items[idx].complete = true;
        Some(self.items[idx].name.clone())
    }

    pub fn is_complete(&self) -> bool {
        self.items.iter().all(|item| item.complete)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_pending_returns_items_in_order() {
        let checklist = Checklist::new("Test", &["A", "B", "C"]);
        assert_eq!(checklist.next_pending(), Some(0));
    }

    #[test]
    fn check_next_marks_the_first_pending_item_and_advances() {
        let mut checklist = Checklist::new("Test", &["A", "B"]);
        assert_eq!(checklist.check_next(), Some("A".to_string()));
        assert_eq!(checklist.next_pending(), Some(1));
        assert_eq!(checklist.check_next(), Some("B".to_string()));
        assert_eq!(checklist.next_pending(), None);
    }

    #[test]
    fn check_next_on_a_complete_checklist_returns_none() {
        let mut checklist = Checklist::new("Test", &["A"]);
        checklist.check_next();
        assert_eq!(checklist.check_next(), None);
    }

    #[test]
    fn is_complete_reflects_item_state() {
        let mut checklist = Checklist::new("Test", &["A", "B"]);
        assert!(!checklist.is_complete());
        checklist.check_next();
        assert!(!checklist.is_complete());
        checklist.check_next();
        assert!(checklist.is_complete());
    }

    #[test]
    fn empty_checklist_is_complete_by_definition() {
        let checklist = Checklist::new("Empty", &[]);
        assert!(checklist.is_complete());
        assert_eq!(checklist.next_pending(), None);
    }
}
