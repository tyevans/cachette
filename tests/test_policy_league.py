"""A rating must order play, and it must refuse to order a seat.

The tool under test rates the stored policies by the games they play against
each other. Two failures would each produce a table that reads like a
ranking, and neither would raise.

**The first failure is the seat.** The seats of this world do not win equally
often, so a schedule that leaves one player in one seat rates the seat. The
tests below build a synthetic world in which the seat decides every game. They
assert that the balanced schedule of the tool rates every player alike over
it. A second test pins one player in one seat over the same synthetic
world and asserts that the rating then follows the seat. **The second test is
the proof that the first is not vacuous.**

**The second failure is an error bar that no resample produced.** A league of
one world gives the same fit on every resample, so the standard deviation of
it is float noise rather than zero. A difference divided by that noise clears
any bar. A smoke run of three games over one world reported two ordered pairs
that way, and nothing failed.

# The fixtures supply extremes, and none of them is a typical league

Every synthetic game in this file is built by hand. The cases here hold, on
purpose:

- a world in which the seat wins every game, which is pure seat luck
- a player that won no game at all, which drives a bare fit to minus infinity
- a league of one world, which can state no error bar
- a game that no win reader ended, which states nothing about strength
- a game that ended exactly at the tick limit, which is the case that
  separates two spellings of the limit rule
- a pair whose games the third player won, which must be out of the pairwise
  denominator

One test starts the engine. It plays a small world, and it is marked slow
because it costs about a minute.
"""

from __future__ import annotations

import importlib.util
import json
import sys
from pathlib import Path
from types import ModuleType, SimpleNamespace
from typing import TYPE_CHECKING, Any

import numpy as np
import pytest

if TYPE_CHECKING:
    from cachette.learn.env import Env
    from cachette.learn.policy import LinearPolicy

ROOT = Path(__file__).resolve().parent.parent


def _load(name: str) -> ModuleType:
    """Import a script by path, because `scripts/` is not a package."""
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / f"{name}.py")
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


league = _load("policy_league")
instrumental = _load("instrumental_population")

# How many players and seats the synthetic schedules use. Three seats is what
# the engine world holds, and four players is the smallest league in which a
# player sits out some games.
PLAYERS = 4
SEATS = 3


def _game(
    index: int,
    world: int,
    players: tuple[int, ...],
    winner: int | None,
    path: str = "territory",
    end_tick: int = 2500,
    reached_limit: bool = True,
) -> object:
    """Build one finished game by hand, so a test states the case it needs."""
    return league.GameResult(
        index=index,
        world=world,
        seed=50_000 + world,
        rotation=index % SEATS,
        players=players,
        winner=winner,
        path=path,
        end_tick=end_tick,
        reached_limit=reached_limit,
        decisions=250,
    )


def _seat_wins(seatings: list[object], seat: int = 0) -> list[object]:
    """Play a synthetic league in which one seat wins every game.

    This is pure seat luck with the quality of the players held equal. A
    rating over it must order nobody, because nothing about the players
    decided anything.
    """
    return [
        _game(
            index=index,
            world=seating.world,
            players=seating.players,
            winner=seating.players[seat],
        )
        for index, seating in enumerate(seatings)
    ]


def _players(count: int) -> list[object]:
    """Build a player list of the given size, with no policy behind any of it."""
    return [
        league.Player(name=f"player-{index}", policy=None) for index in range(count)
    ]


def test_the_schedule_gives_every_player_every_seat_the_same_number_of_times() -> None:
    """The whole design turns on this, and no rating alone would show it."""
    seatings = league.schedule(9, 3, list(range(50_000, 50_012)), 1)
    assert len(seatings) == 84 * 3
    assert len({seating.world for seating in seatings}) == 84
    counted = league.balance_of(seatings)
    assert sorted(counted) == list(range(9))
    for seats in counted.values():
        assert sorted(seats) == [0, 1, 2]
        assert set(seats.values()) == {28}


def test_the_schedule_repeats_over_two_calls() -> None:
    """Two runs of the tool must play the same games in the same order."""
    first = league.schedule(PLAYERS, SEATS, [1, 2, 3], 2)
    second = league.schedule(PLAYERS, SEATS, [1, 2, 3], 2)
    assert first == second


def test_the_rotations_of_one_world_share_a_seed() -> None:
    """The seat turns inside one map, so the turn cancels no map difference."""
    seatings = league.schedule(PLAYERS, SEATS, [7, 8], 1)
    by_world: dict[int, set[int]] = {}
    for seating in seatings:
        by_world.setdefault(seating.world, set()).add(seating.seed)
    assert all(len(seeds) == 1 for seeds in by_world.values())
    counted: dict[int, int] = {}
    for seating in seatings:
        counted[seating.world] = counted.get(seating.world, 0) + 1
    assert set(counted.values()) == {SEATS}


def test_the_schedule_refuses_a_league_that_cannot_fill_the_seats() -> None:
    """Three distinct players cannot be drawn from two."""
    with pytest.raises(ValueError, match="distinct players"):
        league.schedule(2, SEATS, [1], 1)
    with pytest.raises(ValueError, match="at least one seed"):
        league.schedule(PLAYERS, SEATS, [], 1)


def test_the_balance_check_refuses_a_schedule_that_measures_the_seat() -> None:
    """A player pinned to one seat carries that seat's luck into its rating."""
    pinned = [
        league.Seating(world=world, seed=50_000 + world, rotation=0, players=(0, 1, 2))
        for world in range(4)
    ]
    with pytest.raises(ValueError, match="measures the seat"):
        league.require_balance(pinned, SEATS)


def test_a_balanced_schedule_rates_pure_seat_luck_as_nothing() -> None:
    """Seat luck must reach no rating, because the rotation cancels it.

    The synthetic league gives every game to whoever holds seat zero. The
    players are therefore equal, and the balanced schedule must say so.
    """
    seatings = league.schedule(PLAYERS, SEATS, list(range(50_000, 50_020)), 5)
    league.require_balance(seatings, SEATS)
    results = _seat_wins(seatings)
    players = _players(PLAYERS)
    ratings, _spread, raw = league.rate(results, players, 0, draws=200)
    assert {row.wins for row in ratings} == {len(seatings) // PLAYERS}
    assert max(abs(row.elo) for row in ratings) < 1.0
    assert league.ordered_pairs(ratings, league.difference_errors(raw)) == []


def test_an_unbalanced_schedule_rates_the_seat_instead_of_the_play() -> None:
    """This is the artefact the balanced schedule prevents.

    The same synthetic rule decides every game: whoever holds seat zero wins.
    Here one player holds seat zero in every game it plays, so the rating
    ranks it first. **The test above is not vacuous, because this one is the
    same data under a schedule that does not rotate.**
    """
    seatings = [
        league.Seating(
            world=world,
            seed=50_000 + world,
            rotation=0,
            players=(0, 1 + world % (PLAYERS - 1), 1 + (world + 1) % (PLAYERS - 1)),
        )
        for world in range(24)
    ]
    with pytest.raises(ValueError, match="measures the seat"):
        league.require_balance(seatings, SEATS)
    results = _seat_wins(seatings)
    ratings, _spread, raw = league.rate(results, _players(PLAYERS), 0, draws=200)
    by_name = {row.player: row for row in ratings}
    assert by_name["player-0"].wins == len(seatings)
    ordered = league.ordered_pairs(ratings, league.difference_errors(raw))
    assert ordered, "the seat must reach the rating when the schedule pins it"
    assert all(high == "player-0" for high, _low, _gap, _error in ordered)


def test_the_fit_orders_a_player_that_wins_more_often() -> None:
    """A real ordering must clear its own error bar.

    One player wins seven games in ten of the games it sits in, and the
    other three share the rest. The draw comes from a stated seed, so the
    fixture is the same fixture on every run.

    **The first spelling of this fixture gave every undecided game to the
    seat of its own rotation.** That is the same player in every game of a
    triple. The fit then ordered a pair the fixture never made unequal, and
    the test caught the fixture rather than the tool.
    """
    seatings = league.schedule(PLAYERS, SEATS, list(range(50_000, 50_040)), 10)
    draw = np.random.default_rng(5)
    results = [
        _game(
            index=index,
            world=seating.world,
            players=seating.players,
            winner=0
            if 0 in seating.players and draw.random() < 0.7
            else int(draw.choice(seating.players)),
        )
        for index, seating in enumerate(seatings)
    ]
    ratings, _spread, raw = league.rate(results, _players(PLAYERS), 0, draws=200)
    ordered = league.ordered_pairs(ratings, league.difference_errors(raw))
    assert ordered
    assert all(high == "player-0" for high, _low, _gap, _error in ordered)
    strongest = max(ratings, key=lambda row: row.elo)
    assert strongest.player == "player-0"


def test_a_player_that_won_nothing_keeps_a_finite_rating() -> None:
    """A bare fit drives such a player to minus infinity. The prior does not."""
    seatings = league.schedule(PLAYERS, SEATS, list(range(50_000, 50_020)), 5)
    results = [
        _game(
            index=index,
            world=seating.world,
            players=seating.players,
            winner=next(player for player in seating.players if player != 0)
            if 0 in seating.players
            else seating.players[0],
        )
        for index, seating in enumerate(seatings)
    ]
    ratings, _spread, _raw = league.rate(results, _players(PLAYERS), 1, draws=50)
    by_name = {row.player: row for row in ratings}
    assert by_name["player-0"].wins == 0
    assert np.isfinite(by_name["player-0"].elo)
    assert by_name["player-0"].elo < 0.0


def test_a_league_of_one_world_states_no_error_bar() -> None:
    """Every resample of one world draws that world, so nothing varies.

    A smoke run of three games over one world reported two ordered pairs on
    an error bar of float noise. The refusal is the fix, and this is the case
    that produced it.
    """
    results = [
        _game(index=index, world=0, players=(0, 1, 2), winner=0) for index in range(3)
    ]
    with pytest.raises(ValueError, match="states no error"):
        league.bootstrap_strengths(results, 3, draws=10)


def test_the_clearance_test_refuses_an_error_bar_of_float_noise() -> None:
    """A difference divided by float noise clears any bar, and means nothing."""
    ratings = [
        league.Rating(player="high", elo=600.0, error=0.0, games=3, wins=3),
        league.Rating(player="low", elo=-600.0, error=0.0, games=3, wins=0),
    ]
    noise = np.array([[0.0, 2.3e-13], [2.3e-13, 0.0]])
    assert league.ordered_pairs(ratings, noise) == []
    real = np.array([[0.0, 100.0], [100.0, 0.0]])
    assert len(league.ordered_pairs(ratings, real)) == 1


def test_an_overlapping_pair_is_not_ordered() -> None:
    """A rating whose error bars overlap says nothing, and must say so."""
    ratings = [
        league.Rating(player="one", elo=40.0, error=90.0, games=30, wins=12),
        league.Rating(player="two", elo=-40.0, error=90.0, games=30, wins=10),
    ]
    errors = np.array([[0.0, 120.0], [120.0, 0.0]])
    assert league.ordered_pairs(ratings, errors) == []


def test_a_game_with_no_recorded_winner_reaches_no_rating() -> None:
    """Such a game states nothing about which player is stronger."""
    decided = [
        _game(index=index, world=index, players=(0, 1, 2), winner=index % 3)
        for index in range(6)
    ]
    undecided = [
        _game(
            index=6,
            world=6,
            players=(0, 1, 2),
            winner=None,
            path=league.NO_PATH,
            end_tick=100,
            reached_limit=False,
        )
    ]
    members, winners = league.win_table(decided + undecided)
    assert len(winners) == len(decided)
    assert members.shape == (len(decided), SEATS)


def test_the_head_to_head_leaves_out_a_game_the_third_player_won() -> None:
    """A game the third player won states nothing about the other two."""
    results = [
        _game(index=0, world=0, players=(0, 1, 2), winner=0),
        _game(index=1, world=1, players=(0, 1, 2), winner=1),
        _game(index=2, world=2, players=(0, 1, 2), winner=2),
    ]
    wins, together = league.head_to_head(results, 3)
    assert together[0][1] == 3
    assert wins[0][1] == 1
    assert wins[1][0] == 1
    decided = wins[0][1] + wins[1][0]
    assert decided == 2


def test_the_endings_report_the_limit_share_and_the_paths() -> None:
    """The three players of one game share its end tick and its win path."""
    results = [
        _game(
            index=0,
            world=0,
            players=(0, 1, 2),
            winner=0,
            path="domination",
            end_tick=800,
            reached_limit=False,
        ),
        _game(index=1, world=1, players=(0, 1, 2), winner=1, end_tick=2500),
        _game(index=2, world=2, players=(0, 1, 2), winner=2, end_tick=2500),
    ]
    found = league.endings(results, _players(3))
    for row in found:
        assert row.games == 3
        assert row.reached_limit == 2
        assert row.limit_share == pytest.approx(2 / 3)
        assert row.paths == {"domination": 1, "territory": 2}
        assert row.end_tick.lowest == 800.0
        assert row.end_tick.highest == 2500.0
    by_name = {row.player: row for row in found}
    assert by_name["player-0"].won_paths == {"domination": 1}
    assert by_name["player-1"].won_paths == {"territory": 1}


def test_a_player_that_played_no_game_is_refused() -> None:
    """A rating over a player with no game is a rating of the prior alone."""
    results = [_game(index=0, world=0, players=(0, 1, 2), winner=0)]
    with pytest.raises(ValueError, match="played no game"):
        league.endings(results, _players(4))


class _StubWorld:
    """A world that holds only a clock, a tick limit and an end record."""

    def __init__(self, limit: int, tick: int, end: dict[str, Any] | None) -> None:
        """Take the limit, the clock and the end record of the stub."""
        self._limit = limit
        self._tick = tick
        self._end = end

    @property
    def tick(self) -> int:
        """Give back the tick the world stands at."""
        return self._tick

    @property
    def tick_limit(self) -> int:
        """Give back the tick the limit fires at."""
        return self._limit

    def game_end(self) -> dict[str, Any] | None:
        """Give back how the game ended, or nothing while it runs."""
        return self._end


@pytest.mark.parametrize(
    ("limit", "tick", "end"),
    [
        (2500, 2500, {"winner": 0, "path": "territory", "tick": 2500}),
        (2500, 2510, {"winner": 0, "path": "domination", "tick": 2499}),
        (2500, 900, {"winner": 1, "path": "domination", "tick": 890}),
        (0, 4000, {"winner": 1, "path": "renown", "tick": 3990}),
        (2500, 2500, None),
        (2500, 40, None),
    ],
)
def test_the_two_spellings_of_the_tick_limit_rule_agree(
    limit: int, tick: int, end: dict[str, Any] | None
) -> None:
    """One rule, two sites, and a check that fails when the copies disagree.

    The instrumental population script holds this rule inline, and the league
    tool holds it as a function. The cases here include three worlds. One game ended
    exactly at the limit, one game held a clock that ran past the limit, and
    one world holds no limit at all.
    """
    world = _StubWorld(limit, tick, end)
    env = SimpleNamespace(
        world=world,
        outcome="lost",
        decisions=250,
        signals=SimpleNamespace(read_scalars=lambda _array: {}),
        observation=lambda: np.zeros(1, dtype=np.int64),
    )
    theirs = instrumental.read_episode("arm", 1, env)
    ours = league.reached_the_limit(world, instrumental.end_tick_of(world))
    assert ours == theirs.reached_limit


def _checkpoint(directory: Path, name: str, strategy: object) -> None:
    """Write one checkpoint file, so a test states the metadata it needs."""
    payload: dict[str, Any] = {"file": f"{name}.npz"}
    if strategy is not None:
        payload["strategy"] = strategy
    (directory / f"{name}.json").write_text(json.dumps(payload), encoding="utf-8")


def test_a_checkpoint_that_names_no_strategy_is_refused(tmp_path: Path) -> None:
    """A checkpoint with no strategy names no world, so it can play nothing."""
    _checkpoint(tmp_path, "one", None)
    with pytest.raises(ValueError, match="names no strategy"):
        league.stored_players(tmp_path, {"land": (None, None, "linear")}, None)


def test_a_checkpoint_that_names_an_unknown_strategy_is_refused(
    tmp_path: Path,
) -> None:
    """The refusal names the rows the table does hold."""
    _checkpoint(tmp_path, "one", "moon")
    with pytest.raises(ValueError, match=r"\['land'\]"):
        league.stored_players(tmp_path, {"land": (None, None, "linear")}, None)


def test_two_checkpoints_of_one_strategy_are_refused(tmp_path: Path) -> None:
    """A league that seated one strategy twice would rate it against itself."""
    _checkpoint(tmp_path, "one", "land")
    _checkpoint(tmp_path, "two", "land")
    with pytest.raises(ValueError, match="two checkpoints name"):
        league.stored_players(tmp_path, {"land": (None, None, "linear")}, None)


def test_a_strategy_table_of_two_worlds_is_refused() -> None:
    """A rating over two different worlds compares games nobody can compare."""
    from cachette.learn.env import EnvConfig

    small = EnvConfig(width=24, height=24, tick_limit=600, horizon=60)
    large = EnvConfig(width=48, height=48, tick_limit=600, horizon=60)
    with pytest.raises(ValueError, match="more than one world"):
        league.one_world({"a": (small, None, "linear"), "b": (large, None, "linear")})
    assert league.one_world({"a": (small, None, "linear")}) == small


PREFERENCE_SEED = 91
"""The seed the fixed preference order of the engine-driven test draws from."""

PREFERENCE_LIFT = 4.0
"""How far the build rows sit above every other row of that order.

The register measured that all four published policies preferred one build
row at every decision, so the order this test seats holds the same shape.[^1]

References
----------
[^1]: Findings register, FND-707. ``docs/FINDINGS.md``
"""

BUILD_VERB = "build"
"""The verb whose rows the fixed preference order puts first.

The test names the verb and the engine names the rows, so no row index is
declared here.
"""


def a_fixed_preference_order(probe: Env) -> LinearPolicy:
    """Return a policy that scores the action rows in one order, always.

    Every weight over the observation is zero, and the trailing bias weight
    carries the order. The score of a row is therefore the same number at
    every decision, whatever the world holds. **This is the shape the register
    measured in all four published policies.**[^1]

    The build rows sit above the rest, because that is the family the register
    found the strongest constant in. This is one order of that family and it
    is not the maximum over the family, which no test can reach.

    References
    ----------
    [^1]: Findings register, FND-707. ``docs/FINDINGS.md``
    """
    from cachette.learn.policy import LinearPolicy

    weights = np.zeros((probe.action_length, probe.observation_length + 1))
    order = np.random.default_rng(PREFERENCE_SEED).standard_normal(probe.action_length)
    block = probe.action_table.named(BUILD_VERB)
    assert block is not None, f"the action table names no {BUILD_VERB!r} verb"
    order[block.first : block.first + block.rows] += PREFERENCE_LIFT
    weights[:, -1] = order
    return LinearPolicy(weights)


@pytest.mark.slow
def test_the_controller_rates_above_a_fixed_preference_order_and_a_no_op() -> None:
    """The whole tool, driven from the engine, over a known ordering.

    The built-in controller plays a policy that takes the no-op at every
    decision, and a policy that holds one fixed preference order over the
    action rows. **The ordering is known before the games run**, so a rating
    that does not find it is wrong rather than surprising.

    **A no-op alone is the one opponent that says nothing.** The register
    measured the strongest constant preference order at 102.29 and the no-op
    at minus 96.56 under one weighting, so a policy that reads nothing already
    stands far above a passive seat.[^1] A tool that separated a trained
    policy from a no-op would separate a preference order from a no-op just as
    well. Seating both opponents makes the controller gap a gap against a seat
    that acts.

    **The test states no ordering between the two opponents.** The register
    measured its figures on a larger world and a longer game. At this extent
    and this tick limit the no-op won as many games as the preference order,
    and the commit that seated the preference order holds the counts. The
    ordering this test asserts is the controller above each opponent.

    The world is small and the tick limit is short, because the test pays for
    every tick. The world count is eight, which is what the difference needs
    in order to clear two standard errors. A run over one world ordered
    nobody, and the seat won every game of it.

    References
    ----------
    [^1]: Findings register, FND-707. ``docs/FINDINGS.md``
    """
    from cachette.learn.env import Env, EnvConfig, viable_seeds
    from cachette.learn.policy import LinearPolicy

    config = EnvConfig(
        width=24,
        height=24,
        faction_count=SEATS,
        tick_limit=1200,
        horizon=120,
        decision_interval=10,
        threads=1,
    )
    scoring = league.probe_scoring()
    probe = Env(config, scoring)
    idle = LinearPolicy.zeros(probe.action_length, probe.observation_length)
    players = [
        league.Player(name="idle", policy=idle),
        league.Player(name="preference", policy=a_fixed_preference_order(probe)),
        league.Player(name=league.CONTROLLER, policy=None),
    ]
    anchor = len(players) - 1
    seeds = viable_seeds(config, 8, league.HELDOUT_START)
    seatings = league.schedule(len(players), SEATS, seeds, 8)
    league.require_balance(seatings, SEATS)

    results = league.play_league(config, scoring, players, seatings, 12)
    assert len(results) == len(seatings)
    assert all(row.winner is not None for row in results)

    ratings, _spread, raw = league.rate(results, players, anchor, draws=300)
    by_name = {row.player: row for row in ratings}
    for opponent in ("idle", "preference"):
        assert by_name[league.CONTROLLER].wins > by_name[opponent].wins
        assert by_name[league.CONTROLLER].elo > by_name[opponent].elo

    ordered = league.ordered_pairs(ratings, league.difference_errors(raw))
    assert ordered, "the controller must clear the error bar over both opponents"
    assert all(high == league.CONTROLLER for high, _low, _gap, _error in ordered)
    assert {low for _high, low, _gap, _error in ordered} == {"idle", "preference"}

    found = league.endings(results, players)
    assert all(row.games == len(seatings) * SEATS // len(players) for row in found)


def test_the_seat_win_count_reads_the_seat_and_not_the_player() -> None:
    """The seat count is the direct measurement of the seat luck of a world.

    The synthetic league gives every game to whoever holds seat zero. The
    count must therefore put every win in seat zero, whatever the rating
    says about the players.
    """
    seatings = league.schedule(PLAYERS, SEATS, list(range(50_000, 50_020)), 5)
    counted = league.seat_wins(_seat_wins(seatings))
    assert counted == {0: len(seatings)}
    even = league.seat_wins(
        [
            _game(index=index, world=index, players=(0, 1, 2), winner=index % SEATS)
            for index in range(9)
        ]
    )
    assert even == {0: 3, 1: 3, 2: 3}


def test_a_game_with_no_winner_reaches_no_seat_count() -> None:
    """Such a game states nothing about which seat is lucky."""
    results = [
        _game(
            index=0,
            world=0,
            players=(0, 1, 2),
            winner=None,
            path=league.NO_PATH,
            end_tick=100,
            reached_limit=False,
        ),
        _game(index=1, world=1, players=(0, 1, 2), winner=1),
    ]
    assert league.seat_wins(results) == {1: 1}
