"""A seat configuration must reach the engine, and a close pair must stay level.

The tool under test plays versions of the built-in controller against each
other in one world. A version is a seat configuration: the five option weights
of a faction, the hunting ratio of that faction, and whether an external caller
holds the seat. Every one of those values belongs to one faction, so two
versions take two seats of one world.[^1]

Two failures would each produce a report that reads like a ranking, and neither
would raise.

**The first failure is a seating that writes nothing.** The engine draws a
weight vector for every faction from the seed, so a world whose seats were
never written still holds five plausible weights for each of them. A tool that
dropped the write would then rank three drawn vectors under three names. The
test that pins this reads every setting back after the world is built. The test
beside it runs the same check against a world nothing wrote to, and asserts
that the check refuses it. **That second test is the proof that the first is
not vacuous.**

**The second failure is an order the sample cannot hold.** A win share near one
third carries a standard error near 0.059 over 64 games. Two versions three
hundredths apart are one reading and not two, and a table sorted by win share
states an order for them anyway. The tests here build pairs by hand at both
extremes: a pair one version won five times out of six, and a pair that parted
by one game in thirty.

# The fixtures supply extremes, and none of them is a typical run

Every synthetic game in this file is built by hand. The cases here hold, on
purpose:

- a pair that one version won almost every shared game of
- a pair that parted by one game, which no sample of that size can order
- a pair that decided two games between them, where the clearance alone would
  state an order
- a pair in which neither version won any shared game, where the gap and the
  error are both zero
- a version that won no game at all, which a standard error alone gives a
  width of zero

References
----------
[^1]: Report 44, a family of tunable controllers, and how to rank them.
``docs/research/reports/44-a-family-of-tunable-controllers.md``
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path
from types import ModuleType
from typing import Any

import pytest

from cachette import World

ROOT = Path(__file__).resolve().parent.parent

# The seat count of the world the tool plays. Three is what the engine world
# holds, and the tool reads its own count from the training world.
SEATS = 3

# A seed the tool would play. It sits above the held-out start of the league
# tool, so it is a world no training run learned on.
SEED = 50_000

# How many games the synthetic pairs hold. Thirty is enough that a gap of one
# game is far under the error, and small enough to read.
SHARED_GAMES = 30


def _load(name: str) -> ModuleType:
    """Import a script by path, because `scripts/` is not a package."""
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / f"{name}.py")
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


tool = _load("controller_versions")


def _game(index: int, players: tuple[int, ...], winner: int | None) -> object:
    """Build one finished game by hand, so a test states the case it needs."""
    return tool.GameResult(
        index=index,
        world=index // SEATS,
        seed=SEED + index // SEATS,
        rotation=index % SEATS,
        versions=players,
        winner=winner,
        path="renown" if winner is not None else tool.NO_WINNER,
        end_tick=2500,
        reached_limit=False,
    )


def _pair(first_wins: int, second_wins: int) -> list[object]:
    """Build a set of shared games two versions and a third player sat in.

    The third player wins every game neither of the two won, so every game
    holds a winner. The two versions under test hold seats 0 and 1.
    """
    winners = [0] * first_wins + [1] * second_wins
    winners += [2] * (SHARED_GAMES - len(winners))
    return [_game(index, (0, 1, 2), winner) for index, winner in enumerate(winners)]


def _assert_seated(world: World, seated: list[Any]) -> None:
    """Assert that each seat of a built world holds the version that took it.

    This reads every setting a version writes: the weight vector, the hunting
    ratio and the external control flag. A version that names no weight vector
    keeps the vector the seeding drew, so this asserts nothing about it.
    """
    for seat, version in enumerate(seated):
        if version.weights is not None:
            assert world.faction_weights(seat) == version.weights.as_dict(), (
                f"seat {seat} does not hold the weights of {version.name}"
            )
        if version.ratio is not None:
            assert int(world.faction_overmatch_ratio(seat)) == version.ratio, (
                f"seat {seat} does not hold the ratio of {version.name}"
            )
        assert world.is_externally_controlled(seat) is version.external, (
            f"seat {seat} does not hold the control flag of {version.name}"
        )


def test_the_family_holds_every_variant_the_design_report_names() -> None:
    """The tool seats every member of the family the report proposes."""
    named = [
        "mute",
        "quietist",
        "settler",
        "mason",
        "default",
        "warlord",
        "hunter",
    ]
    assert list(tool.FAMILY) == named
    assert [version.name for version in tool.family_versions(named)] == named


def test_a_name_the_family_lacks_is_refused() -> None:
    """A misspelled member names no version, and the refusal lists the family."""
    with pytest.raises(ValueError, match="names no member of the family"):
        tool.family_versions(["masson"])


def test_a_built_world_holds_the_settings_of_the_version_in_each_seat() -> None:
    """Seat three variants in one world, and read every setting back.

    **This is the test that proves the seating reaches the engine.** It goes
    through the same builder the schedule uses, so nothing here drives a
    mechanism the tool does not drive itself.
    """
    seated = tool.family_versions(["warlord", "mason", "mute"])
    world = tool.build_world(SEED, seated)
    _assert_seated(world, seated)


def test_every_member_of_the_family_reaches_a_seat() -> None:
    """Seat every member of the family, three at a time, and read it back.

    A world holds three seats, so no one world holds the whole family. This
    walks the family in groups of three, and it starts each group at a
    different member, so every member takes every seat over the walk.
    """
    named = list(tool.FAMILY)
    for start in range(len(named)):
        group = [named[(start + step) % len(named)] for step in range(SEATS)]
        seated = tool.family_versions(group)
        _assert_seated(tool.build_world(SEED + start, seated), seated)


def test_two_variants_of_one_world_hold_two_weight_vectors() -> None:
    """Two seats of one world hold the two vectors the schedule gave them."""
    seated = tool.family_versions(["warlord", "mason", "mute"])
    world = tool.build_world(SEED, seated)
    assert world.faction_weights(0) != world.faction_weights(1)
    assert world.faction_weights(0)["war"] == 8
    assert world.faction_weights(1)["build"] == 8


def test_the_seating_check_refuses_a_world_that_nothing_wrote_to() -> None:
    """Prove the seating test can fail: skip the write and the check refuses.

    The world here is seeded and never written to, which is what the tool would
    build if the seating step were dropped. Every faction of it still holds a
    drawn weight vector, so only a read-back of the named values finds the
    failure.
    """
    seated = tool.family_versions(["warlord", "mason", "mute"])
    world = tool.build_world(SEED, [])
    with pytest.raises(AssertionError):
        _assert_seated(world, seated)


def test_a_pair_that_differs_only_in_an_inert_weight_is_refused() -> None:
    """Two versions that steer alike are one player twice, and the tool says so.

    No decision that changes the world reads the trade weight or the renown
    weight. A ranking that parted two such versions would measure noise.
    """
    one = tool.Version(
        name="one", weights=tool.Weights(war=4, trade=1, build=4, renown=1, settle=4)
    )
    other = tool.Version(
        name="other", weights=tool.Weights(war=4, trade=8, build=4, renown=8, settle=4)
    )
    with pytest.raises(ValueError, match="steers the world exactly as"):
        tool.require_steering_apart([one, other])


def test_a_pair_that_differs_in_a_steering_weight_is_allowed() -> None:
    """A build weight reaches a decision, so two such versions are two players."""
    one = tool.Version(
        name="one", weights=tool.Weights(war=4, trade=1, build=4, renown=1, settle=4)
    )
    other = tool.Version(
        name="other", weights=tool.Weights(war=4, trade=1, build=8, renown=1, settle=4)
    )
    tool.require_steering_apart([one, other])


def test_two_versions_under_one_name_are_refused() -> None:
    """A report names a row by its version, so one name cannot hold two rows."""
    with pytest.raises(ValueError, match="two versions carry one name"):
        tool.require_named_apart(
            [tool.Version(name="one"), tool.Version(name="one", ratio=0)]
        )


def test_a_win_share_carries_the_standard_error_of_a_share() -> None:
    """The error of a share near one third over 64 games is near 0.059."""
    assert tool.share_error(21, 64) == pytest.approx(0.0589, abs=5e-4)
    assert tool.share_error(0, 0) == 0.0


def test_a_version_that_won_nothing_still_carries_a_width() -> None:
    """A share of zero has a standard error of zero, and an interval of width.

    The error alone would report a passive seat as an exact reading. The
    interval is the figure to read for it.
    """
    assert tool.share_error(0, 64) == 0.0
    interval = tool.share_interval(0, 64)
    assert interval["low"] == 0.0
    assert interval["high"] > 0.0


def test_a_wide_gap_parts_a_pair() -> None:
    """One version won 25 of 30 shared games, and the sample orders the pair."""
    found = tool.paired_gap(_pair(25, 0), 0, 1)
    assert found["games"] == SHARED_GAMES
    assert found["separated"] is True


def test_a_gap_of_one_game_parts_nothing() -> None:
    """A pair that parted by one game in thirty stays level."""
    found = tool.paired_gap(_pair(11, 10), 0, 1)
    assert found["gap"] == pytest.approx(1 / SHARED_GAMES)
    assert found["separated"] is False


def test_a_pair_that_won_no_shared_game_parts_nothing() -> None:
    """Both versions lost every shared game, so the gap and the error are zero.

    A rule that read a gap above zero as a separation would order this pair,
    because the gap is not above zero and the error is not either.
    """
    found = tool.paired_gap(_pair(0, 0), 0, 1)
    assert found["gap"] == 0.0
    assert found["error"] == 0.0
    assert found["separated"] is False


def test_a_run_of_three_games_parts_nothing() -> None:
    """A pair that decided two games between them stays level.

    The gap here is two thirds and the error is 0.272, so the clearance alone
    would part the pair. **A run made to prove the tool would then read as a
    ranking of the family.** The least decided count is what refuses it.
    """
    shared = [
        _game(0, (0, 1, 2), 1),
        _game(1, (0, 1, 2), 1),
        _game(2, (0, 1, 2), 2),
    ]
    found = tool.paired_gap(shared, 0, 1)
    assert found["decided"] == 2
    assert abs(found["gap"]) > tool.CLEARANCE * found["error"]
    assert found["separated"] is False


def test_the_order_names_the_versions_a_sample_cannot_part() -> None:
    """The order block states the ties, so a rank alone implies no order."""
    versions = [tool.Version(name="one"), tool.Version(name="other")]
    results = _pair(11, 10)
    rows = tool.version_rows(versions, results)
    order = tool.order_rows(versions, results, rows)
    assert [row["name"] for row in order] == ["one", "other"]
    assert order[0]["not_separated_from"] == ["other"]
    assert order[1]["not_separated_from"] == ["one"]
    assert rows[0]["win_share_error"] > 0.0


def test_the_separation_block_holds_one_row_for_each_pair() -> None:
    """Every pair of versions gets a row, with the gap and the verdict."""
    versions = [tool.Version(name="one"), tool.Version(name="other")]
    rows = tool.separation_rows(versions, _pair(25, 0))
    assert len(rows) == 1
    assert rows[0]["versions"] == ["one", "other"]
    assert rows[0]["shared_games"] == SHARED_GAMES
    assert rows[0]["separated"] is True


def test_a_version_row_counts_the_paths_it_played_and_the_paths_it_won() -> None:
    """The wonder hypothesis reads the endings, and the win share alone cannot."""
    versions = [tool.Version(name="one"), tool.Version(name="other")]
    rows = tool.version_rows(versions, _pair(2, 1))
    assert rows[0]["paths_played"]["renown"] == SHARED_GAMES
    assert rows[0]["paths_won"] == {"renown": 2}
    assert rows[1]["paths_won"] == {"renown": 1}


def test_the_hunting_ratio_of_one_is_a_whole_number_of_engine_units() -> None:
    """The parity ratio this file names sits on the scale the engine holds.

    The engine gives every new faction a ratio that is a whole multiple of the
    unit. A parity ratio written at another scale would fail here.
    """
    assert tool.default_ratio() % tool.RATIO_ONE == 0
