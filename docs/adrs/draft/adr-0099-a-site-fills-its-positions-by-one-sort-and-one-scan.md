# ADR-0099: A site fills its positions by one sort and one scan

## Context

A site holds a small row of ranked positions. Each position names a kind of
work and a rank, and it holds one unit or nobody. A position that holds nobody
is a state and not an absence.[^1]

A separate record states how many positions a site opens, and it opens them in
proportion to what the site lacks.[^1] That record says nothing about which
unit fills one. This record decides that.

**The two halves were built apart, and the second was never built.** The engine
opened positions and seated nobody in them. A world left to run held sites with
open positions and no worker in any of them, on every frame, and nothing
reported it.[^2]

The choice matters because the obvious method is the expensive one. A site with
`m` applicants and `n` positions has `m` times `n` pairs. A method that scores
each pair is quadratic in the size of a place, and it runs for every site on
every interval. A contributor could reasonably write that method, because it is
the one that a person would write on paper, and the product record rejects it
by name.[^3]

The cheap method is to order both sides and pair them in order. That method is
not merely cheaper. **When the value of putting a unit in a position is the
product of one number about the unit and one number about the position, sorting
both sides and pairing them in order is exactly optimal.** It is not an
approximation of the scoring method; it gives the same answer.[^4] That is the
reasoning a reader will not believe from the code, because the code is a scan
that never compares a unit against a position.

The result carries a duty. It holds when the value factorises into one term per
side. Content that makes the value depend on the pair in a way that does not
factorise loses the guarantee, and the scan then gives an answer that is
defensible but not optimal.

## Decision

### D1. A site pairs its applicants with its positions by one sort and one scan, and nothing scores a pair

The applicants of a site are the units that live there. The pass orders every
applicant of every site once, by the site first and the identity last. It then
walks that order and walks the positions of each site in row order, and it
seats the applicant it is holding in the position it is looking at.

**No part of the pass compares an applicant against a position.** There is no
score, no cost matrix, and no search over pairs. The pairing is a consequence of
the two orders.

The alternative, scoring each pair and taking the best assignment, is rejected.
It is quadratic in the size of a place, it runs on a schedule for every site,
and it gives the same answer as the scan whenever the value factorises. Where
the value does not factorise, the project has no content that asks for it.

The alternative of seating in the order the units were read is rejected. It is
not an order the engine controls: it is the order of the unit storage, and a
separate item may reorder that storage for reasons that have nothing to do with
who works where. An assignment that changed because the storage was repacked
would be a silent behaviour change.

### D2. A position is opened and filled in one pass, in that order

The resize that opens the positions and the scan that fills them run in one
pass, on one schedule, with the resize first. A seat cannot be taken before it
is opened, and a seat opened on one interval and filled on the next would leave
every new position empty for the length of an interval.

The release of a dead holder is not on that schedule. It runs on every frame,
because a unit dies on any frame and no stale identity may cross a barrier.[^5]
The three passes therefore run at two cadences, and this record states which.

### D3. Every applicant of a site is admissible for every position of that site

**No property of the unit limits which position it may take.** The unit row
carries no such property, and this record does not invent one.

The product record requires that a property of the unit limits it, and it does
not say which property.[^3] Choosing one decides the width of the first sort
key and adds a column to the storage of every unit in the world. That is a
second claim, and it is not this one.

**The scan is built so that the limit is a filter and not a rewrite.** A limit
becomes a field of the applicant key, ahead of the identity, and a matching
field on the position. The scan then advances past a position the applicant
cannot take. Nothing about the sort or the scan changes.

This decision is the weakest part of this record and it is stated so that a
reader is not misled. The pass today gives every position at a site to whichever
resident holds the lowest identity among those not yet seated. That is a
deterministic answer and it is not yet a meaningful one.

## Consequences

**A site now shows a consequence of being short.** A position that no applicant
reached stays open, and a reader can tell a site that is short of workers from
one that is not. That was not readable before, because every position was open.

**The project cannot score an assignment without superseding D1.** A later
contributor who wants a value that depends on the pair must write a record that
replaces this one, and must say what the content lost by not factorising.

**The assignment is stable under the storage layout.** D1 forbids the read
order, so a pass that reorders the unit arena cannot change who works where.

**The assignment is not yet meaningful, by D3.** Until a record adds the
limiting property, the pass answers "which units work here" and not "who does
which job". A reader who takes the kind of a filled position as a statement
about the unit in it would be wrong.

**The pass allocates.** It builds one key for each applicant on each interval.
The cost of that is a figure, no measurement of it exists on the target
platform, and the blocker holds every cost figure in this project.[^6]

## References

[^1]: ADR-0065, a group is a site membership, not a region, decisions D1 and D3. `docs/adrs/draft/adr-0065-a-group-is-a-site-membership-not-a-region.md`
[^2]: Findings register, FND-269. `docs/FINDINGS.md`
[^3]: PRD-0017, work is assigned to the people who can do it. `docs/product/accepted/prd-0017-work-is-assigned-to-the-people-who-can-do-it.md`
[^4]: Individual agency and occupations, the rearrangement result. `docs/research/reports/16-individual-agency-and-occupations.md`
[^5]: ADR-0065, a group is a site membership, not a region, decision D2. `docs/adrs/draft/adr-0065-a-group-is-a-site-membership-not-a-region.md`
[^6]: Blockers register, BLK-007. `docs/BLOCKERS.md`
