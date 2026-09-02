---
id: 0155
title: Build a world that produces a chosen distribution
status: proposed
created: 2026-09-01
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

Every item that tests behaviour builds a world first, and each one builds it by
hand.

Item 0059 needs a site at its capacity, a site above its capacity, a site with
capacity and no residents, and a unit that lives nowhere.[^1] Item 0084 needs an
unheld tile, a held tile, and a tile that changed holder on the tick under
test.[^2] Item 0101 needs a fixture that reaches hill and mountain.[^3] Each
item states its distribution in prose, and each one writes its own construction
code to reach it.

The testing rule forbids the cheap route. A fixture copied from the
demonstration binary models the typical case, so it supplies no extreme, and the
assertion never receives the input that would fail it. The rule tells the author
to ask what distribution the test needs and to build the world that produces
it.[^4]

That instruction is correct and it is repeated work. The project has recorded
the defect it prevents twice, in two subsystems, in one session.[^5]

## The shape of the idea

A caller states the distribution it needs. The engine builds a world that
produces it, or refuses. The caller does not place anything.

This keeps the control plane a control plane. Python states the requirement and
sends one command. Rust resolves the requirement and builds the world. Python
never loops over entities.[^6]

## What must be answered before this is refined

**Whether a stated distribution can be satisfied at all.** A requirement may be
unsatisfiable, and a builder that silently returns the nearest world it could
manage is worse than one that refuses. Decide what a refusal looks like and what
it reports.

**Whether the built world is reachable.** A builder that writes columns directly
can produce a world the engine could never reach by stepping. A test against such
a world proves something about an impossible world. Decide whether construction
goes through the same path the engine uses, and say what enforces it.

**Whether this earns a decision record.** Apply the three-condition test.[^7] The
reachability rule above looks like a genuine constraint, because a future
contributor could reasonably choose direct column writes and the cost of choosing
otherwise is paid much later. The rest of the builder may be a mechanism rather
than a constraint.

**Whether a need exists that a product record should hold.** The audience is a
contributor to this repository, not a game developer. Every product record today
serves a game developer. Decide whether this belongs in the product system at
all.

**What calls it first.** Do not build the builder before a test uses it. Name the
item that adopts it, and prefer adopting one existing fixture over writing a new
one.[^8]

## Impact review

Not done. The item stays in `proposed/` until it is.

## Done when

Filled in when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Backlog item 0059. `docs/backlog/proposed/0059-give-a-site-a-capacity-and-a-resident-roll.md`
[^2]: Backlog item 0084. `docs/backlog/refined/0084-give-a-tile-one-faction-column.md`
[^3]: Backlog item 0101. `docs/backlog/proposed/0101-assert-the-terrain-gradient-of-a-holding.md`
[^4]: Testing Rules, a fixture supplies the input. `.claude/rules/testing.md`
[^5]: Findings register, FND-051 and FND-048. `docs/FINDINGS.md`
[^6]: ADR-0040, Python is a control plane, not a data plane. `docs/adrs/REGISTRY.md`
[^7]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^8]: Recurring defect shapes, shape 3. `.claude/rules/recurring-defects.md`
