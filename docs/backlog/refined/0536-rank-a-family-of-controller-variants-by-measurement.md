---
id: 0536
title: Rank a family of controller variants by measurement
status: refined
created: 2026-09-09
implements: [ADR-0156 D3, ADR-0144 D6]
changes: []
creates: []
serves: [PRD-0056, PRD-0057]
blocked-by: []
---

## Why

The project trains a policy against one opponent. That opponent wins 0.333 of
its games in a three-faction world, a seat that sends no action wins 0.000, and
the trained policy wins 0.070. **A policy that loses almost every game receives
almost no win signal.** A set of opponents ordered by strength gives a run an
opponent it can beat early and a stronger one to cross later.

The built-in controller is already tunable. Two of its settings are held for
each faction and both are bound to Python, so several seat configurations can
be seated in one world today. Nothing in the control plane calls either setter,
and no run has ever used one. A design report enumerates every setting, says
which of them steer anything, and proposes six seat configurations with a
predicted order.[^report]

This item builds the harness that seats them and runs the measurement that
puts them in order.

## The architectural impact review

**Governed by.** ADR-0156 D3 states that one verb writes the whole weight
vector of a faction, and that a caller, the built-in controller and a learner
all reach it by that one path. This item calls that verb and adds no second
path.[^adr156] ADR-0144 D6 states that a faction under external control
receives no evaluation, which is what makes the passive seat of the family
possible.[^adr144] ADR-0148 D2 states that a game end is recorded once, which
is what makes a win share countable at all.[^adr148]

**Changes.** No record. This item calls verbs that already exist and changes no
engine behaviour.

**Creates.** No record. The choice of which variants to seat is a measurement
plan and not a constraint, so it holds no decision that a record must carry.

**Blockers.** BLK-007 governs every cost figure this item states, and the
figures below are derived from one measured episode cost.[^blk7] BLK-050
governs every balance value the seat configurations write, and the weight
vector range is an unset row of the balance register.[^balance] This item
invents no value: it writes weights inside the range the engine already
enforces, and it changes no default.

**Precedent.** FND-710 measured the win shares this item builds on, and it
measured the win-path shares that give the family something to be ranked
against.[^fnd710] The testing rule requires a fixture that supplies the input a
defect lives at, and a uniform fixture hides a defect.[^testing] A ranking run
whose members all play the same way measures the seat and not the player, so
the balance check of this item is the seat share the rating tool already
reports.

**What is in flight elsewhere.** Another worker is making the overmatch ratio a
per-faction value. One member of the family needs it. This item runs without
that member and adds it when the work lands, so it is not blocked by it.

## What the work does

**Let the rating tool seat a controller variant.** A player of the tool holds a
stored policy or nothing, and nothing means the built-in controller under its
defaults.[^league] Add a third case: a named set of per-faction settings that
the tool writes to the faction of that seat before the world runs. The setters
are already bound to Python.[^bindings]

**Run the screen.** Seat the six members that need no engine change over 4
world seeds. That is 240 games and 120 games for each member. The standard
error of one win share is 0.043, derived. Adding the seventh member when the
per-faction overmatch ratio lands makes it 420 games and 180 games for each
member, at a standard error of 0.035.

**Run the ranking.** Drop to five members and play 32 world seeds. That is 960
games and 576 games for each member. The standard error of one win share is
0.020 and two members separate at a gap of 0.056, both derived. The bootstrap
of the tool then resamples 32 worlds.

**Report the endings, not only the wins.** Two of the six hypotheses are
written against the share of episodes that end on a named path and against the
reach of the seat along that path. The instrument that reports both already
exists.[^endings]

**Hold the rules of the game fixed.** The renown target, the renown for each
unit felled, the work a wonder costs and the victory claim of the wonder row
are settable and are rules of the game rather than settings of a
player.[^adr175] A run that varied one of them would compare two games rather
than two players.

## Done when

- The rating tool seats a named controller variant, and a test asserts that the
  weight vector of that seat holds the named values after the world is built.
- A test drives the tool and asserts that two variants seated in one world hold
  different weight vectors. **This is the test that proves the seating
  reaches the engine**, and it must be shown to fail when the write is removed.
- The screen ran over 4 seeds with 6 members, and its report names the win
  share of each member.
- The ranking ran over 32 seeds with 5 members, and its report names the
  strength of each member on the Elo scale with its interval.
- The report states, for each of the six hypotheses of the design report,
  whether the measurement refused it.
- The findings register holds one entry for every hypothesis the measurement
  refused, with what was believed, what is true and the evidence.
- The commit body holds the two commands, the machine and the seed sets.

## What this item does not do

**It does not widen the weight bound.** The engine refuses a weight below one,
so no faction can be made to never fight, never build or never found. Widening
the bound down to zero is the cheapest change that would widen this family, and
it moves the drawn weight vector of every seeded world, so it moves every
golden state hash. That cost belongs to its own item, and this one measures the
family at the bound the engine holds today.

**It does not add a weight over the build categories.** The build order draws
one of seven categories uniformly, and the wonder is one of the seven, so no
setting aims a faction at the wonder path. That change is policy that a learner
may also make, so it needs a decision record, and it is larger than this item.

**It does not rank a world-level setting.** One world holds one value of the
evaluation count and of each trade setting, so no round robin can separate two
values of one. A ranking of those needs one run for each value.

**It does not touch the trade settings.** Four settings and one weight already
steer nothing, because the trade subsystem records no offer and no contract
over a whole run. The item that opens that subsystem is already in the
backlog.[^item181]

## Outcome

Filled in when the item moves to `complete/`.

## References

[^report]: Report 44, a family of tunable controllers, and how to rank them. `docs/research/reports/44-a-family-of-tunable-controllers.md`
[^adr156]: ADR-0156, a faction's option weights are policy, set through one verb, decision D3. `docs/adrs/accepted/adr-0156-a-factions-option-weights-are-policy-set-through-one-verb.md`
[^adr144]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D6. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^adr148]: ADR-0148, a game end is recorded once and stops the controllers, decision D2. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
[^blk7]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^balance]: Balance register, the weight vector range. `docs/reference/balance.md`
[^fnd710]: Findings register, FND-710. `docs/FINDINGS.md`
[^testing]: Testing Rules, a fixture supplies the input. `.agents/rules/testing.md`
[^league]: The policy rating tool. `scripts/policy_league.py`
[^bindings]: The faction bindings, the weight verb. `crates/cachette-py/src/world/faction_view.rs`
[^endings]: The endings instrument. `python/cachette/learn/endings.py`
[^adr175]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does, decision D2. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
[^item181]: Backlog item 0181, give a kind of work the commodity it fills. `docs/backlog/proposed/0181-give-a-kind-of-work-the-commodity-it-fills.md`
