# IFR Challenge
### Working title

> **A terminal-first systems simulation about managing uncertainty, not flying airplanes.**

---

# Vision

IFR Challenge is **not** a flight simulator.

It is a real-time procedural systems game where the player acts as the **Pilot Flying (Captain)** of a transport aircraft operating under instrument flight rules (IFR).

The game focuses on:

- decision making
- workload management
- cockpit resource management (CRM)
- incomplete information
- conflicting sensor data
- procedural discipline
- time pressure

The player rarely "flies" the aircraft directly.

Instead, they continuously observe instruments, build hypotheses, distribute workload, communicate with the First Officer, and make operational decisions while the aircraft continues flying.

The core fantasy is:

> **Managing uncertainty inside a complex system.**

---

# Design Philosophy

The project follows several principles.

## No memorization

The game should never require the player to memorize aviation trivia.

Instead, players discover how systems behave through observation.

Example:

Instead of explaining what QNH is, the player notices that changing the barometric setting changes the indicated altitude.

Understanding emerges naturally.

---

## No scripted puzzles

There should never be a single hidden correct solution.

The simulation models consequences.

Multiple solutions may be valid.

Example:

Flaps jammed.

Possible decisions:

- continue landing
- divert
- enter holding
- declare emergency

The simulation evaluates consequences.

Not correctness.

---

## No invisible game rules

Everything should happen because of systems.

Failures.

Weather.

Crew.

Traffic.

ATC.

Aircraft state.

Nothing should happen because "this is Level 5."

---

## Information is imperfect

Displayed information is not necessarily true.

Every sensor can:

- fail
- drift
- freeze
- become unreliable

The player must determine which sources are trustworthy.

---

## Time is a resource

Reading.

Thinking.

Delegating.

Communicating.

Everything consumes time.

The aircraft never stops.

---

## Humans are not perfect

The First Officer is not a menu.

The First Officer is another human working under pressure.

---

# Core Gameplay Loop

Every few seconds the player repeats the same loop.

```
Observe
↓

Form hypothesis

↓

Decide

↓

Execute

↓

Observe consequences
```

Example.

```
IAS disagree

↓

Which pitot failed?

↓

Cross-check standby instruments

↓

Command FO to switch source

↓

Observe stabilization
```

---

# Pillars

## 1. Systems Thinking

The game rewards understanding relationships.

Not reaction speed.

---

## 2. Procedural Discipline

Checklists matter.

Stable approaches matter.

Configuration matters.

---

## 3. Workload Management

The player cannot do everything.

Delegation is required.

---

## 4. Decision Making

The player constantly trades:

- time
- fuel
- safety
- workload
- operational efficiency

---

## 5. Uncertainty

The player almost never has complete information.

---

# What the Player Actually Does

The player:

- reads instruments
- tunes navigation radios
- changes aircraft configuration
- commands the First Officer
- communicates with ATC
- performs checklists
- diagnoses failures
- manages approach energy
- decides whether to continue or go around

The player almost never manually controls pitch or roll.

---

# Aircraft Model

The game uses a fictional aircraft.

Reasons:

- equal starting point for all players
- freedom of interface design
- no licensing
- no endless debates about realism
- systems can be optimized for gameplay

The aircraft should feel believable rather than replicated.

---

# Simulation Layers

```
World
│
├── Weather
├── Airports
├── ATC Constraints
├── Traffic Events
│
Aircraft
│
├── Flight State
├── Engines
├── Electrical
├── Hydraulics
├── Sensors
├── Navigation
│
Crew
│
├── Captain (Player)
├── First Officer
│
Terminal UI
```

Every layer communicates only through events.

---

# First Officer

The FO is an autonomous worker.

The player gives tasks.

The FO performs them.

The FO has characteristics.

Example:

```
Experience

Fatigue

Stress

Assertiveness
```

Possible behaviors:

- delayed execution
- readback mistakes
- forgotten checklist item
- missed callout
- challenging unsafe decisions
- remaining silent

The FO is intentionally imperfect.

---

# Task Queue

The FO executes one task at a time.

Example:

```
[Executing]

Read QRH

10 sec remaining

------------------------

[Pending]

Readback

Flaps 15

Contact Tower

Landing Checklist
```

The player constantly decides:

- interrupt?
- delegate?
- perform manually?

---

# ATC

ATC is a constraint generator.

Not a full air traffic simulator.

Examples:

```
Cross FIX above 4000

Reduce speed 180

Expect runway change

Hold due traffic

Continue approach
```

ATC creates operational pressure.

---

# Weather

Weather is dynamic.

Includes:

- visibility
- ceiling
- QNH
- wind
- gusts
- turbulence
- wind shear
- icing

Weather changes during flight.

---

# Sensors

Every sensor has health.

Possible states:

- healthy
- degraded
- frozen
- noisy
- failed

Examples:

- Pitot
- Static
- GPS
- ILS
- DME
- Radio Altimeter

The player cross-checks information.

---

# Failures

Failures emerge naturally.

Examples:

- unreliable airspeed
- hydraulic leak
- engine flameout
- electrical bus failure
- navigation failure
- flap asymmetry
- landing gear issue
- radio failure

Failures may cascade.

---

# Checklists

Checklists are interactive.

Example:

```
Landing Checklist

Gear

Flaps

Spoilers

Autobrake

Cabin
```

Incorrect configuration has consequences.

---

# Energy Management

The player constantly manages:

- altitude
- speed
- descent profile
- drag
- thrust

High and fast approaches are possible.

Low and slow approaches are possible.

Stable approaches are rewarded.

---

# Stable Approach Concept

Landing is not the objective.

A stabilized approach is.

Go-around is a successful outcome when appropriate.

Unsafe landings are failures even if the aircraft survives.

---

# Career

The game tracks long-term behavior.

Examples:

Repeated unstable approaches.

Late go-arounds.

Checklist discipline.

Energy management.

Crew coordination.

After enough trends appear, the airline reacts.

Examples:

Additional simulator sessions.

Performance review.

Route changes.

Promotion.

---

# Debriefing

Every flight produces a report.

Example:

```
Approach Stability

Checklist Discipline

Crew Coordination

Decision Making

Energy Management

Situational Awareness

Passenger Comfort

Aircraft Handling
```

The game explains consequences.

Not scores.

---

# Emergent Gameplay

The game never says:

```
Today's mission:

Pitot Failure
```

Instead.

The world generates:

- weather
- traffic
- airport status
- aircraft state

Unexpected situations naturally emerge.

---

# Time Model

Simulation runs with a fixed tick.

Example:

```
10 ticks per second
```

Each tick updates:

- aircraft
- crew
- weather
- events
- UI

Deterministic simulation enables:

- replay
- debugging
- black box analysis
- reproducible scenarios

---

# User Interface

Terminal-first.

Possible layout:

```
+-----------------------------------------------------------+
| PFD             ENGINE             AIRCRAFT STATUS         |
+-----------------------------------------------------------+
| ATC             WEATHER            FO TASK QUEUE           |
+-----------------------------------------------------------+
| LOG / WARNINGS                                        |
+-----------------------------------------------------------+
| Command >                                                |
+-----------------------------------------------------------+
```

No graphical cockpit required.

Information density is preferred over visual realism.

---

# Architecture

Language:

Rust

Libraries:

- Ratatui
- Crossterm
- Tokio (if required)
- Serde
- Tracing

Architecture:

Event-driven.

Subsystems never directly manipulate each other.

Everything communicates through events.

---

# Out of Scope (v1)

No multiplayer.

No 3D.

No VR.

No joystick support.

No detailed CFD.

No worldwide navigation database.

No exact aircraft replication.

---

# MVP

The first playable version contains only:

- one fictional aircraft
- one airport
- one runway
- one ILS approach
- one weather model
- one First Officer
- basic ATC
- one checklist
- one failure (Pitot)
- one go-around procedure

If this prototype is engaging, the architecture should naturally support expansion.

---

# Long-Term Goal

Create a game where the player does not feel like they are controlling an airplane.

They feel like they are managing an increasingly uncertain, information-rich, time-critical system.

The airplane is simply the environment where those decisions matter.