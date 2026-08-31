---
id: 0070
title: Settle whether the viewer may make the engine wait
status: proposed
created: 2026-08-31
serves: [PRD-0002]
---

The product record for the first renderable example states that the window
never makes the engine wait, that it drops what it cannot draw, that it
reports the drop, and that the engine costs the same when a viewer is
attached.

The viewer record decides the opposite for now, and says so in its own
consequences. One loop steps and then draws, so the two rates are one number.
The demonstration binary also caps its frame rate, so the engine waits on
every frame that finishes early. Nothing drops a frame and nothing reports a
drop.

The register holds the choice with its options and a recommendation.[^1] This
item is the work that closes it. The likely shape is an amendment to the
product record rather than a change to the engine, because separating the two
rates needs a record that does not exist.

The product record cannot reach `Shipped` while this is open.

Refine this when the register row is decided.

## References

[^1]: Decisions register, DEC-022. `docs/DECISIONS.md`
