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


from direct_die import session  # noqa: E402


def _store(tmp_path):
    return session.Session("hex-tile", "s1", root=tmp_path)


def _feedback(store, index, **fields):
    session.write_json(store.round_path(index) / "feedback.json", {"round": index, **fields})


def _critique(store, index, letter, score, faults=()):
    session.write_json(
        store.round_path(index) / f"variant-{letter}.critique.json",
        {"verdict": "x", "faults": list(faults), "score": score},
    )


def test_the_first_round_has_no_steer(tmp_path):
    found = steer.collect(_store(tmp_path), 0)
    assert found.parents == ()
    assert found.denied == ()
    assert found.note is None


def test_every_like_becomes_a_parent_in_the_order_the_person_gave(tmp_path):
    store = _store(tmp_path)
    _feedback(store, 0, likes=["b", "d"], order=["d", "b"])
    found = steer.collect(store, 1)
    assert found.parents == ("round-00/variant-d", "round-00/variant-b")


def test_the_faults_of_each_parent_come_back_under_its_reference(tmp_path):
    store = _store(tmp_path)
    _critique(store, 0, "b", 50, ["raise the contrast"])
    _critique(store, 0, "d", 40, ["cut a shape"])
    _feedback(store, 0, likes=["b", "d"], order=["b", "d"])
    found = steer.collect(store, 1)
    assert found.faults["round-00/variant-b"] == ["raise the contrast"]
    assert found.faults["round-00/variant-d"] == ["cut a shape"]


def test_a_refusal_in_the_first_round_is_still_refused_two_rounds_later(tmp_path):
    store = _store(tmp_path)
    _feedback(store, 0, likes=["b"], denies=["a"])
    _feedback(store, 1, likes=["c"], denies=["d"])
    found = steer.collect(store, 2)
    assert found.denied == ("round-00/variant-a", "round-01/variant-d")


def test_the_standing_note_comes_from_the_newest_round_that_holds_one(tmp_path):
    store = _store(tmp_path)
    _feedback(store, 0, likes=["a"], note="keep the palette flat")
    _feedback(store, 1, likes=["a"])
    _feedback(store, 2, likes=["a"], note="use fewer shapes")
    found = steer.collect(store, 3)
    assert found.note == "use fewer shapes"


def test_a_round_with_no_note_does_not_clear_the_standing_note(tmp_path):
    store = _store(tmp_path)
    _feedback(store, 0, likes=["a"], note="keep the palette flat")
    _feedback(store, 1, likes=["a"])
    found = steer.collect(store, 2)
    assert found.note == "keep the palette flat"


def test_the_round_note_comes_from_the_round_below_only(tmp_path):
    store = _store(tmp_path)
    _feedback(store, 0, likes=["a"], text="darker base")
    _feedback(store, 1, likes=["a"])
    found = steer.collect(store, 2)
    assert found.text is None


def test_the_best_score_of_every_round_wins_when_nobody_liked_anything(tmp_path):
    store = _store(tmp_path)
    _critique(store, 0, "a", 45)
    _critique(store, 1, "a", 75, ["add depth at the bottom"])
    _critique(store, 2, "a", 45)
    found = steer.collect(store, 3)
    assert found.parents == ("round-01/variant-a",)
    assert found.faults["round-01/variant-a"] == ["add depth at the bottom"]


def test_a_later_round_wins_a_tie_so_the_loop_still_moves(tmp_path):
    store = _store(tmp_path)
    _critique(store, 0, "a", 60)
    _critique(store, 1, "c", 60)
    found = steer.collect(store, 2)
    assert found.parents == ("round-01/variant-c",)


def test_a_refused_drawing_is_never_the_parent_whatever_it_scored(tmp_path):
    store = _store(tmp_path)
    _critique(store, 0, "a", 95)
    _critique(store, 0, "c", 30)
    _feedback(store, 0, denies=["a"])
    found = steer.collect(store, 1)
    assert found.parents == ("round-00/variant-c",)


def test_a_round_below_the_index_is_read_and_a_round_above_it_is_not(tmp_path):
    store = _store(tmp_path)
    _feedback(store, 0, likes=["b"])
    _feedback(store, 2, likes=["d"], denies=["a"])
    found = steer.collect(store, 1)
    assert found.parents == ("round-00/variant-b",)
    assert found.denied == ()
