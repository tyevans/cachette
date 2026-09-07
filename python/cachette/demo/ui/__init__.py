"""The interface a person uses to watch the world, and to play in it.

The sketch draws the world as ink on a paper page. This package draws the
interface in the same two colours, so a card reads as a sheet laid on that
page rather than as a window from another program.

Every module here paints into the frame the renderer already filled. Nothing
here reads a tile or an entity one at a time, and nothing here steps the
world.
"""

from __future__ import annotations

from cachette.demo.ui.chrome import Chrome

__all__ = ["Chrome"]
