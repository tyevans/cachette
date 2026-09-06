---
id: 0111
title: Let the weather change a unit and show it in the window
status: complete
created: 2026-08-31
implements: []
changes: []
creates: []
serves: [PRD-0004, PRD-0005]
blocked-by: []
---

## Why

A condition that nothing reads is scenery. The product record states two
statements that close the need: a unit that stands in the condition behaves
differently from a unit that does not, and a watcher can see the condition
on the map and can tell it apart from the terrain beneath it.[^1]

Neither is met by the update alone. A capability that nothing invokes passes
its own test and ships inert, and this project has a rule for that
shape.[^2] The unit behaviour is what invokes the condition, and the window
is what makes the difference visible.

The record bounds the behaviour. Weather here does not damage a unit, and it
gives a unit no forecast. A unit meets the condition where it stands.

## What is missing before this is refined

- **The condition must exist first.** Items 0109 and 0110 choose it and
  advance it.[^3] [^4]
- **The behaviour is not chosen.** Which of a unit's choices the condition
  changes is an open question, and it depends on the condition that item
  0109 picks.
- **The drawing rule is not chosen.** The window already draws the ground
  and the holdings. A third layer must not hide the two beneath it, and the
  panel must not gain a pass over the world.[^5]

## Done when

Not yet stated. The item is not refined.

## Outcome

**Closed as already done. The work landed under other items.** An audit read the
code on 5 September 2026.

**Weather changes what a unit gets.** A wet cell raises the yield of a gather,
through one named bonus in the step.[^6]

**The window draws it, twice.** The overlay paints the ground water and the air
water as two spans, each with its own colour. A weather panel of the deck shows
the field as text.[^7] [^8]

## References

[^1]: Product record PRD-0004. `docs/product/accepted/prd-0004-the-world-has-weather-that-a-watcher-can-read.md`
[^2]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
[^3]: Backlog item 0109. `docs/backlog/complete/0109-decide-how-the-world-holds-a-condition-that-moves.md`
[^4]: Backlog item 0110. `docs/backlog/complete/0110-advance-a-weather-condition-each-tick.md`
[^5]: Product record PRD-0005. `docs/product/shipped/prd-0005-a-watcher-can-tell-what-is-happening-and-why.md`
[^6]: The wet cell branch and the wet gather bonus. `crates/cachette-core/src/world.rs`
[^7]: The moisture and air overlay spans. `crates/cachette-view/src/overlay.rs`
[^8]: The weather panel. `crates/cachette-view/src/panel/weather.rs`
