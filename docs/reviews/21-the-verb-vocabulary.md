# The Verb Vocabulary, and What a God Needs

A working report on what a downstream game needs from the Python interface of
this engine. Written 3 September 2026.

Cachette is a world simulation engine. The core is Rust and the control plane is
Python. A game called Gods and Congregations is being built on it. The project
owner named the verbs a god will want and asked one question: is this definable
largely in Python, and if not, what must the project enhance?

This document records what the work found, what it corrected, and what it left
undone. The research report holds the argument.[^1]

## 1. What was checked, and against what

Every claim about the engine came from reading the source tree of this worktree.
**Nothing was run and nothing was measured by this author.** One measured figure
appears in the research report, and it belongs to an earlier report that states
what was run.[^2]

The files read were the bindings crate, the world type of the core crate, and
the modules for holding, upgrades, soldiers, choosing, cohorts, the pyramid, the
bridge and the arithmetic. The search commands are in section 9 of the research
report, with what each reported.

## 2. Corrections to the brief

The brief made four checkable claims. Three held and one did not.

**The write surface is eight callables, not seven.** The brief named
`spawn_soldiers`, `despawn_soldiers`, `order_gather`, `found_settlements`,
`found_run_for_every_faction`, `prefer_at_sites` and `set_position_schedule`. It
missed `found_group`, which founds a group and hands back a settlement identity.
Nine methods on the world type take the lock mutably, and one of the nine is
`step`.

**`order_gather` gathers a resource kind and not units.** Confirmed.

**Movement is not orderable, and a record forbids a per-unit search.**
Confirmed. The brief then said this may be the mechanism that gives ordered
movement. It is stronger than that: the mechanism is **built**. The core answers
a return direction for a faction and an address, and it answers an exit
direction for an address and an option. What is missing is that the seed set is
fixed at the live sites of the faction, so the control plane cannot name a
destination.

**"There is no combat, no conversion, and no general build verb" is two thirds
right.** There is no combat and no conversion. **There is a general build
verb.** The world type is public on four build methods, and no line of the
bindings crate and no Python file names any of them.

## 3. What the work produced

**One research report.** It holds the verb table, the design of the territory
read, the line between data and code, an evaluation of the combat sketch, and
the recommendation.[^1]

**Two product records, both at `Shaped`.** One states the verb vocabulary need
and one states the presence read.[^3] [^4] The research report argues for two
rather than one: they fail different gate questions, their bounds are different
kinds of thing, and they ship apart.

**Four findings, three blockers and seven decisions.** Each register row is
listed in the research report where it was reasoned.

**Ten backlog items, all in `proposed/`.** None was refined, because each needs
a choice to close before its impact review can be written, and the item files
say which.

## 4. The combat sketch

The coordinator supplied a sketch for the attack verb during the work, with an
acceptance test from the project owner: one tank still kills four bowmen. The
research report evaluates it in full.[^5] The verdict in short.

**Sound: the threshold before aggregation.** It satisfies the tank test
structurally rather than by tuning a constant, because zero is the identity of
integer addition and a sum of zeroes is zero. It keeps every determinism rule,
because the fold is still exact integer addition over a set.

**Sound: a table over types rather than a fight for each pair.** The cost
follows the type count and never the population.

**Sound: one draw for a group rather than one for each unit.** The project
already holds the rule that does this, in the record for serving rations, and
combat should reuse it rather than invent a second one.

**Wrong: resolving the fight for each level 1 cell.** A cell covers 1024 tiles,
so a fight resolved there kills units across a block far wider than any front
line. The bridge already lists the units standing on one tile, so a tile
resolution needs no new input, and its cost follows the contested tiles rather
than the world. The research report states the measurement that would settle it,
and a blocker holds it.

**One risk the project owner must settle.** The hard threshold produces a cliff:
one point of armour makes a unit immune to a whole class, and it removes
attrition entirely for the pairs it applies to. Attrition is what produces the
crowd behaviour the owner also asked for. That is a game-design judgement and a
decisions register row holds it.

## 5. What was left undone

**No architecture decision record was written.** The brief forbade it. Eleven
are recommended by claim title in the research report, and each has a decisions
register row where the choice is open.

**No code was written and no crate was touched.**

**Nothing was measured.** The cost arguments in the research report say which
term a cost follows. None of them is a result, and one blocker governs every
cost figure in this project.

**Twelve assumptions about the downstream game were made and none was
verified.** They are listed in one section of the research report so that the
project owner can correct each one, and a blocker holds the gap.

## 6. The gate

`just records` runs eight checks. It reported 0 failures across all eight after
this work, with two notes about records that nothing cites and one note about a
fixture the conflict check did not read. Both notes predate this work.

## References

[^1]: Research report 21, what a god needs from this engine. `docs/research/reports/21-what-a-god-needs.md`
[^2]: Research report 20, what the Python interface should be, section 0. `docs/research/reports/20-the-python-interface.md`
[^3]: PRD-0030, a developer builds a game the engine did not anticipate. `docs/product/shaped/prd-0030-a-developer-builds-a-game-the-engine-did-not-anticipate.md`
[^4]: PRD-0031, a god knows whose ground its people stand on. `docs/product/shaped/prd-0031-a-god-knows-whose-ground-its-people-stand-on.md`
[^5]: Research report 21, what a god needs from this engine, section 4. `docs/research/reports/21-what-a-god-needs.md`
