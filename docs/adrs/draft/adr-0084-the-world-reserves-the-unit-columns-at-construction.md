# ADR-0084: The world reserves the unit columns at construction, and a spawn past the reservation is refused

## Context

A unit lives in a generational arena. The arena holds one column for each
field of the unit shape, and the identity of a unit pairs a slot index with a
generation.[^1] [^2] The arena never compacts the slot index space, so the
number of slots it has opened is a high water mark that never falls.[^2]

A column is a growable array. Nothing in the shape of a growable array says
how much it must hold, so a column that is never told grows on demand. It
grows by allocating a larger block, copying what it held, and freeing the old
block. That work happens at the moment of the entry that does not fit.

**The engine grew.** The arena opened as many slots as the slot index holds,
which is the range of the index rather than a chosen bound, and it reserved
nothing for them. Each spawn appended one entry to each column. The refusal
path existed and was unreachable, because a run cannot reach the range of the
index. A product review founded a group through the public interface and read
the capacity back as that range. The findings register holds what was believed
and what was true.[^3]

**The accepted product record states the opposite property.** The storage the
world reserves is sized for the target population, it does not change during a
run, and a run does not stop to grow.[^4] A record the code contradicts is
worse than no record, because it lies.[^5] The project owner closed the choice
between changing the code and changing the record, and chose the code.[^6]

### The forces

**A reallocation lands inside a step, at a moment nobody chose.** The spawn
that does not fit is an ordinary spawn. It is not the largest, and it is not
the last. A run therefore pays for the copy of every column on a tick that
depends on the population, and a caller that reads the frame cost sees a
figure that has nothing to do with the frame. No run has priced the copy, so nobody
can say what it costs, only when it arrives.[^7]

**A cost that arrives at construction is a cost a developer can see.** A
reservation is one call, in one place, before the first frame. A developer who
builds a world knows what the world will hold. A developer who is part way
through a run does not.

**The alternative is not free either.** A reservation is memory that a run
holds and does not use. The product record accepts this by name: a world sized
for its target pays the target from tick zero while a hundred units live in
it.[^4]

**An unreachable refusal is not a refusal.** The arena already returned a
typed refusal when it could open no further slot. Nothing could reach it, so
nothing tested it, and the first caller to meet it would meet it in a run
rather than in a test. A bound a run can reach gives the refusal path a real
case.

**One value stated twice is the defect shape this project keeps meeting.**[^8]
A reservation that the settings state and the arena also defaults would read
back correctly from either place, and the copy that lost would change nothing
that anybody could see.

## Decision

### D1. The world reserves every unit column, and the free queue, at construction

The world reserves the named number of entries in each unit column when it is
built. It reserves the same number in the queue of free slots, because every
opened slot can be free at once when a shortage ends the whole population on
one tick, and a queue that grew on that tick would reallocate inside the step.

The reservation is paid once. No later spawn and no later death pays it.

**A copy of a world keeps the reservation.** A derived copy of a column
allocates for what the column holds, not for what it reserved, so a copied
world would grow where the original does not and nothing would report it. The
copy is written rather than derived, and it reserves what the original
reserved.

**The reservation changes a capacity and never a length.** A reserved slot is
not an opened slot. It carries no generation and no identity, and the arena
mints nothing for it.[^9] The state hash covers the length of each column, the
free queue and the counts, and it covers no capacity, so a reservation moves
no hash and no golden file.[^10]

### D2. The settings of the world state the reservation, and nothing else states it

The reservation is a field of the world settings. The arena takes the value it
is given and holds no default of its own, so the arena has no reservation to
disagree with the settings.

The default of that field is the target population the project owner answered.
The reference table holds the value, and the settings cite it.[^11] The code
states no population of its own.

The reservation counts everybody, because the answered target counts
everybody. Soldiers are a fraction of it rather than a population on top of
the civilians.[^11]

### D3. A spawn past the reservation is refused, and the refusal is a value

A spawn that would open a slot past the reservation returns the typed refusal
the arena already carries. It does not panic, it does not drop the unit
silently, and it does not grow.

A founding is a spawn of a group. It wraps the refusal of the arena in a
variant of its own, so a caller of the founding sees which member of the group
was refused and why.

**A founding that a refusal stops leaves nothing behind.** The founding seats
a settlement before it seats the group, so a refusal part way through the
group would otherwise leave a settlement standing and a fraction of a group
alive. Every refusal after the settlement stands goes through one path that
undoes both.

### D4. The reservation is the bound. There is no fallback that grows

The engine holds one answer to the question, not two. A world that reaches its
reservation refuses. It does not reserve at construction and then grow past
the reservation when a run asks for more, because that arrangement holds both
behaviours and therefore holds two answers to one question.

## Consequences

**A run cannot exceed the number the settings named.** A caller who wants a
larger population states a larger reservation before the run begins. A caller
who discovers part way through a run that the world is too small must build
another world.

**A world holds memory it does not use.** The reservation is sized for the
target and a run begins small, so the difference between the two is held and
untouched for as long as the run takes to grow.

**Every caller that builds a world states the reservation.** The settings are
a plain structure with public fields, so a new field reaches every place that
builds one. The backlog holds an item about that price, and this record pays
it once rather than arguing with it.[^12]

**The refusal is now reachable, so callers must handle it.** A path that
treated a spawn as unable to fail was correct while the bound was the range of
the index. It is not correct now.

**The settlement arena and the character arena still grow.** Both hold the
same shape and neither is in the closed row that this record implements. An
open row states the question they raise, and the character arena raises a
second one, because its ceiling is larger than its target.[^13]

**A reader cannot tell from this record what the reservation costs.** One
blocker holds every cost figure this record would state, so the record gives
the shape of the cost and no figure.[^7]

## References

[^1]: ADR-0012, tiles are dense columns and units are a generational arena, decision D3. `docs/adrs/accepted/adr-0012-tiles-are-dense-columns-and-units-are-a-generational-arena.md`
[^2]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^3]: Findings register, FND-135. `docs/FINDINGS.md`
[^4]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
[^5]: Definition of Done, section 3. `.claude/rules/definition-of-done.md`
[^6]: Decisions register, DEC-059. `docs/DECISIONS.md`
[^7]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^8]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^9]: ADR-0014, entity identity is an index plus a generation, decision D6. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^10]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^11]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^12]: Backlog item 0080. `docs/backlog/proposed/0080-give-the-world-settings-a-constructor.md`
[^13]: Decisions register, DEC-062. `docs/DECISIONS.md`
