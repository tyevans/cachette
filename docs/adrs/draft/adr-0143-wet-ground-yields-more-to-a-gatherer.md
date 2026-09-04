# ADR-0143: Wet ground yields more to a gatherer

## Context

A product record asks that the condition the world holds influence a unit. A
unit that stands in it must behave differently from a unit that does not, and a
developer must be able to point at the difference.[^1]

**A capability that nothing invokes is the defect shape this project names
first.** A reference project shipped nine inert capabilities in one wave, and it
later wrote a record about a list of telemetry keys that nothing emitted.[^2]
Weather that nothing reads is exactly that shape, and it would pass every test
of its own.

The project has one recent instance of the opposite discipline. It counted how
many different luxuries stand on the ground, found that no pass should read the
number yet, and opened a blocker rather than inventing an effect.[^3] [^4] That
is the right answer when nobody has said what the score should change.

**Weather is different, because the product record already says what good looks
like.** It states that the condition must influence a unit as a checkable
statement. So an effect is required, and the question is which one.

Four passes could read weather. Movement decides what a step costs. Gathering
decides what a unit takes from a tile in one tick. Consumption decides what a
unit needs. The choice pass decides what a unit does next. Each is a separate
claim, and this record makes one of them.

## Decision

### D1. The gather resolve reads the weather of the cell, and wet ground yields
more

**A unit that gathers from a tile inside a wet cell takes more in one tick than
a unit on dry ground.** The resolve reads whether the level 1 cell that covers
the tile holds at least a stated quantity of water on its ground.[^5]

The read happens once for the whole run of units that gather one resource from
one tile, beside the deposit and beside the rate that a finished upgrade
gives.[^6] It adds no pass and no allocation to the step.

Gathering is chosen over the three alternatives for one reason each.
**Movement** is the pass this project has already tuned hardest, and its
kernel takes its cost from the terrain table rather than from a field.[^7]
**Consumption** would make weather kill units, and the product record says
plainly that weather does not damage a unit.[^8] **The choice pass** would make
weather change what a unit wants, which is a larger claim than a first wiring
should make.

Gathering is also where the effect is visible without a second subsystem. A
watcher reads the gather log and sees the amounts change.

### D2. The effect is a whole number added to the rate, never a factor

**Wet ground adds a fixed whole number of resource units to what one gather
takes.** It does not multiply the rate.

A multiplier would need a second scale beside the whole numbers that the
resource ledger counts in, and a truncating multiply would make the effect
vanish at a low rate and grow at a high one. The addition is exact at every
rate, and the conservation check on the resource account still balances.[^9]

The quantity that wet ground adds, and the quantity of water at which ground
counts as wet, are content constants that no measurement chose. A blocker holds
the question of what weather should be worth.[^10]

### D3. Weather changes nothing else, and the register says so

**No other pass reads the weather field.** Movement, consumption, the choice
pass and the contest are unchanged.

This is stated rather than left implicit, because an unstated intent becomes an
assumption. A decision register row holds the three passes that could read
weather and do not, with the recommendation for each.[^11]

## Consequences

A game cannot make weather a hindrance through this record. Wet ground helps a
gatherer, and nothing in the engine makes weather cost anything. A game that
wants a storm to slow a march needs the movement claim, and that claim needs its
own record.

The effect is discontinuous. Ground is wet or it is not, and one drop either
side of the mark changes the yield by the whole amount. A continuous effect
would need a divide inside the gather resolve, and the discontinuity is honest
about a rule nobody has priced.

A world that never rains gathers exactly as it did before this record. The
default is the dry rate, so the change adds nothing to a world with no water and
no god.

The gather resolve now depends on the level 1 rebuild of the previous frame. A
unit takes what the ground held when the last solve ended, not what it holds
after this frame's solve.[^12]

## References

[^1]: PRD-0004, the world has weather that a watcher can read, what good looks like. `docs/product/accepted/prd-0004-the-world-has-weather-that-a-watcher-can-read.md`
[^2]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
[^3]: Decisions register, DEC-200. `docs/DECISIONS.md`
[^4]: Blockers register, BLK-110. `docs/BLOCKERS.md`
[^5]: ADR-0140, weather is a field over the level 1 cell lattice, decision D1. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
[^6]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D3. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
[^7]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^8]: PRD-0004, the world has weather that a watcher can read, what this does not do. `docs/product/accepted/prd-0004-the-world-has-weather-that-a-watcher-can-read.md`
[^9]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D5. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
[^10]: Blockers register, BLK-130. `docs/BLOCKERS.md`
[^11]: Decisions register, DEC-237. `docs/DECISIONS.md`
[^12]: ADR-0140, weather is a field over the level 1 cell lattice, decision D3. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
