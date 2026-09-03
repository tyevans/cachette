---
id: 0099
title: Drive or retire the faction mask
status: proposed
created: 2026-08-31
implements: []
changes: []
creates: []
serves: [PRD-0006]
blocked-by: []
---

## Why

The holding work added a faction mask, one per block of tiles, and a union
over two masks. It also reserved a bit for factions outside the addressable
set.

**Nothing in the engine calls the union, and nothing sets the reserved bit.**
The union has a test that constructs two masks and unions them, so the test
passes and proves that the mechanism works. It does not prove that anything
reaches the mechanism.

This is the third shape in the recurring defect list, and the project has
imported evidence of nine capabilities shipping inert in one wave
elsewhere.[^1] It is also the shape a review found in the panel's overflow
detector in the same session, so this is the second local instance.[^2]

The mask itself is not inert. A caller reads the masks to ask which blocks a
faction holds, and that is the query the product record asks for.[^3] The union
and the reserved bit are the parts nothing reaches.

## What the work does

Answer one question for each part: **who is obligated to invoke this, the user
of the crate or the engine?**[^4]

- If the engine, write the test that starts at the engine and let it drive the
  call. A capability the engine must invoke needs a test that begins there.
- If the user of the crate, say so where the capability is declared, and keep
  the direct test.
- If neither, delete it. A capability declared before something calls it is
  the defect, and removing it costs nothing today.

The reserved bit needs the same treatment and may need a different answer: a
constraint a record states, which no code sets yet, is legitimate. If that is
the answer, the module must say so rather than leaving a reader to guess.

## Done when

- Each of the union and the reserved bit is driven by a real caller, declared
  as a crate interface, or removed.
- Anything kept states who must invoke it.
- No test constructs a mechanism and exercises it as the only evidence that
  the mechanism is reached.
- `just check` exits 0.

## References

[^1]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
[^2]: Backlog item 0072. `docs/backlog/complete/0072-run-the-panel-fit-check-in-the-drawing-pass.md`
[^3]: PRD-0006, a place belongs to somebody. `docs/product/accepted/prd-0006-a-place-belongs-to-somebody.md`
[^4]: Testing Rules, section 5. `.claude/rules/testing.md`
