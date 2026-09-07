"""Tests for the generation prompt.

The prompt carries four kinds of direction: the standing note, the note for
one round, the model faults, and the drawings that the art director refused.
The two human notes outrank the model, and the prompt says so.

The refusals go in as SVG source. The generation step sends no picture, and
that is what keeps the prompt inside the model window.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from direct_die import guide, loop, render  # noqa: E402

# The heading of the refusal section. A test checks for this heading, not for
# the bare word "refused", because a style guide may hold that word too.
REFUSAL_HEADING = "DRAWINGS THE ART DIRECTOR REFUSED"

PARENT = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><rect id="parent"/></svg>'
BAD_ONE = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><rect id="badone"/></svg>'
BAD_TWO = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><rect id="badtwo"/></svg>'
BAD_THREE = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"><rect id="badthree"/></svg>'


def _guide():
    return guide.load("hex-tile")


def _sizes():
    return render.sizes_for("hex-tile")


def test_a_refusal_reaches_the_revision_prompt():
    prompt = loop.build_revision_prompt(
        _guide(), "a forest", "be bold", _sizes(), PARENT, [], None,
        denied_sources=[BAD_ONE],
    )
    assert "badone" in prompt
    assert REFUSAL_HEADING in prompt


def test_a_refusal_reaches_the_creation_prompt():
    prompt = loop.build_creation_prompt(
        _guide(), "a forest", "be bold", _sizes(), denied_sources=[BAD_ONE]
    )
    assert "badone" in prompt


def test_the_prompt_takes_the_two_newest_refusals_only():
    prompt = loop.build_revision_prompt(
        _guide(), "a forest", "be bold", _sizes(), PARENT, [], None,
        denied_sources=[BAD_ONE, BAD_TWO, BAD_THREE],
    )
    assert "badone" not in prompt
    assert "badtwo" in prompt
    assert "badthree" in prompt
    assert loop.MAX_REFUSALS == 2


def test_no_refusal_adds_no_refusal_section():
    prompt = loop.build_revision_prompt(
        _guide(), "a forest", "be bold", _sizes(), PARENT, [], None
    )
    assert REFUSAL_HEADING not in prompt


def test_the_standing_note_sits_above_the_model_faults():
    prompt = loop.build_revision_prompt(
        _guide(), "a forest", "be bold", _sizes(), PARENT,
        ["a model fault"], None, standing_note="a standing note",
    )
    assert prompt.index("a standing note") < prompt.index("a model fault")
    assert "outranks" in prompt


def test_the_round_note_sits_below_the_standing_note_and_above_the_model():
    prompt = loop.build_revision_prompt(
        _guide(), "a forest", "be bold", _sizes(), PARENT,
        ["a model fault"], "a round note", standing_note="a standing note",
    )
    assert prompt.index("a standing note") < prompt.index("a round note")
    assert prompt.index("a round note") < prompt.index("a model fault")


def test_the_parent_source_sits_above_the_refused_source():
    prompt = loop.build_revision_prompt(
        _guide(), "a forest", "be bold", _sizes(), PARENT, [], None,
        denied_sources=[BAD_ONE],
    )
    assert prompt.index("parent") < prompt.index("badone")


def test_the_refusal_count_matches_what_the_prompt_holds():
    # meta.json reports this count. It must match the prompt, not the full
    # list of everything the person ever refused.
    taken = loop._taken_refusals([BAD_ONE, BAD_TWO, BAD_THREE])
    assert len(taken) == loop.MAX_REFUSALS == 2
    prompt = loop.build_revision_prompt(
        _guide(), "a forest", "be bold", _sizes(), PARENT, [], None,
        denied_sources=[BAD_ONE, BAD_TWO, BAD_THREE],
    )
    for source in taken:
        assert source.split('id="')[1].split('"')[0] in prompt


def test_the_old_positional_call_still_works():
    prompt = loop.build_revision_prompt(
        _guide(), "a forest", "be bold", _sizes(), PARENT,
        ["a model fault"], "a human note",
    )
    assert prompt.index("a human note") < prompt.index("a model fault")
