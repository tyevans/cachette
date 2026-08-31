---
id: 0019
title: Implement the soldier column set
status: complete
created: 2026-08-30
implements: [ADR-0066 D1, ADR-0066 D3]
changes: []
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

The world has tiles and no entities. PRD-0002 needs entities that appear on
the world and move between tiles, so something must hold them.

ADR-0066 fixes four shapes and gives each its own column set. The soldier is
the mobile one, so it is the shape this sprint needs. The other three wait:
the record fixes four shapes, and it does not require that all four exist at
once.

## Impact review

**Governed by.** ADR-0066 D1 gives each shape its own columns, and D3 makes a
shape that is not one of the four a compile-time error rather than a run-time
table. Row 0012 holds the arena claim and row 0014 holds the identity claim;
item 0029 writes both before this item starts. ADR-0002 D1 keeps every
soldier field exact. ADR-0004 D1 requires an explicit stable order over the
column. ADR-0006 D1 governs any field whose bytes reach the state hash.

**Changes.** None to a record. One change to existing code is required, and
it was found while reviewing ADR-0014.

ADR-0014 D1 says a caller reads the parts of an identity through accessors
and never constructs an identity from parts. `Entity::new(index, generation)`
in the value types module is a public constructor from parts, so the code
contradicts the record. The arena is the only thing that may mint an
identity. A public constructor lets any caller forge one, which manufactures
the silent wrong-entity failure that ADR-0014 exists to prevent.

Make the constructor reachable only by the arena. The accessors stay public,
because D1 requires them.

A second defect in the same type came out of the same review. The identity
packs the generation above the slot index in a value that cannot be zero, so
slot zero at generation zero has no representable identity. Under D3 a fresh
slot starts at its first generation, and the first entity the engine ever
allocates takes slot zero. ADR-0014 D6 now says a generation starts at one,
which removes the case for every slot at once. The arena must honour it, and
a test must allocate the very first entity and assert that it has an
identity, because every test that allocates a second entity first would pass
without it.

**Creates.** None, if items 0029's records hold. A decision this work finds
that no record holds is a deliverable of this item, not a byproduct.

**Blockers.** BLK-007 governs every cost figure. State none, in the code or
in a comment. The faction ceiling is 63 and it is settled, so a soldier's
faction is a bit index and not a parametric value.

**What the caller can get wrong.** The sprint 1 retrospective asks this at
refinement, because two of five items found the answer late. A caller can ask
for more entities than the arena holds, ask for an entity on a tile outside
the world, hold a handle to a despawned entity, and ask for a faction above
the ceiling. Each needs a typed refusal rather than a panic, and each needs a
test.

**Precedent.** FND-041 records that the project stated capabilities nobody
checked. A column set that no system reads is inert, so a test drives the
step and then inspects the arena.

## Done when

- A soldier carries a generational handle, a tile address, and a faction.
- A handle names at most one live soldier. A stale handle reads as absent
  rather than as a different soldier.
- The generation advances when the arena frees a slot, not when it allocates
  one, and a test proves that a stale handle fails immediately after the
  free rather than after the reuse.
- A freed slot returns to use in first-in first-out order.
- No caller outside the arena can construct an identity from an index and a
  generation. The public constructor is gone or restricted, and the whole
  tree is searched for its call sites.
- The first entity that the arena ever allocates has a representable
  identity. A test allocates exactly one entity into an empty arena and
  asserts this, because a test that allocates two would not see the defect.
- The arena refuses, with a typed error, each of the four caller mistakes
  named above.
- A soldier is placed on a tile that the grid contains, and placing one
  outside the world is refused.
- The state hash covers the soldier columns, and the golden files are
  re-recorded after reading the difference.
- A property test asserts that a handle round-trips through the arena.
- A property test asserts that spawning and despawning in any order leaves
  the arena holding exactly the live set.
- The thread-count test covers a world that holds soldiers, at 1, 2 and 12
  threads, and the perturbed build fails it.
- The new tests are checked against a mutation, as sprint 1 did, and the
  mutations are named in the commit body.
- `just check` runs green.

## Outcome

`crates/cachette-core/src/soldier.rs` holds the arena. A soldier carries a
generational handle, a tile address and a faction. The world exposes spawn,
despawn and place, so the arena has a real caller.

Both repairs are done. `Entity::new` is now reachable only inside the crate,
and the four call sites in the value type tests were rewritten to mint
identities through the arena rather than deleted, so they exercise the real
path. Generations start at one, and a test allocates exactly one entity into
an empty arena and asserts it has an identity.

Two things changed from the plan.

The golden test gained a third scenario. The plan asked only that the state
hash cover the soldier columns, which it does. But the two existing golden
scenarios never spawn a soldier, so their stored hashes covered an empty
arena and could not catch a change to how soldier state is represented. The
new scenario spawns, frees and respawns, so it exercises the generation
advance, the free queue and a reused slot. It was checked against a mutation:
swapping the free queue to last-in first-out changes the recorded sequence,
and the two older scenarios stay green.

The mutation check found nothing wrong with the code, and one thing wrong
with the method. A first pass ran only the integration target and reported
that a retired slot returning to the queue killed no test. The tests for that
rule are unit tests inside the module, which that target does not run. The
tests were right and the measurement was wrong. A mutation check must run
every target.

The Python bindings do not expose soldiers. Nothing needs them yet, and the
product record puts the viewer in Rust.

Mutations applied, against every target:

| Mutation | Record | Tests killed |
|---|---|---|
| A generation starts at zero | ADR-0014 D6 | 5 |
| Last-in first-out slot reuse | ADR-0014 D4 | 1 |
| A retired slot returns to the queue | ADR-0014 D5 | 2 |
| The generation does not advance on the free | ADR-0014 D3 | 1 |

## References

[^1]: ADR-0066, entity storage holds four fixed shapes. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^2]: ADR-0002, simulated and aggregated state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^3]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^4]: ADR-0006, an event is plain data and applying it is pure. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
[^5]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^6]: Findings register, FND-041. `docs/FINDINGS.md`
