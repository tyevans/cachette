# ADR-0078: Descent is a bounded record, and a relation is a bounded recursion

## Context

A character is a named individual in the character tier. The tier holds a
bounded population, so a caller may walk it.[^1] A character stands in no
relation to any other character today. The population is a set of strangers,
and it is the same set after every member of it has been replaced.[^2]

Two questions have to be answered together, because one answer constrains the
other.

**Where does a parent edge live?** Every entity lives in a generational
arena, and its identity pairs a slot index with a generation. The arena frees
a slot when the character in it is gone, and it advances the generation at
that moment, so the character created next in that slot never answers to the
old identity.[^3] [^4] A structure that is keyed on the slot therefore names a
different character after the reuse. A parent edge cannot live there, because
a watcher must read a parent after that parent has died.[^2]

**How is the relation between two characters computed?** The obvious
implementation walks two lines upward until they meet. The accepted product
record rejects it by name, because a pair is asked for often and each ask pays
the depth of both lines.[^2]

### The forces

**A dead character keeps its descent and drops its social ties.** The project
answered that question against the storage cost of the ties.[^5] The two are
different structures and only the ties go. Descent must therefore outlive the
character, and the ties must not.

**A relation must be exact.** No floating point number reaches simulated or
aggregated state, because float addition is not associative.[^6] A comparison
between two relations must give one answer whatever order the work ran in.[^2]

**The kinship recursion is exactly representable, and this is not a
concession.** The research gives Karigl's recursion for the kinship
coefficient. Every step of it halves a value, so every value is an integer
over a power of two. The Q16.16 scale holds sixteen fractional bits, so a
truncation depth below sixteen makes every intermediate value exact. The
integer form is the correct form here and the float form is the lossy
one.[^7]

**A closure table cannot be maintained.** A genealogy is a directed acyclic
graph with in-degree two, so a character has up to two to the power of the
generation distinct ancestors. The research rejects a table of every
ancestor-descendant pair on that arithmetic.[^7]

**A character raised from the ranks receives no invented ancestry.** The
project answered that a promoted soldier founds a new line and stands at zero
to everybody.[^8] A line that starts at zero must be a representable state and
must not be a special case in the recursion.

**A walk carries an order, and an order is a determinism property.** Iteration
order is explicit and stable in this project. A label or a result that depends
on the visit order of a graph walk is a determinism hole that a single-threaded
test cannot see.[^9]

## Decision

### D1. Descent is keyed on an identity that the record never reissues

The record of descent holds one row for each character the world creates. It
is append-only. It never removes a row and it never reuses a row, so a descent
identity names one character for the life of the world.

A parent edge names a descent identity. It never names a slot index and it
never names an entity identity.

Each row keeps the entity identity that the arena minted for that character.
A caller resolves that identity against the arena to learn whether the
character is alive. A character who is gone resolves to nothing, because the
generation in the identity no longer matches the generation in the slot.[^4]

**A death releases the slot columns and never the record of descent.** The
arena frees the slot, and the character created next in that slot overwrites
every column of it. A fact about a character that a death must release
therefore belongs in the slot columns, and a fact that must survive belongs in
the record of descent. That is the storage form of the answer about social
ties, and it needs no further mechanism when those ties exist.[^5]

**The record is bounded.** It holds a stated number of rows and refuses a
creation past it. The number is a derived figure and the reference table holds
it with its derivation.[^10]

### D2. A relation is a bounded recursion over the parent edges, never a walk to a common ancestor

The world computes the relation between two characters by the kinship
recursion. The recursion expands the younger of the two characters, and the
record allocates a row in birth order, so the younger character always carries
the larger descent identity and the recursion always ends.

The recursion stops at a stated depth. The reference table holds that depth and
the derivation that makes every value exact at it.[^10]

**The value is exact.** Every step halves a value, and the depth leaves a
fractional bit spare, so no step rounds. The result is a Q16.16 fixed-point
number and the arithmetic goes through the arithmetic module.[^6]

**Two characters with no ancestor in common stand at zero.** A character who
founds a line holds two absent parents, the recursion reads them as zero, and
nothing invents a parent. The founder is not a special case in the
recursion.[^8]

**The world reports Wright's coefficient of relationship**, which is twice the
kinship coefficient. A parent and a child give one half. That is the number
that reads as a relation, and the doubling is exact.[^7]

**The memo is an ordered map, read by key.** The recursion never iterates it,
so no iteration order reaches a result.[^9]

### D3. Every walk over the record returns the set in ascending descent identity order

A walk to the ancestors of a character and a walk to its descendants both sort
each frontier before they expand it, and both return the set sorted. The
result is therefore the birth order of the world, and it holds no visit
order.[^9]

A child list is appended at its tail, and a row is allocated in birth order, so
a child list is already ascending and a walk over it needs no sort.

### D4. A parent must be an older row, and that alone refuses a cycle

The record allocates a row after both parents, so a parent always carries the
smaller descent identity. The record refuses a parent identity that it has not
issued. A character therefore cannot be its own ancestor, and the graph holds
no cycle.

This is a structural refusal and not a search. The record runs no cycle check
when it adds a row.

## Consequences

**The project cannot ask who a dead person knew.** The ties of a dead
character are released with the slot. This record states where that release
happens and does not add a reader for the released state.[^5]

**A parent must be alive when a child is born.** A birth reads the descent
identity of each parent through the arena, and a character who is gone
resolves to nothing. A caller that wants a posthumous birth needs a path that
this record does not give.

**The relation is exact only to the stated depth.** Two characters whose only
common ancestor is further back than that depth stand at zero. The value is
exact at every depth the recursion reaches, and it is truncated beyond it.

**The record of descent grows and never shrinks within one run.** A line that
ends is reported as ended, and its rows stay. A world that reaches the record
ceiling refuses a creation. This record states the bound and does not release
a row.

**A watcher walks the record and never a hash map.** The parent edges, the
child lists and the walk frontiers are all dense arrays and sorted vectors.
Nothing in the record carries an iteration order that no key fixes.[^9]

**A reader cannot tell from this record what a walk costs.** No run has priced
a walk, and one blocker holds every cost figure this record would state.[^11]

## References

[^1]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D1. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
[^2]: PRD-0015, a unit has parents and children. `docs/product/accepted/prd-0015-a-unit-has-parents-and-children.md`
[^3]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^4]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^5]: Decisions register, DEC-003. `docs/DECISIONS.md`
[^6]: ADR-0002, simulated and aggregated state holds no floating point number, decisions D1 and D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^7]: The character graph and inheritance, sections 3.1 and 3.6. `docs/research/reports/14-character-graph-and-inheritance.md`
[^8]: Blockers register, BLK-011. `docs/BLOCKERS.md`
[^9]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^10]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^11]: Blockers register, BLK-007. `docs/BLOCKERS.md`
