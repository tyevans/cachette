"""Tests for the plan that one round runs from.

The plan names the parent of each variant letter, the faults of each parent,
and the SVG source of each refused drawing. It reads the whole session, not
the last round only.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from direct_die import loop, session  # noqa: E402

LETTERS = ("a", "b", "c", "d")


def _svg(name):
    return (
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">'
        f'<rect id="{name}" width="64" height="64"/></svg>'
    )


def _store(tmp_path):
    return session.Session("hex-tile", "s1", root=tmp_path)


def _drawing(store, index, letter, score=50, faults=()):
    session.write_text(store.round_path(index) / f"variant-{letter}.svg", _svg(letter))
    session.write_json(
        store.round_path(index) / f"variant-{letter}.critique.json",
        {"verdict": "x", "faults": list(faults), "score": score},
    )


def _feedback(store, index, **fields):
    session.write_json(
        store.round_path(index) / "feedback.json", {"round": index, **fields}
    )


def test_the_first_round_plans_no_parent(tmp_path):
    plan = loop.plan_round(_store(tmp_path), 0, LETTERS)
    assert plan.parents == {}
    assert plan.denied_sources == ()


def test_two_likes_give_two_parents_across_four_variants(tmp_path):
    store = _store(tmp_path)
    _drawing(store, 0, "b", 50)
    _drawing(store, 0, "d", 40)
    _feedback(store, 0, likes=["b", "d"], order=["d", "b"])
    plan = loop.plan_round(store, 1, LETTERS)
    assert plan.parents == {
        "a": "round-00/variant-d",
        "b": "round-00/variant-b",
        "c": "round-00/variant-d",
        "d": "round-00/variant-b",
    }
    assert len(set(plan.parents.values())) == 2


def test_one_like_gives_the_behaviour_of_a_single_choice(tmp_path):
    store = _store(tmp_path)
    _drawing(store, 0, "b", 50, ["raise the contrast"])
    _feedback(store, 0, likes=["b"])
    plan = loop.plan_round(store, 1, LETTERS)
    assert set(plan.parents.values()) == {"round-00/variant-b"}
    assert plan.faults["round-00/variant-b"] == ["raise the contrast"]


def test_the_refused_source_is_in_the_plan(tmp_path):
    store = _store(tmp_path)
    _drawing(store, 0, "a", 90)
    _drawing(store, 0, "b", 50)
    _feedback(store, 0, likes=["b"], denies=["a"])
    plan = loop.plan_round(store, 1, LETTERS)
    assert len(plan.denied_sources) == 1
    assert 'id="a"' in plan.denied_sources[0]


def test_a_refusal_with_no_svg_on_disk_is_dropped(tmp_path):
    store = _store(tmp_path)
    _drawing(store, 0, "b", 50)
    _feedback(store, 0, likes=["b"], denies=["a"])
    plan = loop.plan_round(store, 1, LETTERS)
    assert plan.denied_sources == ()


def test_the_notes_reach_the_plan(tmp_path):
    store = _store(tmp_path)
    _drawing(store, 0, "b", 50)
    _feedback(store, 0, likes=["b"], note="keep it flat", text="darker base")
    plan = loop.plan_round(store, 1, LETTERS)
    assert plan.note == "keep it flat"
    assert plan.text == "darker base"
