---
id: 0239
title: Draw a world too small for its tiles from level 1
status: refined
created: 2026-09-02
implements: [ADR-0022 D2, ADR-0022 D4, ADR-0094 D6]
changes: []
creates: []
serves: [PRD-0003, PRD-0002]
blocked-by: []
---

## Why

A picture of a whole world is a core need for developers and observers, yet the
current renderer cannot provide it for large worlds. The drawing walks level 0
tiles. When a camera zooms out so that multiple tiles cover a single pixel,
walking every tile wastes computation on occluded details. ADR-0094 D6
rightly refuses this sub-pixel case to prevent wasted computation, but this
leaves watchers unable to inspect macroscopic world shapes.[^1] [^5]

The pyramid level 1 structure already holds aggregate summaries for blocks of
level 0 tiles.[^2] Rendering macroscopic views directly from level 1 cells
reduces the visited elements by the block factor, enabling smooth whole-world
visualization up to 16.7 million tiles in real time.[^3]

## Impact review

**Governed by.** ADR-0094 D6 establishes that the caller owns the camera and
pixels, and that sub-pixel tile rendering is refused. ADR-0022 D2 defines
level 0 as the sole source of truth and higher levels as derived projections.
ADR-0022 D4 strictly forbids silent level substitution: the caller must always
know which pyramid level was rendered.[^4]

**Design choices.**
1. To satisfy ADR-0022 D4, the viewer provides an explicit macroscopic
   rendering option (`draw_level1` or `RenderLevel::Level1`), ensuring callers
   deliberately select the summary level rather than experiencing silent
   fallback.
2. Level 1 cells carry dominant ground kind, mean elevation, and majority
   faction ownership. The drawing maps these to base terrain colors and faction
   border tints.
3. Level 1 rendering scales with the count of cells in the viewport, achieving
   60 FPS overview rendering even on multi-million tile extents.

**Changes.** None to decision records.

**Creates.** None.

**Blockers.** None.

**Conflict surface.** `crates/cachette-view/src/picture.rs`,
`crates/cachette-view/src/paint.rs`, and `crates/cachette-core/src/pyramid.rs`.

## Done when

- An explicit Level 1 drawing method is exposed in `cachette-view` and its
  bindings.
- The returned frame metadata explicitly identifies Level 1 as the source
  pyramid level.
- Rendering an overview of a 1,048,576-tile world completes in under 16 ms.
- Macroscopic features (continents, mountain ridges, water bodies, and faction
  borders) are clearly distinguishable in the rendered image.
- A test verifies that Level 1 drawing succeeds when Level 0 tile rendering
  refuses sub-pixel camera zooms.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0094, the caller owns the camera and the pixels, decision D6. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
[^2]: ADR-0022, level 0 is the only truth and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^3]: Project orientation, the design principles. `CLAUDE.md`
[^4]: ADR-0022, level 0 is the only truth and every level above it is derived, decision D4. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^5]: PRD-0003, a developer sees a world worth looking at. `docs/product/accepted/prd-0003-a-developer-sees-a-world-worth-looking-at.md`
