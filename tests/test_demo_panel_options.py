"""The command line names a panel of the deck and names a tile.

The deck of panels is drawn by the viewer, and a watcher reaches it with a
function key. A run without a watcher had no way to ask for a panel, so no
check outside a window ever drew one. These tests drive the command line, which
is the caller a person uses.

Each test asks the demonstration for one picture and reads what it printed.
Nothing here loops over a tile or an entity.

References
----------
Testing Rules, section 5. ``.agents/rules/testing.md``
"""

from __future__ import annotations

from pathlib import Path

import pytest

from cachette import World
from cachette.demo.app import chosen_panels, chosen_tile, main

# A small world keeps the run short. The panels read what the engine holds,
# and they hold the same rows at any extent.
SMALL = ["--extent", "48", "--factions", "2", "--ticks", "12", "--seed", "0x51"]


def test_the_named_panels_are_the_panels_the_engine_holds() -> None:
    """A name the engine published is accepted, in the order it was given."""
    assert chosen_panels("events,statistics") == ["events", "statistics"]
    assert chosen_panels(" characters , inspector ") == ["characters", "inspector"]
    assert chosen_panels("") == []


def test_a_panel_the_engine_does_not_hold_is_refused_by_name() -> None:
    """A name no panel carries stops the run, and the message names the deck."""
    with pytest.raises(ValueError, match="no panel is called 'nosuch'") as refusal:
        chosen_panels("statistics,nosuch")
    for name in World.panel_names():
        assert name in str(refusal.value)


def test_a_tile_is_two_numbers_and_an_empty_text_names_none() -> None:
    """The pointer takes an address, and no text means nobody pointed."""
    assert chosen_tile("12,-3") == (12, -3)
    assert chosen_tile("") is None
    with pytest.raises(ValueError, match="two numbers"):
        chosen_tile("12")


def picture_size(path: Path) -> tuple[int, int]:
    """Give back the width and the height a PNG states in its header."""
    header = path.read_bytes()[16:24]
    return (int.from_bytes(header[:4], "big"), int.from_bytes(header[4:], "big"))


def test_the_named_panel_reaches_the_picture(tmp_path: Path) -> None:
    """The name a watcher gives chooses which panel the frame draws.

    Two runs of one world name one panel, and they give one file twice. A
    third run names another panel and gives a different file.

    The first assertion is what makes the second one mean something. A run
    that takes the name and draws the whole panel gives a file that holds the
    wall clock, so two such runs differ whatever the name was.
    """
    counts = tmp_path / "counts.png"
    again = tmp_path / "again.png"
    people = tmp_path / "people.png"
    assert main([*SMALL, "--picture", str(counts), "--panels", "statistics"]) == 0
    assert main([*SMALL, "--picture", str(again), "--panels", "statistics"]) == 0
    assert main([*SMALL, "--picture", str(people), "--panels", "characters"]) == 0
    assert counts.read_bytes() == again.read_bytes()
    assert picture_size(counts) == picture_size(people)
    assert counts.read_bytes() != people.read_bytes()


def test_the_named_tile_reaches_the_tile_panel(tmp_path: Path) -> None:
    """The tile panel reads the address the watcher named.

    Two runs name one tile and give one file twice. A third run names another
    tile, and the panel states another ground, another stock and another
    holder, so the file differs.
    """
    here = tmp_path / "here.png"
    again = tmp_path / "again.png"
    there = tmp_path / "there.png"
    named = ["--panels", "inspector", "--point"]
    assert main([*SMALL, "--picture", str(here), *named, "8,8"]) == 0
    assert main([*SMALL, "--picture", str(again), *named, "8,8"]) == 0
    assert main([*SMALL, "--picture", str(there), *named, "20,17"]) == 0
    assert here.read_bytes() == again.read_bytes()
    assert here.read_bytes() != there.read_bytes()


def test_a_picture_of_a_deck_keeps_the_height_of_a_window(tmp_path: Path) -> None:
    """A deck cuts itself to the frame, so the picture does not grow.

    A picture of the whole panel grows to the height the panel needs, which is
    taller than a window a watcher opens.
    """
    plain = tmp_path / "plain.png"
    deck = tmp_path / "deck.png"
    main([*SMALL, "--picture", str(plain)])
    main([*SMALL, "--picture", str(deck), "--panels", "statistics"])
    assert picture_size(deck)[1] == 720
    assert picture_size(plain)[1] > 720


def test_a_refused_panel_name_stops_the_run_before_it_steps(
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    """A bad name gives a message and no file."""
    picture = tmp_path / "none.png"
    assert main([*SMALL, "--picture", str(picture), "--panels", "nosuch"]) == 2
    assert "no panel is called 'nosuch'" in capsys.readouterr().out
    assert not picture.exists()
