# Review 0223: The descent record

## What was reviewed

| Item | Value |
|---|---|
| `docs/adrs/draft/adr-0078-descent-is-a-bounded-record-and-a-relation-is-a-bounded-recursion.md` | `Draft` at review, and `Draft` after it |
| Commit | `3c8f483`, the head of the review branch |
| Code read | the descent module in full, the character arena's descent readers, the world's four character readers, the descent integration tests, and the scale constants table |

The reviewer did not write this record and has no record of its own that cites
it.

**The reviewer compiled nothing.** Other workers hold the machine, and one of
them holds the character module, so the reading is stated against a commit.

## Verdict

**Accept. The status was not changed, and the reason is mechanical.**

Every decision holds against the code, both figures the record refuses to state
are in the reference table with their derivations, and the exactness argument is
held by a compile-time assertion rather than by a comment.

Five objections were attempted. Four failed. The fifth found a real gap between
the record's motivating force and the readers that exist, and section 4 explains
why that is a register row rather than a correction to this record.

**Accepting moves the file, and the record is cited six times by path, in the
descent module and in the character module.** Document work may not touch either,
and one of them is held by another worker right now. The verdict is the
reviewer's judgement; the move and its sweep are one commit for whoever may edit
`crates/`.[^1]

## 1. The decisions against the code

**D1, descent is keyed on an identity the record never reissues.** Holds. The
record allocates a row for each character and never removes one. A parent edge
stores a descent identity, not a slot and not an entity identity. Each row keeps
the entity identity the arena minted, and the lookup from an entity to a descent
identity filters on that stored identity matching, so a reused slot does not
answer to the old character.

**D1's bound.** Holds. The record refuses a creation past a ceiling, and the
ceiling is a named constant in the module and a row in the scale constants
table, with its derivation: sixteen times the character ceiling, above the
dead-to-living ratio the character report derives. The record states the
constraint and no number, which is what the scope rule asks for.[^2]

**D2, a bounded recursion and never a walk to a common ancestor.** Holds, and
the implementation is the recursion the record names. The expansion takes the
larger identity, which is the younger character because rows are allocated in
birth order, so the recursion terminates on the identity as well as on the
depth. The self case is one half of one plus the inbreeding coefficient, and the
inbreeding coefficient is the kinship of the two parents, which is Karigl's
recursion as the research gives it.

**D2's exactness is enforced, not asserted.** The module carries a compile-time
assertion that the relation depth is below the number of fractional bits in the
scale. That is the record's exactness argument, held where the compiler can
check it. A contributor who raises the depth past the safe value does not get a
subtly rounded relation; they get a build failure.

**D2's absent parents.** Holds. A missing parent is a sentinel that the
recursion reads as zero, so a founder is not a special case, which is what the
promoted-soldier answer requires.

**D2's doubling.** Holds. The world reports Wright's coefficient as the kinship
added to itself, through the arithmetic module, and the module's comment gives
the record's reason: a doubling is exact.

**D2's memo.** Holds. It is an ordered map, read by key and never iterated, so no
map order reaches a result.

**D3, every walk returns ascending descent identity order.** Holds. The walk
sorts each frontier before it expands and sorts the result, the frontier is a
sorted vector and never a hash set, and the module says so at both sites.

**D4, a parent must be an older row.** Holds, and it is tested twice: a parent
that is the child is refused, and a parent identity the record has not issued is
refused. There is no cycle check anywhere, which is the point: the refusal is
structural.

## 2. The tests drive the real caller

The relation is exercised through the world, not through the descent module. The
integration tests build a world, bear characters, and assert the relation of a
mother and a child, of two siblings, of a grandparent and a grandchild, of two
unrelated founders, and of a character with itself.

That matters because the testing rule asks who is obligated to invoke a
capability, and requires the test to start where that caller starts.[^3] For
these readers the caller is a user of the engine, and the tests enter where a
user would.

## 3. The objections that failed

**Does the record state a figure?** **No.** Neither the ceiling nor the depth
appears in it. Both are named as quantities the reference table holds with their
derivations, and both are there.

**Is D2's termination argument airtight, or does it rest on the depth alone?**
**The objection fails.** Two things decrease. The depth falls on every call, and
the expansion replaces the larger identity with its parents, which carry smaller
identities. Either alone would end the recursion.

**Does the memo key admit two entries for one pair?** **The objection fails.**
The key orders the pair before it looks up, so the relation of a pair is
computed once for each depth whichever way round the caller asked.

**Does the relation of a character with itself contradict D2's claim that the
result is exact?** **The objection fails.** The self case adds one to the
inbreeding coefficient and halves, then the report doubles it, so an outbred
character stands at one to itself. Every step is a halving or a doubling of an
integer that stays above one at the stated depth.

**Is the relation a capability nothing invokes?** This is the objection that
found something, and as an objection to this record **it fails.** The engine
exposes the reader, the integration tests drive it through the world, and the
record claims no more than that. What it found belongs in section 4.

## 4. The gap the fifth objection found

The record's first force is that a parent edge cannot live in a structure keyed
on a slot, because a watcher must read a parent after that parent has died. The
storage delivers that: the descent record outlives every character in it.

**No reader does.** All four world-level readers take an entity, and an entity
that names a dead character resolves to nothing. The parents of a dead character
return nothing, its ancestors and descendants return an empty list, and its
relation to anybody returns zero. Zero already means two things in this record,
unrelated and beyond the depth, and a dead character makes three.

**A caller can hold a dead character's identity and do nothing with it.** The
ancestor walk returns descent identities, and an ancestor is usually dead. There
is no world-level reader that takes a descent identity, so the answer names
characters the caller cannot ask about. That is the same shape as a boundary
that answers a question a caller cannot act on.[^4]

**This is not a correction to the record.** The record makes no claim that a
reader exists. It states the force, it makes the storage support it, and it stops.
Two other records in this corpus name their own gaps, and both do so because they
claim an enforcement or declare a type; this one claims neither, so silence here
is an omission and not a falsehood.

**It is a choice, and choices go in the register.** Whether a world-level reader
should take a descent identity, or whether the record's force should be narrowed
to what the readers give, is a decision with options. It is filed.[^5]

## 5. Why this record earns its place

The obvious implementation is named in the record and rejected by the accepted
product record: walk two lines upward until they meet. A contributor writes that
first, it is correct, and its cost is the depth of both lines on every ask. The
recursion that replaces it does not look like the natural thing, and nothing in
the module can say why the walk was refused.

The second thing a contributor would reach for is a closure table of every
ancestor and descendant pair, which the research rejects on the arithmetic of a
graph with in-degree two.

The third is a float. The record's strongest paragraph is the one that says the
integer form is the correct form here and the float form is the lossy one,
because every step of the recursion halves and the scale has bits to spare. That
inverts the usual assumption, it is checkable, and the compile-time assertion in
the module is what holds it.

D3 is a determinism decision. A frontier held in a hash set would give a
different result order on a different run, and no single-threaded test would see
it.

## 6. What acceptance does not settle

It does not give the control plane any access to a character. No binding exposes
a character, a parent, a walk or a relation, so the product need behind this
record is served in Rust and not in Python.

It does not settle whether a dead character can be asked about.[^5]

It does not price a walk. The record says so and cites the blocker.[^6]

## References

[^1]: Findings register, FND-197. `docs/FINDINGS.md`
[^2]: Decision Record Scope, section 4.1. `.claude/rules/adr-scope.md`
[^3]: Testing rules, section 5. `.claude/rules/testing.md`
[^4]: Findings register, FND-147. `docs/FINDINGS.md`
[^5]: Decisions register, DEC-092. `docs/DECISIONS.md`
[^6]: Blockers register, BLK-007. `docs/BLOCKERS.md`
