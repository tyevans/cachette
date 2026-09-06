---
id: 0512
title: Add the temperature term to the production pipeline
status: proposed
created: 2026-09-06
implements: []
changes: []
creates: []
serves: [PRD-0004]
blocked-by: [0511]
---

## Why

The production pipeline of a site composes four terms: the ground, the
moisture, the upgrades and the people.[^1] The project owner asked that
temperature vary a site's output in the same way moisture does.

Temperature was left out for one reason. The weather worker who owns the field
states that it is not built and that its range is not measured, and asks that
no weight be written against a number nobody has taken. A weight written now
would be an invented value.

## What the work does

1. Read the temperature of the cell that covers the site, through the reader
   the weather module publishes, and add a fifth term to the pipeline.
2. Choose the weight from the measured range of the field, not from the
   design intent, and put the derivation in the balance register.
3. Test the term at both extremes of the measured range.

## References

[^1]: Backlog item 0511, make a site production rate follow the world each application. `docs/backlog/refined/0511-make-a-site-production-rate-follow-the-world-each-application.md`
