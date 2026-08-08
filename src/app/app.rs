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
                    } else if input.trim().eq_ignore_ascii_case("DELEGATE") {
                        // Nothing generates real tasks yet (checklists are
                        // #8, ATC comms don't exist) -- this queues one
                        // canned demo task so the FO queue mechanic can
                        // actually be exercised from the shell, the same
                        // way PITCH/BANK/THRUST let #6's aircraft state be
                        // exercised before an autopilot existed to drive
                        // it. Not real gameplay content.
                        self.sim.delegate_task("Read QRH (demo task)", Duration::from_secs(10));
                        self.push_log("OK: delegated 'Read QRH (demo task)' to FO".to_string());
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
    lines.push("Other commands: HELP, DELEGATE (queues a demo FO task).".to_string());
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
