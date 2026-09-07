"""Tests for the four styles and for the set driver.

These tests need no endpoint. Run them with `python3 -m pytest tests`
from the tool directory. They touch no part of the engine, and no engine
gate runs them.
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from direct_die import guide, render, setrun, subjects  # noqa: E402

STYLES = ["cartoon", "pencil", "elegant", "sandcastle"]


@pytest.mark.parametrize("style", STYLES)
def test_each_style_loads_as_an_asset_type(style):
    loaded = guide.load(style)
    assert loaded.asset == style
    assert loaded.rules.strip()
    assert loaded.score_scale.strip()


@pytest.mark.parametrize("style", STYLES)
def test_each_style_takes_the_default_size_set(style):
    sizes = render.sizes_for(style)
    assert sizes.display == 64
    assert sizes.inspection == 384


@pytest.mark.parametrize("style", STYLES)
def test_each_style_names_every_subject_of_the_set(style):
    rules = guide.load(style).rules
    missing = [name for name in subjects.SUBJECT_ORDER if name not in rules]
    assert not missing, f"{style} names no rule for: {missing}"


@pytest.mark.parametrize("style", STYLES)
def test_each_style_states_what_it_forbids(style):
    rules = guide.load(style).rules
    assert "forbids" in rules.lower()


@pytest.mark.parametrize("style", STYLES)
def test_no_style_restates_the_score_scale(style):
    """The score scale has one declaration site, and it is `score.md`."""
    rules = guide.load(style).rules
    assert "0 to 100" not in rules
    assert "90 to 100" not in rules


def test_the_four_styles_give_four_different_guide_versions():
    versions = {style: guide.load(style).version for style in STYLES}
    assert len(set(versions.values())) == len(STYLES)


def test_the_subject_set_holds_five_terrains_and_six_upgrades():
    assert len(subjects.TERRAIN) == 5
    assert len(subjects.UPGRADE) == 6
    assert "open" not in subjects.SUBJECTS


def test_resolve_gives_the_whole_set_by_default():
    assert subjects.resolve(None) == subjects.SUBJECT_ORDER
    assert subjects.resolve([]) == subjects.SUBJECT_ORDER


def test_resolve_refuses_a_subject_that_the_set_does_not_hold():
    with pytest.raises(KeyError):
        subjects.resolve(["desert"])


def test_best_of_takes_the_highest_score_of_every_round():
    from direct_die import loop

    first = loop.RoundResult(index=0)
    first.variants = [
        loop.VariantResult(letter="a", critique={"score": 71}),
        loop.VariantResult(letter="b", critique={"score": 55}),
    ]
    second = loop.RoundResult(index=1)
    second.variants = [loop.VariantResult(letter="a", critique={"score": 60})]
    score, where = setrun.best_of([first, second])
    assert score == 71
    assert where == "round-00/variant-a"


def test_best_of_lets_a_later_round_win_a_tie():
    from direct_die import loop

    first = loop.RoundResult(index=0)
    first.variants = [loop.VariantResult(letter="a", critique={"score": 70})]
    second = loop.RoundResult(index=1)
    second.variants = [loop.VariantResult(letter="c", critique={"score": 70})]
    score, where = setrun.best_of([first, second])
    assert score == 70
    assert where == "round-01/variant-c"


def test_best_of_gives_nothing_when_no_variant_scored():
    from direct_die import loop

    empty = loop.RoundResult(index=0)
    empty.variants = [loop.VariantResult(letter="a", error="draw: failed")]
    assert setrun.best_of([empty]) == (None, None)


def test_the_table_names_every_subject_that_ran():
    results = [
        setrun.SubjectResult(name="forest", session_id="x", best_score=64,
                             best_reference="round-00/variant-b"),
        setrun.SubjectResult(name="water", session_id="y", error="no answer"),
    ]
    text = setrun.format_table("cartoon", results)
    assert "cartoon" in text
    assert "forest" in text
    assert "water" in text
    assert "64" in text
