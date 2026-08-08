# glideslope

*(working title: IFR Challenge)*

A terminal-first systems simulation about managing uncertainty, not flying
airplanes. The player is Pilot Flying on an IFR transport aircraft:
observing instruments, delegating to a First Officer, cross-checking
unreliable sensors, and making operational decisions — rarely touching
pitch or roll directly.

See [ROADMAP.md](ROADMAP.md) for the design philosophy and MVP scope.

## Status

Early foundation work (event bus, fixed-timestep sim loop). Not playable
yet — see open issues for what's next.

## Running

```sh
cargo run
cargo test
```

## Architecture

Event-driven: subsystems (aircraft, crew, world, ATC, ...) never
manipulate each other directly, only communicate through `core::event`.
The simulation advances on a fixed 10 Hz tick for determinism (replay,
debugging, reproducible scenarios) regardless of real frame rate.
