//! The MVP's one checklist, matching the roadmap's example exactly.

use super::Checklist;

pub fn landing_checklist() -> Checklist {
    Checklist::new("Landing Checklist", &["Gear", "Flaps", "Spoilers", "Autobrake", "Cabin"])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_roadmap_item_list_in_order() {
        let checklist = landing_checklist();
        let names: Vec<&str> = checklist.items.iter().map(|item| item.name.as_str()).collect();
        assert_eq!(names, vec!["Gear", "Flaps", "Spoilers", "Autobrake", "Cabin"]);
    }
}
