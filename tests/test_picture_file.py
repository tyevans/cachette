"""The picture presenter: the file says what it holds, and it holds a run.

Two defects put these tests here. The picture named a file ``.png`` and wrote
PPM bytes into it, so no viewer opened it. The picture also drew after two
steps, so every subsystem the panel reports read zero and each of those zeros
came from the fixture rather than from the engine.[^1]

Nothing here loops over a tile or an entity. Each test asks the presenter for
a file and reads the file back.

References
----------
Documentation Rules, section 1. ``.claude/rules/documentation.md``
"""

from __future__ import annotations

import struct
import zlib
from pathlib import Path

import numpy as np
import pytest

from cachette.demo.surface import Surface

WIDTH = 7
HEIGHT = 5

# The first eight bytes of every PNG.
SIGNATURE = b"\x89PNG\r\n\x1a\n"


def a_surface() -> Surface:
    """Build a small surface with a colour in every channel.

    A single colour would let a channel that is dropped or swapped pass, so
    each pixel gets a value that differs in all three bytes.
    """
    surface = Surface(WIDTH, HEIGHT)
    for index in range(WIDTH * HEIGHT):
        red = (index * 7) & 0xFF
        green = (index * 13 + 40) & 0xFF
        blue = (index * 29 + 90) & 0xFF
        surface.pixels[index] = (red << 16) | (green << 8) | blue
    return surface


def channels_of(surface: Surface) -> np.ndarray:
    """Give back the expected channel bytes, one row for each pixel."""
    pixels = surface.pixels
    rgb = np.empty((pixels.size, 3), dtype=np.uint8)
    rgb[:, 0] = (pixels >> 16) & 0xFF
    rgb[:, 1] = (pixels >> 8) & 0xFF
    rgb[:, 2] = pixels & 0xFF
    return rgb


def png_pixels(path: Path) -> np.ndarray:
    """Decode a PNG this module wrote, without an image library.

    The decoder reads the header, joins every image block, inflates them and
    drops the filter byte from each row. It accepts filter 0 only, which is
    what the writer emits.
    """
    data = path.read_bytes()
    assert data[:8] == SIGNATURE
    at = 8
    header = b""
    body = b""
    while at < len(data):
        (length,) = struct.unpack(">I", data[at : at + 4])
        kind = data[at + 4 : at + 8]
        chunk = data[at + 8 : at + 8 + length]
        checksum = struct.unpack(">I", data[at + 8 + length : at + 12 + length])[0]
        assert checksum == zlib.crc32(kind + chunk) & 0xFFFFFFFF, kind
        if kind == b"IHDR":
            header = chunk
        elif kind == b"IDAT":
            body += chunk
        at += 12 + length
    width, height, depth, colour, compression, filtering, interlace = struct.unpack(
        ">2I5B", header
    )
    assert (depth, colour) == (8, 2)
    assert (compression, filtering, interlace) == (0, 0, 0)
    raw = zlib.decompress(body)
    stride = width * 3 + 1
    assert len(raw) == stride * height
    rows = np.frombuffer(raw, dtype=np.uint8).reshape(height, stride)
    assert not rows[:, 0].any(), "the writer emits filter 0 on every row"
    return rows[:, 1:].reshape(height * width, 3)


def test_a_png_name_gives_png_bytes(tmp_path: Path) -> None:
    """A file named .png starts with the PNG signature, not with P6."""
    path = tmp_path / "frame.png"
    a_surface().write_image(str(path))
    data = path.read_bytes()
    assert data[:8] == SIGNATURE
    assert not data.startswith(b"P6")


def test_a_ppm_name_gives_ppm_bytes(tmp_path: Path) -> None:
    """A file named .ppm still gets the format it is named for."""
    path = tmp_path / "frame.ppm"
    a_surface().write_image(str(path))
    assert path.read_bytes().startswith(b"P6\n7 5\n255\n")


def test_the_png_holds_every_pixel_the_surface_held(tmp_path: Path) -> None:
    """The decoded picture equals the surface, channel for channel.

    This is the test that a swapped channel or a dropped row fails. A test
    that only read the signature would pass on a file full of one colour.
    """
    surface = a_surface()
    path = tmp_path / "frame.png"
    surface.write_image(str(path))
    assert np.array_equal(png_pixels(path), channels_of(surface))


def test_the_png_declares_the_size_the_surface_had(tmp_path: Path) -> None:
    """The header states the width and the height the surface held."""
    path = tmp_path / "frame.png"
    a_surface().write_image(str(path))
    (width, height) = struct.unpack(">2I", path.read_bytes()[16:24])
    assert (width, height) == (WIDTH, HEIGHT)


def test_the_two_formats_carry_the_same_pixels(tmp_path: Path) -> None:
    """The PPM and the PNG of one surface hold the same channel bytes.

    Two presenters of one frame must not disagree. This is the check that a
    reader who converts a PPM and a reader who opens a PNG see one picture.
    """
    surface = a_surface()
    png = tmp_path / "frame.png"
    ppm = tmp_path / "frame.ppm"
    surface.write_image(str(png))
    surface.write_image(str(ppm))
    header = f"P6\n{WIDTH} {HEIGHT}\n255\n".encode("ascii")
    from_ppm = np.frombuffer(ppm.read_bytes()[len(header) :], dtype=np.uint8)
    assert np.array_equal(png_pixels(png).reshape(-1), from_ppm)


def test_an_unnamed_format_is_refused(tmp_path: Path) -> None:
    """A name that states no format is refused, and the message names both.

    Writing PPM bytes under any name the caller happened to type is what
    produced a file no viewer opened. The presenter refuses rather than
    guessing.
    """
    path = tmp_path / "frame.jpg"
    with pytest.raises(ValueError, match=r"\.png or \.ppm"):
        a_surface().write_image(str(path))
    assert not path.exists()


def test_a_name_with_no_full_stop_is_refused(tmp_path: Path) -> None:
    """A name with no extension at all is refused too.

    Three files named `0`, `40` and `600` were once committed to this
    repository holding PPM data, because a render script redirected to a
    numeric name. This is that case.
    """
    path = tmp_path / "600"
    with pytest.raises(ValueError, match=r"\.png or \.ppm"):
        a_surface().write_image(str(path))
    assert not path.exists()


def test_the_picture_runs_the_world_before_it_draws(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    """The picture mode steps the count it was given, then draws.

    The defect this closes drew after two steps. At tick 2 no seat is held, no
    unit carries a load and no soldier has been promoted, so the panel
    reported every one of those subsystems at zero. Each zero was the fixture
    and not the engine.

    Sixty steps is past the first promotion. The world takes a fixed seed and
    the engine gives one answer at any thread count, so the tick a promotion
    lands on is a property of the engine rather than of this machine. The tick
    itself is not asserted here, because it moves whenever a pass that feeds a
    unit changes.
    """
    from cachette.demo.app import main

    path = tmp_path / "run.png"
    assert main(["--picture", str(path), "--ticks", "60"]) == 0
    printed = capsys.readouterr().out

    written = [line for line in printed.splitlines() if line.startswith("wrote ")]
    assert len(written) == 1
    tick = int(written[0].split("at tick ")[1].split(",")[0])
    assert tick >= 60, printed

    # The line reads "became a character" for one and "became characters" for
    # more than one, so a match on the singular alone counts a promotion of
    # three as no promotion at all.
    promotions = [line for line in printed.splitlines() if "became " in line]
    assert promotions, printed
    assert path.read_bytes()[:8] == SIGNATURE
