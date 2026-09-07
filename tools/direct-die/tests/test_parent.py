"""Tests for the choice of parent between rounds.

The loop must not walk away from its best drawing. A critique names a
fault even in a good drawing, and a revision that acts on that fault can
make the drawing worse. These tests hold that behaviour.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from direct_die import session, steer  # noqa: E402


def _critique(store, index, letter, score, faults=()):
    session.write_json(
        store.round_path(index) / f"variant-{letter}.critique.json",
        {"verdict": "x", "faults": list(faults), "score": score},
    )


def test_the_parent_is_the_best_of_every_round_not_of_the_last(tmp_path):
    store = session.Session("hex-tile", "s1", root=tmp_path)
    _critique(store, 0, "a", 45)
    _critique(store, 1, "a", 75, ["add depth at the bottom"])
    _critique(store, 2, "a", 45)
    _critique(store, 2, "b", 40)
    found = steer.collect(store, 3)
    assert found.parents == ("round-01/variant-a",)
    assert found.faults["round-01/variant-a"] == ["add depth at the bottom"]


def test_a_later_round_wins_a_tie_so_the_loop_still_moves(tmp_path):
    store = session.Session("hex-tile", "s1", root=tmp_path)
    _critique(store, 0, "a", 60)
    _critique(store, 1, "c", 60)
    found = steer.collect(store, 2)
    assert found.parents == ("round-01/variant-c",)


def test_a_human_choice_wins_over_a_higher_score_in_an_earlier_round(tmp_path):
    store = session.Session("hex-tile", "s1", root=tmp_path)
    _critique(store, 0, "a", 90, ["an old fault"])
    _critique(store, 1, "b", 20, ["a chosen fault"])
    session.write_json(
        store.round_path(1) / "feedback.json",
        {
            "round": 1,
            "likes": ["b"],
            "text": "I like the dark palette",
            "at": "2026-09-06T00:00:00+00:00",
        },
    )
    found = steer.collect(store, 2)
    assert found.parents == ("round-01/variant-b",)
    assert found.faults["round-01/variant-b"] == ["a chosen fault"]
    assert found.text == "I like the dark palette"


def test_feedback_text_alone_keeps_the_best_parent(tmp_path):
    store = session.Session("hex-tile", "s1", root=tmp_path)
    _critique(store, 0, "a", 75)
    _critique(store, 1, "a", 30)
    session.write_json(
        store.round_path(1) / "feedback.json",
        {
            "round": 1,
            "likes": [],
            "text": "make it darker",
            "at": "2026-09-06T00:00:00+00:00",
        },
    )
    found = steer.collect(store, 2)
    assert found.parents == ("round-00/variant-a",)
    assert found.text == "make it darker"


def test_the_first_round_has_no_parent(tmp_path):
    store = session.Session("hex-tile", "s1", root=tmp_path)
    found = steer.collect(store, 0)
    assert found.parents == ()
    assert found.faults == {}
    assert found.denied == ()
    assert found.note is None
    assert found.text is None
