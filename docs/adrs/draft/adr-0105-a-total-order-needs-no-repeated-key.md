# ADR-0105: A total order needs no repeated identifier, only no repeated key

## Context

The engine sorts by a key vector rather than by a comparison function that
content supplies.[^1] A key is a list of ordering fields with a stable
identifier as its last field, and the identifier exists to break a tie between
two keys that agree in every earlier field.[^2]

The project cannot recover determinism once it is lost, so an order that
depends on anything the caller did not state is the failure this project is
built to prevent.[^3] An order is safe when the sort is total on the keys: when
no two keys are indistinguishable. Two indistinguishable keys can be emitted in
either order, and which one comes first may then follow the input arrangement,
the chunk a thread was given, or the thread count.

**The sort proved a stronger property than that, and it paid a second sort to
do it.** Before ordering, it collected the identifier of every key into a new
vector and comparison-sorted that vector to find any repeated value. Both
ordering passes of a frame paid it. The findings register holds the
measurement.[^4]

**The stronger property is not the one determinism needs.** Two keys that share
an identifier and differ in an ordering field are separated by the field they
differ in. Nothing is left for the identifier to decide, and the order of the
set is total without it.

**No test in the project exercised the property that determinism needs.** Two
tests asserted the refusal, and both used keys that share an identifier and
differ in an ordering field: a pair that ties nothing. A repeated key, meaning
two keys that agree in every field including the identifier, appeared in no
test in the repository.[^4]

## Decision

### D1. The sort refuses a repeated key, and a repeated key is one that agrees in every field

Two keys that agree in every ordering field and in the identifier are
indistinguishable. The sort refuses such a set and names the identifier of the
repeated key.

This is the whole of what the order needs. A set with no repeated key has a
total order on its keys, so the result depends on the key values and on nothing
else.

### D2. A repeated identifier that ties nothing is accepted

Two keys that share an identifier and differ in an ordering field are ordered by
the field they differ in. The sort accepts them.

This narrows what the sort refuses. A caller that reuses an identifier across
different ordering fields is no longer refused, and such a caller may have a
defect of its own. **That is the caller's business and not the sort's.** A sort
is not the place to enforce an invariant it does not need, at a price every
caller pays on every call.

### D3. The check reads the sorted order, and it costs one pass

Two keys that agree in every field sort adjacent, so a scan of neighbouring
pairs in the finished order is the whole of the test. The check therefore runs
after the ordering rather than before it, allocates nothing, and costs one pass
over a slice the sort already holds.

The check reports the identifier of the lowest repeated key. The report
therefore does not depend on the input arrangement or on the thread count, in
the way the earlier report did not.

### D4. The refusal is a property of the keys, so it does not depend on the thread count

The general sort divides the keys into one chunk for each thread and merges the
sorted runs. Whether a repeated key exists is a property of the set, not of the
division, so the same set is refused at every thread count and the same
identifier is named. A test drives the refusal at five thread counts.[^5]

## Consequences

**Every ordering pass loses a full sort of its own key set.** The saving follows
the key count, so the passes that sort the most gain the most. The cost register
holds what it was worth on the target platform.[^6]

**The sort refuses less than it did, and one class of caller defect now passes.**
A caller that issues one identifier to two keys with different ordering fields
was refused and is now accepted. No caller in the engine produces one: the
admission sort keys on the entity, one intent for each unit, and the bridge
rebuild keys on the entity, one entry for each live soldier. Both are unique by
construction, and neither relied on the sort to tell them so.

**The check can no longer report before the work is done.** It runs after the
ordering, so a set with a repeated key is ordered and then refused, rather than
refused and never ordered. The wasted work is one ordering pass on a set the
caller should not have submitted.

**A test now exists for the property the sort actually guarantees.** Three
tests cover it: a repeated key in the general order, a repeated key in the
bounded order, and a repeated key at five thread counts. Removing the check
fails all three, which is what makes the check something other than
decoration.[^7] The two tests that existed before covered only the wider
property and passed against a sort that guaranteed nothing about a tie.

**The error names a different thing, so it is named differently.**
`RepeatedIdentifier` became `RepeatedKey`. A variant that kept the old name
would say that two keys carry one identifier, which is no longer what the sort
refuses.

## References

[^1]: ADR-0007, content supplies a key vector, never a comparator, decision D1. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
[^2]: ADR-0007, content supplies a key vector, never a comparator, decision D2. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
[^3]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^4]: Findings register, FND-304. `docs/FINDINGS.md`
[^5]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^6]: Target platform costs, every stage of a frame after the sort stopped sorting twice. `docs/reference/graviton-costs.md`
[^7]: Testing Rules, section 1. `.claude/rules/testing.md`
