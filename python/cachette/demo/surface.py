"""The pixels the control plane lends the engine.

A surface is a block of memory this package owns. The engine writes one frame
into it and returns, and holds no reference to it afterwards.[^1]

The window library needs bytes in a particular order. This module holds that
knowledge, so the application code says "draw" and "present" and nothing about
byte order.

References
----------
ADR-0094, the caller owns the camera and the pixels, decision D2.
``docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md``
"""

from __future__ import annotations

import numpy as np

# The engine writes red, green and blue into the low three bytes of each
# value and leaves the top byte clear. A window that reads the top byte as
# opacity would then draw a fully transparent picture, so the surface sets it
# once for the whole array before it presents.
#
# **This is one array operation, not a loop.** It touches every pixel and no
# entity, and its cost follows the size of the window.
OPAQUE = np.uint32(0xFF000000)


class Surface:
    """A block of pixels that the engine fills and a window presents."""

    __slots__ = ("_pixels", "height", "width")

    def __init__(self, width: int, height: int) -> None:
        """Build a block of pixels of this size."""
        if width <= 0 or height <= 0:
            message = f"a surface of {width} by {height} holds no pixel"
            raise ValueError(message)
        self.width = width
        self.height = height
        # The engine refuses a buffer of the wrong size, so the shape is
        # declared once, here, and never recomputed at a call site.
        self._pixels = np.zeros(width * height, dtype=np.uint32)

    @property
    def pixels(self) -> np.ndarray:
        """Give back the array the engine writes into."""
        return self._pixels

    def to_bytes(self) -> bytes:
        """Give back the frame as bytes a window can upload.

        The top byte of each value becomes opaque first, because the engine
        leaves it clear and a window reads it as opacity.
        """
        return (self._pixels | OPAQUE).tobytes()
