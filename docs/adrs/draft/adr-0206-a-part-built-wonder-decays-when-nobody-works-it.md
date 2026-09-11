# ADR-0206: A part-built wonder decays when nobody works it, and resets when its ground changes holder

## Context

A wonder is an upgrade row that carries a victory claim above zero. A finished
wonder that stands on ground a faction holds ends the game for that
faction.[^1] This record calls the progress of an entry toward such a row
wonder work.

A trained policy found a way to win that no rival could answer. It put every
unit on one wonder from the first tick, and it did nothing else. The findings
register holds how often that won.[^2]

Four rules made the work safe after it started:

- A rival that killed the builders stopped the work, and the work stayed.
- A capture gave the part-built wonder to the taker, because an upgrade
  changes hands with the ground.[^3]
- A raze removed only the upgrade on the tile of the site.[^4]
- The wear pass reads only a standing level, so work that was not finished
  never wore.

A product record asks that a player who chooses one way to win can lose to a
player who chooses another.[^5] A wonder that nothing can undo is a way to win
that no rival contests. The project owner ruled on 10 September 2026 that wonder
work becomes fragile.

## Decision

### D1. Wonder work that no builder adds to on a tick loses a stated amount on that tick

On each tick, the build pass states which entries it added work to. Wonder work
that the build pass did not add to loses the wonder decay. The work stops at
nothing. An entry at no level that holds no work is removed, and the tile
returns to the world the generator made.[^6]

A builder attends the work when the build pass adds its contribution. One
builder is enough. For every row that asks for held ground, the build pass
admits only a builder whose faction holds the ground, so a builder that attends
the work is a builder of the holder.[^7]

No grace period applies, and the decay is linear. Each tick that nobody attends
takes the same amount.

The decay is a balance value, and this record states no value. The upgrade
table holds it, so the state hash covers it and a caller sets it at run
time.[^8]

A reviewer finds a violation when wonder work keeps its value through a tick
that nobody attended while the decay is above zero, when attended work loses
work on the tick it was attended, or when wonder work falls below nothing.

### D2. Wonder work returns to nothing when the ground under it changes holder

When the holder of the tile under wonder work changes during a step, the work
returns to nothing on that step. An entry at no level is then removed. An entry
at a standing level keeps its level.

**The rule stores no owner.** An upgrade is stored against a tile, and no owner
is stored on it or beside it.[^4] The holder of the tile is the only answer to
whose the work is. The rule therefore has no stored holder to compare with the
holder column.

**The step watches the ground instead.** At a fixed point in each step, the
step lists the tiles under wonder work. Each write that changes the holder of a
listed tile marks that tile. The pass resets the work on a marked tile, and the
list then ends. The list is working memory for one step. It is not world state,
and the state hash does not cover it, because no later step reads it.[^8]

**A change and a change back both mark the tile.** The spread can give the
ground to nobody, and a land transfer later in the same step can give it back
to the same faction. The two ends of that step name one holder, and the work
still resets.

The rule reads the holder column and never the act. A capture, a raze, the
release of a faction that leaves the game, a land transfer and a lease all
change the holder through one write, and one rule covers all of them.

A reviewer finds a violation when the new holder of a tile can add to, or
finish, work that an earlier holder put in. A reviewer also finds a violation
when an upgrade stores a faction on it or beside it, or when the watch enters
the state hash.

### D3. The rule reads the victory claim column, and it touches no standing level

Wonder work is the progress of an entry toward the row above it, when that row
carries a victory claim above zero. One predicate on the entry states it, and
the predicate names no category.[^9]

This rule never touches a standing level. A finished wonder therefore keeps the
rule of the record that makes it a win path.[^1] Work toward a row with no claim
keeps the rules it had: it keeps its progress while nobody works it, and it
changes hands with the ground.[^3]

A reviewer finds a violation when the rule branches on a category, when it
lowers or removes a standing level, or when it changes work toward a row with
no claim.

### D4. The pass runs after every stage that writes the holder column, and before anything reads the work

The pass runs once for each tick. It runs after the build, the capture, the
spread, the elimination and the land transfer. It runs before the observation
and the game end reader. A change of holder therefore resets the work on the
tick it happens, and the next build never adds to the work of an earlier
holder.

**The watch of D2 starts after the build and the wear, and before the first
stage that writes the holder column.** Work that the build starts on a tick is
therefore watched on that tick. A capture on the first tick of a build resets
that work. No stage writes the holder column between this pass and the start of
the next watch, and no verb writes it between two steps. A write there would be
seen by no rule.

A wonder that finishes on a tick does not decay on that tick. The build runs
first. A finished entry at the top of its category is not wonder work, and the
build attended every entry it advanced.

The walk is serial and in ascending tile order. It makes no random draw. Every
term is a whole number, and the arithmetic goes through the arithmetic
module.[^10] [^11] The watch is ordered by tile, and the one write of the holder
column runs on one thread, so the marks do not depend on the thread count.

A reviewer finds a violation when a stage that writes the holder column runs
before the watch starts or after this pass, when the observation or the game
end reader runs before the pass, or when its result depends on the thread
count.

## The alternatives this rejects

**Keep wonder work durable, and raise the wonder work instead.** Rejected. More
work makes the tactic slower and leaves it safe. No value of the work lets a
rival undo a wonder that it can reach.

**Give the work to the taker.** This was the rule. Rejected, because the taker
then finishes a wonder with the work of the faction it beat.

**Keep the work for the faction that built it, and give it back on a
recapture.** Rejected. It needs a second owner beside the holder, and a rule for
the time that owner leaves the game. The ruling asked for fragile work, not for
a claim that outlives the ground.

**Take a share of the work on each tick instead of a fixed amount.** Rejected. A
share of a whole number never reaches nothing without a floor rule. It also
takes the most from a wonder near its end, so a short raid costs a nearly
finished wonder more than many ticks of building give.

**Wait for a grace period before the decay starts.** Rejected. It needs a stored
tick for each entry. A linear decay already makes a short gap cheap, because one
tick takes one tick of decay.

**Keep an entry at no level with no work.** Rejected. An entry that holds
nothing stands for nothing, and it refuses every other category on its tile.

**Apply the rule to every category.** Rejected. The need is the wonder
path.[^5] A road or a terrace that disappeared half built would change the whole
economy with no stated need. The resolved blocker keeps every other upgrade with
the ground.[^3]

**Reset the work inside the capture and the raze.** Rejected. Those are two of
the paths that change the holder. A rule that one writer of the column states is
a rule that the other writers miss.[^12]

**Store on each entry the holder that its work belongs to, and compare it with
the column.** Rejected. This was the first form of this rule. It put an owner on
every upgrade, and the record that makes an upgrade change hands with the ground
forbids that.[^4] A stored copy of the holder is also a second declaration of
the holder column.[^12] It compared two ends too, so it missed a change and a
change back inside one step.

**List each tile and its holder at the start of the step, and compare the list
at the pass.** Rejected. It compares two ends, so it misses a change and a
change back inside one step. A list taken before the build also does not know
the work that the build starts on that step. A capture on the first tick of a
build would then give that work to the taker.

## Consequences

**A rival can now undo a wonder.** It kills or drives off every builder and
holds the tile, and the work falls. It takes the ground, and the work is gone.

**A faction that builds a wonder must hold its ground and keep a builder on
it.** One builder stops the decay, so the rule does not punish a faction that
keeps building.

**One clause of ADR-0150 D4 no longer holds for wonder work.** That clause keeps
the progress of a build whose ground changed hands. It still holds for every row
that carries no victory claim.[^7]

**A new stage that writes the holder column must run after the watch starts and
before this pass.** A new verb must not write the column between two steps. A
write outside that window is seen by nothing, and it would let the next build
add to the work of an earlier holder.

**No upgrade carries a faction.** The rule costs a watch over the tiles under
wonder work, and those tiles are sparse. It costs no field on any upgrade.

**This record does not change what a finished wonder does.** A wonder that
finishes on the tick its ground changes hands stands for the new holder when
the game end reader runs. The record that makes a wonder a win path states that
reader.[^1]

**The reading of the wonder path can now fall.** A reading that rises and falls
with the work still reaches its top value only on the tick its reader
fires.[^13]

## References

[^1]: ADR-0174, a wonder is a win path and a stock total is not, decision D1. `docs/adrs/draft/adr-0174-a-wonder-is-a-win-path-and-a-stock-total-is-not.md`
[^2]: Findings register, FND-765. `docs/FINDINGS.md`
[^3]: Blockers register, BLK-036. `docs/BLOCKERS.md`
[^4]: ADR-0180, a site changes hands or the taker destroys it, decision D2. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
[^5]: PRD-0057, more than one way to win decides a game. `docs/product/shaped/prd-0057-more-than-one-way-to-win-decides-a-game.md`
[^6]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D4. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
[^7]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D4. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^8]: ADR-0164, every stored value the step reads enters the state hash, decision D1. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
[^9]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
[^10]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^11]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^12]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
[^13]: ADR-0204, every win path holds a bar of its own, decision D4. `docs/adrs/draft/adr-0204-every-win-path-holds-a-bar-of-its-own.md`
