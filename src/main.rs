mod core;
mod sim;

use core::event::Event;
use sim::simulation::Simulation;
use std::time::{Duration, Instant};

/// How long this standalone harness runs before shutting down. Once the
/// Ratatui shell (issue #5) exists it becomes the real driver of this
/// loop; until then this binary exists to prove clock/event-bus/timestep
/// wiring end to end.
const RUN_FOR: Duration = Duration::from_millis(500);

fn main() {
    tracing_subscriber::fmt::init();

    let mut sim = Simulation::new();

    sim.event_bus().subscribe(|event: &Event| {
        tracing::info!(?event, "event");
    });

    tracing::info!("glideslope starting: fixed-tick simulation loop");

    let start = Instant::now();
    let mut last_poll = start;
    while start.elapsed() < RUN_FOR {
        let now = Instant::now();
        sim.advance(now.duration_since(last_poll));
        last_poll = now;
        std::thread::sleep(Duration::from_millis(5));
    }

    sim.stop();
    tracing::info!(tick = sim.clock().tick_count(), "simulation stopped");
}
