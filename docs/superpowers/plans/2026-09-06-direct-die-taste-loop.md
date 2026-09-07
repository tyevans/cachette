# Direct-die Taste Loop Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let a person mark the drawings they like and the drawings they refuse in the direct-die front end, branch the next round from every like, warn the model away from every refusal, and read a model analysis that says what the two groups differ by.

**Architecture:** A new pure module `direct_die/steer.py` owns the feedback shape and turns a session directory into one `Steer` record. `loop.py` reads that record, assigns a parent to each variant letter, and puts refused SVG source into the prompt. A new `direct_die/analyse.py` holds the analysis prompt, its parser and its writer, and a new `analyse` command drives it. On the front end, `review/store.py` gains the same feedback shape behind its own reader, `review/packs.py` gains the new winner rule, and the round page gains like and refusal controls plus an analysis panel.

**Tech Stack:** Python 3.12, FastAPI, Jinja2, pytest, ruff. No client framework and no build step.

**Spec:** `docs/superpowers/specs/2026-09-06-direct-die-taste-loop-design.md`

## Global Constraints

- **All prose follows Simplified Technical English (ASD-STE100).** Short sentences, active voice, one idea per sentence, no metaphor. This covers docstrings, comments, READMEs and commit messages.
- **A document references external material in a footnote only.** Every footnote definition goes in one `## References` section at the end.
- **The tool is outside every engine gate.** It changes nothing under `crates/` and nothing under `python/cachette/`. Its own gate is pytest and ruff inside `tools/direct-die`.
- **The front end never imports the tool package at run time.** It calls the tool through its command line. One test imports `direct_die.steer` to compare two readers, and that is the only import, and it is a test.
- **Every write is atomic.** Write a temporary file in the same directory, then rename.
- **The variant letters are `a`, `b`, `c`, `d`.** `session.VARIANT_LETTERS` and `store.VARIANT_LETTERS` both declare them today. Do not add a third declaration.
- **Two declaration sites need a check.** The feedback shape has a reader in the tool and a reader in the front end. Task 10 adds the test that fails when they disagree. Do not skip it.
- **No test asserts on wall clock time.**
- **A test count in a step is approximate.** The gate is that every test in the named file passes, not that the number matches.
- **`create_app` binds the style guide directory as `styleguide_root`.** Use that name in a new route.
- **Commit after each task.** End every commit message with:
  ```
  Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_015NrPj9XQj2rz1SftMrjeF6
  ```

## How to run the tests

The tool has two test trees. Run them from their own directories.

```bash
cd /home/ty/workspace/cachette/tools/direct-die && python -m pytest tests -q
cd /home/ty/workspace/cachette/tools/direct-die/review && python -m pytest -q
```

`tests/test_styles.py` loads and rasterises every style guide, so it is slow. Run one file while you work, and the whole tree before you commit.

## Where the pieces go

| Path | Responsibility | Task |
|------|----------------|------|
| `direct_die/steer.py` | New. The feedback shape, the old-field rule, and the walk over a session. | 1, 2 |
| `direct_die/loop.py` | Parent assignment, the refusal section of the prompt, the per-variant metadata. | 3, 4, 5 |
| `direct_die/analyse.py` | New. The analysis prompt, the parser, and the writer. | 6 |
| `direct_die/cli.py` | The `analyse` command. | 7 |
| `review/store.py` | The new feedback shape, the writer, and the analysis reader. | 8 |
| `review/packs.py` | Which drawing stands for an asset. | 9 |
| `review/rules.py` | New. Append one accepted rule to a style rules file. | 11 |
| `review/app.py` | The feedback form, the analysis start, and the rule accept. | 12, 13 |
| `review/templates/round.html` | The like and refusal controls, and the analysis panel. | 12, 13 |
| `review/make_fixtures.py` | Fixtures for the new shape. | 8 |
| `README.md`, `review/README.md` | The new contracts. | 14 |

---

### Task 1: The feedback shape

**Files:**
- Create: `tools/direct-die/direct_die/steer.py`
- Create: `tools/direct-die/tests/test_steer.py`

**Interfaces:**
- Consumes: `direct_die.session.VARIANT_LETTERS`, a tuple of `("a", "b", "c", "d")`.
- Produces:
  - `Feedback` frozen dataclass with fields `likes: tuple[str, ...]`, `denies: tuple[str, ...]`, `order: tuple[str, ...]`, `note: str | None`, `text: str | None`.
  - `read_feedback(value: dict | None) -> Feedback`.
  - `EMPTY: Feedback`, the record with everything empty.

The reader is total. It never raises. A malformed field becomes an empty field, because the loop must keep running when a person writes a file by hand. The writer on the front end is the part that refuses bad input, and Task 8 builds it.

- [ ] **Step 1: Write the failing test**

Create `tools/direct-die/tests/test_steer.py`:

```python
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
```

- [ ] **Step 2: Run the test and see it fail**

```bash
cd /home/ty/workspace/cachette/tools/direct-die && python -m pytest tests/test_steer.py -q
```

Expected: a collection error, `ModuleNotFoundError: No module named 'direct_die.steer'`.

- [ ] **Step 3: Write the module**

Create `tools/direct-die/direct_die/steer.py`:

```python
"""The steer that a person gives to the loop.

The front end writes one file into a round directory: `feedback.json`. The
generation loop is its only reader. This module holds the shape of that file
and the walk over a session that turns it into one record.

The reader is total. It never raises. A malformed field becomes an empty
field, because the loop must keep running when a person edits the file by
hand. The front end refuses bad input at the moment of the write.

## The old field

The file held a `choice` field, which named one variant. Nothing writes that
field now. A file that holds `choice` and no `likes` reads as one like and a
one-entry order, so every session already on disk still opens. This rule
lives in one function.
"""

from __future__ import annotations

from dataclasses import dataclass, field

from .session import VARIANT_LETTERS


@dataclass(frozen=True)
class Feedback:
    """What a person said about one round."""

    likes: tuple[str, ...] = ()
    denies: tuple[str, ...] = ()
    order: tuple[str, ...] = ()
    note: str | None = None
    text: str | None = None


# The record of a round that holds no feedback.
EMPTY = Feedback()


def _letters(value: object) -> tuple[str, ...]:
    """Take the variant letters out of a value, once each, in order."""
    if not isinstance(value, list):
        return ()
    found: list[str] = []
    for item in value:
        if isinstance(item, str) and item in VARIANT_LETTERS and item not in found:
            found.append(item)
    return tuple(found)


def _text(value: object) -> str | None:
    """Take a text field, or give None when it holds nothing."""
    if not isinstance(value, str):
        return None
    stripped = value.strip()
    return stripped or None


def read_feedback(value: dict | None) -> Feedback:
    """Read one feedback object into a record.

    The function never raises. It drops what it cannot read.
    """
    if not isinstance(value, dict):
        return EMPTY

    denies = _letters(value.get("denies"))

    if "likes" in value:
        likes = _letters(value.get("likes"))
    else:
        choice = value.get("choice")
        likes = (choice,) if isinstance(choice, str) and choice in VARIANT_LETTERS else ()

    # A refusal beats a like. The two lists must not hold the same letter, and
    # the front end refuses that write, but a hand-edited file can hold it.
    likes = tuple(letter for letter in likes if letter not in denies)

    ranked = [letter for letter in _letters(value.get("order")) if letter in likes]
    ranked.extend(letter for letter in likes if letter not in ranked)

    return Feedback(
        likes=likes,
        denies=denies,
        order=tuple(ranked),
        note=_text(value.get("note")),
        text=_text(value.get("text")),
    )
```

- [ ] **Step 4: Run the test and see it pass**

```bash
cd /home/ty/workspace/cachette/tools/direct-die && python -m pytest tests/test_steer.py -q
```

Expected: 14 passed.

- [ ] **Step 5: Commit**

```bash
cd /home/ty/workspace/cachette
git add tools/direct-die/direct_die/steer.py tools/direct-die/tests/test_steer.py
git commit -F - <<'EOF'
Add the feedback shape that carries a like and a refusal

The front end writes one file into a round directory. It held one chosen
letter and free text. It now holds a like list, a refusal list, an order
between the likes, a standing note and a note for one round.

The reader is total and never raises, because the generation loop must keep
running when a person edits the file by hand. A file that holds the old
`choice` field and no `likes` reads as one like, so every session already on
disk still opens.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_015NrPj9XQj2rz1SftMrjeF6
EOF
```

---

### Task 2: Walk a session and collect the steer

**Files:**
- Modify: `tools/direct-die/direct_die/steer.py` (append)
- Modify: `tools/direct-die/tests/test_steer.py` (append)

**Interfaces:**
- Consumes: `Feedback`, `read_feedback` from Task 1. `direct_die.session.Session` with methods `existing_rounds() -> list[int]`, `feedback(index) -> dict | None`, `critique(index, letter) -> dict | None`, and the module function `round_name(index) -> str`.
- Produces:
  - `Steer` frozen dataclass with fields `parents: tuple[str, ...]`, `faults: dict[str, list[str]]`, `denied: tuple[str, ...]`, `note: str | None`, `text: str | None`.
  - `reference(index: int, letter: str) -> str`, which gives `"round-01/variant-b"`.
  - `collect(store, index: int) -> Steer`.

The rules this function holds:

1. A refusal accumulates over the whole session. A variant refused in round 1 is still refused in round 4. `denied` holds every refused reference, oldest first.
2. A like does not accumulate. The likes of the round directly below `index` decide the parents, best first, from the `order` field.
3. The standing note is the `note` of the newest round that holds one.
4. `text` is the note of the round directly below `index` only.
5. When no like exists, the parent is the highest scoring variant of every round below `index`, and a later round wins a tie. This is the rule today. A refused reference can never be the parent.

- [ ] **Step 1: Write the failing test**

Append to `tools/direct-die/tests/test_steer.py`:

```python
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
```

- [ ] **Step 2: Run the test and see it fail**

```bash
cd /home/ty/workspace/cachette/tools/direct-die && python -m pytest tests/test_steer.py -q
```

Expected: FAIL with `AttributeError: module 'direct_die.steer' has no attribute 'collect'`.

- [ ] **Step 3: Write the code**

Append to `tools/direct-die/direct_die/steer.py`:

```python
@dataclass(frozen=True)
class Steer:
    """What every round below one index says about the next round."""

    parents: tuple[str, ...] = ()
    faults: dict[str, list[str]] = field(default_factory=dict)
    denied: tuple[str, ...] = ()
    note: str | None = None
    text: str | None = None


def reference(index: int, letter: str) -> str:
    """Name one variant of one round, as the round metadata names it."""
    return f"{session_module.round_name(index)}/variant-{letter}"


def split_reference(value: str) -> tuple[int, str]:
    """Take the round index and the variant letter out of a reference.

    Raise ValueError when the value is not a reference.
    """
    head, _, tail = value.partition("/")
    if not head.startswith("round-") or not tail.startswith("variant-"):
        raise ValueError(f"not a variant reference: {value!r}")
    return int(head[len("round-") :]), tail[len("variant-") :]


def collect(store, index: int) -> Steer:
    """Read every round below one index, and give the steer for that index.

    A refusal holds for the whole session. A like holds for the next round
    only, so the round directly below the index decides the parents. The
    standing note is the newest note that any round holds.
    """
    if index <= 0:
        return Steer()

    below = [number for number in store.existing_rounds() if number < index]
    previous = index - 1

    denied: list[str] = []
    note: str | None = None
    text: str | None = None
    parents: tuple[str, ...] = ()

    for number in below:
        found = read_feedback(store.feedback(number))
        denied.extend(reference(number, letter) for letter in found.denies)
        if found.note is not None:
            note = found.note
        if number == previous:
            text = found.text
            parents = tuple(reference(number, letter) for letter in found.order)

    if not parents:
        best_key: tuple[int, int, str] | None = None
        best: str | None = None
        for number in below:
            for letter in VARIANT_LETTERS:
                critique = store.critique(number, letter)
                if not isinstance(critique, dict):
                    continue
                score = critique.get("score")
                if isinstance(score, bool) or not isinstance(score, int):
                    continue
                candidate = reference(number, letter)
                if candidate in denied:
                    continue
                key = (score, number, letter)
                if best_key is None or key >= best_key:
                    best_key = key
                    best = candidate
        parents = (best,) if best else ()

    faults: dict[str, list[str]] = {}
    for candidate in parents:
        number, letter = split_reference(candidate)
        critique = store.critique(number, letter) or {}
        listed = critique.get("faults")
        faults[candidate] = (
            [item for item in listed if isinstance(item, str)]
            if isinstance(listed, list)
            else []
        )

    return Steer(
        parents=parents,
        faults=faults,
        denied=tuple(denied),
        note=note,
        text=text,
    )
```

Change the import block at the top of the file from `from .session import VARIANT_LETTERS` to:

```python
from . import session as session_module
from .session import VARIANT_LETTERS
```

- [ ] **Step 4: Run the test and see it pass**

```bash
cd /home/ty/workspace/cachette/tools/direct-die && python -m pytest tests/test_steer.py -q
```

Expected: 25 passed.

- [ ] **Step 5: Commit**

```bash
cd /home/ty/workspace/cachette
git add tools/direct-die/direct_die/steer.py tools/direct-die/tests/test_steer.py
git commit -F - <<'EOF'
Collect the steer of a whole session, not of the last round

A refusal holds for the whole session, so a drawing refused in round 1 is
still refused in round 4. A like holds for the next round only. The standing
note is the newest note that any round holds.

The fallback rule does not change: when nobody liked anything, the parent is
the highest scoring variant of every round so far, and a later round wins a
tie. A refused drawing is now never the parent, whatever it scored.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_015NrPj9XQj2rz1SftMrjeF6
EOF
```

---

### Task 3: Assign one parent to each variant letter

**Files:**
- Modify: `tools/direct-die/direct_die/loop.py`
- Create: `tools/direct-die/tests/test_assign.py`

**Interfaces:**
- Consumes: `steer.Steer`, `steer.collect` from Task 2.
- Produces: `loop.assign_parents(parents: Sequence[str], letters: Sequence[str]) -> dict[str, str]`.

The function cycles the parents across the letters, so each parent gets a spread of lenses rather than one lens. Two parents and four letters give `a` and `c` to the first parent, and `b` and `d` to the second.

- [ ] **Step 1: Write the failing test**

Create `tools/direct-die/tests/test_assign.py`:

```python
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
```

- [ ] **Step 2: Run the test and see it fail**

```bash
cd /home/ty/workspace/cachette/tools/direct-die && python -m pytest tests/test_assign.py -q
```

Expected: FAIL with `AttributeError: module 'direct_die.loop' has no attribute 'assign_parents'`.

- [ ] **Step 3: Write the code**

Add to `tools/direct-die/direct_die/loop.py`, directly above `def choose_parent`:

```python
def assign_parents(
    parents: Sequence[str], letters: Sequence[str]
) -> dict[str, str]:
    """Give each variant letter the parent that it revises.

    A person can like more than one drawing. The round cycles the liked
    parents across the variant letters, so each parent gets a spread of
    directions rather than one direction. One parent goes to every letter,
    which is the behaviour of a single choice.
    """
    if not parents:
        return {}
    return {
        letter: parents[position % len(parents)]
        for position, letter in enumerate(letters)
    }
```

Add `from collections.abc import Sequence` to the imports at the top of `loop.py`.

- [ ] **Step 4: Run the test and see it pass**

```bash
cd /home/ty/workspace/cachette/tools/direct-die && python -m pytest tests/test_assign.py -q
```

Expected: 6 passed.

- [ ] **Step 5: Commit**

```bash
cd /home/ty/workspace/cachette
git add tools/direct-die/direct_die/loop.py tools/direct-die/tests/test_assign.py
git commit -F - <<'EOF'
Cycle the liked parents across the variant letters

A person can like more than one drawing of a round. Each liked drawing
becomes a parent of the next round. The round cycles the parents across the
four variant letters, so each parent gets a spread of directions rather than
one direction.

One parent goes to every letter, which is the behaviour of a single choice.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_015NrPj9XQj2rz1SftMrjeF6
EOF
```

---

### Task 4: Put refused drawings into the prompt

**Files:**
- Modify: `tools/direct-die/direct_die/loop.py`
- Create: `tools/direct-die/tests/test_prompt.py`

**Interfaces:**
- Consumes: `guide_module.Guide`, `render.SizeSet`, both already in `loop.py`.
- Produces:
  - `loop.MAX_REFUSALS`, an integer constant with the value `2`.
  - `loop.build_creation_prompt(the_guide, subject, lens, sizes, denied_sources: Sequence[str] = (), standing_note: str | None = None) -> str`
  - `loop.build_revision_prompt(the_guide, subject, lens, sizes, parent_svg, faults, human_text, standing_note: str | None = None, denied_sources: Sequence[str] = ()) -> str`

The existing positional arguments of both functions do not move, so the test in `tests/test_direct_die.py` that calls `build_revision_prompt` with seven positional arguments keeps working.

The order inside the prompt, top to bottom: the rules, the parent source, the refused sources, the standing note, the round note, the model faults, the lens. The standing note and the round note both outrank the model faults, and the prompt says so.

`denied_sources` holds SVG source text, newest last. The builder takes the last `MAX_REFUSALS` entries, because an SVG document runs about 800 tokens and the model window is 16384.

- [ ] **Step 1: Write the failing test**

Create `tools/direct-die/tests/test_prompt.py`:

```python
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
    assert "refused" in prompt.lower()


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
    assert "refused" not in prompt.lower()


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


def test_the_old_positional_call_still_works():
    prompt = loop.build_revision_prompt(
        _guide(), "a forest", "be bold", _sizes(), PARENT,
        ["a model fault"], "a human note",
    )
    assert prompt.index("a human note") < prompt.index("a model fault")
```

- [ ] **Step 2: Run the test and see it fail**

```bash
cd /home/ty/workspace/cachette/tools/direct-die && python -m pytest tests/test_prompt.py -q
```

Expected: several failures with `TypeError: build_revision_prompt() got an unexpected keyword argument 'denied_sources'`.

- [ ] **Step 3: Write the code**

In `tools/direct-die/direct_die/loop.py`, add the constant below `VARIANT_LENSES`:

```python
# How many refused drawings go into one prompt. An SVG document of this tool
# runs about 800 tokens, and the model window is 16384. The newest refusals
# win, because they answer the drawing that the person just saw.
MAX_REFUSALS = 2
```

Add this helper directly above `build_creation_prompt`:

```python
def _refusal_block(denied_sources: Sequence[str]) -> str:
    """Give the prompt section that shows the drawings a person refused.

    Give the empty string when nobody refused anything. The section holds SVG
    source, because the generation step sends no picture, and that is what
    keeps the prompt inside the model window.
    """
    taken = [item for item in denied_sources if item and item.strip()][-MAX_REFUSALS:]
    if not taken:
        return ""
    bodies = "\n-----\n".join(item.strip() for item in taken)
    return (
        "\nDRAWINGS THE ART DIRECTOR REFUSED. Do not draw like these. Do not "
        "reuse their shapes or their palette.\n"
        "-----\n"
        f"{bodies}\n"
        "-----\n"
    )
```

Replace `build_creation_prompt` with:

```python
def build_creation_prompt(
    the_guide: guide_module.Guide,
    subject: str,
    lens: str,
    sizes: render.SizeSet,
    denied_sources: Sequence[str] = (),
    standing_note: str | None = None,
) -> str:
    """Build the prompt that makes the first drawing of an asset."""
    standing = ""
    if standing_note:
        standing = (
            "\nDIRECTION FROM THE HUMAN ART DIRECTOR. This outranks every "
            "other note. Do what it says first.\n"
            f"{standing_note.strip()}\n"
        )
    return (
        f"Draw one {the_guide.asset} for a strategy game world map.\n"
        f"The subject is: {subject}\n\n"
        "The style guide follows. Obey every rule in it.\n"
        "-----\n"
        f"{the_guide.rules}\n"
        "-----\n"
        + _refusal_block(denied_sources)
        + standing
        + f"\nYour direction for this drawing: {lens}\n\n"
        f"The map draws this asset at {sizes.display} pixels. A person "
        f"inspects it at {sizes.inspection} pixels. It must read at the "
        "smaller size.\n\n"
        "Answer with the SVG document only."
    )
```

Replace the body of `build_revision_prompt` with this, and give it the two new keyword arguments:

```python
def build_revision_prompt(
    the_guide: guide_module.Guide,
    subject: str,
    lens: str,
    sizes: render.SizeSet,
    parent_svg: str,
    faults: list[str],
    human_text: str | None,
    standing_note: str | None = None,
    denied_sources: Sequence[str] = (),
) -> str:
    """Build the prompt that revises a drawing.

    The two human notes go above the model critique, and the prompt says that
    the human outranks the model. The standing note goes above the note for
    this round, because it holds for the whole session.
    """
    direction = []
    if standing_note:
        direction.append(
            "STANDING DIRECTION FROM THE HUMAN ART DIRECTOR. It holds for "
            "the whole session, and it outranks every other note below it.\n"
            f"{standing_note.strip()}"
        )
    if human_text:
        direction.append(
            "DIRECTION FROM THE HUMAN ART DIRECTOR FOR THIS ROUND. This "
            "outranks every model note below it. Do what it says first.\n"
            f"{human_text.strip()}"
        )
    if faults:
        listed = "\n".join(f"- {item}" for item in faults)
        direction.append(
            "Notes from the model critique of this same drawing. They "
            "rank below the human direction.\n" + listed
        )
    if not direction:
        direction.append(
            "No critique came back. Improve the weakest part of the "
            "drawing at the display size."
        )

    return (
        f"Revise one {the_guide.asset} for a strategy game world map.\n"
        f"The subject is: {subject}\n\n"
        "The style guide follows. Obey every rule in it.\n"
        "-----\n"
        f"{the_guide.rules}\n"
        "-----\n\n"
        "This is the current SVG source.\n"
        "-----\n"
        f"{parent_svg.strip()}\n"
        "-----\n"
        + _refusal_block(denied_sources)
        + "\n"
        + "\n\n".join(direction)
        + f"\n\nYour direction for this variant: {lens}\n\n"
        f"The map draws this asset at {sizes.display} pixels. It must "
        "read at that size.\n\n"
        "Answer with the revised SVG document only. Keep what already "
        "works. Change what the notes name."
    )
```

The word `outranks` stays in the revision prompt, so the existing test in `tests/test_direct_die.py` keeps passing.

- [ ] **Step 4: Run the tests and see them pass**

```bash
cd /home/ty/workspace/cachette/tools/direct-die && python -m pytest tests/test_prompt.py tests/test_direct_die.py -q
```

Expected: 8 passed in `test_prompt.py`, and no regression in `test_direct_die.py`.

- [ ] **Step 5: Prove the test can fail**

Put the defect back. Edit `_refusal_block` so its first line is `return ""`. Then run:

```bash
cd /home/ty/workspace/cachette/tools/direct-die && python -m pytest tests/test_prompt.py -q
```

Expected: FAIL. At least `test_a_refusal_reaches_the_revision_prompt`, `test_a_refusal_reaches_the_creation_prompt` and `test_the_prompt_takes_the_two_newest_refusals_only` must fail. If they pass, the fixture does not reach the code, and the test is decoration. Fix the test before you go on.

Undo the edit and run again to confirm 8 passed.

- [ ] **Step 6: Commit**

```bash
cd /home/ty/workspace/cachette
git add tools/direct-die/direct_die/loop.py tools/direct-die/tests/test_prompt.py
git commit -F - <<'EOF'
Put the refused drawings into the generation prompt as source

A person who refuses a drawing now reaches the model. The refusal goes in as
SVG source, under a heading that says not to draw like it. The model writes
SVG, so it can act on SVG.

The generation step still sends no picture, which is what keeps the prompt
inside the 16384 token window. An SVG document of this tool runs about 800
tokens, so the prompt takes the two newest refusals only.

The prompt gains a standing note above the note for one round. Both outrank
the model faults, and the prompt says so.

Proved the tests can fail by making the refusal block return the empty
string. Three tests then failed.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_015NrPj9XQj2rz1SftMrjeF6
EOF
```

---

### Task 5: Run a round from the steer

**Files:**
- Modify: `tools/direct-die/direct_die/loop.py`
- Modify: `tools/direct-die/tests/test_parent.py`
- Modify: `tools/direct-die/tests/test_direct_die.py`
- Create: `tools/direct-die/tests/test_round_plan.py`

**Interfaces:**
- Consumes: `steer.collect`, `steer.split_reference`, `loop.assign_parents`, the two prompt builders.
- Produces:
  - `loop.RoundPlan` frozen dataclass with fields `parents: dict[str, str]`, `faults: dict[str, list[str]]`, `denied_sources: tuple[str, ...]`, `note: str | None`, `text: str | None`.
  - `loop.plan_round(store, index: int, letters: Sequence[str]) -> RoundPlan`.
  - `run_round` writes `parents` into `meta.json` and no longer writes `parent`.

`loop.choose_parent` goes away. Its two callers are `run_round` and the tests in `tests/test_parent.py` and `tests/test_direct_die.py`. Task 2 moved its rules into `steer.collect`, and those tests move with them.

- [ ] **Step 1: Write the failing test**

Create `tools/direct-die/tests/test_round_plan.py`:

```python
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
```

- [ ] **Step 2: Run the test and see it fail**

```bash
cd /home/ty/workspace/cachette/tools/direct-die && python -m pytest tests/test_round_plan.py -q
```

Expected: FAIL with `AttributeError: module 'direct_die.loop' has no attribute 'plan_round'`.

- [ ] **Step 3: Write the code**

In `tools/direct-die/direct_die/loop.py`, add `from . import steer as steer_module` to the imports.

Delete the whole `choose_parent` function. Put this in its place:

```python
@dataclass(frozen=True)
class RoundPlan:
    """What one round revises, refuses, and reads as direction."""

    parents: dict[str, str] = field(default_factory=dict)
    faults: dict[str, list[str]] = field(default_factory=dict)
    denied_sources: tuple[str, ...] = ()
    note: str | None = None
    text: str | None = None


def plan_round(
    store: session.Session, index: int, letters: Sequence[str]
) -> RoundPlan:
    """Read the session and give the plan of one round.

    The plan names the parent of each variant letter, the faults of each
    parent, and the SVG source of each drawing that the person refused.

    A refusal whose SVG is not on disk drops out. The picture is what the
    prompt shows, and there is nothing to show.
    """
    found = steer_module.collect(store, index)
    sources: list[str] = []
    for reference in found.denied:
        number, letter = steer_module.split_reference(reference)
        text = store.svg(number, letter)
        if text and text.strip():
            sources.append(text)
    return RoundPlan(
        parents=assign_parents(found.parents, letters),
        faults=dict(found.faults),
        denied_sources=tuple(sources),
        note=found.note,
        text=found.text,
    )
```

Now replace the head of `run_round`. Change:

```python
    sizes = render.sizes_for(the_guide.asset)
    parent, faults, human_text = choose_parent(store, index)
    parent_svg = None
    if parent:
        previous_index = int(parent.split("/")[0][len("round-") :])
        letter = parent.rsplit("-", 1)[1]
        parent_svg = store.svg(previous_index, letter)

    result = RoundResult(index=index, parent=parent)
    letters = session.VARIANT_LETTERS[:variants]
    round_path = store.round_path(index)
```

to:

```python
    sizes = render.sizes_for(the_guide.asset)
    letters = session.VARIANT_LETTERS[:variants]
    plan = plan_round(store, index, letters)
    result = RoundResult(index=index, parent=plan.parents.get(letters[0]))
    round_path = store.round_path(index)
```

Inside the `for letter in letters:` loop, replace the prompt block:

```python
        if parent_svg:
            prompt = build_revision_prompt(
                the_guide, subject, lens, sizes, parent_svg, faults, human_text
            )
        else:
            prompt = build_creation_prompt(the_guide, subject, lens, sizes)
```

with:

```python
        reference = plan.parents.get(letter)
        parent_svg = None
        if reference:
            number, parent_letter = steer_module.split_reference(reference)
            parent_svg = store.svg(number, parent_letter)
        if parent_svg:
            prompt = build_revision_prompt(
                the_guide,
                subject,
                lens,
                sizes,
                parent_svg,
                plan.faults.get(reference, []),
                plan.text,
                standing_note=plan.note,
                denied_sources=plan.denied_sources,
            )
        else:
            prompt = build_creation_prompt(
                the_guide,
                subject,
                lens,
                sizes,
                denied_sources=plan.denied_sources,
                standing_note=plan.note,
            )
```

Replace the `meta.json` write at the end of `run_round`:

```python
    result.seconds = sum(variant.seconds for variant in result.variants)
    summary = "first drawing" if not plan.parents else "revision"
    if plan.note or plan.text:
        summary += "; the human gave direction"
    if plan.denied_sources:
        summary += f"; {len(plan.denied_sources)} refused drawings in the prompt"
    session.write_json(
        round_path / "meta.json",
        {
            "round": index,
            "prompt_summary": f"{subject}; {summary}",
            "parents": dict(plan.parents),
        },
    )
    return result
```

Add `field` to the `dataclasses` import line if it is not there. It already is.

- [ ] **Step 4: Move the two tests that named the old function**

In `tools/direct-die/tests/test_parent.py`, replace the import line and both test bodies. The rules did not change, only the function that holds them. Replace the whole file body below the docstring with:

```python
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
```

Keep any other test that the file holds below line 40, and change each one the same way: `loop.choose_parent(store, n)` becomes `steer.collect(store, n)`, and `parent, faults, human` becomes `found.parents[0]`, `found.faults[found.parents[0]]` and `found.note`.

In `tools/direct-die/tests/test_direct_die.py`, replace `test_a_human_choice_beats_the_best_score` and `test_the_best_score_wins_when_no_feedback_exists`:

```python
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
```

Add `steer` to the `from direct_die import ...` line at the top of `tests/test_direct_die.py`.

- [ ] **Step 5: Run the whole tool test tree and see it pass**

```bash
cd /home/ty/workspace/cachette/tools/direct-die && python -m pytest tests -q
```

Expected: every test passes. Confirm with a search that nothing names the deleted function:

```bash
cd /home/ty/workspace/cachette && grep -rn "choose_parent" tools/direct-die
```

Expected: no output.

- [ ] **Step 6: Prove the branching test can fail**

Put the defect back. In `plan_round`, replace `assign_parents(found.parents, letters)` with `assign_parents(found.parents[:1], letters)`, so every variant takes the first parent. Then run:

```bash
cd /home/ty/workspace/cachette/tools/direct-die && python -m pytest tests/test_round_plan.py -q
```

Expected: FAIL on `test_two_likes_give_two_parents_across_four_variants`. If it passes, the fixture holds one like and the test measures nothing. Fix the test.

Undo the edit and run the tree again.

- [ ] **Step 7: Commit**

```bash
cd /home/ty/workspace/cachette
git add tools/direct-die/direct_die/loop.py tools/direct-die/tests/
git commit -F - <<'EOF'
Run a round from a plan that names a parent for each variant

A round took one parent and gave it to all four lenses. It now reads a plan
that names the parent of each variant letter, the faults of each parent, and
the SVG source of every refused drawing.

The round metadata therefore holds a `parents` object and no `parent` string.
A reader that finds the old field takes that one value for every variant.

Deleted `loop.choose_parent`. Its rules moved to `steer.collect`, and the two
tests that held them moved with it. Searched the whole tool for the name:

    grep -rn "choose_parent" tools/direct-die

Proved the branching test can fail by planning every variant from the first
parent. One test then failed.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_015NrPj9XQj2rz1SftMrjeF6
EOF
```

---

### Task 6: The analysis

**Files:**
- Create: `tools/direct-die/direct_die/analyse.py`
- Create: `tools/direct-die/tests/test_analyse.py`

**Interfaces:**
- Consumes: `guide_module.load`, `render.render`, `render.sizes_for`, `client.Image`, `client.ask_with_images`, `session.Session`, `steer.read_feedback`.
- Produces:
  - `analyse.ANALYST_SYSTEM`, a string.
  - `analyse.build_analysis_prompt(the_guide, subject: str, liked: Sequence[str]) -> str`
  - `analyse.parse_analysis(text: str, liked: Sequence[str]) -> dict`
  - `analyse.AnalysisError(RuntimeError)`
  - `analyse.run(asset: str, session_id: str, index: int, root: Path = session.SESSION_ROOT, exemplar_limit: int = guide_module.DEFAULT_EXEMPLAR_LIMIT, ask=client.ask_with_images, log=print) -> dict`

`run` takes the model call as an argument, so a test drives the whole path without a model.

`parse_analysis` follows `loop.parse_critique`: it finds the first JSON object in the answer and validates every field.

- [ ] **Step 1: Write the failing test**

Create `tools/direct-die/tests/test_analyse.py`:

```python
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
```

- [ ] **Step 2: Run the test and see it fail**

```bash
cd /home/ty/workspace/cachette/tools/direct-die && python -m pytest tests/test_analyse.py -q
```

Expected: a collection error, `ModuleNotFoundError: No module named 'direct_die.analyse'`.

- [ ] **Step 3: Write the module**

Create `tools/direct-die/direct_die/analyse.py`:

```python
"""The analysis of a liked set against a refused set.

The analysis runs between two rounds, and not inside one. A person ranks the
drawings of a round, reads this analysis, tunes the prompt, and then asks for
the next round.

It answers three questions. What do the liked drawings share that the refused
ones lack? Which liked drawing is best? Which rule of the guide would have
told the artist this without a person in the room?

## Why an order and not a score

A large change to a drawing moves the absolute score by a few points.[^1] An
order between two drawings does not have that defect, because it compares the
two drawings and not each drawing against a table. The order does not replace
the score. The score still decides which drawing wins when no person chose.

## References

[^1]: The tool guide, the known limits. `tools/direct-die/README.md`
"""

from __future__ import annotations

import json
import re
from collections.abc import Sequence
from pathlib import Path

from . import guide as guide_module
from . import render, session, steer
from .client import ClientError, Image, ask_with_images

ANALYST_SYSTEM = (
    "You are the assistant of an art director for a strategy game. The art "
    "director accepted some drawings and refused others. You say what the "
    "two groups differ by. You answer with one JSON object and nothing else."
)

ANALYSIS_SHAPE = (
    "Answer with one JSON object of this exact shape:\n"
    '{"preference": "<two sentences or fewer>", '
    '"order": ["<letter>", "..."], '
    '"reasons": {"<letter>": "<one sentence>"}, '
    '"guide_edit": {"section": "<a section name>", "rule": "<one sentence>"}}\n'
    "Rules for the answer:\n"
    "- Write the preference in the vocabulary of the style guide above.\n"
    "- Name a difference only when you can see it in a picture.\n"
    "- The field 'order' holds every accepted letter once, best first, and "
    "no other letter.\n"
    "- The field 'reasons' gives one sentence for each accepted letter. Say "
    "why it beats the letter below it.\n"
    "- The field 'guide_edit' proposes one rule that the guide does not "
    "state, which would have stopped the refused drawings. Write it as an "
    "instruction to an artist. Use null when the guide already states it.\n"
    "- Write no text outside the JSON object."
)


class AnalysisError(RuntimeError):
    """The analysis could not run, or the answer stayed malformed."""


_JSON_BLOCK = re.compile(r"\{.*\}", re.DOTALL)


def build_analysis_prompt(
    the_guide: guide_module.Guide, subject: str, liked: Sequence[str]
) -> str:
    """Build the prompt that compares the accepted set to the refused set."""
    letters = ", ".join(f'"{letter}"' for letter in liked)
    return (
        f"The drawings above are candidates for one {the_guide.asset}. Their "
        f"subject is: {subject}\n\n"
        "Each label says whether the art director accepted the drawing or "
        "refused it.\n\n"
        "The style guide follows.\n"
        "-----\n"
        f"{the_guide.rules}\n"
        "-----\n\n"
        f"The accepted letters are: {letters}\n\n"
        "Say what the accepted drawings share that the refused drawings "
        "lack. Judge at the display size, because that is where a player "
        "sees the asset.\n\n" + ANALYSIS_SHAPE
    )


def parse_analysis(text: str, liked: Sequence[str]) -> dict:
    """Take the analysis object out of an answer, and check its shape.

    Raise ValueError when the answer holds no valid object.
    """
    match = _JSON_BLOCK.search(text)
    if not match:
        raise ValueError("the answer holds no JSON object")
    try:
        value = json.loads(match.group(0))
    except json.JSONDecodeError as error:
        raise ValueError(f"the JSON does not parse: {error}") from error
    if not isinstance(value, dict):
        raise ValueError("the JSON is not an object")

    preference = value.get("preference")
    if not isinstance(preference, str) or not preference.strip():
        raise ValueError("the field 'preference' is missing or empty")

    order = value.get("order")
    if not isinstance(order, list) or not all(isinstance(item, str) for item in order):
        raise ValueError("the field 'order' is not a list of strings")
    if sorted(order) != sorted(liked):
        raise ValueError(
            "the field 'order' must hold every accepted letter once: "
            f"{sorted(liked)}"
        )

    reasons_in = value.get("reasons")
    reasons = {
        letter: (
            reasons_in.get(letter, "").strip()
            if isinstance(reasons_in, dict) and isinstance(reasons_in.get(letter), str)
            else ""
        )
        for letter in order
    }

    edit = value.get("guide_edit")
    if edit is None:
        parsed_edit = None
    elif isinstance(edit, dict):
        rule = edit.get("rule")
        if not isinstance(rule, str) or not rule.strip():
            raise ValueError("the guide edit holds no rule")
        section = edit.get("section")
        parsed_edit = {
            "section": section.strip() if isinstance(section, str) else "",
            "rule": rule.strip(),
        }
    else:
        raise ValueError("the field 'guide_edit' is not an object and not null")

    return {
        "preference": preference.strip(),
        "order": list(order),
        "reasons": reasons,
        "guide_edit": parsed_edit,
    }


def _rasters(
    store: session.Session, index: int, letters: Sequence[str], verdict: str, sizes
) -> list[Image]:
    """Rasterise each named variant at the display size, with a label."""
    images: list[Image] = []
    for letter in letters:
        source = store.svg(index, letter)
        if not source:
            continue
        try:
            data = render.render(source, sizes.display)
        except render.RenderError:
            continue
        images.append(
            Image(
                label=(
                    f"{verdict} BY THE ART DIRECTOR: variant {letter}, at "
                    f"{sizes.display} pixels, the size a player sees."
                ),
                data=data,
            )
        )
    return images


def run(
    asset: str,
    session_id: str,
    index: int,
    root: Path = session.SESSION_ROOT,
    exemplar_limit: int = guide_module.DEFAULT_EXEMPLAR_LIMIT,
    ask=ask_with_images,
    log=print,
) -> dict:
    """Analyse one round, write `analysis.json`, and give what it wrote.

    Raise `AnalysisError` when the round holds no like, when the round is not
    on disk, and when the answer stays malformed after a second ask.
    """
    store = session.Session(asset, session_id, root=root)
    directory = store.round_path(index)
    if not directory.is_dir():
        raise AnalysisError(f"no such round: {asset}/{session_id}/round-{index:02d}")

    found = steer.read_feedback(store.feedback(index))
    if not found.likes:
        raise AnalysisError(
            "the analysis needs at least one liked drawing. Like one, then "
            "ask again."
        )

    the_guide = guide_module.load(asset, limit=exemplar_limit)
    sizes = render.sizes_for(asset)
    images = _rasters(store, index, found.order, "ACCEPTED", sizes)
    if not images:
        raise AnalysisError("no liked drawing has a picture on disk")
    images.extend(_rasters(store, index, found.denies, "REFUSED", sizes))

    meta = session.read_json(directory / "meta.json") or {}
    summary = meta.get("prompt_summary")
    subject = summary.split(";")[0].strip() if isinstance(summary, str) else asset

    prompt = build_analysis_prompt(the_guide, subject, found.order)
    log(f"analysing {asset}/{session_id}/round-{index:02d}: {len(images)} pictures")

    last_error = "no answer"
    for attempt in range(2):
        text = prompt if attempt == 0 else prompt + (
            "\n\nYour last answer was not valid JSON of the shape above. "
            "Answer with the JSON object only, and write nothing else."
        )
        try:
            reply = ask(
                text,
                images,
                system=ANALYST_SYSTEM,
                temperature=0.3 if attempt == 0 else 0.0,
            )
        except ClientError as error:
            raise AnalysisError(f"the endpoint failed: {error}") from error
        try:
            parsed = parse_analysis(reply.text, found.order)
        except ValueError as error:
            last_error = str(error)
            continue
        payload = {"round": index, **parsed, "at": session.now_iso()}
        session.write_json(directory / "analysis.json", payload)
        log(f"  the analysis ranks {', '.join(parsed['order'])}")
        return payload

    raise AnalysisError(f"the analysis stayed malformed: {last_error}")
```

- [ ] **Step 4: Run the test and see it pass**

```bash
cd /home/ty/workspace/cachette/tools/direct-die && python -m pytest tests/test_analyse.py -q
```

Expected: 21 passed.

- [ ] **Step 5: Commit**

```bash
cd /home/ty/workspace/cachette
git add tools/direct-die/direct_die/analyse.py tools/direct-die/tests/test_analyse.py
git commit -F - <<'EOF'
Analyse a liked set against a refused set

The analysis runs between two rounds, and not inside one. It shows the model
the accepted drawings and the refused drawings together, and it answers with
a preference statement, an order between the accepted drawings, and one
proposed rule for the style guide.

It gives an order and not a score. The tool already records that a large
change to a drawing moves the absolute score by a few points. A comparison of
two drawings does not have that defect.

The runner takes the model call as an argument, so every test here drives the
whole path without a model.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_015NrPj9XQj2rz1SftMrjeF6
EOF
```

---

### Task 7: The `analyse` command

**Files:**
- Modify: `tools/direct-die/direct_die/cli.py`
- Create: `tools/direct-die/tests/test_cli.py`

**Interfaces:**
- Consumes: `analyse.run`, `analyse.AnalysisError`.
- Produces: the command `python -m direct_die analyse --asset X --session Y --round N`. It returns 0 on success, 2 on a guide error, 3 on an endpoint error, and 4 on an analysis error.

- [ ] **Step 1: Write the failing test**

Create `tools/direct-die/tests/test_cli.py`:

```python
"""Tests for the command line of the tool.

The front end calls the tool this way and no other way, so the exit codes and
the argument names are a contract.
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from direct_die import analyse, cli  # noqa: E402


def test_the_analyse_command_calls_the_runner(monkeypatch, capsys):
    seen = {}

    def fake_run(asset, session_id, index, **rest):
        seen["asset"] = asset
        seen["session"] = session_id
        seen["round"] = index
        return {"preference": "flat fills read better", "order": ["d", "b"]}

    monkeypatch.setattr(analyse, "run", fake_run)
    code = cli.main(["analyse", "--asset", "cartoon", "--session", "s1", "--round", "2"])
    assert code == 0
    assert seen == {"asset": "cartoon", "session": "s1", "round": 2}
    assert "flat fills read better" in capsys.readouterr().out


def test_the_analyse_command_reports_an_analysis_error(monkeypatch, capsys):
    def fake_run(*args, **rest):
        raise analyse.AnalysisError("the analysis needs at least one liked drawing")

    monkeypatch.setattr(analyse, "run", fake_run)
    code = cli.main(["analyse", "--asset", "cartoon", "--session", "s1", "--round", "0"])
    assert code == 4
    assert "at least one liked drawing" in capsys.readouterr().err


def test_the_analyse_command_needs_a_session_and_a_round():
    with pytest.raises(SystemExit):
        cli.main(["analyse", "--asset", "cartoon"])
```

- [ ] **Step 2: Run the test and see it fail**

```bash
cd /home/ty/workspace/cachette/tools/direct-die && python -m pytest tests/test_cli.py -q
```

Expected: FAIL. `argparse` exits with code 2 because `analyse` is not a command.

- [ ] **Step 3: Write the code**

In `tools/direct-die/direct_die/cli.py`, add `analyse` to the import line:

```python
from . import analyse as analyse_module
from . import guide as guide_module
from . import loop, render, session, setrun, subjects
```

Add the handler below `_set`:

```python
def _analyse(arguments: argparse.Namespace) -> int:
    """Analyse one round, and print what the model said."""
    try:
        found = analyse_module.run(
            arguments.asset,
            arguments.session,
            arguments.round,
            exemplar_limit=arguments.exemplars,
        )
    except guide_module.GuideError as error:
        print(f"guide error: {error}", file=sys.stderr)
        return 2
    except ClientError as error:
        print(f"endpoint error: {error}", file=sys.stderr)
        return 3
    except analyse_module.AnalysisError as error:
        print(f"analysis error: {error}", file=sys.stderr)
        return 4

    print()
    print(found["preference"])
    order = found.get("order") or []
    if order:
        print("order: " + " > ".join(order))
    edit = found.get("guide_edit")
    if edit:
        print(f"proposed rule ({edit['section']}): {edit['rule']}")
    return 0
```

The test replaces `analyse.run` on the module object, so the handler must call it through the module and not through a name it imported directly. It does.

Add the parser below the `set` parser, above the `guide` parser:

```python
    analysis = commands.add_parser(
        "analyse", help="say what the liked drawings of a round share"
    )
    analysis.add_argument("--asset", default="hex-tile", help="the asset type")
    analysis.add_argument("--session", required=True, help="the session identifier")
    analysis.add_argument("--round", type=int, required=True, help="the round index")
    analysis.add_argument(
        "--exemplars",
        type=int,
        default=guide_module.DEFAULT_EXEMPLAR_LIMIT,
        help="how many exemplar images to attach",
    )
    analysis.set_defaults(handler=_analyse)
```

- [ ] **Step 4: Run the test and see it pass**

```bash
cd /home/ty/workspace/cachette/tools/direct-die && python -m pytest tests/test_cli.py -q
cd /home/ty/workspace/cachette/tools/direct-die && python -m direct_die analyse --help
```

Expected: 3 passed, and the help text names `--asset`, `--session` and `--round`.

- [ ] **Step 5: Commit**

```bash
cd /home/ty/workspace/cachette
git add tools/direct-die/direct_die/cli.py tools/direct-die/tests/test_cli.py
git commit -F - <<'EOF'
Add the analyse command

The front end calls the tool through its command line and never imports the
tool package, so the analysis needs a command. It takes the asset, the
session and the round index, and it writes analysis.json into that round.

The exit codes say which part failed: 2 for the guide, 3 for the endpoint,
and 4 for the analysis itself.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_015NrPj9XQj2rz1SftMrjeF6
EOF
```

---

### Task 8: The front end reads and writes the new shape

**Files:**
- Modify: `tools/direct-die/review/store.py`
- Modify: `tools/direct-die/review/make_fixtures.py`
- Modify: `tools/direct-die/review/test_review.py`

**Interfaces:**
- Consumes: nothing from the tool package. This reader is a second declaration site of the shape that Task 1 built, and Task 10 adds the check that fails when the two disagree.
- Produces, on `store.Round`:
  - `likes -> tuple[str, ...]`, `denies -> tuple[str, ...]`, `order -> tuple[str, ...]`
  - `winner -> str | None`, the first entry of `order`
  - `note -> str`, and the existing `feedback_text -> str`
  - `parents -> dict[str, str]`
  - `analysis -> dict | None`
  - The property `choice` goes away.
- Produces, on `store.SessionStore`:
  - `write_feedback(asset, session_id, round_name, likes: Sequence[str], denies: Sequence[str], order: Sequence[str], note: str, text: str) -> Path`
  - `round_index(round_name: str) -> int`

`write_feedback` raises `ContractError` when a letter is not a variant letter, when a letter is in both lists, when `order` is not the same set as `likes`, and when the round directory is absent.

- [ ] **Step 1: Write the failing test**

Append to `tools/direct-die/review/test_review.py`:

```python
# -- the feedback shape ------------------------------------------------------


def _round_directory(root: Path, key: str, round_name: str) -> Path:
    asset, session_id = key.split("/")
    return root / asset / session_id / round_name


def test_the_writer_records_the_likes_and_the_refusals(root: Path) -> None:
    store = SessionStore(root)
    asset, session_id = HEALTHY.split("/")
    path = store.write_feedback(
        asset, session_id, "round-02", ["b", "d"], ["a"], ["d", "b"], "keep it flat", "darker base"
    )
    written = json.loads(path.read_text(encoding="utf-8"))
    assert written["round"] == 2
    assert written["likes"] == ["b", "d"]
    assert written["denies"] == ["a"]
    assert written["order"] == ["d", "b"]
    assert written["note"] == "keep it flat"
    assert written["text"] == "darker base"
    assert "choice" not in written


def test_the_writer_refuses_a_letter_that_is_in_both_lists(root: Path) -> None:
    store = SessionStore(root)
    asset, session_id = HEALTHY.split("/")
    with pytest.raises(ContractError):
        store.write_feedback(asset, session_id, "round-02", ["b"], ["b"], ["b"], "", "")


def test_the_writer_refuses_an_order_that_is_not_the_likes(root: Path) -> None:
    store = SessionStore(root)
    asset, session_id = HEALTHY.split("/")
    with pytest.raises(ContractError):
        store.write_feedback(asset, session_id, "round-02", ["b", "d"], [], ["b"], "", "")


def test_the_writer_refuses_a_letter_that_is_not_a_variant(root: Path) -> None:
    store = SessionStore(root)
    asset, session_id = HEALTHY.split("/")
    with pytest.raises(ContractError):
        store.write_feedback(asset, session_id, "round-02", ["z"], [], ["z"], "", "")


def test_an_old_choice_file_reads_as_one_like(root: Path) -> None:
    store = SessionStore(root)
    asset, session_id = HEALTHY.split("/")
    current = store.load_round(asset, session_id, "round-00")
    assert current.likes == ("b",)
    assert current.winner == "b"
    assert current.denies == ()


def test_a_round_with_no_feedback_reads_as_nothing(root: Path) -> None:
    store = SessionStore(root)
    asset, session_id = EMPTY.split("/")
    session = store.load_session(asset, session_id)
    assert session.rounds == []


def test_the_round_reads_a_parents_object(root: Path) -> None:
    store = SessionStore(root)
    asset, session_id = HEALTHY.split("/")
    current = store.load_round(asset, session_id, "round-01")
    assert current.parents
    assert set(current.parents.values())


def test_a_round_with_no_analysis_gives_none(root: Path) -> None:
    store = SessionStore(root)
    asset, session_id = HEALTHY.split("/")
    assert store.load_round(asset, session_id, "round-02").analysis is None


def test_a_round_with_an_analysis_gives_it(root: Path) -> None:
    store = SessionStore(root)
    asset, session_id = HEALTHY.split("/")
    current = store.load_round(asset, session_id, "round-01")
    assert current.analysis is not None
    assert current.analysis["order"]
```

Add `import pytest` to the imports of the file if it is not there. It is.

- [ ] **Step 2: Run the test and see it fail**

```bash
cd /home/ty/workspace/cachette/tools/direct-die/review && python -m pytest test_review.py -q
```

Expected: many failures. `write_feedback` takes the old arguments, and `Round` has no `likes`.

- [ ] **Step 3: Write the reader and the writer**

In `tools/direct-die/review/store.py`, add these module functions above `class Round`:

```python
def _letters(value: object) -> tuple[str, ...]:
    """Take the variant letters out of a value, once each, in order."""
    if not isinstance(value, list):
        return ()
    found: list[str] = []
    for item in value:
        if isinstance(item, str) and item in VARIANT_LETTERS and item not in found:
            found.append(item)
    return tuple(found)


def read_feedback(value: dict | None) -> tuple[tuple[str, ...], tuple[str, ...], tuple[str, ...], str, str]:
    """Read one feedback object into likes, refusals, order, note and text.

    The function never raises. It drops what it cannot read, because a person
    can edit the file by hand and a page must still render.

    The file held a `choice` field, which named one variant. Nothing writes
    that field now. A file that holds `choice` and no `likes` reads as one
    like, so every session already on disk still opens. The generation loop
    holds the same rule in its own reader, and one test compares the two.[^1]

    ## References

    [^1]: The reader of the loop. `tools/direct-die/direct_die/steer.py`
    """
    if not isinstance(value, dict):
        return (), (), (), "", ""
    denies = _letters(value.get("denies"))
    if "likes" in value:
        likes = _letters(value.get("likes"))
    else:
        choice = value.get("choice")
        likes = (choice,) if isinstance(choice, str) and choice in VARIANT_LETTERS else ()
    likes = tuple(letter for letter in likes if letter not in denies)
    ranked = [letter for letter in _letters(value.get("order")) if letter in likes]
    ranked.extend(letter for letter in likes if letter not in ranked)
    note = value.get("note")
    text = value.get("text")
    return (
        likes,
        denies,
        tuple(ranked),
        note.strip() if isinstance(note, str) else "",
        text if isinstance(text, str) else "",
    )
```

In `class Round`, add the field `analysis: dict | None = None` beside `feedback`, delete the `choice` property, and add:

```python
    @property
    def _read(self) -> tuple:
        """Read the feedback once, for the properties below."""
        return read_feedback(self.feedback)

    @property
    def likes(self) -> tuple[str, ...]:
        """Give the letters that the person accepted."""
        return self._read[0]

    @property
    def denies(self) -> tuple[str, ...]:
        """Give the letters that the person refused."""
        return self._read[1]

    @property
    def order(self) -> tuple[str, ...]:
        """Give the accepted letters, best first."""
        return self._read[2]

    @property
    def note(self) -> str:
        """Give the standing note. Give "" when there is none."""
        return self._read[3]

    @property
    def winner(self) -> str | None:
        """Give the letter that stands for this round, or `None`.

        It is the first of the order, which is the drawing the person put
        first. A refused drawing is never the winner, because the reader
        drops a refused letter from the likes.
        """
        found = self.order
        return found[0] if found else None

    @property
    def parents(self) -> dict[str, str]:
        """Give the parent that each variant of this round revised.

        The loop wrote one `parent` string before it could branch. A round
        that holds the old field gives that one value to every variant.
        """
        if self.meta is None:
            return {}
        found = self.meta.get("parents")
        if isinstance(found, dict):
            return {
                letter: value
                for letter, value in found.items()
                if letter in VARIANT_LETTERS and isinstance(value, str)
            }
        single = self.meta.get("parent")
        if isinstance(single, str):
            return {variant.letter: single for variant in self.present_variants}
        return {}
```

Change `feedback_text` to read through `_read[4]`:

```python
    @property
    def feedback_text(self) -> str:
        """Give the note for this round only. Give "" when there is none."""
        return self._read[4]
```

Delete the old `parent` property.

In `SessionStore.load_round`, load the analysis beside the feedback:

```python
        feedback = read_json(directory / "feedback.json")
        analysis = read_json(directory / "analysis.json")
```

and pass `analysis=analysis` into the `Round(...)` call.

Add the index helper to `SessionStore`:

```python
    @staticmethod
    def round_index(round_name: str) -> int:
        """Give the index that a round directory name holds.

        Raise `ContractError` when the name is not a round directory name.
        """
        if not round_name.startswith("round-"):
            raise ContractError(f"not a round name: {round_name!r}")
        try:
            return int(round_name[len("round-") :])
        except ValueError as error:
            raise ContractError(f"not a round name: {round_name!r}") from error
```

Replace `write_feedback` with:

```python
    def write_feedback(
        self,
        asset: str,
        session_id: str,
        round_name: str,
        likes: Sequence[str],
        denies: Sequence[str],
        order: Sequence[str],
        note: str,
        text: str,
    ) -> Path:
        """Write `feedback.json` into a round directory, and give its path.

        The write is atomic. It writes a temporary file in the same directory
        and then renames it, so the generation loop never reads half a file.

        Raise `ContractError` when a letter is not a variant letter, when a
        letter is in both lists, when the order is not the same set as the
        likes, and when the round directory does not exist. The loop owns
        that directory, so this module does not create one.
        """
        index = self.round_index(round_name)
        for group in (likes, denies, order):
            for letter in group:
                if letter not in VARIANT_LETTERS:
                    raise ContractError(f"not a variant letter: {letter!r}")
        both = sorted(set(likes) & set(denies))
        if both:
            raise ContractError(
                f"a letter cannot be liked and refused: {', '.join(both)}"
            )
        if sorted(set(order)) != sorted(set(likes)):
            raise ContractError("the order must hold every liked letter once")
        directory = self.round_directory(asset, session_id, round_name)
        if not directory.is_dir():
            raise ContractError(f"no such round: {asset}/{session_id}/{round_name}")
        payload = {
            "round": index,
            "likes": list(dict.fromkeys(likes)),
            "denies": list(dict.fromkeys(denies)),
            "order": list(dict.fromkeys(order)),
            "note": note.strip(),
            "text": text,
            "at": datetime.now(UTC)
            .isoformat(timespec="seconds")
            .replace("+00:00", "Z"),
        }
        return write_json_atomically(directory / "feedback.json", payload)
```

Add `from collections.abc import Sequence` to the imports of `store.py`.

- [ ] **Step 4: Give the fixture tree the new shape**

In `tools/direct-die/review/make_fixtures.py`, the healthy session writes feedback into round 0 and round 1. Keep round 0 as it is, with `"choice": "b"`, because one fixture must hold the old shape and prove the reader still opens it. Change the round 1 write from `{"choice": None, ...}` to the new shape, and give the healthy session an analysis and a `parents` object.

Replace the round 1 feedback write with:

```python
    write_json(
        healthy / "round-01" / "feedback.json",
        {
            "round": 1,
            "likes": ["b", "d"],
            "denies": ["a"],
            "order": ["d", "b"],
            "note": "keep the palette flat",
            "text": "raise the contrast of the base",
            "at": "2026-09-01T10:20:00Z",
        },
    )
    write_json(
        healthy / "round-01" / "analysis.json",
        {
            "round": 1,
            "preference": "The accepted drawings hold three shapes and one flat fill.",
            "order": ["d", "b"],
            "reasons": {"d": "the silhouette reads at 64 pixels", "b": ""},
            "guide_edit": {
                "section": "Shape language",
                "rule": "Use three shapes or fewer inside the hexagon.",
            },
            "at": "2026-09-01T10:25:00Z",
        },
    )
```

Find the `meta.json` write of the healthy session's round 1 and round 2 and change `"parent": "round-00/variant-b"` to `"parents": {"a": "round-00/variant-b", "b": "round-00/variant-d", "c": "round-00/variant-b", "d": "round-00/variant-d"}`. Leave one round of the partial session with the old `"parent"` string, so a fixture holds that case too. Put a comment on that line that says so.

In `build_style_sessions`, the `cartoon/20260904-090000-forest` session writes `{"choice": "b", ...}` into `round-01`. `test_frontend.py` reads that tree, and Task 13 needs an analysis there. Replace that write with:

```python
    write_json(
        chosen / "round-01" / "feedback.json",
        {
            "round": 1,
            "likes": ["b", "d"],
            "denies": ["a"],
            "order": ["b", "d"],
            "note": "Keep the canopy shape.",
            "text": "B reads at tile size.",
            "at": (created + timedelta(days=3, minutes=5))
            .isoformat(timespec="seconds")
            .replace("+00:00", "Z"),
        },
    )
    write_json(
        chosen / "round-01" / "analysis.json",
        {
            "round": 1,
            "preference": "The accepted drawings hold three shapes and one flat fill.",
            "order": ["d", "b"],
            "reasons": {"d": "the silhouette reads at 64 pixels", "b": ""},
            "guide_edit": {
                "section": "Shape language",
                "rule": "Use three shapes or fewer inside the hexagon.",
            },
            "at": (created + timedelta(days=3, minutes=8))
            .isoformat(timespec="seconds")
            .replace("+00:00", "Z"),
        },
    )
```

That session's `round-01` is written by `write_round(chosen, 1, parent="round-00/variant-b", ...)`. Leave that `parent` string as it is. It is the fixture of the old metadata shape, and one test reads it. Put a comment on the line that says so.

Note that `write_round` draws four variants, so `d` exists and can be liked and `a` exists and can be refused. Check that before you write the feedback above:

```bash
cd /home/ty/workspace/cachette/tools/direct-die/review && grep -n "def write_round" -A 12 make_fixtures.py
```

- [ ] **Step 5: Repair the tests that named the old writer**

Search for every call site:

```bash
cd /home/ty/workspace/cachette && grep -rn "write_feedback\|\.choice\|round\.parent" tools/direct-die/review
```

Change each `store.write_feedback(asset, session_id, round_name, "b", "text")` to `store.write_feedback(asset, session_id, round_name, ["b"], [], ["b"], "", "text")`. Change each `round.choice` to `round.winner`. Change each `round.parent` to `round.parents`.

`app.py` and `round.html` also hold call sites. Task 12 rewrites them. Until then the server will not import cleanly, so run only `test_review.py` in this task, and expect `test_frontend.py` to fail. Say so in the commit.

- [ ] **Step 6: Run the store tests and see them pass**

```bash
cd /home/ty/workspace/cachette/tools/direct-die/review && python -m pytest test_review.py -q -k "feedback or shape or parent or analysis or choice or writer"
```

Expected: every selected test passes.

- [ ] **Step 7: Commit**

```bash
cd /home/ty/workspace/cachette
git add tools/direct-die/review/store.py tools/direct-die/review/make_fixtures.py tools/direct-die/review/test_review.py
git commit -F - <<'EOF'
Read and write the feedback file in its new shape

The file held one chosen letter and free text. It now holds a like list, a
refusal list, an order between the likes, a standing note and a note for one
round. The writer refuses a letter that is in both lists, and an order that
is not the same set as the likes.

The reader takes the old `choice` field as one like, so every session already
on disk still opens. The round metadata reader takes the old `parent` string
for every variant, for the same reason. One fixture holds each old shape on
purpose.

Searched the front end for every call site:

    grep -rn "write_feedback\|\.choice\|round\.parent" tools/direct-die/review

test_frontend.py fails at this commit, because app.py and round.html still
call the old writer. The task that rewrites the round page repairs them.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_015NrPj9XQj2rz1SftMrjeF6
EOF
```

---

### Task 9: Which drawing stands for an asset

**Files:**
- Modify: `tools/direct-die/review/packs.py`
- Modify: `tools/direct-die/review/test_frontend.py`

**Interfaces:**
- Consumes: `store.Round.winner` and `store.Round.denies` from Task 8.
- Produces: no new name. `pick_for` keeps its signature and changes its rule.

The three changes:

1. A human choice is the `winner` of a round, which is the first entry of `order`.
2. A refused variant never wins the score fallback, whatever it scored.
3. Everything else stays. The rule reads the sessions newest first, and a later round wins a tie in the fallback.

- [ ] **Step 1: Write the failing test**

Append to `tools/direct-die/review/test_frontend.py`:

```python
# -- which drawing stands for an asset ---------------------------------------


def test_the_first_of_the_order_stands_for_the_asset(tmp_path: Path) -> None:
    root = _one_session_tree(tmp_path, scores={"b": 30, "d": 90}, feedback={
        "round": 0, "likes": ["b", "d"], "denies": [], "order": ["b", "d"], "note": "", "text": ""
    })
    store = SessionStore(root)
    pick = packs.pick_for(store, "cartoon", "forest")
    assert pick is not None
    assert pick.letter == "b"
    assert pick.source == "human"


def test_a_refused_drawing_never_wins_on_score(tmp_path: Path) -> None:
    root = _one_session_tree(tmp_path, scores={"a": 95, "c": 30}, feedback={
        "round": 0, "likes": [], "denies": ["a"], "order": [], "note": "", "text": ""
    })
    store = SessionStore(root)
    pick = packs.pick_for(store, "cartoon", "forest")
    assert pick is not None
    assert pick.letter == "c"
    assert pick.source == "score"


def test_the_highest_score_still_wins_when_nobody_said_anything(tmp_path: Path) -> None:
    root = _one_session_tree(tmp_path, scores={"a": 40, "c": 80}, feedback=None)
    store = SessionStore(root)
    pick = packs.pick_for(store, "cartoon", "forest")
    assert pick is not None
    assert pick.letter == "c"
    assert pick.source == "score"
```

These tests go in `test_frontend.py`. Its import block already binds `packs` as `pack_module` and `SessionStore` from `store`. Add `write_json_atomically` to the `from store import ...` line, and change `packs.pick_for` to `pack_module.pick_for` in the three tests above.

Add this helper above those tests:

```python
def _one_session_tree(tmp_path: Path, scores: dict, feedback: dict | None) -> Path:
    """Write one style, one session and one round, and give the sessions root.

    The fixture takes a score for each variant, so a test can put the highest
    score on the drawing that the person refused. A fixture that scores every
    variant the same cannot prove that the refusal changed the answer.
    """
    root = tmp_path / "sessions"
    directory = root / "cartoon" / "forest-20260901-1000" / "round-00"
    directory.mkdir(parents=True)
    write_json_atomically(
        root / "cartoon" / "forest-20260901-1000" / "session.json",
        {"asset": "forest", "created": "2026-09-01T10:00:00Z", "model": "m", "guide_version": "v"},
    )
    write_json_atomically(directory / "meta.json", {"round": 0, "prompt_summary": "a forest", "parents": {}})
    for letter, score in scores.items():
        (directory / f"variant-{letter}.svg").write_text(
            '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"/>', encoding="utf-8"
        )
        (directory / f"variant-{letter}.png").write_bytes(b"\x89PNG\r\n\x1a\n")
        write_json_atomically(
            directory / f"variant-{letter}.critique.json",
            {"verdict": "x", "faults": [], "score": score},
        )
    if feedback is not None:
        write_json_atomically(directory / "feedback.json", feedback)
    return root
```

The import line becomes:

```python
from store import SessionStore, write_json_atomically  # noqa: E402
```

- [ ] **Step 2: Run the test and see it fail**

```bash
cd /home/ty/workspace/cachette/tools/direct-die/review && python -m pytest test_frontend.py -q -k "stands_for or refused or highest_score"
```

Expected: FAIL with `AttributeError: 'Round' object has no attribute 'choice'` inside `pick_for`.

- [ ] **Step 3: Write the code**

In `tools/direct-die/review/packs.py`, inside `pick_for`, change the human loop:

```python
    for session in reversed(found):
        for entry in reversed(session.rounds):
            letter = entry.winner
```

and inside the score loop, skip a refused letter:

```python
            for variant in entry.present_variants:
                if variant.svg is None:
                    continue
                if variant.letter in entry.denies:
                    # A person refused this drawing. It never stands for the
                    # asset, whatever the model scored it.
                    continue
```

Update the module docstring of `packs.py`. Replace "takes the first human choice that it finds" with "takes the drawing that the person put first". Add one sentence: "A refused drawing never wins, whatever the model scored it."

- [ ] **Step 4: Run the test and see it pass**

```bash
cd /home/ty/workspace/cachette/tools/direct-die/review && python -m pytest test_frontend.py -q -k "stands_for or refused or highest_score"
```

Expected: 3 passed.

- [ ] **Step 5: Prove the refusal test can fail**

Put the defect back. Remove the `if variant.letter in entry.denies: continue` block, then run the same command.

Expected: FAIL on `test_a_refused_drawing_never_wins_on_score`. The fixture gives the refused drawing the highest score on purpose, so the test can see the difference. If it passes, the fixture is wrong.

Undo the edit.

- [ ] **Step 6: Commit**

```bash
cd /home/ty/workspace/cachette
git add tools/direct-die/review/packs.py tools/direct-die/review/test_frontend.py
git commit -F - <<'EOF'
Let the drawing a person put first stand for the asset

The rule took the one letter a person chose. It now takes the first entry of
the order, which is the drawing they put at the top of the liked set.

A refused drawing never wins the score fallback, whatever the model scored
it. The fixture gives the refused drawing the highest score on purpose, so
the test can see the difference.

Proved the test can fail by removing the refusal check. One test then failed.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_015NrPj9XQj2rz1SftMrjeF6
EOF
```

---

### Task 10: The check that the two readers agree

**Files:**
- Create: `tools/direct-die/review/test_readers_agree.py`

**Interfaces:**
- Consumes: `store.read_feedback` from Task 8 and `direct_die.steer.read_feedback` from Task 1.
- Produces: no name. It is a check.

The feedback shape has two readers. The tool reads it to steer the loop, and the front end reads it to draw the page. The project rule is that a second declaration site needs a check that fails when the copies disagree. This is that check.

This is the one place where a test imports the tool package. The server itself does not.

- [ ] **Step 1: Write the test**

Create `tools/direct-die/review/test_readers_agree.py`:

```python
"""The two readers of the feedback file must agree.

The generation loop reads `feedback.json` to steer the next round. The front
end reads it to draw the page. That is two declaration sites of one shape, and
a second site needs a check that fails when the copies disagree.

This is the only test that imports the tool package. The server never does. It
calls the tool through its command line.
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

HERE = Path(__file__).resolve().parent
TOOL = HERE.parent
for entry in (str(HERE), str(TOOL)):
    if entry not in sys.path:
        sys.path.insert(0, entry)

from direct_die import steer  # noqa: E402
from store import read_feedback as front_end_reader  # noqa: E402

CASES = [
    None,
    {},
    ["b"],
    {"choice": "b", "text": "more contrast"},
    {"choice": None, "text": ""},
    {"choice": "a", "likes": ["c"]},
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
```

- [ ] **Step 2: Run the test**

```bash
cd /home/ty/workspace/cachette/tools/direct-die/review && python -m pytest test_readers_agree.py -q
```

Expected: 15 passed. If one fails, one reader is wrong. Fix the reader, not the test.

- [ ] **Step 3: Prove the check can fail**

In `store.py`, change `read_feedback` so it ignores the `denies` field: replace `denies = _letters(value.get("denies"))` with `denies = ()`. Run the test again.

Expected: FAIL on at least two cases. Undo the edit.

- [ ] **Step 4: Commit**

```bash
cd /home/ty/workspace/cachette
git add tools/direct-die/review/test_readers_agree.py
git commit -F - <<'EOF'
Check that the two readers of the feedback file agree

The generation loop reads feedback.json to steer the next round, and the
front end reads it to draw the page. That is one value with two declaration
sites, which is the defect shape this project sees most.

The rule is that a second site needs a check that fails when the copies
disagree. This is that check. It drives both readers over fifteen inputs,
including every old-field case and every malformed case.

This is the only test that imports the tool package. The server never does.

Proved the check can fail by making the front end reader ignore the refusal
list. Two cases then failed.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_015NrPj9XQj2rz1SftMrjeF6
EOF
```

---

### Task 11: Append an accepted rule to a style guide

**Files:**
- Create: `tools/direct-die/review/rules.py`
- Create: `tools/direct-die/review/test_rules.py`

**Interfaces:**
- Consumes: `store.write_json_atomically` is for JSON, so this module writes its own atomic text helper. Follow the same pattern: a temporary file in the same directory, then a rename.
- Produces:
  - `rules.ADDED_HEADING`, the string `"## Rules the art director added"`.
  - `rules.rules_file(styleguide_root: Path, style: str) -> Path`
  - `rules.append_rule(styleguide_root: Path, style: str, rule: str) -> Path`
  - `rules.RuleError(RuntimeError)`

The function appends one line under a heading that it creates once. It does not try to find the section that the model named. A section name from a model is a guess, and a wrong guess puts a rule where nobody reads it. One heading at the end of the file is honest and a person can move the line later.

The rule goes in as a list item. The function raises when the rule is empty, when it holds a newline, and when the style has no rules file.

- [ ] **Step 1: Write the failing test**

Create `tools/direct-die/review/test_rules.py`:

```python
"""Tests for the one write into a style rules file.

The analysis proposes a rule. A person accepts it, and this module appends it.
It appends under one heading that it creates once. It does not guess where the
rule belongs, because a wrong guess puts a rule where nobody reads it.
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

import rules  # noqa: E402

BODY = "# cartoon\n\n## 1. Geometry\n\n- The tile is a hexagon.\n"


def _tree(tmp_path: Path) -> Path:
    root = tmp_path / "styleguide"
    root.mkdir()
    (root / "cartoon.md").write_text(BODY, encoding="utf-8")
    return root


def test_the_rule_lands_under_a_new_heading(tmp_path: Path) -> None:
    root = _tree(tmp_path)
    path = rules.append_rule(root, "cartoon", "Use three shapes or fewer.")
    text = path.read_text(encoding="utf-8")
    assert rules.ADDED_HEADING in text
    assert "- Use three shapes or fewer." in text
    assert text.index("## 1. Geometry") < text.index(rules.ADDED_HEADING)


def test_a_second_rule_reuses_the_heading(tmp_path: Path) -> None:
    root = _tree(tmp_path)
    rules.append_rule(root, "cartoon", "Use three shapes or fewer.")
    path = rules.append_rule(root, "cartoon", "Keep the palette to four colours.")
    text = path.read_text(encoding="utf-8")
    assert text.count(rules.ADDED_HEADING) == 1
    assert "- Use three shapes or fewer." in text
    assert "- Keep the palette to four colours." in text


def test_the_original_rules_survive(tmp_path: Path) -> None:
    root = _tree(tmp_path)
    path = rules.append_rule(root, "cartoon", "Use three shapes or fewer.")
    assert "- The tile is a hexagon." in path.read_text(encoding="utf-8")


def test_the_write_leaves_no_temporary_file(tmp_path: Path) -> None:
    root = _tree(tmp_path)
    rules.append_rule(root, "cartoon", "Use three shapes or fewer.")
    assert sorted(item.name for item in root.iterdir()) == ["cartoon.md"]


def test_an_empty_rule_is_refused(tmp_path: Path) -> None:
    root = _tree(tmp_path)
    with pytest.raises(rules.RuleError):
        rules.append_rule(root, "cartoon", "   ")


def test_a_rule_with_a_newline_is_refused(tmp_path: Path) -> None:
    root = _tree(tmp_path)
    with pytest.raises(rules.RuleError):
        rules.append_rule(root, "cartoon", "one line\nand another")


def test_a_style_with_no_rules_file_is_refused(tmp_path: Path) -> None:
    root = _tree(tmp_path)
    with pytest.raises(rules.RuleError):
        rules.append_rule(root, "pencil", "Use three shapes or fewer.")


def test_a_style_name_with_a_separator_is_refused(tmp_path: Path) -> None:
    root = _tree(tmp_path)
    with pytest.raises(rules.RuleError):
        rules.append_rule(root, "../secrets", "Use three shapes or fewer.")
```

- [ ] **Step 2: Run the test and see it fail**

```bash
cd /home/ty/workspace/cachette/tools/direct-die/review && python -m pytest test_rules.py -q
```

Expected: a collection error, `ModuleNotFoundError: No module named 'rules'`.

- [ ] **Step 3: Write the module**

Create `tools/direct-die/review/rules.py`:

```python
"""The one write into a style rules file.

The analysis proposes a rule that the guide does not state. A person reads it,
edits it, and accepts it. This module appends it.

It appends under one heading that it creates once. It does not try to find the
section that the model named. A section name from a model is a guess, and a
wrong guess puts a rule where nobody reads it. A person moves the line later
if they want it somewhere else.

The style guide version is a digest of the rules and the exemplars, so an
accepted rule changes the version that the next session records.[^1]

## References

[^1]: The tool guide, the style guide. `tools/direct-die/README.md`
"""

from __future__ import annotations

import os
import tempfile
from pathlib import Path

# The heading that holds every rule a person accepted from an analysis.
ADDED_HEADING = "## Rules the art director added"


class RuleError(RuntimeError):
    """The rule cannot be written."""


def rules_file(styleguide_root: Path, style: str) -> Path:
    """Give the rules file of one style.

    Raise `RuleError` when the name is not one path segment, and when the
    file does not exist.
    """
    if not style or "/" in style or "\\" in style or style in (".", ".."):
        raise RuleError(f"not a style name: {style!r}")
    path = Path(styleguide_root) / f"{style}.md"
    if not path.is_file():
        raise RuleError(f"no rules file for {style!r}")
    return path


def append_rule(styleguide_root: Path, style: str, rule: str) -> Path:
    """Append one rule to a style rules file, and give the path.

    The write is atomic. It writes a temporary file in the same directory and
    then renames it, so a reader never sees half a file.

    Raise `RuleError` when the rule is empty and when it holds a newline. A
    rule is one line, because the heading holds a list.
    """
    path = rules_file(styleguide_root, style)
    text = rule.strip()
    if not text:
        raise RuleError("the rule is empty")
    if "\n" in text or "\r" in text:
        raise RuleError("a rule is one line")

    body = path.read_text(encoding="utf-8").rstrip("\n")
    if ADDED_HEADING not in body:
        body += f"\n\n{ADDED_HEADING}\n"
    body += f"\n- {text}\n"

    directory = path.parent
    handle, temporary = tempfile.mkstemp(dir=directory, suffix=".tmp")
    try:
        with os.fdopen(handle, "w", encoding="utf-8") as file:
            file.write(body)
        os.replace(temporary, path)
    except BaseException:
        Path(temporary).unlink(missing_ok=True)
        raise
    return path
```

- [ ] **Step 4: Run the test and see it pass**

```bash
cd /home/ty/workspace/cachette/tools/direct-die/review && python -m pytest test_rules.py -q
```

Expected: 8 passed.

- [ ] **Step 5: Commit**

```bash
cd /home/ty/workspace/cachette
git add tools/direct-die/review/rules.py tools/direct-die/review/test_rules.py
git commit -F - <<'EOF'
Append an accepted rule to a style rules file

The analysis proposes a rule that the guide does not state. A person edits it
and accepts it, and this module appends it under one heading that it creates
once.

It does not try to find the section that the model named. A section name from
a model is a guess, and a wrong guess puts a rule where nobody reads it. A
person moves the line later.

The write is atomic, like every other write of this server.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_015NrPj9XQj2rz1SftMrjeF6
EOF
```

---

### Task 12: The like and refusal controls on the round page

**Files:**
- Modify: `tools/direct-die/review/app.py`
- Modify: `tools/direct-die/review/templates/round.html`
- Modify: `tools/direct-die/review/static/review.css`
- Modify: `tools/direct-die/review/test_review.py`

**Interfaces:**
- Consumes: `store.write_feedback` from Task 8, `Round.likes`, `.denies`, `.order`, `.note`, `.winner`, `.parents`.
- Produces: the route `POST /s/{asset}/{session_id}/{round_name}/feedback` takes these form fields:
  - `like` — a repeated field, one entry for each liked letter
  - `deny` — a repeated field, one entry for each refused letter
  - `order` — one string, the liked letters best first, separated by commas
  - `note` — one string, the standing note
  - `text` — one string, the note for this round

Each variant gets three radio buttons in one group named `mark-<letter>`: no mark, like, refuse. The server turns the three groups into the two lists. A radio group is the honest control, because a drawing is liked, refused, or neither, and those are exclusive.

- [ ] **Step 1: Write the failing test**

Append to `tools/direct-die/review/test_review.py`:

```python
# -- the round form ----------------------------------------------------------


def test_the_form_writes_the_likes_and_the_refusals(client: TestClient, root: Path) -> None:
    response = client.post(
        f"/s/{HEALTHY}/round-02/feedback",
        data={
            "mark-a": "deny",
            "mark-b": "like",
            "mark-c": "none",
            "mark-d": "like",
            "order": "d,b",
            "note": "keep the palette flat",
            "text": "raise the contrast",
        },
        follow_redirects=False,
    )
    assert response.status_code == 303
    asset, session_id = HEALTHY.split("/")
    written = json.loads(
        (root / asset / session_id / "round-02" / "feedback.json").read_text()
    )
    assert written["likes"] == ["b", "d"]
    assert written["denies"] == ["a"]
    assert written["order"] == ["d", "b"]
    assert written["note"] == "keep the palette flat"
    assert written["text"] == "raise the contrast"


def test_an_order_the_person_left_blank_takes_the_like_order(
    client: TestClient, root: Path
) -> None:
    client.post(
        f"/s/{HEALTHY}/round-02/feedback",
        data={"mark-b": "like", "mark-d": "like", "order": "", "note": "", "text": ""},
        follow_redirects=False,
    )
    asset, session_id = HEALTHY.split("/")
    written = json.loads(
        (root / asset / session_id / "round-02" / "feedback.json").read_text()
    )
    assert written["order"] == ["b", "d"]


def test_an_order_that_names_a_refused_letter_is_dropped(
    client: TestClient, root: Path
) -> None:
    client.post(
        f"/s/{HEALTHY}/round-02/feedback",
        data={"mark-a": "deny", "mark-b": "like", "order": "a,b", "note": "", "text": ""},
        follow_redirects=False,
    )
    asset, session_id = HEALTHY.split("/")
    written = json.loads(
        (root / asset / session_id / "round-02" / "feedback.json").read_text()
    )
    assert written["order"] == ["b"]
    assert written["denies"] == ["a"]


def test_a_form_that_marks_nothing_writes_two_empty_lists(
    client: TestClient, root: Path
) -> None:
    client.post(
        f"/s/{HEALTHY}/round-02/feedback",
        data={"order": "", "note": "", "text": "nothing yet"},
        follow_redirects=False,
    )
    asset, session_id = HEALTHY.split("/")
    written = json.loads(
        (root / asset / session_id / "round-02" / "feedback.json").read_text()
    )
    assert written["likes"] == []
    assert written["denies"] == []
    assert written["text"] == "nothing yet"


def test_the_round_page_shows_the_three_marks_for_each_variant(client: TestClient) -> None:
    response = client.get(f"/s/{HEALTHY}/round-01")
    assert response.status_code == 200
    for letter in ("a", "b", "c", "d"):
        assert f'name="mark-{letter}"' in response.text
    assert 'name="note"' in response.text
    assert 'name="order"' in response.text


def test_the_round_page_marks_the_saved_like_and_refusal(client: TestClient) -> None:
    response = client.get(f"/s/{HEALTHY}/round-01")
    assert "refused" in response.text.lower()
    assert "liked" in response.text.lower()


def test_the_round_page_names_the_parent_of_each_variant(client: TestClient) -> None:
    response = client.get(f"/s/{HEALTHY}/round-01")
    assert "round-00/variant-b" in response.text
    assert "round-00/variant-d" in response.text
```

- [ ] **Step 2: Run the test and see it fail**

```bash
cd /home/ty/workspace/cachette/tools/direct-die/review && python -m pytest test_review.py -q
```

Expected: FAIL. The route still takes `choice`, and the template still names `round.choice`.

- [ ] **Step 3: Write the route**

In `tools/direct-die/review/app.py`, replace `submit_feedback` with:

```python
    @app.post("/s/{asset}/{session_id}/{round_name}/feedback")
    async def submit_feedback(
        request: Request, asset: str, session_id: str, round_name: str
    ) -> Response:
        """Write the feedback of one round, then show the round again.

        Each variant carries one radio group named `mark-<letter>`, whose
        value is `like`, `deny` or `none`. A drawing is liked, refused, or
        neither, and those are exclusive, so a radio group is the honest
        control.

        The order field holds the liked letters, best first, separated by
        commas. A blank order takes the letters in the order of the page.
        """
        form = await request.form()
        likes: list[str] = []
        denies: list[str] = []
        for letter in VARIANT_LETTERS:
            mark = form.get(f"mark-{letter}")
            if mark == "like":
                likes.append(letter)
            elif mark == "deny":
                denies.append(letter)

        ranked = [
            item.strip()
            for item in str(form.get("order") or "").split(",")
            if item.strip() in likes
        ]
        ranked = list(dict.fromkeys(ranked))
        ranked.extend(letter for letter in likes if letter not in ranked)

        try:
            store.write_feedback(
                asset,
                session_id,
                round_name,
                likes,
                denies,
                ranked,
                str(form.get("note") or ""),
                str(form.get("text") or ""),
            )
        except ContractError as error:
            return page("error.html", request, message=str(error))
        target = f"/s/{asset}/{session_id}/{round_name}?saved=1"
        return no_store(RedirectResponse(target, status_code=303))
```

`Form` is no longer needed on this route. Leave the import if another route uses it; `/promote` and `/start` do.

Pass the analysis into the template. In `round_page`, the call already passes `round=current`, and `Round.analysis` reaches the template through it. Add nothing.

- [ ] **Step 4: Write the template**

In `tools/direct-die/review/templates/round.html`, replace the `Parent` row of the `dl.meta` block with nothing, and instead show the parent under each variant.

Replace the `<label class="pick">` block inside the variant section with:

```html
              <div class="mark">
                <span class="letter">{{ variant.letter }}</span>
                <label>
                  <input type="radio" name="mark-{{ variant.letter }}" value="like"
                    {{ 'checked' if variant.letter in round.likes else '' }}>
                  Liked
                </label>
                <label>
                  <input type="radio" name="mark-{{ variant.letter }}" value="deny"
                    {{ 'checked' if variant.letter in round.denies else '' }}>
                  Refused
                </label>
                <label>
                  <input type="radio" name="mark-{{ variant.letter }}" value="none"
                    {{ 'checked' if variant.letter not in round.likes and variant.letter not in round.denies else '' }}>
                  No mark
                </label>
              </div>
              {% if round.parents.get(variant.letter) %}
                <p class="parent">
                  Revised from
                  <a href="{{ base }}/{{ round.parents[variant.letter].split('/')[0] }}">{{ round.parents[variant.letter] }}</a>
                </p>
              {% endif %}
```

Change the `<section class="variant...">` class line to:

```html
            <section class="variant{{ ' picked' if round.winner == variant.letter else '' }}{{ ' refused' if variant.letter in round.denies else '' }}">
```

Replace the whole `<div class="submit">` block with:

```html
        <div class="submit">
          <label class="text">
            The order of the liked drawings, best first
            <input
              type="text"
              name="order"
              value="{{ round.order | join(',') }}"
              placeholder="d,b"
            >
            <span class="note">
              Leave it blank to take the order of the page. The first drawing
              stands for the asset in a pack.
            </span>
          </label>
          <label class="text">
            The standing note. Every later round of this session reads it.
            <textarea
              name="note"
              rows="3"
              placeholder="Say what holds for the whole session."
            >{{ round.note }}</textarea>
          </label>
          <label class="text">
            The note for the next round only
            <textarea
              name="text"
              rows="4"
              placeholder="Say what to keep and what to change."
            >{{ round.feedback_text }}</textarea>
          </label>
          <button type="submit">Write the feedback</button>
          {% if round.feedback is not none %}
            <p class="written">
              Written at {{ round.feedback.get("at") or "an unrecorded time" }}.
              Submitting again replaces it.
            </p>
          {% endif %}
        </div>
```

- [ ] **Step 5: Add the two styles**

Append to `tools/direct-die/review/static/review.css`:

```css
/* A refused drawing stays on the page. A person compares it with the drawing
   they liked, and the loop puts its source into the next prompt. */
.variant.refused {
  opacity: 0.55;
  border-color: #a33;
}

.variant .mark {
  display: flex;
  gap: 0.75rem;
  align-items: center;
  flex-wrap: wrap;
  font-size: 0.85rem;
}

.variant .parent {
  font-size: 0.8rem;
  opacity: 0.7;
  margin: 0.25rem 0 0;
}
```

- [ ] **Step 6: Run the whole front end tree and see it pass**

```bash
cd /home/ty/workspace/cachette/tools/direct-die/review && python -m pytest -q
```

Expected: every test passes, including the `test_frontend.py` tests that Task 8 left failing.

Search for any call site the sweep missed:

```bash
cd /home/ty/workspace/cachette && grep -rn "\.choice\b\|round\.parent\b\|name=\"choice\"" tools/direct-die
```

Expected: no output.

- [ ] **Step 7: Commit**

```bash
cd /home/ty/workspace/cachette
git add tools/direct-die/review/app.py tools/direct-die/review/templates/round.html tools/direct-die/review/static/review.css tools/direct-die/review/test_review.py
git commit -F - <<'EOF'
Give each variant a like, a refusal and no mark

The round page had one radio button for the whole round. Each variant now
carries its own radio group: liked, refused, or no mark. A drawing is one of
those three and never two, so a radio group is the honest control.

The page gains an order field for the liked letters, a standing note that
every later round reads, and the note for one round that it already had.

A refused drawing stays on the page at a lower opacity. A person compares it
with the drawing they liked, and the loop puts its source into the next
prompt.

Each variant now names the parent it revised, because a round can branch from
more than one drawing.

Searched the whole tool for the call sites of the field that went away:

    grep -rn "\.choice\b\|round\.parent\b\|name=\"choice\"" tools/direct-die

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_015NrPj9XQj2rz1SftMrjeF6
EOF
```

---

### Task 13: The analysis panel

**Files:**
- Modify: `tools/direct-die/review/app.py`
- Modify: `tools/direct-die/review/templates/round.html`
- Modify: `tools/direct-die/review/static/review.css`
- Modify: `tools/direct-die/review/test_frontend.py`

**Interfaces:**
- Consumes: `rules.append_rule` from Task 11, `Round.analysis` from Task 8, `RunManager.python` and `RunManager.tool_directory` from `runs.py`.
- Produces:
  - `app.analysis_command(python: str, style: str, session_id: str, index: int) -> list[str]`
  - `POST /s/{asset}/{session_id}/{round_name}/analyse`
  - `POST /rules/{style}/append`

**The analysis runs in the foreground, and this is a deliberate difference from a run.** A run draws up to eleven subjects at about seventy seconds each, so no page waits for it. The analysis is one model call. The person is looking at the page waiting for exactly that answer, and a job record for a ten second call would be a second state machine for no gain. The route runs the child with a timeout of 180 seconds and shows an error page when it fails.

The panel shows the preference, the order and the proposed rule. The preference and the order are not separate writes. The panel pre-fills the note field and the order field of the feedback form, and the person edits them and submits that one form. One value has one write path.

- [ ] **Step 1: Write the failing test**

Append to `tools/direct-die/review/test_frontend.py`:

```python
# -- the analysis ------------------------------------------------------------


def test_the_panel_says_so_when_no_analysis_exists(client: TestClient) -> None:
    response = client.get(f"/s/{CHOSEN}/round-00")
    assert response.status_code == 200
    assert "No analysis" in response.text


def test_the_panel_shows_the_preference_the_order_and_the_rule(
    client: TestClient,
) -> None:
    response = client.get(f"/s/{CHOSEN}/round-01")
    assert "three shapes and one flat fill" in response.text
    assert "Use three shapes or fewer inside the hexagon." in response.text
    assert 'value="d,b"' in response.text


def test_the_analysis_command_is_the_tool_command_line() -> None:
    found = app_module.analysis_command("python3", "cartoon", "forest-1", 2)
    assert found == [
        "python3", "-m", "direct_die", "analyse",
        "--asset", "cartoon", "--session", "forest-1", "--round", "2",
    ]


def test_the_analyse_route_runs_the_command(monkeypatch, client: TestClient) -> None:
    seen = {}

    class Done:
        returncode = 0
        stdout = "the analysis ranks d, b"
        stderr = ""

    def fake_run(command, **rest):
        seen["command"] = command
        seen["cwd"] = rest.get("cwd")
        return Done()

    monkeypatch.setattr(app_module.subprocess, "run", fake_run)
    response = client.post(f"/s/{CHOSEN}/round-01/analyse", follow_redirects=False)
    assert response.status_code == 303
    assert seen["command"][3] == "analyse"
    assert "--round" in seen["command"]


def test_the_analyse_route_shows_the_error_the_tool_printed(
    monkeypatch, client: TestClient
) -> None:
    class Failed:
        returncode = 4
        stdout = ""
        stderr = "analysis error: the analysis needs at least one liked drawing"

    monkeypatch.setattr(app_module.subprocess, "run", lambda command, **rest: Failed())
    response = client.post(f"/s/{CHOSEN}/round-01/analyse", follow_redirects=True)
    assert "at least one liked drawing" in response.text


def test_accepting_a_rule_appends_it_to_the_style(
    client: TestClient, paths: dict[str, Path]
) -> None:
    response = client.post(
        "/rules/cartoon/append",
        data={
            "rule": "Use three shapes or fewer inside the hexagon.",
            "back": f"/s/{CHOSEN}/round-01",
        },
        follow_redirects=False,
    )
    assert response.status_code == 303
    text = (paths["styleguide"] / "cartoon.md").read_text(encoding="utf-8")
    assert "- Use three shapes or fewer inside the hexagon." in text


def test_accepting_an_empty_rule_shows_an_error(client: TestClient) -> None:
    response = client.post(
        "/rules/cartoon/append",
        data={"rule": "   ", "back": f"/s/{CHOSEN}/round-01"},
        follow_redirects=True,
    )
    assert "empty" in response.text.lower()


def test_accepting_a_rule_for_a_style_with_no_file_shows_an_error(
    client: TestClient,
) -> None:
    response = client.post(
        "/rules/nosuchstyle/append",
        data={"rule": "Use three shapes or fewer.", "back": "/"},
        follow_redirects=True,
    )
    assert "no rules file" in response.text.lower()
```

The fixture names in `test_frontend.py` are `paths` (a dict with the keys `sessions`, `styleguide`, `packs` and `runs`) and `client`. The session constant `CHOSEN` is `"cartoon/20260904-090000-forest"`, and it holds `round-00` and `round-01`. Add `import app as app_module  # noqa: E402` beside the existing `from app import create_app` line.

Task 8 gave the `cartoon/20260904-090000-forest` session an analysis on `round-01`. It went into `build_style_sessions`, and `test_frontend.py` reads that tree. Confirm the analysis is there before you write these tests:

```bash
cd /home/ty/workspace/cachette/tools/direct-die/review && grep -n "analysis.json" make_fixtures.py
```

If Task 8 only wrote the analysis into the `test_review.py` tree, add the same block to `build_style_sessions` for `chosen / "round-01"`, and give that round the new feedback shape with `likes: ["b", "d"]`.

- [ ] **Step 2: Run the test and see it fail**

```bash
cd /home/ty/workspace/cachette/tools/direct-die/review && python -m pytest test_frontend.py -q -k "analysis or analyse or rule"
```

Expected: FAIL with `AttributeError: module 'app' has no attribute 'analysis_command'`.

- [ ] **Step 3: Write the routes**

In `tools/direct-die/review/app.py`, add `import subprocess` to the imports, and `import rules as rule_module` beside the other tool-directory imports.

Add the command builder beside `no_store`:

```python
# How long the analysis may take. It is one model call with up to eight
# pictures. A run of a whole subject takes about seventy seconds, and this is
# smaller than that.
ANALYSIS_TIMEOUT_SECONDS = 180.0


def analysis_command(
    python: str, style: str, session_id: str, index: int
) -> list[str]:
    """Build the command line that analyses one round.

    The server calls the tool this way and no other way. The two halves of
    the tool share a directory layout, and nothing else.
    """
    return [
        python,
        "-m",
        "direct_die",
        "analyse",
        "--asset",
        style,
        "--session",
        session_id,
        "--round",
        str(index),
    ]
```

Add the two routes below `submit_feedback`:

```python
    @app.post("/s/{asset}/{session_id}/{round_name}/analyse")
    def start_analysis(
        request: Request, asset: str, session_id: str, round_name: str
    ) -> Response:
        """Analyse one round, and show the round again.

        This waits, and a run does not. A run draws up to eleven subjects at
        about seventy seconds each, so no page waits for it. The analysis is
        one model call, and the person is looking at the page waiting for
        exactly that answer. A job record for a call this short would be a
        second state machine for no gain.
        """
        try:
            index = store.round_index(round_name)
        except ContractError as error:
            return page("error.html", request, message=str(error))
        try:
            done = subprocess.run(
                analysis_command(runner.python, asset, session_id, index),
                cwd=str(runner.tool_directory),
                capture_output=True,
                text=True,
                timeout=ANALYSIS_TIMEOUT_SECONDS,
            )
        except subprocess.TimeoutExpired:
            return page(
                "error.html",
                request,
                message=(
                    "The analysis did not answer in "
                    f"{int(ANALYSIS_TIMEOUT_SECONDS)} seconds. The endpoint "
                    "may be busy. Ask again."
                ),
            )
        except OSError as error:
            return page("error.html", request, message=f"the tool did not start: {error}")
        if done.returncode != 0:
            message = (done.stderr or done.stdout or "").strip()
            return page(
                "error.html",
                request,
                message=message or f"the analysis failed with code {done.returncode}",
            )
        target = f"/s/{asset}/{session_id}/{round_name}?analysed=1"
        return no_store(RedirectResponse(target, status_code=303))

    @app.post("/rules/{style}/append")
    def append_rule(
        request: Request,
        style: str,
        rule: str = Form(default=""),
        back: str = Form(default="/"),
    ) -> Response:
        """Append one accepted rule to a style rules file."""
        try:
            rule_module.append_rule(styleguide_root, style, rule)
        except rule_module.RuleError as error:
            return page("error.html", request, message=str(error))
        target = back if back.startswith("/") else "/"
        joiner = "&" if "?" in target else "?"
        return no_store(RedirectResponse(f"{target}{joiner}ruled=1", status_code=303))
```

`styleguide_root` is the name that `create_app` binds. The `/promote` route already closes over it.

In `round_page`, pass the two new banner flags:

```python
            analysed=request.query_params.get("analysed") == "1",
            ruled=request.query_params.get("ruled") == "1",
```

- [ ] **Step 4: Write the panel**

In `tools/direct-die/review/templates/round.html`, add the two banners beside the `saved` banner:

```html
      {% if analysed %}
        <p class="saved">The analysis is written. It is below the drawings.</p>
      {% endif %}
      {% if ruled %}
        <p class="saved">
          The rule is now in the style guide of {{ session.asset }}. The next
          run reads it.
        </p>
      {% endif %}
```

Add the panel below the feedback form and above the promote card:

```html
      <section class="card analysis">
        <h2>What the liked drawings share</h2>
        <p class="note">
          The analysis compares the drawings you liked with the drawings you
          refused. It runs one model call and takes about half a minute. Mark
          at least one drawing as liked before you ask for it.
        </p>
        <form method="post" action="{{ base }}/{{ round.name }}/analyse" class="inline">
          <button type="submit">Analyse this round</button>
        </form>

        {% if round.analysis is none %}
          <p class="gap">No analysis yet for this round.</p>
        {% else %}
          <p class="preference">{{ round.analysis.get("preference") }}</p>

          {% if round.analysis.get("order") %}
            <h3>The order it proposes</h3>
            <ol class="ranking">
              {% for letter in round.analysis["order"] %}
                <li>
                  <strong>{{ letter }}</strong>
                  {{ round.analysis.get("reasons", {}).get(letter, "") }}
                </li>
              {% endfor %}
            </ol>
            <p class="note">
              Copy this into the order field above to use it. The form is the
              one place that writes the order.
            </p>
          {% endif %}

          {% if round.analysis.get("guide_edit") %}
            <h3>The rule it proposes</h3>
            <p class="note">
              This goes into the style guide of {{ session.asset }} under one
              heading at the end of the file. Edit it before you accept it.
              Every later run of every subject reads it.
            </p>
            <form method="post" action="/rules/{{ session.asset }}/append">
              <input type="hidden" name="back" value="{{ base }}/{{ round.name }}">
              <input
                type="text"
                name="rule"
                class="wide"
                value="{{ round.analysis['guide_edit'].get('rule', '') }}"
              >
              <button type="submit" class="small">Accept the rule</button>
            </form>
          {% endif %}
        {% endif %}
      </section>
```

The panel pre-fills nothing by itself. Add a small script at the end of the `script` block that copies the analysis order into the order field when a person clicks a button:

```html
    {% if round.analysis and round.analysis.get("order") %}
    <script>
      (function () {
        var field = document.querySelector('input[name="order"]');
        var note = document.querySelector('textarea[name="note"]');
        var button = document.getElementById("take-analysis");
        if (!button) { return; }
        button.addEventListener("click", function () {
          field.value = {{ (round.analysis["order"] | join(",")) | tojson }};
          note.value = {{ round.analysis.get("preference", "") | tojson }};
        });
      })();
    </script>
    {% endif %}
```

Add the button inside the panel, above the ranking list:

```html
            <button type="button" id="take-analysis" class="small">
              Take this order and this preference into the form above
            </button>
```

- [ ] **Step 5: Add the styles**

Append to `tools/direct-die/review/static/review.css`:

```css
.analysis .preference {
  font-size: 1.05rem;
  line-height: 1.5;
  margin: 0.75rem 0;
}

.analysis .ranking {
  margin: 0.5rem 0 0.75rem 1.25rem;
}

.analysis input.wide {
  width: 100%;
  box-sizing: border-box;
  margin: 0.4rem 0;
}
```

- [ ] **Step 6: Run the whole front end tree and see it pass**

```bash
cd /home/ty/workspace/cachette/tools/direct-die/review && python -m pytest -q
```

Expected: every test passes.

- [ ] **Step 7: Commit**

```bash
cd /home/ty/workspace/cachette
git add tools/direct-die/review/app.py tools/direct-die/review/templates/round.html tools/direct-die/review/static/review.css tools/direct-die/review/test_frontend.py
git commit -F - <<'EOF'
Show what the liked drawings share, and accept the rule it proposes

The round page gains an analysis panel. It runs the analyse command of the
tool, shows the preference statement, the order it proposes and the rule it
proposes, and lets a person accept the rule into the style guide.

The analysis waits and a run does not. A run draws up to eleven subjects at
about seventy seconds each, so no page waits for it. The analysis is one
model call, and the person is looking at the page waiting for that answer. A
job record for a call this short would be a second state machine for no gain.

The panel writes no feedback. It copies its order and its preference into the
one form that writes them, so each value keeps one write path.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_015NrPj9XQj2rz1SftMrjeF6
EOF
```

---

### Task 14: The documents state the new contracts

**Files:**
- Modify: `tools/direct-die/README.md`
- Modify: `tools/direct-die/review/README.md`

**Interfaces:**
- Consumes: everything above.
- Produces: no code.

Three statements in the two READMEs are now false. Repair them, and add what is new.

- [ ] **Step 1: Find every false statement**

```bash
cd /home/ty/workspace/cachette && grep -rn "choice\|changes no rules file\|The human outranks the model\|feedback.json" tools/direct-die/README.md tools/direct-die/review/README.md
```

Read each hit and decide whether it is still true.

- [ ] **Step 2: Repair the tool README**

In `tools/direct-die/README.md`:

Replace the section "The human outranks the model" with:

```markdown
## The human outranks the model

The review interface writes `feedback.json` into a round directory. This
tool reads that file and never writes it.

The file holds four things. It names the drawings that a person liked, the
drawings they refused, an order between the liked drawings, and two notes.

Every liked drawing becomes a parent of the next round. The round cycles the
parents across the four variant lenses, so each parent gets a spread of
directions. One like gives every lens that one parent.

Every refused drawing goes into the next prompt as SVG source, under a
heading that says not to draw like it. The refusal holds for the whole
session, and the prompt carries the two newest.

The standing note holds for the whole session. The round note holds for the
next round only. Both go above the model critique, and the prompt says that
the human direction outranks every model note.

The file names the round that it belongs to, by index. This tool refuses the
file when that index disagrees with the directory, because a mismatch means
one of the two is wrong, and neither is safe to guess from.

When nobody liked anything, the parent is the highest scoring variant of
every round so far, and not only of the last round. A critique names a fault
even in a good drawing, and a revision that acts on that fault can make the
drawing worse. The loop must not walk away from its best work when that
happens. A later round wins a tie, so the loop still moves. A refused drawing
is never the parent, whatever it scored.
```

Add a section after it:

```markdown
## The analysis

The analysis compares what a person liked with what they refused. It runs
between two rounds, and not inside one.

```
python3 -m direct_die analyse --asset hex-tile --session 20260906-190000 --round 1
```

It shows the model the display render of each liked drawing and of each
refused drawing, beside the rules. It answers with three things: a sentence
that says what the liked drawings share, an order between the liked drawings,
and one rule that the guide does not state. It writes `analysis.json` into
the round directory.

It gives an order and not a score. A large change to a drawing moves the
absolute score by a few points, which the known limits below record. A
comparison of two drawings does not have that defect. The order does not
replace the score. The score still decides which drawing wins when no person
chose.
```

Update the on-disk layout block. Add `analysis.json` beside `feedback.json`, and say which side writes each one.

- [ ] **Step 3: Repair the front end README**

In `tools/direct-die/review/README.md`:

Replace item 2 of "The five things a person does":

```markdown
2. **Steer.** Read the four variants of a round side by side, read the
   critique the model wrote for each one, and mark each drawing as liked,
   refused, or neither. Order the liked ones. Type a standing note for the
   session and a note for the next round. Ask for an analysis of what the
   liked drawings share. The server writes `feedback.json`, and the loop
   reads it on its next round.
```

Replace the paragraph in "Which drawing stands for an asset" that says "takes the first human choice it finds" with:

```markdown
The person outranks the model. The rule reads the sessions of one style and
one asset, newest session first, and takes the drawing that a person put
first in the order of a round. When no person ordered anything, it takes the
highest score of every round of every session, and a later round wins a tie.
A refused drawing never stands for an asset, whatever the model scored it.
```

Replace the "What this server writes" list:

```markdown
- `feedback.json` in a round directory.
- A job record and its logs under the runs directory.
- An exemplar SVG under `styleguide/exemplars/<style>/`, when a person
  promotes a drawing.
- One rule appended to `styleguide/<style>.md`, when a person accepts a rule
  that the analysis proposed. It goes under one heading at the end of the
  file, which the server creates once.
- A pack under the packs directory.

It writes nothing else. It changes no drawing, and it writes no
`analysis.json`. The tool writes that file.
```

Update the on-disk contract block to hold the new `feedback.json` shape and `analysis.json`.

Add a section:

```markdown
## The analysis waits, and a run does not

A run draws up to eleven subjects at about seventy seconds each, so no page
waits for it. A job record holds its state.

The analysis is one model call. A person clicks the button and looks at the
page until the answer arrives, so the request waits for the child and shows
the answer. A job record for a call this short would be a second state
machine for no gain. The request gives up after 180 seconds and says so.
```

Add `rules.py` and `test_readers_agree.py` to the file table at the end.

- [ ] **Step 4: Check the prose**

Read both files once for the documentation rule: short sentences, active voice, one idea per sentence, no metaphor, and every external reference in a footnote.

- [ ] **Step 5: Run every test one last time**

```bash
cd /home/ty/workspace/cachette/tools/direct-die && python -m pytest tests -q
cd /home/ty/workspace/cachette/tools/direct-die/review && python -m pytest -q
cd /home/ty/workspace/cachette/tools/direct-die/review && python -m ruff check .
```

Expected: every test passes, and ruff reports nothing.

- [ ] **Step 6: Commit**

```bash
cd /home/ty/workspace/cachette
git add tools/direct-die/README.md tools/direct-die/review/README.md
git commit -F - <<'EOF'
Say what the two halves of the tool now write and read

Three statements were false after the taste loop landed. The feedback file no
longer holds one chosen letter. The server now writes into a style rules
file. The rule for which drawing stands for an asset changed.

Searched both documents for the false statements:

    grep -rn "choice\|changes no rules file\|feedback.json" \
        tools/direct-die/README.md tools/direct-die/review/README.md

Added the analysis command, the new feedback shape, and the reason the
analysis waits while a run does not.

Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_015NrPj9XQj2rz1SftMrjeF6
EOF
```

---

## The end state

After Task 14 a person can do this in a browser.

1. Start a run for one asset or for the whole set.
2. Read the four variants of a round, with the critique of each one.
3. Mark each drawing as liked, refused, or neither, and order the liked ones.
4. Ask for an analysis, and read what the liked drawings share.
5. Take the order and the preference into the form, edit them, and save.
6. Accept the proposed rule into the style guide.
7. Add a round, which branches from every liked drawing and refuses every
   refused one.
8. Promote a drawing to an exemplar, and export the style as a pack.

Three pieces of the original request stay open, and each has its own design
ahead of it: settings and prompt tuning in the browser, the composited
preview, and numbered pack snapshots.
