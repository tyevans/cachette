---
id: 0133
title: Let a watcher reach a panel longer than the window
status: complete
created: 2026-09-01
implements: [ADR-0067 D1, ADR-0067 D2, ADR-0067 D3, ADR-0070 D1, ADR-0070 D2]
changes: []
creates: []
serves: [PRD-0005]
blocked-by: []
---

## Why

The panel was longer than the window. It cut at the foot and said so with a
notice on the last line. A watcher who read the notice could not then reach the
rows below it. There was no scroll, no fold and no other page.

The panel grew each time the viewer gained a section, and every new section
forced the same placement choice again. In the demonstration window the cut
removed the ground legend and the cost rows, so the window did not name every
colour it drew, which a product record asks for.[^1]

The notice was honest, and honesty is not the same as reachable.

## Impact review

**Governed by.**

- **ADR-0067 D1.** The viewer reads the world through the public interface and
  writes nothing to it. The cards take a shared reference to the readout.[^2]
- **ADR-0067 D2.** The engine holds no value that exists for the viewer. The
  work adds no engine field. The key state lives for one frame in the caller,
  in the same way the camera does.[^2]
- **ADR-0067 D3.** Floating point begins at the viewer boundary and never
  returns.[^2]
- **ADR-0070 D1.** The panel adds no pass over the world. The cards read the
  same readout the panel reads, which is taken once. They add no reader.[^3]
- **ADR-0070 D2.** A number the panel cannot afford is absent, never estimated.
  A card states a dash rather than a zero when the engine will not answer.[^3]

**Changes.** No record changes. The work contradicts none of them.

**Creates.** No record. The claim behind the overlay is worth one, and the
number is not allocated. The register holds the reasoning meanwhile.[^4]

**Blockers.** None.

**Precedent.** FND-198 records that a second layout of one reading loses the
corrections the first one earned. FND-199 records that the agent inspection
tool cannot carry what the window stopped showing, which is why the detail went
to a rendered picture instead.[^5]

## What the work did

**The window draws cards, not a panel.** Three cards hold what changes moment
to moment: the run, the ground under the crosshair, and the nearest unit. They
sit at the corners and the map fills the rest of the window.

**A key reveals what a watcher checks occasionally.** The faction legend, the
ground legend, the camera position and the cost appear while the key is held.
One mechanism serves all four, and the key holds no state between frames.

**The panel still holds every section.** It moved from the window to a rendered
picture, which one build recipe produces at a height that never cuts. A line
that is always visible names that command.

## The candidates this item listed, and what happened to them

The item offered a fold, a scroll and a second column, and asked whether the
order of the sections was the cheaper answer.

The order was tried first and it is recorded as a measurement: it bought one
placement and could not buy a second.[^6] The project owner then chose the
overlay over all three candidates, so none of them was built. The register
records that outcome against the row that recommended the scroll.[^6]

## Done when

- The window draws cards, and every card sits inside the window at several
  sizes, with the key up and with it down.
- The glass covers a small part of the window, and less of it than the panel
  did.
- The window states how many units the world holds beside how many it shows,
  and labels both.
- A key names every faction the world holds and every kind of ground, and names
  no colour that no faction uses.
- The same key states where the person is looking and what the step and the
  drawing cost.
- One command renders every number the window does not show, and the window
  names that command.
- Each test has a proven failure mode.
- `just check` passes.

## Outcome

Done. The window shows the moment and the panel moved to a picture.

**What the tests catch.** Each defect below was put back on its own, and the
suite was run each time.

- The run card reads the window count into the world row: one test fails.
- The legend draws whatever the key says: two tests fail.
- The legend is sized by the colour table rather than the world: one test
  fails.
- The food row states what the ground gave rather than what is left: one test
  fails.
- A card is placed from a corner without its own size: one test fails.
- The chosen option is read from the intent rather than the winner: one test
  fails.
- The recipe is renamed and the window is not: **this passed at first.** The
  check asked whether a recipe line starts with the name, and a recipe named
  `inspect-the-panel` starts with `inspect`. The predicate now asks for the
  name followed by an argument or a colon, and the rename then fails it.

**What is not done.** The agent inspection tool still cannot show what the
panel shows. FND-199 holds the count of what it lacks, and a separate item
holds the work.[^5]

## References

[^1]: PRD-0005, a watcher can tell what is happening and why. `docs/product/shipped/prd-0005-a-watcher-can-tell-what-is-happening-and-why.md`
[^2]: ADR-0067, the viewer reads the world and never writes to it. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^3]: ADR-0070, the head-up display reports what the drawing pass read. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
[^4]: Decisions register, DEC-084. `docs/DECISIONS.md`
[^5]: Backlog item 0206. `docs/backlog/proposed/0206-let-the-agent-tool-read-what-the-panel-reads.md`
[^6]: Decisions register, DEC-078. `docs/DECISIONS.md`
