---
id: 0506
title: Put the wealth path out of easy reach
status: complete
created: 2026-09-05
implements: [ADR-0148 D3]
changes: [ADR-0148]
creates: [ADR-0165]
serves: [PRD-0053]
blocked-by: []
---

## Why

**The wealth clause ends every game, and the balance register cannot stop it.**
A sweep of 32 seeds of the demonstration world, each played to 20000 ticks,
ended 32 games on the wealth-or-wonder path. The stock clause ended 29 of them
and the wonder clause ended the other 3. Domination and territory ended none.
The project owner asked for wealth to sit far out of reach, so that domination
and territory decide a game.[^1]

**The value has a ceiling that is one seventh above where it stands.** A
faction founds one settlement, the commodity count is one, and a store is a
`Fix32`, so the stock total of a faction clamps just below 32768 whole units.
The target stands at 28672, which is seven eighths of the clamp. The store
rises steadily and reaches the clamp, so the reader fires at every target the
engine permits, given enough ticks. Raising the row buys ticks and nothing
else.

**A row that cannot be set is worse than a row that is unset.** The stock
target has a register row, and the row invites a reader to believe the value is
free. It is not. A worker who raises it will measure a later end tick and
report a success, and the path will still end every game.

## Done when

- The wealth clause reads a value that a bar can stand out of reach of, and a
  record holds the decision and its alternatives.
- A check inside the build fails when the bar falls back under the ceiling.
- A unit test drives the reader at each extreme: a world at the old clamp wins
  nothing, a world above the new bar wins, and the value the reader totals
  stays exact at the target scale. Each test goes red when its defect is put
  back.
- The 32-seed sweep at 20000 ticks runs again, and the wealth-or-wonder path
  ends most games no longer.

## Outcome

**The reader now compares against a bar that stands above the stock one
settlement can hold.** The stock total was never the narrow part: it already
sums every commodity of every live settlement into a 64-bit accumulator. The
narrow part is the store, which is a fixed-point value of a fixed width. A bar
under that ceiling is crossed by a faction that founds one settlement and
waits. The bar is now a whole multiple, greater than one, of the share one
settlement carries, and a compile-time assertion stops the build when the bar
is not above the ceiling. The record holds the decision, the alternatives and
their costs.[^2]

**A wider accumulator was never an available answer.** Two registers and this
item named it as the cheap fix. The reader already had it. A finding records
the correction.[^3]

**The wealth bar was declared in three places.** Two scripts held a copy of the
bar and reported a share against it. The binding now exposes the bar and each
script reads it. A second finding records that.[^4]

**The sweep result is in the commit body, and the balance register rows follow
in a separate change**, because another worker holds that file. The
wealth-or-wonder path no longer ends most games. The wonder clause now carries
the path, and the stock clause fires in no seed, because nothing in the engine
founds a second settlement yet.[^5]

## References

[^1]: Findings register, FND-543. `docs/FINDINGS.md`
[^2]: ADR-0165, the wealth bar stands above what one settlement can hold. `docs/adrs/draft/adr-0165-the-wealth-bar-stands-above-what-one-settlement-can-hold.md`
[^3]: Findings register, FND-548. `docs/FINDINGS.md`
[^4]: Findings register, FND-549. `docs/FINDINGS.md`
[^5]: Findings register, FND-542. `docs/FINDINGS.md`
