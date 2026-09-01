---
id: 0093
title: Say what the founding chose in the panel
status: refined
created: 2026-08-31
implements: [ADR-0067 D1, ADR-0067 D2, ADR-0070 D1, ADR-0070 D2, ADR-0075 D5]
changes: []
creates: []
serves: [PRD-0012, PRD-0005]
blocked-by: []
---

## Why

A run now begins with a small group in a place the engine chose. The panel
still describes the world it was written for: a full world of soldiers spread
everywhere.

The panel says how many units are alive, how many the window draws, how many
of each faction the window draws, and what ground the window shows. Every one
of those rows is correct. None of them says the thing a watcher of a founded
run wants to know: where the group is, and why the engine put it there.

The founding already computes the answer and returns it. The chooser reports
the place it took, the quantities it read there, and the candidates it
rejected with theirs.[^1] Nothing shows any of it. A watcher must read the
terminal, and the terminal prints the line once and then scrolls away.

## What the work does

1. The panel names the founded place and says whether the window shows it.
2. The panel gives the quantities that made the place the choice, as the
   survey read them.
3. The panel says how many places the founding compared, so a watcher can tell
   a choice from a default.
4. A watcher compares the chosen place against a place that was not chosen.

## The two questions the proposed item held, and their answers

**May the panel hold a value that the drawing pass did not read?** Yes, and it
needs no third category. The panel record names three sources and says there
is no fourth.[^2] The second source is a value the viewer computes from its
own state, and it already covers the camera, the zoom and the extent of the
window. The founding report is a value of exactly that kind: the viewer holds
it, the viewer does not recompute it, and reading it starts no loop over the
world and no loop over the units. The record therefore needs no change.

**Does the founding report belong to the world?** No. It belongs to the
program that owns the loop, and that program passes it to the panel.

The boundary record decides that the engine holds no value that exists because
something draws it.[^3] The world does not keep the survey today, and a field
added to keep it would exist for the panel and for nothing else. The same
record already states the general form of the answer: a value the viewer needs
and the world does not have is the viewer's to hold. The exemption in that
record covers the demonstration binary, which owns both ends of the loop.[^4]

**There is therefore one copy and one declaration site.** The founding call
returns one value, the binary holds that one value, and the panel borrows it.
Nothing recomputes a score, so nothing can disagree with the choice that was
made.[^5] [^6]

## Impact review

**Governed by.**

- ADR-0070 D1. The panel adds no pass over the world. The founding report is a
  value the viewer holds, and reading a field of it costs one read.[^2]
- ADR-0070 D2. A number the panel cannot afford is absent and is never
  estimated. The rows state what the survey read at the places the survey
  visited. They state nothing about the rest of the world, and they label the
  place they describe.[^7]
- ADR-0067 D2. The engine gains no field. The report stays with the caller,
  which is what makes this checkable by a reviewer.[^3]
- ADR-0067 D1. The panel holds a shared reference to the world and writes
  nothing to it. The founding report it reads is already computed, so the
  panel calls nothing that founds anything.[^8]
- ADR-0075 D5. The chooser reports the properties that made the place the
  choice, and the report is the output of the choice. The panel restates that
  output and derives nothing from it.[^1]

**Changes.** No record changes. Both questions above resolve inside the
records as written.

**Creates.** No record. **This is a deliberate judgement against the scope
rule.**[^9] The claim that would be recorded is "the panel states a value the
viewer holds from before the first frame", and it fails condition one: the
panel record already names the viewer's own state as a source and forbids a
fourth. A record that restated it would state a claim an accepted record
holds, in a second place.

**Blockers.**

- BLK-007 governs every cost figure, so this item states none.[^10]
- BLK-018 asks how many groups found a world, and it is open.[^11] The item is
  therefore written parametrically: **the panel describes the foundings the
  binary holds, and it does not assume there is one.** A panel row that named
  "the founding" would state something false the moment a second group founds.
  The row states which founding it describes, and the layout is written for a
  list whose length the binary chooses.

**Precedent.**

- FND-071 records that the whole-world pass the pyramid gave up was still
  alive in the demonstration binary, and that a cost claim reaches the
  examples and not only the engine.[^12] This item adds panel rows to that
  same binary, so it must add no pass of its own.
- FND-051 records that a fixture chosen for realism hides the defect it should
  show.[^13] The picture test must cover a founding whose rejected candidates
  differ from the chosen one, or the comparison row is untested.
- FND-061 records that a fixture assertion belongs over the outcome and not
  over the inputs.[^14]

**Serves.** PRD-0012 and PRD-0005.

PRD-0012 asks that a watcher see the founding place and compare it against the
places that were not chosen, and that a watcher ask which properties of the
place made it the choice.[^15] The engine answers both today and no person can
read the answer. This item is what meets the need, and it meets it in full for
one founding.

PRD-0005 asks that every number the window states be a number a reader can
check, and that a number the viewer cannot know be absent.[^16] Each new row
is a value the engine computed and reported, so each is checkable against the
report the run prints.

**Conflict surface.** `crates/cachette-view/src/hud.rs` at the readout, the
layout and the bounds. `crates/cachette-view/src/lib.rs` at the frame call,
because the readout gains an input. `crates/cachette-view/src/main.rs`, which
already holds the founding and must pass it. `crates/cachette-view/tests/
pictures/panel.txt` and the picture test that reads it.
`crates/cachette-core/src/founding.rs` is read and not changed.

**It cannot run beside item 0069** or **item 0072**, which both change the
panel and its stored picture. **It cannot run beside item 0085**, which
changes the frame call in the same crate. It does not conflict with any engine
item, because it changes no engine file.

## Done when

- The panel names each founded place as a tile address, and says whether the
  window shows it.
- The panel states the quantities the survey read at the chosen place, and
  they equal the quantities the founding reported. A test asserts the
  equality against the report rather than against a constant.
- The panel states how many places the founding compared.
- The panel states one place that was not chosen, with its quantities, so a
  watcher compares the two.
- No row states a value that nothing computed, and no row estimates.
- The panel starts no loop over the units and no loop over the tiles. A
  reviewer finds no such loop in the reporting code.
- The engine gains no field. The world holds no founding report.
- The rows describe the foundings the binary holds, and the layout does not
  assume there is one.[^11]
- A picture test covers the new rows, in the same way the existing rows are
  covered, and its fixture holds a rejected candidate that differs from the
  chosen one.[^13]
- Every new value fits its column. A value that does not fit is cut, and a cut
  value states a number other than the one it was given.
- The two determinism tests are unaffected, and the commit body says so. The
  viewer is outside them.
- `just check` runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D5. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
[^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
[^3]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^4]: ADR-0067, the viewer reads the world and never writes to it, decision D4. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^5]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^6]: Findings register, FND-066. `docs/FINDINGS.md`
[^7]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
[^8]: ADR-0067, the viewer reads the world and never writes to it, decision D1. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^9]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^10]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^11]: Blockers register, BLK-018. `docs/BLOCKERS.md`
[^12]: Findings register, FND-071. `docs/FINDINGS.md`
[^13]: Findings register, FND-051. `docs/FINDINGS.md`
[^14]: Findings register, FND-061. `docs/FINDINGS.md`
[^15]: PRD-0012, a world starts small and grows. `docs/product/shaped/prd-0012-a-world-starts-small-and-grows.md`
[^16]: PRD-0005, a watcher can tell what is happening and why. `docs/product/shaped/prd-0005-a-watcher-can-tell-what-is-happening-and-why.md`
