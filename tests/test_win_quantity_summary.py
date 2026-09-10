"""A distribution tool must state the spread, the leader and the arrival right.

The tool under test reads the trajectories one probe recorded and states the
quantiles a win threshold would be set against.[^1] Three failures would each
give a report that reads like a measurement, and none of them would raise.

**The first failure is a leader that is really a seat.** Every summary here
reports the seat value beside the leading value, and the leading value is the
highest over the factions of the game. A tool that read faction zero would
report the seat twice under two names. The fixture for this holds a game whose
faction zero never leads any quantity, so a reader of the seat gives a
different answer from a reader of the leader. **A fixture whose faction zero
happens to lead would pass either way**, so the fixture states the extreme on
purpose.

**The second failure is an arrival that reads a constant.** The arrival figure
says how far into a game a quantity first stood at nine tenths of its end
value. A quantity that never moves stands at its end value from the first
sample, so it reads an arrival of zero. That reading is right and it is not a
measurement of saturation, and the fixtures hold the case so that a reader of
the report knows to expect it. A series that ends at zero has no arrival at
all.

**The third failure is a trend that reads past the end of a game.** The trend
table reads each game at fixed ticks. A game shorter than the tick must
contribute nothing there, and a tool that took the last sample instead would
report a finished game as if it stood at every later tick.

# The fixtures supply extremes, and none of them is a typical game

Every game in this file is built by hand. The cases hold, on purpose: a
quantity that rises to its end value only at the last tenth, a quantity that
arrives in the first fifth and then stands still, a quantity that never
moves, a quantity that ends at zero, a game that ends long before the trend
table stops, and a set of one number.

References
----------
[^1]: The trajectory summariser. ``scripts/win_quantity_summary.py``
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path
from types import ModuleType
from typing import Any

import pytest

ROOT = Path(__file__).resolve().parent.parent

# The seat count of the games the fixtures build. Three is what the training
# world holds.
SEATS = 3

# How many episodes the standard error case names. A share near one third over
# sixty-four games is the figure the project rules quote, and this pins it.
EPISODES = 64


def _script(name: str) -> ModuleType:
    """Import a script of the repository, because the scripts are no package."""
    path = ROOT / "scripts" / f"{name}.py"
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:  # pragma: no cover - import contract
        message = f"cannot import the script at {path}"
        raise ImportError(message)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


summary = _script("win_quantity_summary")


def a_game(
    *,
    field: str = "calm",
    path: str = "wonder",
    end_tick: int = 1000,
    seats: tuple[str, ...] = ("quietist", "settler", "mason"),
    winner: str | None = "mason",
    series: dict[str, list[list[float]]],
    ticks: list[int],
) -> dict[str, Any]:
    """Build one game whose factions follow the series the caller states.

    Each entry of the series names a quantity and gives one row for each
    sample, and each row holds one value for each faction in seat order.
    """
    samples = [
        {
            "tick": tick,
            "shares": {name: rows[index] for name, rows in series.items()},
            "counts": {name: rows[index] for name, rows in series.items()},
        }
        for index, tick in enumerate(ticks)
    ]
    return {
        "field": field,
        "path": path,
        "end_tick": end_tick,
        "reached_limit": False,
        "seats": list(seats),
        "winner_seat": None if winner is None else list(seats).index(winner),
        "winner": winner,
        "samples": samples,
        "index": 0,
        "world": 0,
        "seed": 50_000,
        "rotation": 0,
    }


def test_spread_states_the_quantiles_and_the_highest_value() -> None:
    """The spread of a known set holds its deciles, its median and its top."""
    found = summary.spread([float(value) for value in range(11)])
    assert found["count"] == 11
    assert found["p10"] == pytest.approx(1.0)
    assert found["median"] == pytest.approx(5.0)
    assert found["p90"] == pytest.approx(9.0)
    assert found["max"] == pytest.approx(10.0)


def test_spread_of_no_number_states_a_count_of_zero() -> None:
    """An empty set reads zero everywhere, and the count says it was empty."""
    found = summary.spread([])
    assert found["count"] == 0
    assert found["median"] == pytest.approx(0.0)


def test_the_standard_error_of_a_third_over_sixty_four_games() -> None:
    """A share near one third over sixty-four games carries an error of 0.059."""
    assert summary.share_error(EPISODES // 3, EPISODES) == pytest.approx(
        0.059, abs=5e-4
    )


def test_the_error_of_a_share_of_zero_is_zero_and_too_narrow() -> None:
    """A share of zero gives an error of zero, which the report must not read."""
    assert summary.share_error(0, EPISODES) == pytest.approx(0.0)


def test_the_leader_is_the_highest_faction_and_never_the_seat() -> None:
    """A game whose faction zero never leads parts the leader from the seat.

    The rows rise for every faction, and faction zero holds the lowest value
    at every sample. A reader of faction zero would answer 3.0 at the end, and
    the leader answers 9.0.
    """
    game = a_game(
        series={"held_tiles": [[1.0, 2.0, 3.0], [2.0, 5.0, 6.0], [3.0, 7.0, 9.0]]},
        ticks=[0, 500, 1000],
    )
    assert summary.leader_series(game, "counts", "held_tiles") == [3.0, 6.0, 9.0]
    assert summary.own_values(game, "counts", "held_tiles") == [3.0, 7.0, 9.0]


def test_a_quantity_that_arrives_late_reads_a_late_arrival() -> None:
    """A series that reaches nine tenths only at the last sample reads near one."""
    ticks = list(range(0, 1001, 100))
    rising = [[float(tick) / 10.0] * SEATS for tick in ticks]
    game = a_game(series={"store_total": rising}, ticks=ticks)
    series = summary.leader_series(game, "counts", "store_total")
    assert summary.arrival_of(series, ticks) == pytest.approx(0.9)


def test_a_quantity_that_saturates_early_reads_an_early_arrival() -> None:
    """A series that reaches its end value in the first fifth reads a fifth."""
    ticks = list(range(0, 1001, 100))
    held = [[min(float(tick), 200.0)] * SEATS for tick in ticks]
    game = a_game(series={"held_tiles": held}, ticks=ticks)
    series = summary.leader_series(game, "counts", "held_tiles")
    assert summary.arrival_of(series, ticks) == pytest.approx(0.2)


def test_a_constant_quantity_reads_an_arrival_of_zero() -> None:
    """A quantity that never moves stands at its end value from the first tick."""
    ticks = [0, 500, 1000]
    flat = [[1.0 / 3.0] * SEATS for _ in ticks]
    game = a_game(series={"domination_progress": flat}, ticks=ticks)
    series = summary.leader_series(game, "shares", "domination_progress")
    assert summary.arrival_of(series, ticks) == pytest.approx(0.0)


def test_a_quantity_that_ends_at_zero_has_no_arrival() -> None:
    """A series whose end value is zero states nothing about when it arrived."""
    ticks = [0, 500, 1000]
    empty = [[0.0] * SEATS for _ in ticks]
    game = a_game(series={"rival_seats_held": empty}, ticks=ticks)
    series = summary.leader_series(game, "counts", "rival_seats_held")
    assert summary.arrival_of(series, ticks) is None


def test_a_series_that_only_accumulates_rises_at_every_step() -> None:
    """A quantity that never falls reads a rise share of one."""
    assert summary.rise_share([1.0, 2.0, 3.0, 9.0]) == pytest.approx(1.0)


def test_a_series_that_a_contest_moves_rises_at_some_steps() -> None:
    """A quantity that falls at one step of four reads three quarters."""
    assert summary.rise_share([1.0, 2.0, 1.0, 2.0, 3.0]) == pytest.approx(0.75)


def test_a_level_step_counts_for_neither_direction() -> None:
    """A step that holds level is out of the share, and a flat series reads none.

    The first series holds level twice and rises once, so the share reads one
    over one and not one over three.
    """
    assert summary.rise_share([1.0, 1.0, 1.0, 2.0]) == pytest.approx(1.0)
    assert summary.rise_share([1.0, 1.0, 1.0]) is None


def test_the_trend_reads_no_game_that_ended_before_the_tick() -> None:
    """A game shorter than a trend tick contributes nothing at that tick."""
    short = a_game(
        series={"held_tiles": [[1.0] * SEATS, [4.0] * SEATS]},
        ticks=[0, 500],
        end_tick=500,
    )
    long = a_game(
        series={"held_tiles": [[1.0] * SEATS, [4.0] * SEATS, [9.0] * SEATS]},
        ticks=[0, 500, 1000],
        end_tick=1000,
    )
    rows = summary.trend_at([short, long], "counts", "held_tiles")
    assert rows["500"]["games"] == 2
    assert rows["500"]["median"] == pytest.approx(4.0)
    assert rows["1000"]["games"] == 1
    assert rows["1000"]["median"] == pytest.approx(9.0)


def test_the_ending_shares_sum_to_one_and_carry_an_error() -> None:
    """Every game falls under one path, and each share states its own error."""
    games = [
        a_game(path="wonder", series={"a": [[1.0] * SEATS]}, ticks=[0]),
        a_game(path="wonder", series={"a": [[1.0] * SEATS]}, ticks=[0]),
        a_game(path="renown", series={"a": [[1.0] * SEATS]}, ticks=[0]),
        a_game(path="territory", series={"a": [[1.0] * SEATS]}, ticks=[0]),
    ]
    rows = summary.ending_rows(games)
    assert sum(row["share"] for row in rows.values()) == pytest.approx(1.0)
    assert rows["wonder"]["share"] == pytest.approx(0.5)
    assert rows["wonder"]["share_error"] == pytest.approx(0.25)


def test_a_player_is_counted_in_every_game_it_sat_in() -> None:
    """Each seat of each game counts once, and only the winner counts a win."""
    games = [
        a_game(winner="mason", series={"a": [[1.0] * SEATS]}, ticks=[0]),
        a_game(winner=None, series={"a": [[1.0] * SEATS]}, ticks=[0]),
    ]
    rows = summary.winner_rows(games)
    assert rows["mason"]["games"] == 2
    assert rows["mason"]["wins"] == 1
    assert rows["quietist"]["wins"] == 0
    assert rows["quietist"]["share"] == pytest.approx(0.0)
