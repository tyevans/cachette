# ADR-0072: A tile stock is generated, and only what was taken is stored

## Context

A tile holds an amount of a resource. A unit standing on the tile takes from
it. The engine must answer two questions about any tile: what the tile started
with, and what remains.

The target scale is 16.7 million tiles. A dense array of one amount for each
tile and each kind is a real cost, and it is a cost the world pays for every
tile whether anybody ever visits it. The ground of this world already avoids
that cost. Terrain is a pure function of the seed and the tile address, and the
engine stores no terrain map.[^1]

What a unit has taken is different. It is a fact that a unit created, and no
function of the seed can produce it. The engine must store it.

The two halves therefore have opposite shapes. One is large, static and
derivable. The other is small, dynamic and irreducible.

An amount must be exact. An aggregate over tiles combines in any order, so the
addition must be exactly associative, and floating point addition is not.[^2]

Every draw must be keyed on the tuple of system, frame, entity and draw index.
A stateful generator makes the world depend on the order that the threads
visited it in.[^3]

A resource that a unit wants is what makes one place better than another. A
field spread evenly over the world gives a map texture and no geography, so the
generator must take the ground into account.

## Decision

### D1. The stock a tile started with is a pure function of the seed and the address

The engine computes the original stock of a tile from the world seed, the tile
address and the ground of the tile. It stores no map of stocks. Two callers
that visit the world in different orders, on different thread counts, read the
same world.

The draw key holds the tile address in its entity slot and the resource kind in
its draw index slot. Both components of the address reach the key. A key that
dropped one would give a field that varies along one axis and is constant along
the other, and both determinism tests would still pass, because the defect
repeats.[^4]

The frame slot holds a constant, because the stock a tile started with does not
change with time. The slot stays in the key, because the key shape is fixed by
the record that defines it.[^3]

### D2. The ground sets what a tile can hold

A table gives, for each ground and each kind, the largest stock that a tile of
that ground holds and how often such a tile carries a deposit at all. Open
water carries nothing of any kind. The tables are content, and they sit beside
the ground table until a content pipeline exists.

A deposit that exists holds at least one. A deposit of nothing is the same as
no deposit, and two ways to state one thing is the defect shape this project
keeps meeting.[^5]

### D3. The catalogue is a small table of kinds, not a set of verbs

A resource kind is an index into the tables of D2. It is not a type, not a
trait and not a verb. Adding a kind adds a row and no code.

The catalogue starts at three kinds: what a unit eats, what grows on a wooded
tile, and what high ground carries. Three is the smallest catalogue that shows
the shape the tables need. One kind would let a later reader believe the field
is a scalar. A larger catalogue would state a content decision that no need has
been recorded for.

### D4. The engine stores what was taken, and nothing else

A sparse ledger holds one entry for each tile and kind that somebody gathered
from. A world in which nothing was gathered holds no entry at all. The memory
cost therefore follows the gathering and not the size of the world.

The remaining stock of a tile is the original less what the ledger holds. The
engine never writes a remaining stock anywhere, so no second copy of the amount
can disagree with the first.[^5]

The ledger holds its entries sorted by a key that packs the tile and the kind,
and it merges new amounts in ascending runs. The order of the entries therefore
depends on the key values alone and never on how the entries were
gathered.[^6]

### D5. Conservation is a world invariant, checked for each kind

What has left the tiles equals what the live units carry, plus what left the
world in the hands of a unit that died. The equality holds for each kind on its
own, because a gather never turns one kind into another.

The world invariant check holds this rule. A rule that leaked the same amount
on every run would repeat perfectly, so neither determinism test can see it.
Only a check of the invariant can.[^7]

A unit that dies takes its load out of the world. The world adds that load to a
register of what departed rather than letting it disappear, so the equality
stays exact and the loss stays visible.

## Consequences

**A world costs nothing for the resources nobody touched.** The field is the
seed and the extent. The ledger is the gathering.

**A change to the generator moves every world.** The state hash therefore folds
the generated stock of every tile, and not the seed alone. The seed is the
input of the generator; only the tiles report a change to the generator itself.

**The project cannot place a deposit by hand.** Nothing writes an original
stock, so a scenario that needs a deposit at a stated tile must find one that
the seed produced. A later need for authored content is a new decision and
needs a record.

**A resource cannot regrow under this record.** Regrowth would move the
original stock, which is generated and therefore fixed. A regrowth rule must
either subtract from the ledger or add a second stored term, and either is a
new decision.

**The ledger grows and never shrinks.** An entry stays after its deposit is
empty, because the entry is what says the deposit is empty. A world whose units
gather everywhere will store an entry for every deposit. That is the same bound
as a dense array over the deposits, and it is reached only by a world that
gathered everywhere.

## Alternatives rejected

**Store a dense array of remaining stock.** This is the obvious shape and it is
what the ground already refused. It pays the whole world for the tiles nobody
touches, and it holds the same fact twice: the remaining amount and the
original amount that produced it.

**Store the remaining stock sparsely, rather than what was taken.** This costs
the same and reads slightly faster. It is rejected because a sparse remaining
stock cannot be told apart from a tile nobody has touched: an absent entry
would have to mean "full", so a deposit emptied to zero would read as full
unless something wrote a zero entry. Storing what was taken makes the absent
entry mean "nothing has happened here", which is true.

**Give every tile the same stock.** This removes the tables of D2 and makes the
field trivial. It is rejected because a map without good places has no
geography, and the need this work answers is exactly that a unit has no reason
to prefer one tile to another.

**Make an amount a fixed-point value.** The project has one fixed-point scale,
and a stock could use it. It is rejected because a fraction of a unit of stone
is not a thing the world holds, and because a whole number states the same
thing with no scale to get wrong.

## References

[^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
[^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^3]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^4]: Testing Rules, a determinism test cannot tell correct from consistently wrong. `.claude/rules/testing.md`
[^5]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^6]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^7]: Findings register, FND-048. `docs/FINDINGS.md`
