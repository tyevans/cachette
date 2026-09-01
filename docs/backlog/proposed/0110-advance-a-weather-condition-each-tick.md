---
id: 0110
title: Advance a weather condition each tick
status: proposed
created: 2026-08-31
implements: []
changes: []
creates: []
serves: [PRD-0004]
blocked-by: [BLK-007]
---

## Why

The world needs a condition that moves. The product record states what the
condition must do: it varies over the map and over time, a rule advances it
each tick, it conserves what it should conserve, it stays bounded, terrain
influences it, and the same seed gives the same result at every thread
count.[^1]

A table of values read by date does not satisfy the record. The rule is the
work.

This item builds the condition and its update. It also builds the tests that
prove the record's statements: an exchange between two places that loses
nothing and gains nothing, a bound that holds under a long run, a height or
a tile kind that changes what the condition does, and a thread-count
comparison.

## What is missing before this is refined

- **The decision record comes first.** Item 0109 chooses the condition and
  the shape that holds it.[^2] This item cannot be refined before that
  record exists.
- **The conservation test needs a fixture that reaches the extreme.** A
  fixture built from the demonstration world supplies no extreme, and the
  test then measures the fixture.[^3] The review must say what distribution
  the test needs.
- **No measurement exists on the target platform.**[^4] The item states a
  cost shape, not a figure.

## Done when

Not yet stated. The item is not refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Product record PRD-0004. `docs/product/accepted/prd-0004-the-world-has-weather-that-a-watcher-can-read.md`
[^2]: Backlog item 0109. `docs/backlog/proposed/0109-decide-how-the-world-holds-a-condition-that-moves.md`
[^3]: Testing Rules, section 2a. `.claude/rules/testing.md`
[^4]: Blockers register, BLK-007. `docs/BLOCKERS.md`
