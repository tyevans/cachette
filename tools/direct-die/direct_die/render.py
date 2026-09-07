"""Rasterise an SVG asset at more than one size.

An asset is judged where a player sees it. A hex tile on the map is small.
A drawing that only reads at 512 pixels is of no use. Every render
therefore makes two rasters: one at the display size, and one at the
inspection size. The loop shows both to the model in the same turn, and it
says which is which.
"""

from __future__ import annotations

import re
from dataclasses import dataclass

import cairosvg

# The background colour under a transparent asset. A mid grey shows both
# light art and dark art.
BACKGROUND = "#8a8f94"


class RenderError(RuntimeError):
    """The SVG source did not rasterise."""


@dataclass(frozen=True)
class SizeSet:
    """The two sizes at which the tool judges one asset type."""

    display: int
    inspection: int

    def as_pairs(self) -> list[tuple[str, int]]:
        """Give the sizes as (name, pixels) pairs, smallest first."""
        return [("display", self.display), ("inspection", self.inspection)]


# The size set of each asset type. A new asset type needs a row here.
SIZES: dict[str, SizeSet] = {
    "hex-tile": SizeSet(display=64, inspection=384),
    "unit": SizeSet(display=32, inspection=384),
    "icon": SizeSet(display=24, inspection=288),
}

# The size set for an asset type that no row names.
DEFAULT_SIZES = SizeSet(display=64, inspection=384)


def sizes_for(asset: str) -> SizeSet:
    """Give the size set of one asset type."""
    return SIZES.get(asset, DEFAULT_SIZES)


_SVG_START = re.compile(r"<svg[\s>]", re.IGNORECASE)


def extract_svg(text: str) -> str:
    """Take the SVG document out of a model answer.

    The model often wraps the source in a fence, or writes a sentence
    before it. This function keeps the first `<svg>` element and drops the
    rest.
    """
    start = text.find("<svg")
    if start < 0 or not _SVG_START.search(text[start:]):
        raise RenderError("the answer holds no <svg> element")
    end = text.rfind("</svg>")
    if end < 0:
        raise RenderError("the answer holds no closing </svg> tag")
    return text[start : end + len("</svg>")].strip()


def render(svg_text: str, pixels: int) -> bytes:
    """Rasterise one SVG document to a square PNG of the given size."""
    try:
        png = cairosvg.svg2png(
            bytestring=svg_text.encode("utf-8"),
            output_width=pixels,
            output_height=pixels,
            background_color=BACKGROUND,
        )
    except Exception as error:  # cairosvg raises many types.
        raise RenderError(f"cairosvg failed: {error}") from error
    if not png:
        raise RenderError("cairosvg produced an empty image")
    return png


def render_both(svg_text: str, sizes: SizeSet) -> dict[str, bytes]:
    """Rasterise one SVG document at the display size and at the
    inspection size.

    The keys of the result are `display` and `inspection`.
    """
    return {name: render(svg_text, pixels) for name, pixels in sizes.as_pairs()}
