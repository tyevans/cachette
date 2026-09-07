"""The two colours of the page, as whole numbers.

**One thing declares the page, and everything else derives from it.** The
sketch declares the paper and the ink as three channels each, because its own
arithmetic works on channels. A card and a disc blend whole colours, so they
need the same two colours packed into one number.

A second copy of the two numbers would be a second declaration site. Nothing
would fail when the sketch changed its paper and the interface kept the old
one, and every card would then sit on a page of a different colour. This
module packs the numbers the sketch declares, so there is nothing to keep in
step.
"""

from __future__ import annotations

import numpy as np

from cachette.demo.sketch import INK as _INK_CHANNELS
from cachette.demo.sketch import PAPER as _PAPER_CHANNELS


def _packed(channels: np.ndarray) -> int:
    """Pack red, green and blue into one whole number."""
    red, green, blue = (round(float(value)) for value in channels)
    return (red << 16) | (green << 8) | blue


# The colour of the page, and the colour of the ink on it.
PAPER = _packed(_PAPER_CHANNELS)
INK = _packed(_INK_CHANNELS)
