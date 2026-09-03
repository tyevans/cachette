---
id: 0274
title: Show the load a unit carries and the home it keeps
status: complete
created: 2026-09-02
implements: [ADR-0070]
changes: []
creates: []
serves: [PRD-0005]
blocked-by: []
---

## Why

**Three things the engine already does were invisible to a watcher.** An audit
of a run of the demonstration found them, and none of the three needed new
behaviour to become visible. They needed a reader.

- Every live unit holds a home site, and the reader that answers which one had
  no caller outside the core crate.
- A share of the units haul a load. Nothing drew it and no panel counted it.
- The store of a site rations when it cannot serve what its cohorts asked for.
  It happens more than once in a tick, and the only way to see it was to read
  the log.

**A watcher must be able to tell what is happening and why.** A quantity the
engine produces and no surface reports is a quantity the product record asks
for and does not get.

## Done when

- The window and the panel both report what the drawn units carry, and how
  many of them hold a home site.
- The panel reports how many sites rationed in the last step and by how much.
- Every count of the units is a count of the window. The drawing reads at the
  units it painted and starts no pass over the arena, and a test reads the
  count of reads rather than the totals, because a layer that swept the arena
  would report the same totals.
- No surface reports a subsystem that produces nothing.

## How it came out

**The reads sit on the loop that already paints.** The drawing visits every
unit it paints in order to draw it, and it now asks that unit for its load and
its home on the same pass. The count of reads equals the count of units
painted, which is the assertion that separates this from a sweep. A test reads
that equality at two world sizes with one camera.

**The card and the section appear only when something is being carried.** A
surface that reported a permanent zero would take space from the map on every
frame of every run to say nothing, and it would still say nothing if the
subsystem behind it broke. The count decides, so the surface cannot outlive
the behaviour it reports.

**The fixture had to be built for the case, and the first one was not.** A
group the ground carries is never short, so it never forages, never gathers
and never carries. The first fixture seated groups of 48 on ground that fed
them, so nothing was ever hauled and every assertion about a load passed
against two zeros. A test that asserts the fixture's own outcome caught it,
and it is the reason that test exists.[^1] The group is now larger than the
ground carries, which is the same relation an earlier item established for the
demonstration itself.[^2]

**The shortfall is in accumulator units and was nearly reported raw.** An
accumulator is fixed point at a scale of 65536, so the first render stated a
shortfall about sixty-five thousand times the real one. It is formatted at
every surface now, and the reader that hands it to the control plane names the
unit in its key, because a caller cannot see the scale from the number.

## References

[^1]: Testing rules, section 2a. `.claude/rules/testing.md`
[^2]: Backlog item 0240, let the demonstration make a unit hungry. `docs/backlog/complete/0240-let-the-demonstration-make-a-unit-hungry.md`
