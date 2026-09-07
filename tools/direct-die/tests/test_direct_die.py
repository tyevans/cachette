"""Tests for the parts of direct-die that need no endpoint.

Run them with `python3 -m pytest tests` from the tool directory. These
tests touch no part of the engine, and no engine gate runs them.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from direct_die import guide, loop, render, session, steer  # noqa: E402

SQUARE = (
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">'
    '<rect x="8" y="8" width="48" height="48" fill="#336699"/></svg>'
)


def test_extract_svg_takes_the_document_out_of_a_fence():
    text = "Here it is:\n```svg\n" + SQUARE + "\n```\nI hope it helps."
    assert render.extract_svg(text) == SQUARE


def test_extract_svg_refuses_an_answer_with_no_svg():
    with pytest.raises(render.RenderError):
        render.extract_svg("I cannot draw that.")


def test_render_both_gives_two_different_sizes():
    sizes = render.SizeSet(display=16, inspection=128)
    rasters = render.render_both(SQUARE, sizes)
    assert set(rasters) == {"display", "inspection"}
    assert len(rasters["inspection"]) > len(rasters["display"])


def test_render_refuses_broken_source():
    with pytest.raises(render.RenderError):
        render.render("<svg><rect", 32)


def test_parse_critique_accepts_a_valid_object():
    text = 'Here: {"verdict": "good", "faults": ["fix the edge"], "score": 72}'
    assert loop.parse_critique(text) == {
        "verdict": "good",
        "faults": ["fix the edge"],
        "score": 72,
    }


def test_parse_critique_clamps_the_score_into_the_scale():
    high = loop.parse_critique('{"verdict": "x", "faults": [], "score": 480}')
    low = loop.parse_critique('{"verdict": "x", "faults": [], "score": -8}')
    assert high["score"] == 100
    assert low["score"] == 0


@pytest.mark.parametrize(
    "text",
    [
        "no object here",
        '{"verdict": "x", "faults": []}',
        '{"verdict": "", "faults": [], "score": 5}',
        '{"verdict": "x", "faults": "one fault", "score": 5}',
        '{"verdict": "x", "faults": [1, 2], "score": 5}',
    ],
)
def test_parse_critique_refuses_a_malformed_object(text):
    with pytest.raises(ValueError):
        loop.parse_critique(text)


def test_write_json_leaves_no_temporary_file(tmp_path):
    target = tmp_path / "deep" / "value.json"
    session.write_json(target, {"a": 1})
    assert json.loads(target.read_text()) == {"a": 1}
    assert [path.name for path in target.parent.iterdir()] == ["value.json"]


def test_the_header_holds_no_round_count(tmp_path):
    store = session.Session("hex-tile", "s1", root=tmp_path)
    store.write_header("a-model", "abc123")
    header = json.loads((store.path / "session.json").read_text())
    assert "rounds" not in header
    assert header["asset"] == "hex-tile"


def test_existing_rounds_counts_the_directories(tmp_path):
    store = session.Session("hex-tile", "s1", root=tmp_path)
    for index in (0, 1, 2):
        session.write_text(store.round_path(index) / "variant-a.svg", SQUARE)
    assert store.existing_rounds() == [0, 1, 2]


def test_feedback_comes_back_when_the_round_matches(tmp_path):
    store = session.Session("hex-tile", "s1", root=tmp_path)
    session.write_json(
        store.round_path(1) / "feedback.json",
        {"round": 1, "choice": "b", "text": "more contrast", "at": "2026-09-06T00:00:00+00:00"},
    )
    found = store.feedback(1)
    assert found is not None
    assert found["choice"] == "b"


def test_feedback_is_refused_when_the_round_disagrees(tmp_path):
    store = session.Session("hex-tile", "s1", root=tmp_path)
    session.write_json(
        store.round_path(1) / "feedback.json",
        {"round": 7, "choice": "b", "text": "more contrast", "at": "2026-09-06T00:00:00+00:00"},
    )
    assert store.feedback(1) is None


def test_a_human_choice_beats_the_best_score(tmp_path):
    store = session.Session("hex-tile", "s1", root=tmp_path)
    session.write_json(
        store.round_path(0) / "variant-a.critique.json",
        {"verdict": "x", "faults": ["fault a"], "score": 90},
    )
    session.write_json(
        store.round_path(0) / "variant-c.critique.json",
        {"verdict": "x", "faults": ["fault c"], "score": 10},
    )
    session.write_json(
        store.round_path(0) / "feedback.json",
        {
            "round": 0,
            "likes": ["c"],
            "note": "keep the dark palette",
            "at": "2026-09-06T00:00:00+00:00",
        },
    )
    found = steer.collect(store, 1)
    assert found.parents == ("round-00/variant-c",)
    assert found.faults["round-00/variant-c"] == ["fault c"]
    assert found.note == "keep the dark palette"


def test_the_best_score_wins_when_no_feedback_exists(tmp_path):
    store = session.Session("hex-tile", "s1", root=tmp_path)
    session.write_json(
        store.round_path(0) / "variant-a.critique.json",
        {"verdict": "x", "faults": [], "score": 40},
    )
    session.write_json(
        store.round_path(0) / "variant-d.critique.json",
        {"verdict": "x", "faults": [], "score": 65},
    )
    found = steer.collect(store, 1)
    assert found.parents == ("round-00/variant-d",)
    assert found.note is None


def test_the_revision_prompt_puts_the_human_above_the_model():
    loaded = guide.load("hex-tile")
    sizes = render.sizes_for("hex-tile")
    prompt = loop.build_revision_prompt(
        loaded, "a forest", "be bold", sizes, SQUARE, ["a model fault"], "a human note"
    )
    assert prompt.index("a human note") < prompt.index("a model fault")
    assert "outranks" in prompt


def test_the_guide_holds_a_score_scale_and_covers_the_hex_tile():
    loaded = guide.load("hex-tile")
    assert "hex-tile" in guide.asset_types()
    assert "score" not in guide.asset_types()
    assert "0 to 100" in loaded.score_scale
    assert loaded.exemplars, "the guide must ship at least one exemplar"


def test_the_four_variants_use_four_different_temperatures():
    temperatures = [value[1] for value in loop.VARIANT_LENSES.values()]
    assert len(set(temperatures)) == len(temperatures)
