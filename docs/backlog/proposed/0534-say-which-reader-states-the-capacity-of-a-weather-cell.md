---
id: 0534
title: Say which reader states the capacity of a weather cell
status: proposed
created: 2026-09-09
implements: []
changes: []
creates: []
serves: [PRD-0004]
blocked-by: []
---

## Why

**Two places state what the air over a weather cell can hold, and they answer
different numbers.** The published reader answers the capacity of air at rest
over the cell, which follows the temperature of that cell alone. The settle pass
bounds the air at a travelling capacity, which takes the same temperature and
then applies the cooling the air met and the slope it went up. The slope term is
signed, because air that descends warms and holds what it carries, so the
travelling capacity can stand above the published one.

**A test asserts the wrong one of the two, and it is red today.** It reads the
whole plane after twenty steps and asks that no cell stand above its own
published capacity. One cell stood 23 drops above 1526, with no storm over it. A
finding holds the measurement.[^1]

**The test is right to fail and it must not be relaxed.** It asserts a bound
that nothing promises. A test changed to match a picture that fails its own rule
states something false with the authority of a test.

**Nothing is visibly wrong in the picture.** The reader that turns the pair into
a share of a sky clamps its answer, so a watcher never sees more than a whole
sky. The clamp is why this stood unread.

**This is one value with two declaration sites.** Nothing fails when the two
disagree, which is the shape this project keeps meeting.[^2]

## Impact review

**This item is proposed, and refining it is the work.** The review below states
what a refinement must answer, and it does not answer them.

**Governed by.** ADR-0141 holds that a weather pass moves water and never scales
it, so any repair must keep the water account exact. ADR-0162 holds where water
enters the air and where it falls, and its second decision is the one that makes
the slope term signed. A refinement must say whether the answer changes that
record or works inside it.

**Creates.** A record, if the answer is that one reader states the capacity and
the other is derived from it. Which reader is authoritative is a choice a
contributor could reasonably make otherwise, and the published reader reaches
the drawing and the boundary.

**Blockers.** None. Every figure the work needs is a property of the model
rather than a cost.

**Serves.** PRD-0004.

## What the work does

Decide which figure the phrase "the capacity of a cell" names, and make one
reader state it. Say what the other becomes: a second published reader with its
own name, or a value the first derives.

Then repair the test against the answer, and repair the doc comment of the
published reader, which says today that air which cooled or climbed holds less
than the figure it returns and does not mention descent.

## What good looks like

One reader states the bound the settle pass applies. A test asserts that bound
over the whole plane and passes. The doc comment of each reader says which of
the two figures it answers. The water account still balances and the state
hashes do not move, because the work changes what is published and not what the
solve computes.

## What it does not do

It does not change the settle pass, unless the answer is that the settle pass is
wrong. The measurement says it is behaving as its own record describes.

It does not change the cloud share reader. That reader clamps its answer, and
the clamp is correct whichever figure it divides by.

## References

[^1]: Findings register, FND-730. `docs/FINDINGS.md`
[^2]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
