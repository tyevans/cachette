"""Tests for the parent that each variant of a round revises.

A person can like more than one drawing. Each liked drawing becomes a parent,
and the round cycles the parents across the variant letters, so each parent
gets a spread of directions rather than one direction.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from direct_die import loop  # noqa: E402

LETTERS = ("a", "b", "c", "d")


def test_no_parent_gives_no_assignment():
    assert loop.assign_parents((), LETTERS) == {}


def test_one_parent_goes_to_every_letter():
    found = loop.assign_parents(("round-00/variant-b",), LETTERS)
    assert found == {letter: "round-00/variant-b" for letter in LETTERS}


def test_two_parents_alternate_across_four_letters():
    found = loop.assign_parents(("p1", "p2"), LETTERS)
    assert found == {"a": "p1", "b": "p2", "c": "p1", "d": "p2"}


def test_three_parents_cycle_across_four_letters():
    found = loop.assign_parents(("p1", "p2", "p3"), LETTERS)
    assert found == {"a": "p1", "b": "p2", "c": "p3", "d": "p1"}


def test_four_parents_and_two_letters_use_the_first_two_parents():
    found = loop.assign_parents(("p1", "p2", "p3", "p4"), ("a", "b"))
    assert found == {"a": "p1", "b": "p2"}


def test_each_parent_gets_more_than_one_lens_when_the_letters_allow_it():
    found = loop.assign_parents(("p1", "p2"), LETTERS)
    lenses_of_p1 = [letter for letter, parent in found.items() if parent == "p1"]
    assert len(lenses_of_p1) == 2
