# ADR-0065: A group is a site membership, not a region

## Context

This engine simulates a hex world. A unit stands on one tile. A settlement
stands on one tile and pools a store. The engine holds a small fixed set of
entity shapes, and a settlement is one of them.[^1]

A group is a set of units that belong together. Two kinds of group exist in
this project. A workforce is the set of units that work for one place. A
formation is the set of units that march under one command.

Both kinds face one question. **What names the members of a group?**

There are two answers a contributor can reach for.

The first answer defines a group by where its members are. A workforce is the
units standing on the tiles of a place. A formation is the units inside its
bounding area.

The second answer defines a group by a stored membership. Each member carries
the identity of its group. The group carries a reverse index.

The first answer is attractive. It costs no storage. It needs no bookkeeping
when a unit joins or leaves. It also reads like the way a person describes a
crowd.

**The first answer does not survive movement.** A region changes what it
contains between frames, because the units inside it moved. A command sent to
a region therefore changes its own recipient set while it runs. The project
has already recorded this, and it recorded it against a formation.[^2] The
same register records the military answer: membership is an ownership column
plus a reverse index, and it is not a spatial region.[^3]

The research on group spatial dynamics reaches the same place from a
different direction. It states membership as an ownership column on the unit
row, with a reverse index built by a counting sort.[^4] It treats the spatial
extent of a group as a derived summary, and not as the definition of the
group.[^4] It also states the rule that governs what a group may share: share
what is configured, never share what is accumulated.[^5]

So the project holds the military answer in a register and the civilian
answer nowhere. One question then has two answers, recorded in two places.
Nothing fails when the two drift apart, and that is a failure shape this
project has already met.[^6]

The second force is cost. A membership must be stored, and the largest count
in this project is the tile count. A membership held per tile is the
allocation the project refuses. A membership held per site is not. The number
of settlements is far below the number of tiles, and a site holds a small
fixed number of members.

The third force is staleness. A stored membership names an entity. An entity
dies, and the storage gives its slot to the entity created next.[^7] A
membership that stored a bare slot index would follow the dead member into
its replacement, and nothing would fail.

## Decision

### D1. A group is a membership held by the group, and every member is named by its whole identity

**A group is not a region.** No group in this engine is defined by the tiles
its members stand on, by a bounding area, or by any other spatial test. The
workforce of a place and the formation under a command take one shape.

The group holds one row of member entries. Each entry names one member, or it
names nobody. The entry stores the whole identity of the member, which pairs
the slot index with the generation the slot carried.[^7] It never stores a
bare slot index.

**An entry that names nobody is a state, not an absence.** The entry exists.
It carries what the group wants of the member that fills it, and its member
field is empty. A group that stored only its filled entries could not say
what it is short of. Being short of something is what makes a group ask for a
member.

Every read of a member resolves the stored identity against the storage that
minted it. The storage refuses a generation that has moved on. A group
therefore never reports the entity that took the slot of a dead member.[^8]

**The civilian case and the military case are one claim.** A workforce is a
membership held by a site. A formation is a membership held by a command
node. The register already resolved the military case this way.[^3] This
record is the whole rule, and not a second declaration of half of it.

The alternative, defining a group by a region, is rejected. A region is not
stable under movement. A command sent to a region therefore changes its own
recipient set between frames.[^2] The alternative is not cheaper either. The
spatial extent of a group stays available as a derived summary over the
members, and a derived summary cannot disagree with its source.

The alternative of storing a bare slot index is rejected. It is smaller. It
is also wrong in the one case that matters, which is the case where a member
died and the storage reused its slot. The failure is silent, because a bare
index reads back correctly and names the wrong entity.

### D2. A membership that names an entity that no longer exists is a defect, and a check refuses it

The invariant check fails when any member entry names an entity that the
storage no longer holds. A stale identity in stored state is the defect that
the generation exists to catch. Nothing may carry one across a frame barrier.

**The engine releases a dead member on every frame.** It does not wait for
the pass that resizes the membership. A member dies on any frame, so the
release must be as frequent as the deaths. A release on the resize interval
would leave a stale identity in the world between two resizes.

A group that is destroyed releases every entry it held. The group created
next in that storage slot inherits nothing.

### D3. The size of a membership is bounded by the ground, and it changes on a schedule

A membership held at a place cannot be larger than what can stand in that
place. The ground of a tile states how many units it admits. That table is
the one declaration of the bound.[^9] The width of a member row is folded
from the same table, so raising a capacity raises the row width. **The two
bounds are one bound.** A value declared in two places drifts, and nothing
fails when it does.[^6]

**The membership answers to what the group has and lacks.** A group that
lacks nothing needs nobody. A group that lacks something opens entries in
proportion to what it lacks. The split between several wants is exact. Each
want takes the truncated proportion. The remainder then goes one entry at a
time, in ascending want order. The parts sum to the whole, and no tie needs a
draw.[^10]

**The interval is a parameter, and no kernel holds it as a constant.** The
pass that resizes a membership runs on a schedule that the world carries. A
caller changes how often a group reconsiders without touching the engine.

The control plane changes what a set of groups wants with one command. **The
command names no member.** It states what a place wants, and the engine turns
that into a number of entries. A command that named the members would be the
control plane looping over entities, which this project forbids.[^11]

The alternative of resizing on every frame is rejected. Nothing needs it. A
pass over every group on every frame spends the frame for an answer that
rarely changes.

## Consequences

The project can no longer define any group by a spatial test. A feature that
wants the units near a place must ask for a membership, or accept that its
answer changes between frames.

A membership costs storage that grows with the number of groups. The project
accepts this cost for a per-group row and refuses it for a per-tile row.

Every read of a member costs a resolution against the storage. The engine
cannot hand out a member reference that skips the generation check.

A group cannot share accumulated state between its members. Each member
carries its own accumulated values. Exact allocation gives each member a
different integer, so the values diverge after the first application.[^5]

The spatial extent of a group is now a derived summary. Nothing stores it as
truth, so nothing can disagree with the members.

A pass that resizes a membership writes one row for each group and reads
nothing outside it. Each thread takes a contiguous span of groups, so the
result does not depend on which thread finished first.[^12]

## References

[^1]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^2]: Findings register, FND-010. `docs/FINDINGS.md`
[^3]: Blockers register, BLK-010. `docs/BLOCKERS.md`
[^4]: Report 17, group spatial dynamics, section 0. `docs/research/reports/17-group-spatial-dynamics.md`
[^5]: Report 17, group spatial dynamics, section 5.6. `docs/research/reports/17-group-spatial-dynamics.md`
[^6]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^7]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^8]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^9]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^10]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^11]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^12]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
