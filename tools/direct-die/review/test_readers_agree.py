"""The two readers of the feedback file must agree.

The generation loop reads `feedback.json` to steer the next round. The front
end reads it to draw the page. That is two declaration sites of one shape, and
a second site needs a check that fails when the copies disagree.

This is the only test that imports the tool package. The server never does. It
calls the tool through its command line.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

import pytest

HERE = Path(__file__).resolve().parent
TOOL = HERE.parent
for entry in (str(HERE), str(TOOL)):
    if entry not in sys.path:
        sys.path.insert(0, entry)

from direct_die import session as session_module  # noqa: E402
from direct_die import steer  # noqa: E402
from store import SessionStore  # noqa: E402
from store import read_feedback as front_end_reader  # noqa: E402

CASES = [
    None,
    {},
    ["b"],
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


def test_a_copied_feedback_naming_the_wrong_round_is_ignored_by_both_readers(
    tmp_path: Path,
) -> None:
    """Catch a feedback file copied from one round into another.

    A person, or a script, can copy `round-01/feedback.json` into
    `round-03/`. The copy still names round 1. `direct_die.session.Session`
    and `store.SessionStore` must both drop that file, because a mismatch
    means one of the two numbers is wrong and neither is safe to guess
    from. This test fails if either reader accepts the copy.
    """
    asset = "hex-forest"
    session_id = "sess-20260901-1000"
    root = tmp_path / "sessions"
    round_dir = root / asset / session_id / "round-03"
    round_dir.mkdir(parents=True)
    misfiled = {"round": 1, "likes": ["b"], "order": ["b"], "denies": []}
    (round_dir / "feedback.json").write_text(json.dumps(misfiled), encoding="utf-8")

    loop_session = session_module.Session(asset, session_id, root=root)
    assert loop_session.feedback(3) is None

    store = SessionStore(root)
    round_record = store.load_round(asset, session_id, "round-03")
    assert round_record.feedback is None
    assert round_record.winner is None
