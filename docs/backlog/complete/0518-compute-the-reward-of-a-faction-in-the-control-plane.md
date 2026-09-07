---
id: 0518
title: Compute the reward of a faction in the control plane, from the observation and the game end
status: complete
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

**This item writes no Rust.** It adds a module to the Python package. Floating
point is allowed there, and only there, because the simulation holds none and
the learner's side of the boundary is the exception the record names.[^4] [^5]

## Impact review

**Governed by.** ADR-0154 D1 makes the observation schema the only declaration
of the array layout, so the module states no position and asks for the schema
instead.[^5] ADR-0154 D3 binds the observation reader to the sight rule, and the
module reads that reader.[^6] ADR-0148 D1 and D2 give the game end record and the
running standing of a faction.[^7] [^8] ADR-0002 D1 keeps floating point out of
simulated state, and D4 admits it on the control plane side.[^4] ADR-0040 D1 and
D2 forbid a loop over entities in Python, and the module loops over a fixed set
of named positions and never over a unit or a tile.[^9]

**Changes.** None. No record changed and no record was superseded.

**Creates.** None. A reward is a training choice, and a contributor may choose
otherwise at no cost to the project, because the reward enters no hash. The
second condition of the scope rule therefore fails, and a design document
reaches the same conclusion for the same reason.[^2] [^10]

**Blockers.** BLK-050 holds the rules of the downstream game, and it governs
every weight.[^11] The work states no weight. It adds one unset row for each
weight to a new register, and the module refuses to run while a weight a caller
asked for is unset.[^12]

**Precedent.** FND-568 says the standing reports the work toward a victory
claim, while the wonder reader compares the claim itself.[^13] The work reads
the observation array rather than the standing, because the array carries both.

## What the work decided

The item asked five questions before it could be refined. Each one is answered
below.

**It reads the observation array of the faction, and one public fact.** The
array holds what that faction observes and nothing else.[^6] The public fact is
the winner of a game that has already ended, which the game end record
holds.[^7] Every player learns who won when a game ends, so that fact is not
hidden from anybody. The module reads it only after the array says the game is
over.

**The reward may not read the truth of the world, and the cost of that is
stated.** The first checkable statement of the product record is that the
harness never shows a learner anything a player of that faction could not
see.[^1] A reward is not an observation, but a reward reaches the policy through
the gradient. A term the observation never held therefore still teaches the
policy something a player could not know. The cost is a weaker signal: the
module cannot weigh a rival's standing, and it cannot weigh ground the faction
has not seen. Choosing otherwise would have bought a denser shaping term and
would have broken the statement above.

**The reward is neither shaped nor terminal, because the weights decide.** The
module needs no mode selection. A weighting whose shaped weights are all zero is
exactly the terminal reward. The choice is therefore a choice of weights, which
keeps the terms data rather than code, and it removes a mode a caller could set
against its own weights.

**A faction that loses its last unit is not eliminated, and that answer is
measured.** Such a faction keeps its held ground and its seat, and the territory
reader compares held ground at the tick limit, so it may still win. A terminal
that fired on the loss of the last unit would end an episode the faction could
still win. The finding holds the measurement.[^14] The module therefore reports
three terminal outcomes and one boolean beside them, and the boolean is true
while the faction holds a unit or a person.

**The reward is a module beside the wrapper, and not a module of it.** It takes
a world and a faction number, so a test builds one, steps a real world and reads
it. The wrapper calls it.[^15] A reward only the wrapper could reach could not
be tested on its own.

## What the work found

**The array omits one win threshold and carries the other.** It carries the tick
limit, which the territory path compares. It carries no renown target, which the
renown path compares. The finding holds the reading.[^16] The reward can
therefore weigh the best renown and cannot weigh the distance to the target.
This is an asymmetry of the array and not a rule, and a later item may close it.

## Done when

- A module of the Python package computes the reward of one faction. Done.
- The module states no weight, and every weight is a parameter. Done. It refuses
  to run while a weight a caller asked for is unset, and the refusal names each
  unset weight and names the blocker.[^11]
- A register holds one unset row for each weight, and each row names the
  blocker. Done.[^12]
- A test through the public interface steps a real world and shows the reward
  move when the thing it rewards moves. Done.
- A test proves the assertion measures the weight and not the fixture. Done. The
  same fixture with the weight at zero pays nothing while the term still moves.
- The module reads no reader that answers for more than one faction, except the
  game end record. Done.
- The lint, the format check and the type check pass on the files the work adds.
  Done.

## Outcome

The module lives in the learner package of the control plane, beside an
`__init__` that exports it. It offers a weighting, a reward over one run, and a
reading for each decision. A reading reports the value, the shaped part, the
terminal part, the outcome, whether the run ended, whether the faction can act,
what each term contributed, and the raw change of each term.

The register is new. It holds eight shaped rows and three terminal rows, and
every row is unset under BLK-050. Two tests compare the register against the
module, because two declaration sites with nothing to fail is the defect shape
this project names first.[^17]

Two findings came out of the work. One says the array carries the tick limit and
no renown target.[^16] One says a faction that loses every unit keeps its
ground.[^14]

**What is left undone.** The full check command did not run in this branch. The
work ran the lint, the format check, the type check and its own tests, and it
ran no Rust command, because the item writes no Rust.

## References

[^1]: PRD-0056, a learner plays one faction against the controllers. `docs/product/accepted/prd-0056-a-learner-plays-one-faction-against-the-controllers.md`
[^2]: Design, a learner plays one faction against the controllers, section 3. `docs/superpowers/specs/2026-09-05-reinforcement-learning-design.md`
[^3]: Research report 31, the state of the learner surface. `docs/research/reports/31-the-state-of-the-learner-surface.md`
[^4]: ADR-0002, state holds no floating point number, decisions D1 and D4. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^5]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D1. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^6]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D3. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^7]: ADR-0148, a game end is recorded once and stops the controllers, decision D1. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
[^8]: ADR-0148, a game end is recorded once and stops the controllers, decision D2. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
[^9]: ADR-0040, Python is a control plane, not a data plane, decisions D1 and D2. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^10]: Decision Record Scope, section 1. `.agents/rules/adr-scope.md`
[^11]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^12]: Reinforcement learning parameters, the reward rows. `docs/reference/rl-costs.md`
[^13]: Findings register, FND-568. `docs/FINDINGS.md`
[^14]: Findings register, FND-580. `docs/FINDINGS.md`
[^15]: Backlog item 0520, wrap the engine as an environment one learning stack can drive. `docs/backlog/proposed/0520-wrap-the-engine-as-an-environment-one-learning-stack-can-drive.md`
[^16]: Findings register, FND-579. `docs/FINDINGS.md`
[^17]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
