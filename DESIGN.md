# Design notes

Decisions made in discussion that later issues should treat as settled,
not re-litigate. Update this file when a decision changes; don't leave
stale rationale here once it does.

## Flight phase model

The original issue list (#1-#10) covers foundation, UI, aircraft state,
commands, FO queue, and checklists -- none of it covers the actual
structure of a flight (gate -> taxi -> takeoff -> climb -> cruise ->
descent -> approach -> landing). That gap is intentional: those pieces
depend on systems (airport, ATC, navigation) that didn't exist yet.
Filling it needs new issues, ordered so nothing is built against a fake
trigger.

### Phases

Not a strictly linear list -- one loop (go-around) and one modifier
(holding) that isn't a phase of its own:

```
Ground (at gate)
  -> Taxi
    -> TakeoffHold (holding short, waiting for clearance)
      -> Takeoff -> Climb
        -> Cruise
          -> Descent
            -> Approach
              -> Landing -> TaxiToGate -> Shutdown
              -> GoAround -> Climb (second circuit)
```

**Holding is a modifier, not a phase.** "Hold due traffic" can happen
during Climb, Descent, or Approach -- making it its own phase would mean
tracking which phase to return to afterward, which a modifier avoids for
free. Model as something like `holding: Option<HoldReason>` alongside
the current phase, not as a phase value itself.

### Transition triggers

Every transition must be a consequence of a real event -- an ATC
clearance, a checklist completion, an altitude/speed threshold, or a
player command. Never a timer or a scripted "you've been in this phase
long enough" check (see ROADMAP.md: "No invisible game rules").

| Transition | Trigger | Needs |
|---|---|---|
| Ground -> Taxi | ATC pushback/taxi clearance | ATC (#12) |
| Taxi -> TakeoffHold | abstracted -- see below | ATC (#12) |
| TakeoffHold -> Takeoff | ATC takeoff clearance + before-takeoff checklist + player commits (thrust up) | ATC (#12), checklist already exists (#8) |
| Climb -> Cruise | reaching target cruise altitude | nothing new -- `AircraftState.altitude_ft` already exists |
| Cruise -> Descent | ATC descent clearance, or player decision | ATC (#12) |
| Descent -> Approach | crossing a fix / proximity to the airport | navigation/airport (#11) |
| Approach -> Landing / GoAround | stabilized-approach criteria met/not met | partially -- speed/VS exist, ILS capture doesn't (#9) |

Climb -> Cruise is the one transition buildable today with zero new
dependencies. Everything touching position relative to the airport
needs #11 first -- this is why #11 (world::airport) comes before the
phase machine gets built, not after.

### Taxi: abstracted, not simulated

Taxi does **not** model ground position on a taxiway graph. It's an
abstracted waiting/configuration period between Ground and TakeoffHold,
not a physical-movement simulation. Rationale: the roadmap's own MVP
scope ("one airport, one runway") never asked for taxiway routing, and
the design's stated priority is decision-making over movement -- a full
ground-movement sim would be real effort spent on something outside
that priority.

### Phase gating: soft, not hard

Phases do not hard-block player actions except where a hard block is a
physical/state-consistency truth (e.g. the sim cannot be in Cruise phase
while altitude is 0 -- that's not a game rule, it's incoherent state).
Everything else is a **soft** gate: ATC can issue a clearance the
aircraft isn't actually configured for, and the consequence plays out
rather than the game refusing the input. Matches the roadmap directly:
"Multiple solutions may be valid... the simulation evaluates
consequences. Not correctness." A missed before-takeoff checklist item
should be a real problem the player discovers, not a blocked keystroke.

### Open / not yet decided

- Exact criteria for "stabilized approach" (Approach -> Landing) --
  depends on what's real once ILS (#9) exists.
- Whether GoAround always re-enters Climb, or can shortcut back into a
  vectored re-approach without a full climb -- revisit once ATC (#12)
  exists and holding/vectoring is real.
