//! Application shell: owns the simulation, the terminal event loop, and
//! the bits of UI state (log buffer, command input) that don't belong to
//! the sim itself. This is the seam between "headless simulation" and
//! "thing a person looks at" -- nothing simulation-specific should leak
//! in here beyond what [`crate::sim::simulation::Simulation`] exposes.

use crate::core::event::Event;
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
            let mut log = log_for_subscriber.lock().unwrap();
            if log.len() == LOG_CAPACITY {
                log.pop_front();
            }
            log.push_back(format!("{event:?}"));
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
                    self.push_log(format!("> {}", self.command_input));
                    // No parser yet (issue #4) -- the command is only
                    // echoed to the log, never interpreted.
                    self.command_input.clear();
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
        if log.len() == LOG_CAPACITY {
            log.pop_front();
        }
        log.push_back(entry);
    }

    fn draw(&self, frame: &mut ratatui::Frame) {
        let log = self.log.lock().unwrap();
        let log_entries: Vec<String> = log.iter().cloned().collect();
        let state = ScreenState {
            tick_count: self.sim.clock().tick_count(),
            aircraft: self.sim.aircraft(),
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
