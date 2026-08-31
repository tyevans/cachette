---
id: 0040
title: Record where a change made outside a frame gets its barrier
status: proposed
created: 2026-08-31
---

The step now opens by rebuilding the derived structure when a spawn or a
despawn made between two frames left it stale. Admission reads the occupancy
of a target from that structure, so it must describe the arena before the
intents are admitted.

There are two call sites for one operation. The rebuild at the end of the step
is the barrier of that frame and stays last. The one at the top gives the
caller's own changes the barrier they never had.

The register holds the choice and the assumption.[^1] Promote it to a record
when a second structural apply lands inside the frame, because the ordering
between the two is a real decision then and a comment is not the mechanism
this project accepts for that class of fact.

**The barrier itself is settled.** The item that asked for it is complete, and
four tests read the ordering from outside: a rebuild that ran before the
structural apply leaves the derived structure stale when the step ends.[^2]
This item is about the other call site, the one at the top of the step, which
serves a change the caller made outside any frame. That one has no test that
could fail, because a caller who forgets to rebuild is served rather than
refused, which is the whole point of the assumption.

## References

[^1]: Decisions register, DEC-021. `docs/DECISIONS.md`
[^2]: Backlog item 0030. `docs/backlog/complete/0030-enforce-the-barrier-ordering.md`
