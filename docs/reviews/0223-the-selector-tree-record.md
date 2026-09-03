# Review 0223: The selector tree record

## What was reviewed

| Item | Value |
|---|---|
| `docs/adrs/accepted/adr-0051-a-selector-is-a-lazy-expression-tree.md` | `Draft` at review, `Accepted` after it |
| Commit | `3aa63c8`, the head of the review branch |
| Code read | the bindings, the type stubs, the tier module, the public interface tests, and the Python package |

The reviewer did not write this record. The reviewer returned ADR-0040 and
ADR-0043 earlier in this session, and both are cited by this one, so the
reviewer came to it expecting to return a third. Section 4 states what the
reviewer did about that.

**The reviewer compiled nothing.** Other workers hold the machine.

## Verdict

**Accept.**

**Nothing implements this record, and the acceptance says so plainly.** No
selector type exists in Python or in Rust. The Python package says in its own
text that the selector API is not written. The record states this in its last
consequence rather than leaving a reader to find it. The registry permits
acceptance ahead of the code on exactly that condition, and this is that
case.[^1]

**The record was moved and the registry row was set.** The move cost five path
citations, all in documents, and the reviewer holds all five. No source file
names this record, so the sweep that makes a move expensive elsewhere does not
apply here.[^2] Section 6 records the move.

**Five objections were attempted. Four failed outright and one failed
narrowly.** The narrow one is in section 3 with a clarification the author may
take or leave; it blocks nothing.

## 1. Why this record earns acceptance before its code

The scope rule asks three questions and this record answers all three.[^3]

**Could a contributor reasonably choose otherwise?** Yes, and the record names
the four choices they would reach for: compute the set in Python, send a query
string, offer a fixed set of named reads, or evaluate each predicate eagerly.
Each is a real design that ships in real libraries. Eager evaluation in
particular is what a contributor writes first, because it is the one that can be
tested one predicate at a time.

**Does choosing otherwise cost more than changing it later?** Yes, and this is
the strongest part of the case. The owner named the set-valued command as how
the control plane reaches a set, so every verb written from now on is written
against whatever this record decides.[^4] A verb written against eager sets is
not adjusted later; it is rewritten, along with its tests and its stubs.

**Is the reasoning visible in the artefact?** There is no artefact. The record
is the only place the reasoning can live.

**The counter-test also applies.** D5 is a determinism decision: it fixes the
result order by a stable key and forbids a float in a predicate. A contributor
writing a parallel evaluator would not otherwise know that the order of the
result is part of the contract, and determinism is the property this project
cannot recover.[^5]

## 2. The decisions, read one at a time

**D1, a selector describes a set and is not a set.** Sound, and it is the claim
the title states. Building reads nothing, so composition costs nothing.

**D2, the tree crosses once with the verb that consumes it.** Sound and
consistent with the boundary rule it cites. A tree that crossed node by node
would make the crossing count a function of the expression, which is the failure
D2 of the boundary record names.

**D3, a mixed combination fails when the tree is built.** Sound. The alternative
it names, an empty result at run time, is the worse failure precisely because a
test cannot distinguish it from a correct empty answer. Naming the bridges in
both directions is what keeps the consultation of the unit-to-tile bridge
visible, which matters because that structure rebuilds at the barrier.

**D4, a selector holds no snapshot.** Sound, and the record states the surprise
it creates rather than hiding it. Its rejection of the alternative is the right
shape: binding a selector to a frame would make the engine hold a result whose
size is a function of the population.

**D5, evaluation fixes its order.** Sound, and it is the determinism decision.
The float clause is not decoration: a predicate is the one place where a
comparison could quietly re-enter through a threshold a caller supplied.

**D6, a selector reports the evaluation it will perform.** Sound, and section 3
records the attempt to split it out.

## 3. The objections

**Should D6 be its own record?** The scope rule says to split a record that
states two claims a reviewer could accept separately, and an explain operation
looks separable from a lazy tree.[^6] **The objection fails.** D6 exists because
D1 and the first consequence together hide the strategy from the caller: the
engine owns evaluation and the caller cannot override it. A caller who cannot
choose the strategy and cannot see it has no way to tell a slow schema from a
slow query. Accepting D1 while rejecting D6 leaves a deliberate blindness with
no remedy, so they are one claim about one mechanism.

**Does the record forbid the verbs that exist today, which take an explicit
column of identities?** **The objection fails**, and the record answers it in
its own alternatives: a named read remains correct for a question the engine
answers better than a general tree can, and the record does not forbid one. The
existing verbs are not superseded by this record and do not need to be.

**Does D3 promise a static check that Python cannot give?** The wording is "an
error at the moment the caller writes it", which reads like an editor-time
check. **The objection fails** on the decision's own heading, which says the
failure happens when the tree is built. Composition time is what is claimed and
composition time is achievable.

**Does the node cap in the consequences smuggle a figure into a record?**
**The objection fails.** The record states that a cap exists and that the
refusal is typed, and states no number. That is the correct division: the rule
is the decision and the number is living reference.[^7]

**Does the context contradict the boundary record?** This one **fails
narrowly.** The context says a caller who names a set cannot name it by listing
it, because the list is the population. The boundary record says the opposite in
its own D1: the rule binds the direction of travel and not the volume, and one
column of results is one answer that crosses once. A caller passing a column
back to a verb is naming a set by listing it, in one crossing, and the boundary
record's D3 requires the verbs to accept exactly that.

Read in its paragraph the sentence means something true and narrower: a caller
that must *describe* a set it does not already hold cannot produce the
enumeration without the world. Nothing downstream of the sentence depends on the
wider reading, and no decision changes either way.

**A clarification the author may take.** Change "cannot name it by listing it,
because the list is the population" to "cannot produce that list without the
world, because the list is the population". This blocks nothing and the
reviewer did not apply it.

## 4. What the reviewer did about expecting to return it

Two records had already been returned in this session, and a reviewer who has
found a defect twice looks for a third rather than reading what is there.

The reviewer therefore did not begin with the current code, which is what
produced both earlier objections. It began with the scope test, asking whether
the record should exist at all, and then read each decision on its own terms
looking for a claim that could be false. Only after that did it read the record
against the code, which is where the fifth objection came from.

**The result is that the accept rests on a different reading than the two
returns did.** The two returned records both make claims about the current
surface. This one makes none, which is why the same reading finds nothing to
return.

## 5. What acceptance does not settle

**It does not make the selector exist.** The record binds the verbs written
after it and describes no code. A reader who meets an accepted record and
assumes an implementation is the failure this project keeps recording, so the
record's last consequence and this section both say otherwise.[^8]

**It does not settle whether the result is a list.** That argument is the
sibling record, reviewed separately.[^9]

**It does not close the gap that made the record necessary.** The control plane
still cannot say where to act. The item that closes it is open.[^10]

## 6. The move

Accepting a record moves its file, and every citation of the old path then names
nothing. That cost is why two records at verdict Accept are still drafts.[^2]

Here the cost is five path citations: the registry, the decisions register, and
three sibling records. All five are documents, and no source file names this
record. The reviewer performed the move and repaired all five in the same
commit, because the alternative is a registry row and a file that disagree,
which is the shape this project punishes.

Two of the three sibling records are themselves returned for amendment. Editing
a footnote path in a returned draft is a mechanical repair and not a rewrite of
the record.

## References

[^1]: ADR Registry, who reviews. `docs/adrs/REGISTRY.md`
[^2]: Findings register, FND-197. `docs/FINDINGS.md`
[^3]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^4]: Decisions register, DEC-063. `docs/DECISIONS.md`
[^5]: Decision Record Scope, the counter-test. `.claude/rules/adr-scope.md`
[^6]: Decision Record Scope, section 2. `.claude/rules/adr-scope.md`
[^7]: Decision Record Scope, section 4.1. `.claude/rules/adr-scope.md`
[^8]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
[^9]: ADR-0052, a selector result may be a range, not only an enumerated set. `docs/adrs/draft/adr-0052-a-selector-result-may-be-a-range.md`
[^10]: Backlog item 0161. `docs/backlog/proposed/0161-let-a-selector-say-where-to-act.md`
