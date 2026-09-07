"""Tests for the feedback shape that the front end writes.

The reader is total. A malformed field becomes an empty field, because the
generation loop must keep running when a person edits the file by hand.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from direct_die import steer  # noqa: E402


def test_a_missing_file_gives_the_empty_record():
    assert steer.read_feedback(None) == steer.EMPTY


def test_a_value_that_is_not_an_object_gives_the_empty_record():
    assert steer.read_feedback(["b"]) == steer.EMPTY


def test_the_three_lists_come_back_in_order():
    found = steer.read_feedback(
        {"likes": ["b", "d"], "denies": ["a"], "order": ["d", "b"]}
    )
    assert found.likes == ("b", "d")
    assert found.denies == ("a",)
    assert found.order == ("d", "b")


def test_an_old_choice_reads_as_one_like_and_one_order():
    found = steer.read_feedback({"choice": "b", "text": "more contrast"})
    assert found.likes == ("b",)
    assert found.order == ("b",)
    assert found.denies == ()
    assert found.text == "more contrast"


def test_an_old_choice_of_none_reads_as_no_like():
    found = steer.read_feedback({"choice": None, "text": ""})
    assert found.likes == ()
    assert found.order == ()
    assert found.text is None


def test_likes_win_over_an_old_choice_when_both_are_present():
    found = steer.read_feedback({"choice": "a", "likes": ["c"]})
    assert found.likes == ("c",)


def test_an_order_that_is_missing_takes_the_like_order():
    found = steer.read_feedback({"likes": ["d", "b"]})
    assert found.order == ("d", "b")


def test_an_order_entry_that_no_like_holds_is_dropped():
    found = steer.read_feedback({"likes": ["b"], "order": ["d", "b"]})
    assert found.order == ("b",)


def test_a_like_that_the_order_leaves_out_goes_on_the_end():
    found = steer.read_feedback({"likes": ["a", "b", "c"], "order": ["c"]})
    assert found.order == ("c", "a", "b")


def test_a_letter_in_both_lists_is_a_refusal():
    found = steer.read_feedback({"likes": ["b"], "denies": ["b"]})
    assert found.likes == ()
    assert found.denies == ("b",)
    assert found.order == ()


def test_a_letter_that_is_not_a_variant_is_dropped():
    found = steer.read_feedback({"likes": ["b", "z", 3], "denies": ["q"]})
    assert found.likes == ("b",)
    assert found.denies == ()


def test_a_repeated_letter_appears_once():
    found = steer.read_feedback({"likes": ["b", "b", "d"]})
    assert found.likes == ("b", "d")


def test_blank_text_reads_as_no_text():
    found = steer.read_feedback({"note": "   ", "text": "\n"})
    assert found.note is None
    assert found.text is None


def test_text_is_stripped():
    found = steer.read_feedback({"note": "  keep it flat  "})
    assert found.note == "keep it flat"


def test_a_field_of_the_wrong_type_is_dropped():
    found = steer.read_feedback({"likes": "bd", "note": 7, "denies": {"a": 1}})
    assert found.likes == ()
    assert found.denies == ()
    assert found.note is None
