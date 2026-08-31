# ADR-0022: Level 0 is the only truth, and every level above it is derived

Status: Draft

## Context

The engine simulates a hex world at three levels of detail. Level 0 holds
individual tiles and units. Level 1 summarises blocks of tiles at city scale.
Level 2 summarises blocks of level 1 cells at region scale.

A summary can be built in two ways. It can be derived from the level below it,
recomputed whenever that level changes. Or it can be simulated in its own
right, with its own rules, and reconciled with the level below from time to
time.

The second is tempting. A region that runs its own economy costs a fraction of
the tiles it covers, and a strategy game usually wants a region to behave as a
thing rather than as a sum. Several published engines take that route.

It has one failure that this project cannot absorb. When a level runs its own
rules, the two levels disagree, and nothing says which is right. The
disagreement is not a defect that a test can name: both levels are internally
consistent, and only the relationship between them is wrong. A player who
zooms in sees a different world from the one the summary described, and the
engine has no rule for what to do about it.

The project also hashes the whole world state each frame and compares it
against a stored file.[^1] A derived level adds nothing to that hash that the
level below does not already determine. A simulated level adds state that must
be hashed, ordered, and reproduced, at every level.

## Decision

### D1. Level 0 is the only source of truth

**Every fact about the world lives at level 0.** A tile field, a unit, a
faction, a stockpile: each is stored once, at level 0, and nowhere else.

No level above level 0 holds a fact of its own. A value that appears only at
level 1 is a defect, not a feature.

### D2. Every level above level 0 is a pure function of level 0

**A level 1 cell equals the exact combination of the level 0 tiles it covers.
A level 2 cell equals the exact combination of the level 1 cells it covers.**

The equality is exact and not approximate. A test may recompute any cell from
the level below and compare, and the two must be equal.

This is what makes an incremental update legal. An incremental update is an
optimisation that must give the answer a full rebuild would give, and D2 is
the statement it is checked against.

### D3. No system writes to a level above level 0

A simulation system reads level 0 and writes level 0. It never writes a
summary. The pyramid is maintained by one mechanism, not by every system that
happens to change a tile.

A system that wants to change a summary changes the tiles the summary covers.

### D4. A reader may read any level, and the level it read is part of the
answer

A caller that asks a question of level 1 gets an answer about level 1. The
engine does not silently substitute one level for another, and it does not
promise that a level 1 answer equals a level 0 answer to a question that level
1 cannot express.

A count of units in a region is the same at both levels, because a count is a
sum. The identity of the largest settlement is not, unless the summary was
built to carry it.

## Consequences

**A region cannot have a rule of its own.** A design that wants a province to
behave differently from the sum of its tiles must express that difference as a
tile field. This is a real cost, and it is the price of never having two
levels disagree.

**The pyramid can always be rebuilt.** A save file need not hold it. A
corrupted summary is repaired by recomputing it, and no information is lost,
because no information lived there.

**A summary field must be expressible as a combination.** A quantity that
cannot be combined from its parts cannot be a summary field at all. What
combination means, and which fields qualify, is the subject of another
record.[^2] [^3]

**The state hash covers level 0 and need not cover the pyramid.** A pyramid
that disagrees with level 0 is a defect the equality check finds, and the hash
would find it too but would not say which level was wrong.

**An incremental update is always optional.** The engine may rebuild any cell
at any time and get the same answer. That is what makes the two update paths a
cost decision rather than a correctness one.[^4]

## References

[^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^2]: ADR-0023, an aggregate combines exactly, in any order. `docs/adrs/draft/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^3]: ADR-0024, every summary field is declared extensive or intensive. `docs/adrs/draft/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
[^4]: ADR Registry, row 0025. `docs/adrs/REGISTRY.md`
