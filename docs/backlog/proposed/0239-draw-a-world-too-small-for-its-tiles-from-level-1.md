---
id: 0239
title: Draw a world too small for its tiles from level 1
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: [PRD-0003]
blocked-by: []
---

## Why

**A picture of a whole world is a thing a person wants and the drawing cannot
give.** The drawing walks level 0. The tiles one frame covers go as the area of
the frame divided by the area of a tile, so a picture of a large world at a
small scale walks every tile the world has to fill a few pixels.

The record that inverts the drawing boundary refuses that case rather than
serving it.[^1] Below one pixel for each tile, more than one tile falls on the
same pixel, so the work beyond the first tile of each pixel cannot change what
the frame holds. The verb refuses and its error names the bound. **The refusal
is correct and it is not an answer.** A person who wants to see the shape of a
world still cannot.

**The structure that answers it already exists.** Level 1 summarises blocks of
level 0 tiles, and every level above level 0 is a pure function of level 0.[^2]
A frame drawn from level 1 costs the cells the frame covers, which is smaller
than the tiles by the block area. That is this project's own principle that a
set-valued command permits a cheaper algorithm, rather than a batched loop over
the tiles.[^3]

**The record forbids the easy version of this.** A verb that quietly read level
1 when level 0 grew expensive would substitute one level for another without
saying so, and the level a reader read is part of the answer.[^4] So the work
is not a fallback inside the existing verb. Whatever it is, the caller must be
able to tell which level answered.

## What is missing before this is refined

- The impact review.
- **How a caller learns which level answered.** The options are a caller that
  names the level, a verb that reports the level it used, or two verbs. The
  record that governs the levels rules out a silent choice and does not choose
  between the rest.[^4]
- **What a level 1 cell looks like as a pixel.** The drawing colours a tile
  from its kind and its height and its food. A summary holds neither a kind nor
  a height in the same sense, and a mean of a kind is not a kind. Which fields
  a cell must carry to be drawable is the first real question, and it may
  require a summary field that does not exist.
- Whether level 2 has the same argument one step further out. It probably does,
  and the answer should not be written twice.
- What a test asserts. A count of tiles read is checkable; a picture is not. A
  test that draws one world at both levels and compares a quantity both levels
  can express would show the substitution the record forbids.

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0094, the caller owns the camera and the pixels, decision D6. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
[^2]: ADR-0022, level 0 is the only truth and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^3]: Project orientation, the design principles. `CLAUDE.md`
[^4]: ADR-0022, level 0 is the only truth and every level above it is derived, decision D4. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
