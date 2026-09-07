---
id: 0518
title: Compute the reward of a faction in the control plane, from the standing and the game end
status: proposed
created: 2026-09-06
implements: []
changes: []
creates: []
serves: [PRD-0056]
blocked-by: [BLK-050]
---

## Why

**A learner needs a reward, and the engine must not hold one.** A product record
says plainly that it does not decide what a faction should be rewarded for,
because that is a rule of the downstream game.[^1] A design document gives the
determinism reason for keeping the reward on the other side of the boundary: a
reward inside the engine would enter the state hash, so every golden file would
move when a researcher changed their mind.[^2]

**So the absence of a reward in the engine is a decision, and not a gap.** A
survey searched the whole tree for it, found nothing, and says to report it that
way.[^3] This item builds the reward where it belongs, which is the control
plane.

**Both inputs exist today.** One reader returns the standing of a faction on
each win path, and it answers for the named faction alone. One record holds the
game end once and stops the controllers.[^4] [^5] A survey checked the standing
reader closely and found every value it returns belongs to the faction that was
named.[^3]

**This item writes no Rust.** It adds a module to the Python package. Floating
point is allowed there, and only there, because the simulation holds none and
the learner's side of the boundary is the exception the record names.[^6] [^7]

**It is independent of the observation and of the action table.** It reads two
calls that exist, so it can be built and tested before either lands. It is not
the harness on its own, and the item that wraps it says so.[^8]

## What is missing before this can be refined

- **What a faction should be rewarded for.** The product record refuses to
  decide it, and one blocker holds the rules of the downstream game.[^1] [^9]
  The work must express each term as a parameter, add one row for each term to
  the reference tables with the value unset, and name the blocker on every
  row.[^10] It must not invent a weight.

- **Whether the reward is shaped or terminal.** A terminal reward reads the game
  end and gives nothing until a run finishes. A shaped reward reads the standing
  every step and gives a difference. The two ask different things of the run
  length, and nothing in the tree settles which the project wants. The work must
  decide, or must build both behind one selection and say which is the default.

- **What the reward reads when a faction is eliminated before the game ends.**
  The standing reader answers for a faction that holds nothing, and a game end
  is recorded once for the run.[^5] The work must say what the reward is for a
  faction whose run ended early, because a zero that a learner cannot tell from
  a real zero is the worst of the answers.

- **Whether the reward may read anything the faction cannot see.** The first
  checkable statement of the product record is that the harness never shows a
  learner anything a player of that faction could not see.[^1] A reward is not
  an observation, and a reward computed from the truth is a signal a player
  would not have. The work must decide whether that rule reaches the reward, and
  it must say so plainly either way.

- **Whether the reward belongs to this package at all.** A separate item wraps
  the engine as an environment.[^8] The work must say whether the reward is a
  module of that wrapper or a module beside it that the wrapper calls, and a
  reward that only the wrapper can reach cannot be tested on its own.

## Done when

Filled in when the item moves to `refined/`.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: PRD-0056, a learner plays one faction against the controllers. `docs/product/accepted/prd-0056-a-learner-plays-one-faction-against-the-controllers.md`
[^2]: Design, a learner plays one faction against the controllers. `docs/superpowers/specs/2026-09-05-reinforcement-learning-design.md`
[^3]: Research report 31, the state of the learner surface. `docs/research/reports/31-the-state-of-the-learner-surface.md`
[^4]: ADR-0148, a game end is recorded once and stops the controllers, decision D1. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
[^5]: ADR-0148, a game end is recorded once and stops the controllers, decision D2. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
[^6]: ADR-0002, state holds no floating point number, decision D4. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^7]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables the engine owns, decision D2. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^8]: Backlog item 0520, wrap the engine as an environment one learning stack can drive. `docs/backlog/proposed/0520-wrap-the-engine-as-an-environment-one-learning-stack-can-drive.md`
[^9]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^10]: Reference registers. `docs/reference/`
