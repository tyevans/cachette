---
id: 0024
title: Write the viewer record
status: proposed
created: 2026-08-30
---

Registry row 0067 holds the claim: the viewer reads a published frame and
never writes to the world.

The record must state what the viewer may do with floating point. ADR-0002
allows floating point in rendering, because rendering does not feed back into
the world. The viewer is where that permission is used, so the boundary needs
stating once, where a reviewer can find a violation.

Refine this at sprint 4 planning.
