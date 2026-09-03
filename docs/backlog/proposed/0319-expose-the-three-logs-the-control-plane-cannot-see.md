---
id: 0319
title: Expose the three logs the control plane cannot see
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

**The control plane cannot see a unit die.** The engine keeps six logs of the
step that just ran. The bindings expose three of them: the tile changes, the
gathers and the sites that rationed. They expose none of the other three.

A unit that a shortage ended, a unit that a step promoted, and a site that
could not pay its upkeep are each recorded in the engine and each invisible to
Python. A caller can read one aggregate count of promotions from the drawing
report, and nothing else.

**This was found while building a panel, and the panel does not need it.** The
event feed reads the logs in Rust, where they are all available. The gap is on
the Python side alone, and nothing failed, because nothing asked.

**The three that are exposed set the shape.** Each of them crosses as a
dictionary of columns, one column for each field of the event, keyed by the
field name in the Rust source. A reader takes a field by name and holds no byte
offset.

## Done when

- A caller reads the units a shortage ended, the units a step promoted, and the
  sites that fell short, each as one dictionary of columns.
- Each crossing is one call and it names no entity.
- The type stub declares the shape of each dictionary.
- A test asks the engine for each log through the public interface, after
  driving the engine into the condition that writes it.

## What is not yet worked out

- Whether a caller wants the raw bytes as well. Three logs expose a bytes
  method beside the column method, and three do not.
- Whether an identity in a log crosses as the packed identity the engine
  mints, which is what every other verb takes, or as an index a reader can
  print.[^1]

## References

[^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
