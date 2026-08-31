# Review: the three storage records

**Reviewed:** ADR-0012, ADR-0014 and ADR-0018 in `docs/adrs/draft/`, against
the six records they must not contradict, the registers, and the core crate,
at commit `5b80f6f`.

**Reviewer:** an agent that wrote no part of the three records. The approver
read them separately and reached two of the same findings independently; this
file records both readings and says which found what.

**Verdicts:** ADR-0012 `ACCEPT WITH AMENDMENT`. ADR-0014 `ACCEPT WITH
AMENDMENT`, two of them, both blocking. ADR-0018 verdict outstanding.

## ADR-0012, tiles are dense columns and units are a generational arena

Passes the three-condition test. The fork is live, because several engines do
make a tile an entity. The split is expensive to reverse, because every
system reads a tile by index and a unit by identity. The rejected cost, which
is a location table with one row for every tile, is invisible in code that
merely subscripts an array.

**The amendment that held.** D1 said a tile lookup is an array subscript and
never a resolution. Under ADR-0016, which stores tiles in block-tiled order,
the axial index is not the array offset: a shift and a mask sit between them.
As written, D1 pre-empted a record that has not been written yet. ADR-0018
confronts the same point openly; ADR-0012 did not.

**Applied.** D1 now says a tile lookup derives its position from the address
by arithmetic and never consults a table, and states that a block order adds
a shift and a mask but adds no lookup. The constraint is the absence of a
resolution, not the presence of a bare subscript.

**A bundling observation, not grounds for rejection.** D2 states that each
tile field is its own dense column. That is separately rejectable: a reviewer
could accept that a tile is not an entity and still store tiles as an array
of structures. The alternatives section argues only D1, and D2 overlaps two
reserved rows. It stays, because D2 is load-bearing for the zero-copy
consequence that ADR-0066 already depends on. It is recorded here so that a
future split has its reasoning ready.

**Objections attempted that failed.** That D2 is a module arrangement: failed,
a data layout with a stated alternative is a constraint, not a statement of
where code lives. That the record duplicates ADR-0066: failed, ADR-0066
assumes the split and says it does not state it. That it holds a volatile
figure: failed, "grows with the tile count" names a relation and no value.

## ADR-0014, entity identity is an index plus a generation

Two amendments, both blocking, both now applied.

**The first was found by the approver and confirmed here.** D4 argued for
first-in first-out slot reuse on the grounds that last-in first-out would let
a stale identity match a new occupant. D3 forecloses that: the generation
advances on the free, so a captured identity at one generation meets a slot
at the next and correctly fails. The sentence reads like a fragment of an
argument written against a design that advances the generation on allocation.

The decision survives on a different force. First-in first-out maximises the
number of allocations between a free and its reuse, which defers generation
wraparound on any one slot, which defers the permanent retirement that D5
imposes. As first written, D4 was a real constraint resting on a false
premise, and the first contributor to trace it would have concluded the
constraint was unfounded and replaced the queue with a stack.

**The second was found by this review alone, and the approver had missed it.**
The identity packs the generation above the slot index into a value that
cannot be zero. Under D3 a fresh slot starts at its first generation. If that
generation is zero, then slot zero at generation zero packs to zero, which
the type cannot hold. **The first entity the engine ever allocates takes slot
zero, so it would have no representable identity.**

The failure appears once, at the first allocation, for one slot. Every test
that allocates a second entity before checking anything would pass.

**Applied as a new decision.** A generation starts at one and never at zero.
The rejected alternative is to forbid the allocator from issuing slot zero,
which wastes a slot and puts the rule where each future allocator must
remember it, rather than in the identity where it holds once. Generation zero
now means a slot has never been used.

**The third finding, reached by both readings.** D1 says a caller never
constructs an identity from parts, and the value types module exposes a
public constructor that takes exactly those two parts. The record contradicts
the code on the day it would be accepted. The record is right: only the arena
may mint an identity, because a public constructor lets any caller forge the
silent wrong-entity failure the record exists to prevent. Backlog item 0019
carries the repair and names the call sites.

## ADR-0018, the unit-to-tile bridge

**Outstanding.** The verdict was truncated in transit and has not been
re-delivered. ADR-0018 therefore stays a draft with no review, and it is not
accepted. This is recorded rather than quietly carried, because a missing
verdict and a clean verdict look the same in a summary.

## What this review changed about the ceremony

The reviewer found one defect the approver did not. That is the whole
argument for the second reading, and it is the reason a delegated review
needs an agent that did not write the work.

## References

[^1]: Decision Record Scope. `.claude/rules/adr-scope.md`
[^2]: Reviews index. `docs/reviews/README.md`
[^3]: Findings register, FND-043. `docs/FINDINGS.md`
