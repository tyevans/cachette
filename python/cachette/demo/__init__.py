"""A window that shows the world, driven from the control plane.

This package opens a window, drives the engine, and presents a surface. It
draws nothing itself. The engine fills a buffer this package owns, and this
package puts the result on a screen.[^1]

**Nothing here loops over a tile or over an entity.** The camera names the
tiles a picture covers without naming them one at a time, and one call fills
the whole frame. A loop on this side of the boundary would cross it once for
each tile, and the crossing costs more than the drawing.[^2]

The window is one presenter. The Rust binary is another. Both stand on one
drawing path, so neither can disagree with the other about the world.[^3]

References
----------
ADR-0094, the caller owns the camera and the pixels, decision D2.
``docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md``

ADR-0094, the caller owns the camera and the pixels, decision D1.
``docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md``

ADR-0094, the caller owns the camera and the pixels, decision D5.
``docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md``
"""

from cachette.demo.app import Demo, main

__all__ = ["Demo", "main"]
