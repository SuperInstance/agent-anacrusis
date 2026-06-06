# agent-anacrusis

> *Anacrusis* — the notes that come before the downbeat. In music, these pickup
> notes prepare the listener for what's coming. The best agents do the same:
> they start preparing before the task officially begins.

**Pickup notes and preparation for agent systems.**

## Overview

Most agent systems wait for a task to arrive, then scramble to get ready. Load
config, warm caches, establish connections — all while the clock is ticking. But
what if the agent could start preparing *before* the task begins? That's the
anacrusis: the preparation phase that precedes the official start.

`agent-anacrusis` provides the building blocks for proactive agent preparation:

- **Anacrusis** — The full preparation phase before a main action. Orchestrates
  the anticipation, preparation window, pickup notes, and final verification
  into a cohesive pre-task workflow.

- **PickupNote** — A small preparatory action that sets up the main event.
  Warm caches, load configs, establish connections, prefetch data. Each pickup
  has an estimated duration, criticality level, and optional dependencies on
  other pickups.

- **PreparationWindow** — The time budget before the main action starts. Track
  how much preparation time is available, how much has been used, and whether
  there's enough time for remaining pickups. Detect urgency when time is
  running low.

- **AnticipationDetector** — Sense when something is about to start. Watch for
  signals that a task is imminent — user activity patterns, calendar events,
  schedule cues — and begin preparation before the official start signal arrives.

- **PreemptiveAction** — Act before being asked. Compute expected value (benefit
  × confidence − cost − waste risk) to decide whether a proactive action is
  worthwhile. Track whether preemptive actions were actually needed.

- **SetupVerification** — Confirm that preparation was sufficient before the
  main action begins. Run checks, categorize them as critical or optional, and
  produce a readiness verdict: Ready, PartiallyReady (warnings), or NotReady
  (critical failures).

## When to Use This

- **Proactive caching** — Preload data before a user requests it, based on
  observed patterns.
- **Connection warming** — Establish database, API, and service connections
  before the first request arrives.
- **Configuration loading** — Parse and validate config files during the
  anacrusis phase rather than on first use.
- **Anticipation** — Detect that a task is coming (calendar, schedule, user
  activity) and start preparing early.
- **Setup validation** — Verify all prerequisites are met before committing
  to the main action.

## Quick Start

```rust
use agent_anacrusis::{
    Anacrusis, PickupNote, AnticipationDetector, VerificationCheck,
    PreemptiveAction,
};
use std::time::Duration;

// Create an anacrusis for a main action
let mut prep = Anacrusis::new("process-user-request", Duration::from_secs(5));

// Anticipate the task is coming
prep.add_signal(AnticipationDetector::signal("user-activity", 0.8));
prep.add_signal(AnticipationDetector::timed_signal(
    "scheduled-task", 0.9, Duration::from_secs(30),
));

// Add preparatory actions
prep.add_pickup(
    PickupNote::new("warm-cache", "Warm the response cache")
        .with_duration(Duration::from_secs(2))
        .with_criticality(8) // essential
);
prep.add_pickup(
    PickupNote::new("prefetch-data", "Prefetch likely-needed data")
        .with_duration(Duration::from_secs(1))
        .with_criticality(3) // optional
        .depends_on("warm-cache")
);

// Verify setup
prep.add_check(VerificationCheck::pass("cache-warm"));
prep.add_check(VerificationCheck::pass("data-loaded"));

// Run the preparation phase
prep.begin_preparation();
// ... execute pickups ...
// ... verify setup:
let result = prep.finalize();
```

## Core Concepts

### The Anacrusis Philosophy

In music, the anacrusis isn't a mistake — it's intentional preparation. The
pickup notes set up the rhythm, key, and emotional tone before the first strong
beat. A well-prepared downbeat feels effortless. A poorly prepared one feels
jarring. The same applies to agent systems: a well-prepared task starts fast
and runs smooth; an unprepared one stumbles through initialization.

### Pickup Notes as Micro-Preparation

Each `PickupNote` is a small, focused preparatory action. They can have
dependencies on each other (warm cache before prefetching), criticality levels
(essential vs optional), and estimated durations for time budgeting.

### Expected Value of Preemption

`PreemptiveAction` computes expected value to decide whether proactive work is
worthwhile: `EV = confidence × benefit − (1 − confidence) × waste_cost − cost`.
High confidence + high benefit + low waste = do it. Low confidence + high waste
= don't.

## License

MIT
