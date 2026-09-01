---
id: 0136
title: Provision a founded site from the ground it reaches
status: complete
created: 2026-09-01
implements: [ADR-0062 D1, ADR-0063 D2, ADR-0075 D5]
changes: []
creates: []
serves: [PRD-0012]
blocked-by: []
---

## Why

The demonstration founds four groups and every person is dead within one
hundred ticks. The founding gives a group a home site with an empty store, and
sets no rate, so the store holds nothing on any frame. Each unit draws nothing,
each deficit rises without a bound below it, and the whole map ends at once.

Nothing was broken. Nothing connected the founding to the economy. The
findings register holds the evidence and what the project believed.[^1]

## Impact review

**Governed by.**

- **ADR-0062 D1** puts a rate on a site and never on a unit. The rate this
  work sets belongs to the settlement.
- **ADR-0062 D2** requires a rate at or above zero, and forbids an upkeep
  expressed as a production below zero. The rate here is a production.
- **ADR-0063 D2** makes a unit draw from the store of its site. The founding
  already sets the home site, so the store this work fills is the one the unit
  draws from.
- **ADR-0075 D5** makes the chooser report the properties that made the place
  the choice. The food that report holds is the input to the rate.
- **ADR-0002 D1** forbids a floating point quantity. The rate is a fixed-point
  multiply through the arithmetic module.

**Changes.** No record changes. Every record above already states the
constraint this work obeys.

**Creates.** No record. A future contributor could set the rates from the
control plane instead, but the choice is cheap to reverse and the reasoning is
visible in the call: the founding sets the rate because it is the one call that
holds both the site and the survey. The three-condition test therefore does not
ask for a record.[^2] The rule governs no draw, no order and no parallel
result, so the determinism counter-test does not ask for one either.

**Blockers.** None.

**Registers.** The findings register gains one entry.[^1]

## What the work does

1. A founding sets the production rate of the site it seats, for the commodity
   that a unit eats. One unit of food that the place reaches feeds one person,
   at the ration the need rule holds.
2. The ration is read from the need rule and never repeated, so the amount a
   person eats has one declaration site.[^3]
3. A test founds a run, steps it past the span that killed it, and asserts that
   every person is fed.
4. A second test removes the rate the founding set and asserts that the same
   group ends. This is the proof that the first test reaches the case.[^4]

## Done when

- A founded run survives longer than the span that ended it.
- The rate of each site equals the ration times the food its survey reported.
- Taking the rate away ends the group inside that span.
- The golden state hash is recorded again from the merged source, and only the
  founding scenario changes.
- `just check` exits 0.

## Outcome

**Done.** A founding now sets the production rate of the site it seats, from
the food the survey measured. In the demonstration world the four sites reach
44, 37, 52 and 60 food for thirty people each, so each holds a surplus and each
group is fed on every frame of a two hundred frame run. The commit body holds
the figures and the command.

**The survey score now decides something.** The score weighs food, and until
this work that weight changed nothing after the place was chosen. A place that
reaches less food than its group needs now runs that group short.

**Three tests cover it, and one of them is the negated case.** The rate test
asserts the rule rather than a value. The survival test drives the founding
into the run, which no test did before. The starvation test removes the rate
and watches the population end, so the survival assertion is known to reach the
case rather than to measure the fixture.

**One golden file changed, and only the founding scenario in it.** The other
scenarios found nothing, so they hold no site and no rate.

**What this does not do.** A unit still gathers into its own carry, and no pass
moves a carry into a store. A site therefore produces without anybody working,
and the store grows without a bound. Bounding it is the growth work, which
waits on housing.[^5]

## References

[^1]: Findings register, FND-124. `docs/FINDINGS.md`
[^2]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^3]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^4]: Testing rules, section 2a. `.claude/rules/testing.md`
[^5]: Backlog item 0060, grow the population from the store and the housing. `docs/backlog/refined/0060-grow-the-population-from-the-store-and-the-housing.md`
