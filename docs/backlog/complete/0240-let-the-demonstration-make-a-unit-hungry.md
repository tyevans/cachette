---
id: 0240
title: Let the demonstration make a unit hungry
status: complete
created: 2026-09-02
implements: []
changes: []
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

**The same work is filed as item 0216 on another branch.** That branch is not
merged. This row exists so that the duplicate is visible and labelled rather
than silent, and the dispatcher reconciles the two at merge. This one carries
the measurements.

**No unit of the demonstration was ever hungry.** Over twelve hundred ticks
with four groups of thirty, every unit was fed at every sampled tick, none was
ever short or starved, the ration never failed, and not one tile of two hundred
and eighty-one thousand six hundred was ever gathered from. The forage option
scores against the deficit of a unit, so it scored zero on every unit on every
frame whatever the ground held. The food layer of the window decided
nothing.[^1]

**The cause is the size of the group, and it acts through a relation that is
exact.** The founding sets the production rate of a site to a sixteenth of the
food its survey reached. The default need rule gives a person a ration of a
sixteenth of a full need for each application. The two cancel, so **a site
feeds exactly as many people as the food its survey measured.** The four
founded sites reach 44, 37, 52 and 60, and a group of thirty sits under all
four.

**Two candidate causes were eliminated before anything changed.** The starting
store is not it: at the first tick every store is zero and every unit is still
fed, so an empty store does not make a unit hungry. The declared upkeep rate is
not it either: it is zero at every group size, because the founding never sets
one, and a cohort draws its own ration rather than the declared upkeep.[^1]

## Done when

- The group size falls inside the spread of what the founded sites reach, so
  some ground cannot carry its group and other ground can. A watcher sees both
  conditions at once and the choice a unit makes varies across the map.
- The size is justified by that spread and not by a rendered picture. **The
  split follows the ground; the number only has to fall inside it.**
- The founding report states for each site whether its ground carries its
  group, and the run says so when every seated group came out the same way. A
  fixture that produces one condition everywhere measures itself, and the
  defect this item repairs went unnoticed because nothing said so.[^2]
- A run shows food falling where a crowd stands, and the level 1 summary
  falling with it.

## What this item does not do

**It does not make a unit hungry by moving it.** Feeding has no distance term,
so a unit draws from its home site wherever it stands and nothing it does can
make it hungry. That is an engine fact with consequences past this item, and a
finding records it.[^3] A demonstration cannot repair it and this item does not
try.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-232. `docs/FINDINGS.md`
[^2]: Testing rules, section 2a. `.claude/rules/testing.md`
[^3]: Findings register, FND-231. `docs/FINDINGS.md`
