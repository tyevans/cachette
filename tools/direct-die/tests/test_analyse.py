"""Tests for the analysis of a liked set against a refused set.

The analysis runs between two rounds. It reads what the person liked and what
they refused, and it answers with a preference statement, an order between the
likes, and one proposed rule.

No test here calls the model. The runner takes the model call as an argument.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from direct_die import analyse, client, session  # noqa: E402

SQUARE = (
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">'
    '<rect width="64" height="64" fill="#4a7"/></svg>'
)

GOOD = json.dumps(
    {
        "preference": "The liked drawings hold three shapes and a flat fill.",
        "order": ["d", "b"],
        "reasons": {"d": "the silhouette reads at 64 pixels", "b": ""},
        "guide_edit": {"section": "Shape language", "rule": "Use three shapes or fewer."},
    }
)


# -- the parser --------------------------------------------------------------


def test_the_parser_accepts_a_valid_object():
    found = analyse.parse_analysis(GOOD, ("b", "d"))
    assert found["order"] == ["d", "b"]
    assert found["guide_edit"]["rule"] == "Use three shapes or fewer."
    assert found["reasons"]["d"].startswith("the silhouette")


def test_the_parser_takes_the_object_out_of_prose():
    found = analyse.parse_analysis("Here it is:\n" + GOOD + "\nThat is all.", ("b", "d"))
    assert found["order"] == ["d", "b"]


def test_a_null_guide_edit_is_valid():
    text = json.dumps({"preference": "x", "order": ["b"], "guide_edit": None})
    found = analyse.parse_analysis(text, ("b",))
    assert found["guide_edit"] is None


def test_a_missing_reasons_field_gives_one_empty_reason_for_each_like():
    text = json.dumps({"preference": "x", "order": ["b", "d"]})
    found = analyse.parse_analysis(text, ("b", "d"))
    assert found["reasons"] == {"b": "", "d": ""}


@pytest.mark.parametrize(
    "text",
    [
        "no object here",
        "{not json}",
        json.dumps(["b"]),
        json.dumps({"order": ["b"]}),
        json.dumps({"preference": "  ", "order": ["b"]}),
        json.dumps({"preference": "x"}),
        json.dumps({"preference": "x", "order": ["b", "a"]}),
        json.dumps({"preference": "x", "order": ["b", "b"]}),
        json.dumps({"preference": "x", "order": []}),
        json.dumps({"preference": "x", "order": ["b"], "guide_edit": {"rule": ""}}),
    ],
)
def test_the_parser_refuses_a_malformed_object(text):
    with pytest.raises(ValueError):
        analyse.parse_analysis(text, ("b", "d"))


def test_the_order_must_hold_every_like_and_no_other_letter():
    text = json.dumps({"preference": "x", "order": ["d"]})
    with pytest.raises(ValueError):
        analyse.parse_analysis(text, ("b", "d"))


# -- the prompt --------------------------------------------------------------


def test_the_prompt_asks_for_the_three_things():
    from direct_die import guide

    prompt = analyse.build_analysis_prompt(guide.load("hex-tile"), "a forest", ("b", "d"))
    assert "preference" in prompt
    assert "order" in prompt
    assert "guide_edit" in prompt
    assert '"b"' in prompt and '"d"' in prompt


# -- the runner --------------------------------------------------------------


def _session_with(tmp_path, **fields):
    store = session.Session("hex-tile", "s1", root=tmp_path)
    for letter in ("a", "b", "d"):
        session.write_text(store.round_path(0) / f"variant-{letter}.svg", SQUARE)
    session.write_json(store.round_path(0) / "feedback.json", {"round": 0, **fields})
    return store


def _replier(text):
    seen = {}

    def ask(prompt, images, system=None, temperature=0.4, max_tokens=1200, **rest):
        seen["prompt"] = prompt
        seen["images"] = images
        return client.Reply(text=text, prompt_tokens=10, completion_tokens=5)

    return ask, seen


def test_the_runner_writes_the_analysis_file(tmp_path):
    _session_with(tmp_path, likes=["b", "d"], denies=["a"])
    ask, _ = _replier(GOOD)
    found = analyse.run("hex-tile", "s1", 0, root=tmp_path, ask=ask, log=lambda *_: None)
    written = json.loads(
        (tmp_path / "hex-tile" / "s1" / "round-00" / "analysis.json").read_text()
    )
    assert written["round"] == 0
    assert written["order"] == ["d", "b"]
    assert written["at"]
    assert found["preference"] == written["preference"]


def test_the_runner_shows_the_liked_and_the_refused_drawings(tmp_path):
    _session_with(tmp_path, likes=["b", "d"], denies=["a"])
    ask, seen = _replier(GOOD)
    analyse.run("hex-tile", "s1", 0, root=tmp_path, ask=ask, log=lambda *_: None)
    labels = [image.label for image in seen["images"]]
    accepted = [label for label in labels if "ACCEPTED" in label]
    refused = [label for label in labels if "REFUSED" in label]
    assert len(accepted) == 2
    assert len(refused) == 1
    assert any("variant b" in label for label in accepted)
    assert any("variant a" in label for label in refused)


def test_the_runner_raises_when_nobody_liked_anything(tmp_path):
    _session_with(tmp_path, denies=["a"])
    ask, _ = _replier(GOOD)
    with pytest.raises(analyse.AnalysisError):
        analyse.run("hex-tile", "s1", 0, root=tmp_path, ask=ask, log=lambda *_: None)


def test_the_runner_raises_when_the_round_does_not_exist(tmp_path):
    ask, _ = _replier(GOOD)
    with pytest.raises(analyse.AnalysisError):
        analyse.run("hex-tile", "s1", 4, root=tmp_path, ask=ask, log=lambda *_: None)


def test_the_runner_asks_once_more_when_the_answer_is_malformed(tmp_path):
    _session_with(tmp_path, likes=["b", "d"])
    calls = []

    def ask(prompt, images, **rest):
        calls.append(prompt)
        text = "nonsense" if len(calls) == 1 else GOOD
        return client.Reply(text=text)

    analyse.run("hex-tile", "s1", 0, root=tmp_path, ask=ask, log=lambda *_: None)
    assert len(calls) == 2


def test_the_runner_raises_when_the_answer_stays_malformed(tmp_path):
    _session_with(tmp_path, likes=["b", "d"])

    def ask(prompt, images, **rest):
        return client.Reply(text="nonsense")

    with pytest.raises(analyse.AnalysisError):
        analyse.run("hex-tile", "s1", 0, root=tmp_path, ask=ask, log=lambda *_: None)
