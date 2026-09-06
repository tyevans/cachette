---
id: 0503
title: Apply the window settings in the order the window takes
status: complete
created: 2026-09-05
implements: [ADR-0094 D2, ADR-0094 D5, ADR-0067 D1]
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

Pressing the fullscreen key did not draw the window full screen, and pressing
it again crashed the run. The project owner reported both faults from a run of
the demonstration.

Four faults sat behind them. The settings sent the window size before the
fullscreen state, so leaving fullscreen set a size the window library refuses.
The refusal arrived as a raise, and the code named only a method the window
does not have, so the raise passed through the key handler. The surface took
its size from the setting rather than from the window, and a fullscreen window
is the size of the screen. The picture the window draws was built once, with a
fixed pitch, so a surface of a new size handed it a buffer of the wrong length.

## Impact review

**Governed by.** ADR-0094 D2 gives the caller the pixels, so the surface size
is the caller's to choose and the engine holds none of it. ADR-0094 D5 makes
the window one presenter of one renderer, so the window path changes no drawing
and adds no second drawing path. ADR-0067 D1 keeps the viewer out of the world,
and a window size is a viewer value that enters no state hash.

**Changes.** None. No record states the order of the window calls.

**Creates.** None. The order is a property of the window library, not a choice
this project makes, so it fails the first test for a record.

**Blockers.** None.

**Precedent.** FND-497 records the belief this work corrected. The recurring
defect rule names the shape: one fact, the size of the surface, was declared in
the settings, in the surface and in the pitch of the picture, with nothing that
failed when the three disagreed.

## Done when

- The fullscreen state reaches the window before the size, and a fullscreen
  window is given no size at all.
- A window that refuses a call by raising joins the refusal list.
- The surface follows the size the window reports, not the size the settings
  hold.
- The picture and its pitch are rebuilt when the surface changes size.
- A test covers each of the four, and each test goes red when the defect
  returns.
- The demonstration tests, the lint, the format check and the type check pass.

## Outcome

All six statements hold. The four fixes are in the settings and in the
demonstration application.

The picture the window draws is now one class that owns the size, the pitch and
the image together. It takes a maker, so a test builds it over a fake image and
needs no display. That removed the third declaration of the surface size.

One existing test asserted the old order as correct. It now asserts the new
order, and the finding records why a fake that accepts every call proved
nothing.

**The window itself is still untested, and no test here opens one.** The gate
has no display. The finding names what stays untested and how someone tests it
on a machine that has a display.
