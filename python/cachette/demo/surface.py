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

import struct
import zlib

import numpy as np

# The engine writes red, green and blue into the low three bytes of each
# value and leaves the top byte clear. A window that reads the top byte as
# opacity would then draw a fully transparent picture, so the surface sets it
# once for the whole array before it presents.
#
# **This is one array operation, not a loop.** It touches every pixel and no
# entity, and its cost follows the size of the window.
OPAQUE = np.uint32(0xFF000000)

# The eight bytes every PNG starts with. A reader that does not find them
# reports the file as damaged.
PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"


def _png_chunk(kind: bytes, body: bytes) -> bytes:
    """Wrap one block of a PNG in its length, its name and its checksum."""
    return (
        struct.pack(">I", len(body))
        + kind
        + body
        + struct.pack(">I", zlib.crc32(kind + body) & 0xFFFFFFFF)
    )


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

    def write_image(self, path: str) -> None:
        """Write the frame to a file that an image tool reads.

        A window is one presenter and a file is another. Both ask the engine
        for a frame and put the result somewhere, and neither draws anything
        itself, so a picture on a disk cannot disagree with a picture on a
        screen.

        The name of the file chooses the format. A name that ends in ``.png``
        gives a PNG, and a name that ends in ``.ppm`` gives a binary PPM. Any
        other name is refused, because a file whose bytes disagree with its
        name is worse than no file: the tool that opens it reports a download
        error and the reader looks for a network fault that is not there.

        Neither format needs a dependency or a display. PNG needs ``zlib``,
        which the standard library holds.
        """
        suffix = path.rsplit(".", 1)[-1].lower() if "." in path else ""
        if suffix == "png":
            self.write_png(path)
        elif suffix == "ppm":
            self.write_ppm(path)
        else:
            message = (
                f"cannot name the format of {path!r}: the name must end in .png or .ppm"
            )
            raise ValueError(message)

    def _rgb(self) -> np.ndarray:
        """Give back the frame as one row of red, green and blue bytes.

        The engine writes the three channels into the low bytes of each value,
        so this is a shift and a mask over the whole array. It touches every
        pixel and no entity.
        """
        pixels = self._pixels
        rgb = np.empty((pixels.size, 3), dtype=np.uint8)
        rgb[:, 0] = (pixels >> 16) & 0xFF
        rgb[:, 1] = (pixels >> 8) & 0xFF
        rgb[:, 2] = pixels & 0xFF
        return rgb

    def write_ppm(self, path: str) -> None:
        """Write the frame as a binary PPM.

        The format is one text header and then the channel bytes.
        """
        header = f"P6\n{self.width} {self.height}\n255\n".encode("ascii")
        with open(path, "wb") as out:
            out.write(header)
            out.write(self._rgb().tobytes())

    def write_png(self, path: str) -> None:
        """Write the frame as a PNG.

        The image holds eight bits for each of three channels and no
        transparency, so the colour type is 2 and the bit depth is 8. Every
        row carries the filter byte 0, which means the row holds the values
        themselves. A filter that predicted each byte would make the file
        smaller and would make this function a compressor, and the picture
        is a demonstration rather than an asset.
        """
        rows = self._rgb().reshape(self.height, self.width * 3)
        # One filter byte in front of each row. This builds the whole block
        # with two array operations rather than a loop over the rows.
        raw = np.zeros((self.height, self.width * 3 + 1), dtype=np.uint8)
        raw[:, 1:] = rows
        header = struct.pack(">2I5B", self.width, self.height, 8, 2, 0, 0, 0)
        with open(path, "wb") as out:
            out.write(PNG_SIGNATURE)
            out.write(_png_chunk(b"IHDR", header))
            out.write(_png_chunk(b"IDAT", zlib.compress(raw.tobytes(), 6)))
            out.write(_png_chunk(b"IEND", b""))

    def to_bytes(self) -> bytes:
        """Give back the frame as bytes a window can upload.

        The top byte of each value becomes opaque first, because the engine
        leaves it clear and a window reads it as opacity.
        """
        return (self._pixels | OPAQUE).tobytes()
