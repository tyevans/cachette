"""The two readers of the feedback file must agree.

The generation loop reads `feedback.json` to steer the next round. The front
end reads it to draw the page. That is two declaration sites of one shape, and
a second site needs a check that fails when the copies disagree.

This is the only test that imports the tool package. The server never does. It
calls the tool through its command line.
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

HERE = Path(__file__).resolve().parent
TOOL = HERE.parent
for entry in (str(HERE), str(TOOL)):
    if entry not in sys.path:
        sys.path.insert(0, entry)

from direct_die import steer  # noqa: E402
from store import read_feedback as front_end_reader  # noqa: E402

CASES = [
    None,
    {},
    ["b"],
    {"choice": "b", "text": "more contrast"},
    {"choice": None, "text": ""},
    {"choice": "a", "likes": ["c"]},
    {"likes": ["b", "d"], "denies": ["a"], "order": ["d", "b"]},
    {"likes": ["b"], "denies": ["b"]},
    {"likes": ["b", "z", 3], "denies": ["q"]},
    {"likes": ["b", "b", "d"]},
    {"likes": ["a", "b", "c"], "order": ["c"]},
    {"likes": ["b"], "order": ["d", "b"]},
    {"likes": "bd", "note": 7, "denies": {"a": 1}},
    {"note": "  keep it flat  ", "text": "  "},
    {"round": 3, "likes": ["d"], "order": ["d"], "note": "x", "text": "y"},
]


@pytest.mark.parametrize("value", CASES)
def test_the_two_readers_give_the_same_answer(value) -> None:
    theirs = steer.read_feedback(value)
    likes, denies, order, note, text = front_end_reader(value)
    assert likes == theirs.likes
    assert denies == theirs.denies
    assert order == theirs.order
    assert note == (theirs.note or "")
    assert text == (theirs.text or "")
