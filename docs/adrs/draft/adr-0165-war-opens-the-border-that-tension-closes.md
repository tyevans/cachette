# ADR-0165: War opens the border that tension closes

## Context

Each ordered pair of factions carries one signed integer. A band is a
threshold on that integer, and a pass compares the integer to an edge rather
than to a name. The bands the game names are alliance, peace, tension and
war.[^1]

Two passes read that integer for the same border, and today they read it in a
way that closes the border to the only faction that wants to cross it.

**The movement pass refuses a guest.** A holder refuses a unit of another
faction onto ground the holder holds, when the holder is below the guest edge
toward that faction. The guest edge is a balance value, and it stands at the
peace edge.[^2] [^3]

**The campaign pass raises only against a faction in the war band.** A
controller picks an objective inside the ground of a faction it is at war
with, and it sends a cohort at that objective.[^4]

**The two edges close on the same act.** A faction in the war band is below
the peace edge by definition, so it is below the guest edge. The ground of the
objective therefore refuses the cohort exactly when the cohort is raised. A
sweep of thirty-two seeds, each played to the tick limit of the balance
harness, raised campaigns at every faction and won none. The objective never
changed holder in any seed.[^5]

**No balance value opens this.** Moving the guest edge below the war edge
opens the border to every faction at every band, and the refusal then means
nothing. Moving the war edge above the guest edge makes war a band that no
campaign can be raised in. The edges cannot be ordered so that one band both
raises a campaign and admits its cohort.

Three forces fix the shape.

**A refusal is a border that a host keeps.** A host that refuses a guest is
exercising a right over its own ground. A host at war with a faction is not
exercising a right over that faction. It is fighting it.

**The engine must not name a band.** No pass takes a branch on a value whose
type is the band. A pass compares the integer to an edge.[^1]

**A tile's lease follows the units that stand on it.** A faction takes ground
by standing on it for enough ticks, and it must be able to reach that ground
first. A record already states that a faction erodes a border only where the
two factions are at peace, and that consequence follows from the rule this
record changes.[^6]

## Decision

**A holder refuses a guest when the relation is below the guest edge and at or
above the war edge. A holder in the war band toward a faction refuses that
faction nothing.**

### D1. The guest refusal is a window between two edges, and the war band lies below it

**This changes the movement clause of ADR-0146 D4.** That decision says that
movement refuses entry to ground another faction holds when the holder is
below a stated edge toward the guest. This record adds a floor to that test:
the refusal holds only while the relation is at or above the war edge.

The other three clauses of ADR-0146 D4 stand. The contest still fires only
when at least one of the pair is in the war band. Conversion still reads the
conversion edge. Trade still refuses an offer when either side is in the war
band.[^1]

Both edges stay balance values, and no pass names a band.[^3]

The engine holds one statement of the movement rule, and this test lives
inside it. A caller that reads the rule reads one function.

A reviewer finds a violation when a pass compares the relation to a literal,
when the refusal test reads one edge only, or when a second site states the
rule.

### D2. War admits a guest. It does not protect one

Admission is not safety. A cohort that crosses a hostile border stands on
ground the holder holds, beside the units of the holder, and the meeting
resolves at the tile.[^7] The holder's answer to an invasion is the contest
and not the border.

The order of the two passes is unchanged. Movement admits, the barrier
rebuilds the derived unit structure, and the meeting resolves after it.[^7]

A reviewer finds a violation when the movement pass takes a casualty, or when
a pass makes a guest immune because it crossed at war.

## The alternatives this rejects

**Exempt a campaign cohort from the refusal.** The admission pass would read
the campaign register, and the rule would then depend on why a unit is where
it is. Rejected because the refusal is a property of the pair of factions and
not of the errand. A faction that walks a unit onto hostile ground without
raising a campaign would still be refused, which makes the border a rule about
paperwork.

**Move the guest edge below the war edge.** Rejected because the refusal would
then never fire, and a peaceful faction would lose a border it never gave
away. The register would hold a row that changes nothing.

**Move the war edge above the guest edge.** Rejected because the bands would
then overlap in a way that no reader can hold: a faction would be at war and
above the edge that closes the border, and the tension band would be the only
band that fights.

**Give a campaign a right of passage that a treaty grants.** Rejected for now
because no record holds a treaty, and a decision that needs a mechanism nobody
has written is an intent rather than a fact.

## Consequences

**A border is open at war and closed in tension.** A watcher sees units cross
a border after a declaration and stop crossing it after a peace. That reads
backwards until the reader knows that the closed border is the diplomatic act.

**A faction erodes a border at war as well as at peace.** The record that
gives a tile a lease states that a faction erodes a border only where the two
factions are at peace, and that consequence followed from the rule this record
changes. It no longer holds.[^6]

**A faction cannot shut an enemy out by declaring war.** The only defence of
ground is the units that stand on it.

**The tension band gains the whole of the refusal.** Every closed border in
the engine is now a border between two factions in the tension band. A change
to the tension edges therefore moves every closed border at once.

**A campaign can reach its objective.** That is the point of this record. What
happens when it arrives is decided by the meeting and by the lease, and not
here.[^6] [^7]

## References

[^1]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decisions D2 and D4. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
[^2]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^3]: Balance register, the band below which a holder refuses a guest. `docs/reference/balance.md`
[^4]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D5. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^5]: Findings register, FND-542. `docs/FINDINGS.md`
[^6]: ADR-0153, a tile's lease follows the units that stand on it, decisions D2 and D5. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
[^7]: ADR-0121, a meeting between two factions resolves at the tile, decisions D1 and D2. `docs/adrs/draft/adr-0121-a-meeting-between-two-factions-resolves-at-the-tile.md`
