//! Application shell: owns the simulation, the terminal event loop, and
//! the bits of UI state (log buffer, command input) that don't belong to
//! the sim itself. This is the seam between "headless simulation" and
//! "thing a person looks at" -- nothing simulation-specific should leak
//! in here beyond what [`crate::sim::simulation::Simulation`] exposes.

use crate::core::command::Command;
use crate::core::event::Event;
use crate::parser;
use crate::sim::simulation::Simulation;
use crate::ui::screen::{self, ScreenState};
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::backend::Backend;
use ratatui::Terminal;
use std::collections::VecDeque;
use std::io;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// How many log lines to keep on screen. Older entries are dropped rather
/// than accumulated forever -- this is a live status view, not the
/// permanent record (that's what `replay::recorder` will be, once it
/// exists).
const LOG_CAPACITY: usize = 200;

/// Upper bound on how long a single poll waits for terminal input before
/// giving the sim loop a chance to run. Keeps the UI responsive to both
/// keystrokes and the passage of sim time.
const POLL_INTERVAL: Duration = Duration::from_millis(33); // ~30 Hz redraw

pub struct App {
    sim: Simulation,
    log: Arc<Mutex<VecDeque<String>>>,
    command_input: String,
    should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        let mut sim = Simulation::new();
        let log = Arc::new(Mutex::new(VecDeque::with_capacity(LOG_CAPACITY)));

        let log_for_subscriber = Arc::clone(&log);
        sim.event_bus().subscribe(move |event: &Event| {
            // Tick fires 10x/sec (see core::time::TICKS_PER_SECOND) and
            // would drown out everything else in the log within a couple
            // of seconds -- it exists for subsystems that need to know
            // time passed, not for this operator-facing log. Only
            // discrete, notable events belong here (start/stop today;
            // failures, checklist steps, ATC calls once those issues
            // land). Command confirmations/errors are pushed directly by
            // handle_key, not through the bus, so they're unaffected by
            // this filter either way.
            if matches!(event, Event::Tick { .. }) {
                return;
            }

            let mut log = log_for_subscriber.lock().unwrap();
            push_capped(&mut log, format!("{event:?}"));
        });

        Self { sim, log, command_input: String::new(), should_quit: false }
    }

    /// Run the event loop until the user quits. Returns on a clean exit;
    /// terminal setup/teardown is the caller's responsibility (see
    /// `main.rs`) so this stays testable without a real terminal.
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        let mut last_frame = Instant::now();

        while !self.should_quit {
            if event::poll(POLL_INTERVAL)? {
                if let CrosstermEvent::Key(key) = event::read()? {
                    // Only handle presses -- crossterm on some platforms
                    // also reports key release/repeat, which would
                    // otherwise double-handle input.
                    if key.kind == KeyEventKind::Press {
                        self.handle_key(key.code, key.modifiers);
                    }
                }
            }

            let now = Instant::now();
            self.sim.advance(now.duration_since(last_frame));
            last_frame = now;

            terminal.draw(|frame| self.draw(frame))?;
        }

        self.sim.stop();
        Ok(())
    }

    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        match code {
            KeyCode::Esc => self.should_quit = true,
            KeyCode::Enter => {
                if !self.command_input.is_empty() {
                    let input = std::mem::take(&mut self.command_input);
                    self.push_log(format!("> {input}"));

                    // HELP isn't a Command the simulation acts on -- it's
                    // a UI convenience, so it's handled before the real
                    // parser rather than adding a no-op arm to
                    // Simulation::apply_command for something that isn't
                    // really a simulation command.
                    if input.trim().eq_ignore_ascii_case("HELP") {
                        for line in help_lines() {
                            self.push_log(line);
                        }
                    } else if input.trim().eq_ignore_ascii_case("CHECKLIST") {
                        for line in checklist_lines(self.sim.checklist()) {
                            self.push_log(line);
                        }
                    } else if input.trim().eq_ignore_ascii_case("CHECK") {
                        match self.sim.check_next_checklist_item() {
                            Some(name) => self.push_log(format!("OK: checked '{name}'")),
                            None => self.push_log(
                                "ERROR: checklist already complete (or nothing to check)"
                                    .to_string(),
                            ),
                        }
                    } else if input.trim().eq_ignore_ascii_case("DELEGATE") {
                        match self.sim.delegate_next_checklist_item() {
                            Some(name) => self.push_log(format!("OK: delegated '{name}' to FO")),
                            None => self.push_log(
                                "ERROR: nothing left to delegate (checklist complete, or \
                                 every remaining item is already with the FO)"
                                    .to_string(),
                            ),
                        }
                    } else if input.trim().eq_ignore_ascii_case("AIRPORT") {
                        for line in airport_lines(self.sim.airport()) {
                            self.push_log(line);
                        }
                    } else if input.trim().eq_ignore_ascii_case("ATC") {
                        for line in atc_lines(self.sim.atc()) {
                            self.push_log(line);
                        }
                    } else if let Some(name) = strip_word(&input, "REQUEST") {
                        match crate::atc::constraints::ClearanceType::lookup(name) {
                            Some(clearance) => {
                                self.sim.request_clearance(clearance);
                                self.push_log(format!("ATC: cleared for {}", clearance.name()));
                            }
                            None => self.push_log(format!(
                                "ERROR: unknown clearance '{name}' (try ATC or HELP)"
                            )),
                        }
                    } else {
                        match parser::parse(&input) {
                            Ok(command) => {
                                let confirmation = describe(&command);
                                self.sim.apply_command(command);
                                self.push_log(confirmation);
                            }
                            Err(err) => self.push_log(format!("ERROR: {err}")),
                        }
                    }
                }
            }
            KeyCode::Backspace => {
                self.command_input.pop();
            }
            // Reserve Ctrl+C/Ctrl+D as a hard quit alongside Esc, since
            // people reach for it out of habit -- checked before the
            // general Char arm so plain 'c'/'d' still type normally.
            KeyCode::Char('c') | KeyCode::Char('d')
                if modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.should_quit = true;
            }
            KeyCode::Char(c) => {
                self.command_input.push(c);
            }
            _ => {}
        }
    }

    fn push_log(&mut self, entry: String) {
        let mut log = self.log.lock().unwrap();
        push_capped(&mut log, entry);
    }

    fn draw(&self, frame: &mut ratatui::Frame) {
        let log = self.log.lock().unwrap();
        let log_entries: Vec<String> = log.iter().cloned().collect();
        let state = ScreenState {
            tick_count: self.sim.clock().tick_count(),
            aircraft: self.sim.aircraft(),
            fo_queue: self.sim.fo_queue(),
            log_entries: &log_entries,
            command_input: &self.command_input,
        };
        screen::draw(frame, &state);
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Push `entry` onto `log`, evicting the oldest entry first if already at
/// `LOG_CAPACITY`. Shared by the event-bus subscriber and `App::push_log`
/// so the cap is enforced in exactly one place.
fn push_capped(log: &mut VecDeque<String>, entry: String) {
    if log.len() == LOG_CAPACITY {
        log.pop_front();
    }
    log.push_back(entry);
}

/// Generates the `HELP` listing from `parser::grammar::ALL` so it can
/// never drift out of sync with what the parser actually accepts.
fn help_lines() -> Vec<String> {
    let mut lines = vec!["Commands:".to_string()];
    for spec in crate::parser::grammar::ALL {
        lines.push(format!(
            "  {:<7} {:>6}..{:<6} {}",
            spec.name,
            spec.range.start(),
            spec.range.end(),
            spec.description
        ));
    }
    lines.push("Type a command and press Enter, e.g. 'PITCH 5'.".to_string());
    lines.push("Other commands:".to_string());
    lines.push("  HELP       show this text".to_string());
    lines.push("  CHECKLIST  show the active checklist and its status".to_string());
    lines.push("  CHECK      check off the next pending item yourself".to_string());
    lines.push("  DELEGATE   hand the next pending item to the FO".to_string());
    lines.push("  AIRPORT    show the current airport and runway".to_string());
    lines.push("  ATC        show clearance status".to_string());
    lines.push("  REQUEST <name>  request a clearance, e.g. 'REQUEST TAXI'".to_string());
    lines
}

/// Renders a checklist as `[x]`/`[ ]` lines for the log, since there's no
/// dedicated screen region for it in the roadmap's layout.
fn checklist_lines(checklist: &crate::checklist::Checklist) -> Vec<String> {
    let mut lines = vec![format!("{}:", checklist.name)];
    for item in &checklist.items {
        let mark = if item.complete { "x" } else { " " };
        lines.push(format!("  [{mark}] {}", item.name));
    }
    if checklist.is_complete() {
        lines.push("Status: COMPLETE".to_string());
    }
    lines
}

/// If `input` starts with `word` (case-insensitive) followed by
/// whitespace, returns whatever follows, trimmed. Used for the one
/// pseudo-command that takes an argument (`REQUEST <clearance>`) --
/// the others are all bare keywords, matched with `eq_ignore_ascii_case`
/// directly.
fn strip_word<'a>(input: &'a str, word: &str) -> Option<&'a str> {
    let trimmed = input.trim();
    if trimmed.len() <= word.len() {
        return None; // no room for both the word and an argument
    }
    let head = trimmed.get(..word.len())?; // None if word.len() isn't a char boundary
    let tail = &trimmed[word.len()..];
    if !head.eq_ignore_ascii_case(word) || !tail.starts_with(char::is_whitespace) {
        return None;
    }
    let rest = tail.trim();
    if rest.is_empty() {
        None
    } else {
        Some(rest)
    }
}

/// Static facts about the airport -- no clearances, no ATC dialogue
/// (that's #12's job once it exists). Just what's actually known: name,
/// identifier, and the one runway.
fn airport_lines(airport: &crate::world::airport::Airport) -> Vec<String> {
    vec![
        format!("{} ({})", airport.name, airport.icao),
        format!(
            "Runway {}: {:.0} ft, elevation {:.0} ft",
            airport.runway.designator(),
            airport.runway.length_ft,
            airport.runway.elevation_ft
        ),
    ]
}

/// Every clearance type and whether it's been granted. No pending/denied
/// state to show yet -- see `atc::controller`'s module doc for why.
fn atc_lines(atc: &crate::atc::controller::Controller) -> Vec<String> {
    let mut lines = vec!["ATC clearances:".to_string()];
    for clearance in crate::atc::constraints::ClearanceType::ALL {
        let status = if atc.is_granted(clearance) { "GRANTED" } else { "not requested" };
        lines.push(format!("  {:<9} {status}", clearance.name()));
    }
    lines
}

/// Human-readable confirmation shown in the log after a command is
/// applied. Kept separate from `Command`'s `Debug` output so the log
/// reads like an operator log, not a struct dump.
fn describe(command: &Command) -> String {
    match command {
        Command::Pitch(deg) => format!("OK: pitch target {deg:.1}"),
        Command::Bank(deg) => format!("OK: bank target {deg:.1}"),
        Command::Thrust(percent) => format!("OK: thrust target {percent:.1}%"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    fn type_str(app: &mut App, input: &str) {
        for c in input.chars() {
            app.handle_key(KeyCode::Char(c), KeyModifiers::NONE);
        }
    }

    #[test]
    fn ticks_do_not_reach_the_operator_log() {
        let mut app = App::new();
        for _ in 0..50 {
            app.sim.tick();
        }
        let log = app.log.lock().unwrap();
        assert!(log.iter().all(|entry| !entry.contains("Tick")));
    }

    #[test]
    fn valid_command_logs_a_confirmation_and_updates_controls() {
        let mut app = App::new();
        type_str(&mut app, "PITCH 5");
        app.handle_key(KeyCode::Enter, KeyModifiers::NONE);

        let log = app.log.lock().unwrap();
        assert!(log.iter().any(|entry| entry.starts_with("OK: pitch target 5.0")));
        drop(log);

        for _ in 0..10 {
            app.sim.tick();
        }
        assert!(app.sim.aircraft().pitch_deg > 2.5);
    }

    #[test]
    fn invalid_command_logs_an_error_and_leaves_controls_untouched() {
        let mut app = App::new();
        let pitch_before = app.sim.aircraft().pitch_deg;

        type_str(&mut app, "FLAPS 15");
        app.handle_key(KeyCode::Enter, KeyModifiers::NONE);

        let log = app.log.lock().unwrap();
        assert!(log.iter().any(|entry| entry.starts_with("ERROR:")));
        drop(log);

        assert_eq!(app.sim.aircraft().pitch_deg, pitch_before);
    }

    #[test]
    fn check_command_checks_off_the_next_item() {
        let mut app = App::new();
        type_str(&mut app, "check");
        app.handle_key(KeyCode::Enter, KeyModifiers::NONE);

        let log = app.log.lock().unwrap();
        assert!(log.iter().any(|entry| entry.contains("Gear")));
        drop(log);

        assert!(app.sim.checklist().items[0].complete);
    }

    #[test]
    fn delegate_command_delegates_the_next_item_to_the_fo() {
        let mut app = App::new();
        type_str(&mut app, "delegate");
        app.handle_key(KeyCode::Enter, KeyModifiers::NONE);

        let log = app.log.lock().unwrap();
        assert!(log.iter().any(|entry| entry.contains("Gear")));
        drop(log);

        assert!(app.sim.fo_queue().executing().is_some());
        assert!(!app.sim.checklist().items[0].complete); // not done yet, only delegated
    }

    #[test]
    fn checklist_command_lists_all_items() {
        let mut app = App::new();
        type_str(&mut app, "checklist");
        app.handle_key(KeyCode::Enter, KeyModifiers::NONE);

        let log = app.log.lock().unwrap();
        let joined = log.iter().cloned().collect::<Vec<_>>().join("\n");
        for name in ["Gear", "Flaps", "Spoilers", "Autobrake", "Cabin"] {
            assert!(joined.contains(name), "CHECKLIST output should mention {name}");
        }
    }

    #[test]
    fn airport_command_shows_name_and_runway() {
        let mut app = App::new();
        type_str(&mut app, "airport");
        app.handle_key(KeyCode::Enter, KeyModifiers::NONE);

        let log = app.log.lock().unwrap();
        let joined = log.iter().cloned().collect::<Vec<_>>().join("\n");
        assert!(joined.contains("Glideslope Regional"));
        assert!(joined.contains("Runway 09"));
    }

    #[test]
    fn request_command_grants_a_valid_clearance() {
        let mut app = App::new();
        type_str(&mut app, "request taxi");
        app.handle_key(KeyCode::Enter, KeyModifiers::NONE);

        let log = app.log.lock().unwrap();
        assert!(log.iter().any(|entry| entry.contains("TAXI")));
        drop(log);

        assert!(app.sim.atc().is_granted(crate::atc::constraints::ClearanceType::Taxi));
    }

    #[test]
    fn request_command_rejects_an_unknown_clearance() {
        let mut app = App::new();
        type_str(&mut app, "request flightlevelchange");
        app.handle_key(KeyCode::Enter, KeyModifiers::NONE);

        let log = app.log.lock().unwrap();
        assert!(log.iter().any(|entry| entry.starts_with("ERROR:")));
    }

    #[test]
    fn atc_command_lists_every_clearance_type() {
        let mut app = App::new();
        type_str(&mut app, "atc");
        app.handle_key(KeyCode::Enter, KeyModifiers::NONE);

        let log = app.log.lock().unwrap();
        let joined = log.iter().cloned().collect::<Vec<_>>().join("\n");
        for clearance in crate::atc::constraints::ClearanceType::ALL {
            assert!(joined.contains(clearance.name()));
        }
    }

    #[test]
    fn strip_word_extracts_the_argument() {
        assert_eq!(strip_word("REQUEST TAXI", "REQUEST"), Some("TAXI"));
        assert_eq!(strip_word("  request   taxi  ", "REQUEST"), Some("taxi"));
    }

    #[test]
    fn strip_word_rejects_a_word_without_an_argument() {
        assert_eq!(strip_word("REQUEST", "REQUEST"), None);
        assert_eq!(strip_word("REQUEST ", "REQUEST"), None);
    }

    #[test]
    fn strip_word_does_not_match_a_longer_word_with_the_same_prefix() {
        // "REQUESTED FOO" must not be treated as "REQUEST"ed with
        // argument "ED FOO".
        assert_eq!(strip_word("REQUESTED FOO", "REQUEST"), None);
    }

    #[test]
    fn sim_starts_on_the_ground_at_the_airport_not_in_cruise() {
        let app = App::new();
        let airport = app.sim.airport();
        assert_eq!(app.sim.aircraft().altitude_ft, airport.runway.elevation_ft);
        assert_eq!(app.sim.aircraft().indicated_airspeed_kt, 0.0);
        assert_eq!(app.sim.aircraft().engine.n1_percent, 0.0);
    }

    #[test]
    fn help_command_lists_every_grammar_command() {
        let mut app = App::new();
        type_str(&mut app, "help");
        app.handle_key(KeyCode::Enter, KeyModifiers::NONE);

        let log = app.log.lock().unwrap();
        let joined = log.iter().cloned().collect::<Vec<_>>().join("\n");
        for spec in crate::parser::grammar::ALL {
            assert!(joined.contains(spec.name), "HELP output should mention {}", spec.name);
        }
    }

    #[test]
    fn command_input_clears_after_submission() {
        let mut app = App::new();
        type_str(&mut app, "BANK 10");
        app.handle_key(KeyCode::Enter, KeyModifiers::NONE);
        assert!(app.command_input.is_empty());
    }

    #[test]
    fn log_evicts_oldest_entry_once_at_capacity() {
        let mut log = VecDeque::new();
        for i in 0..LOG_CAPACITY {
            push_capped(&mut log, format!("entry {i}"));
        }
        push_capped(&mut log, "overflow".to_string());

        assert_eq!(log.len(), LOG_CAPACITY);
        assert_eq!(log.front().unwrap(), "entry 1");
        assert_eq!(log.back().unwrap(), "overflow");
    }
}
