"""A small font the control plane paints into the pixels it owns.

The engine draws the world. It takes no text from the caller, so a line the
caller wants over the map has to come from the caller.[^1] This module holds
that font and one blending routine.

**This is chrome, not the world.** Every pixel this module writes sits over a
frame the engine already filled, and nothing here reads a tile or an entity.
The two drawing paths therefore cannot disagree about the world, because only
one of them draws it.

The glyphs are five pixels wide and seven tall. The set holds the capital
letters, the digits and a little punctuation. A lower case letter takes the
capital glyph, and a character the set does not hold takes a hollow box, so a
missing glyph is visible rather than silent.

References
----------
ADR-0094, the caller owns the camera and the pixels, decision D2.
``docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md``
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import numpy as np

if TYPE_CHECKING:
    from cachette.demo.surface import Surface

# The width and the height of one glyph, in pixels before any scaling.
GLYPH_WIDTH = 5
GLYPH_HEIGHT = 7

# The gap between two glyphs, in pixels before any scaling.
GLYPH_GAP = 1

# The glyphs, as one block of rows for each character. A hash is ink and a
# full stop is paper. Every block holds seven rows of five characters, and a
# builder below asserts that shape, so a mistyped glyph stops the import
# rather than drawing a broken letter.
_GLYPH_ROWS: dict[str, str] = {
    " ": ".....;.....;.....;.....;.....;.....;.....",
    "A": ".###.;#...#;#...#;#####;#...#;#...#;#...#",
    "B": "####.;#...#;#...#;####.;#...#;#...#;####.",
    "C": ".###.;#...#;#....;#....;#....;#...#;.###.",
    "D": "####.;#...#;#...#;#...#;#...#;#...#;####.",
    "E": "#####;#....;#....;####.;#....;#....;#####",
    "F": "#####;#....;#....;####.;#....;#....;#....",
    "G": ".###.;#...#;#....;#.###;#...#;#...#;.###.",
    "H": "#...#;#...#;#...#;#####;#...#;#...#;#...#",
    "I": "#####;..#..;..#..;..#..;..#..;..#..;#####",
    "J": "..###;...#.;...#.;...#.;...#.;#..#.;.##..",
    "K": "#...#;#..#.;#.#..;##...;#.#..;#..#.;#...#",
    "L": "#....;#....;#....;#....;#....;#....;#####",
    "M": "#...#;##.##;#.#.#;#.#.#;#...#;#...#;#...#",
    "N": "#...#;##..#;#.#.#;#.#.#;#..##;#...#;#...#",
    "O": ".###.;#...#;#...#;#...#;#...#;#...#;.###.",
    "P": "####.;#...#;#...#;####.;#....;#....;#....",
    "Q": ".###.;#...#;#...#;#...#;#.#.#;#..#.;.##.#",
    "R": "####.;#...#;#...#;####.;#.#..;#..#.;#...#",
    "S": ".####;#....;#....;.###.;....#;....#;####.",
    "T": "#####;..#..;..#..;..#..;..#..;..#..;..#..",
    "U": "#...#;#...#;#...#;#...#;#...#;#...#;.###.",
    "V": "#...#;#...#;#...#;#...#;#...#;.#.#.;..#..",
    "W": "#...#;#...#;#...#;#.#.#;#.#.#;##.##;#...#",
    "X": "#...#;#...#;.#.#.;..#..;.#.#.;#...#;#...#",
    "Y": "#...#;#...#;.#.#.;..#..;..#..;..#..;..#..",
    "Z": "#####;....#;...#.;..#..;.#...;#....;#####",
    "0": ".###.;#...#;#..##;#.#.#;##..#;#...#;.###.",
    "1": "..#..;.##..;..#..;..#..;..#..;..#..;.###.",
    "2": ".###.;#...#;....#;...#.;..#..;.#...;#####",
    "3": "#####;...#.;..#..;...#.;....#;#...#;.###.",
    "4": "...#.;..##.;.#.#.;#..#.;#####;...#.;...#.",
    "5": "#####;#....;####.;....#;....#;#...#;.###.",
    "6": "..##.;.#...;#....;####.;#...#;#...#;.###.",
    "7": "#####;....#;...#.;..#..;.#...;.#...;.#...",
    "8": ".###.;#...#;#...#;.###.;#...#;#...#;.###.",
    "9": ".###.;#...#;#...#;.####;....#;...#.;.##..",
    ".": ".....;.....;.....;.....;.....;.##..;.##..",
    ",": ".....;.....;.....;.....;.##..;.##..;.#...",
    ":": ".....;.##..;.##..;.....;.##..;.##..;.....",
    "-": ".....;.....;.....;#####;.....;.....;.....",
    "'": "..#..;..#..;.....;.....;.....;.....;.....",
    "(": "...#.;..#..;.#...;.#...;.#...;..#..;...#.",
    ")": ".#...;..#..;...#.;...#.;...#.;..#..;.#...",
    "!": "..#..;..#..;..#..;..#..;..#..;.....;..#..",
    "?": ".###.;#...#;....#;...#.;..#..;.....;..#..",
}

# The glyph a character outside the set takes. A hollow box is visible, so a
# sentence that reaches for a character nobody drew says so on the screen.
_UNKNOWN = "#####;#...#;#...#;#...#;#...#;#...#;#####"


def _mask(rows: str) -> np.ndarray:
    """Turn one block of rows into an array of ink flags."""
    lines = rows.split(";")
    if len(lines) != GLYPH_HEIGHT:
        message = f"a glyph holds {GLYPH_HEIGHT} rows, not {len(lines)}"
        raise ValueError(message)
    for line in lines:
        if len(line) != GLYPH_WIDTH:
            message = f"a glyph row holds {GLYPH_WIDTH} pixels, not {len(line)}"
            raise ValueError(message)
    return np.array(
        [[character == "#" for character in line] for line in lines],
        dtype=bool,
    )


# The glyphs as arrays, built once at import. The build checks the shape of
# every block, so a mistyped glyph stops the import.
GLYPHS: dict[str, np.ndarray] = {
    character: _mask(rows) for character, rows in _GLYPH_ROWS.items()
}
UNKNOWN_GLYPH = _mask(_UNKNOWN)


def text_width(text: str, scale: int) -> int:
    """Give back how wide a line of text is, in pixels, at this scale."""
    if not text:
        return 0
    return (len(text) * (GLYPH_WIDTH + GLYPH_GAP) - GLYPH_GAP) * scale


def text_height(scale: int) -> int:
    """Give back how tall a line of text is, in pixels, at this scale."""
    return GLYPH_HEIGHT * scale


def line_mask(text: str, scale: int) -> np.ndarray:
    """Give back one array of ink flags for a whole line of text.

    The array is as tall as a glyph and as wide as the line, both at the
    scale. **The glyphs are copied into one array and then scaled once.** A
    caller that blended each glyph on its own would blend once for each
    character of the line.
    """
    width = max(text_width(text, 1), 1)
    mask = np.zeros((GLYPH_HEIGHT, width), dtype=bool)
    step = GLYPH_WIDTH + GLYPH_GAP
    for at, character in enumerate(text):
        glyph = GLYPHS.get(character.upper(), UNKNOWN_GLYPH)
        left = at * step
        mask[:, left : left + GLYPH_WIDTH] = glyph
    if scale > 1:
        mask = np.repeat(np.repeat(mask, scale, axis=0), scale, axis=1)
    return mask


def _blend(under: np.ndarray, colour: int, weight: float) -> np.ndarray:
    """Mix a colour into a block of pixels by a weight from zero to one.

    The pixels hold red, green and blue in the low three bytes. Each channel
    is mixed on its own and the result is packed back.

    **This is one array operation over a block, not a loop over pixels.**
    """
    share = float(min(max(weight, 0.0), 1.0))
    kept = 1.0 - share
    mixed = np.zeros_like(under)
    for shift in (16, 8, 0):
        was = (under >> shift) & 0xFF
        wants = (colour >> shift) & 0xFF
        now = np.clip(was * kept + wants * share, 0, 255).astype(np.uint32)
        mixed |= now << shift
    return mixed


def paint_block(
    surface: Surface,
    left: int,
    top: int,
    width: int,
    height: int,
    colour: int,
    weight: float,
) -> None:
    """Mix one colour over a rectangle of the frame.

    A rectangle that falls outside the frame is cut to the frame. A rectangle
    wholly outside it paints nothing.
    """
    first_x = max(left, 0)
    first_y = max(top, 0)
    last_x = min(left + width, surface.width)
    last_y = min(top + height, surface.height)
    if last_x <= first_x or last_y <= first_y:
        return
    frame = surface.pixels.reshape(surface.height, surface.width)
    block = frame[first_y:last_y, first_x:last_x]
    frame[first_y:last_y, first_x:last_x] = _blend(block, colour, weight)


def paint_text(
    surface: Surface,
    left: int,
    top: int,
    text: str,
    colour: int,
    scale: int = 1,
    weight: float = 1.0,
) -> None:
    """Paint a line of text into the frame at this place.

    The weight is how much of the colour reaches the pixels, from zero to one.
    A caller fades a line by lowering it.

    Text that falls outside the frame is cut to the frame.
    """
    if not text or weight <= 0.0:
        return
    mask = line_mask(text, scale)
    rows, columns = mask.shape
    first_x = max(left, 0)
    first_y = max(top, 0)
    last_x = min(left + columns, surface.width)
    last_y = min(top + rows, surface.height)
    if last_x <= first_x or last_y <= first_y:
        return
    inside = mask[first_y - top : last_y - top, first_x - left : last_x - left]
    frame = surface.pixels.reshape(surface.height, surface.width)
    block = frame[first_y:last_y, first_x:last_x]
    frame[first_y:last_y, first_x:last_x] = np.where(
        inside, _blend(block, colour, weight), block
    )
