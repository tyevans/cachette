"""A world that decides itself must leave the pool, and must move no centre.

Some seeds build a world where no policy can matter. The engine seats each
faction in turn, and it refuses the whole world only when it seats nobody. A
world that seats one faction of three is therefore a world the engine
accepts, and that world ends on the first tick: the seated faction holds
every settlement, so the engine records a domination win. A seat whose site
reaches no food starves inside about a hundred ticks.

Every candidate of a generation scores the same number on such a world. The
ranking then ranks a set of equal numbers, and the rank of an equal set is
the index of the candidate, so the update takes a step of the full learning
rate along a direction the noise alone chose.

Two things must hold. The seed filter must refuse the world, and the trainer
must refuse the step when it meets one anyway.

**Each claim here has a proven failure mode.** A test puts the old behaviour
back through one name, and asserts that the world it built then reaches the
thing the fix removed.[^1]

References
----------
[^1]: Testing Rules, section 1 and section 2a. ``.agents/rules/testing.md``
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

import numpy as np
import pytest

from cachette import World
from cachette.learn import env as env_module
from cachette.learn import search as search_module
from cachette.learn.env import EnvConfig, seats_every_faction, viable_seeds
from cachette.learn.policy import load_policy
from cachette.learn.reward import Weighting
from cachette.learn.train import TrainConfig, train

if TYPE_CHECKING:
    from pathlib import Path

    from cachette._core import FoundingReport

# The world a training run plays, and the base the run starts its search
# from. **The extent, the faction count and the base are the run's own**, and
# a test that used a smaller world would not measure the pool the run draws.
POOL_WORLD = EnvConfig(width=48, height=48, faction_count=3, seat=0)
POOL_START = 1000

# How many seeds the pool test asks for. A run asks for one for each
# generation and a few spare, and the seed this test names sits inside this
# many.
POOL_SIZE = 120

# A seed of the pool world that seats one faction of three. The world ends on
# the first decision, and the score of every candidate is the same number.
ONE_SEAT_SEED = 1065

# A small world for the trainer test, and a seed of it that seats one faction
# of three. A run of the real size takes hours.
SMALL = EnvConfig(
    width=24,
    height=24,
    faction_count=3,
    seat=0,
    tick_limit=200,
    horizon=20,
    decision_interval=10,
)
SMALL_ONE_SEAT_SEED = 902

WEIGHTING = Weighting(terms={"held_tiles": 1.0}, won=100.0, lost=-100.0, drawn=0.0)


def founding(config: EnvConfig, seed: int) -> list[FoundingReport]:
    """Build one world and return what the engine says about each seat."""
    world = World(
        width=config.width,
        height=config.height,
        seed=seed,
        faction_count=config.faction_count,
    )
    return list(world.seed_world())


def test_the_named_seed_seats_one_faction_of_three() -> None:
    """The fixture supplies the case the two tests below are about.

    A seed that seated every faction would pass both of them and prove
    nothing, so this reads the founding report and says what the world is.
    """
    reports = founding(POOL_WORLD, ONE_SEAT_SEED)
    seated = [row["faction"] for row in reports if row["seated"]]
    assert len(seated) == 1, f"seed {ONE_SEAT_SEED} seats {seated}, not one faction"

    reports = founding(SMALL, SMALL_ONE_SEAT_SEED)
    seated = [row["faction"] for row in reports if row["seated"]]
    assert len(seated) == 1, (
        f"small seed {SMALL_ONE_SEAT_SEED} seats {seated}, not one faction"
    )


def test_the_pool_a_run_draws_holds_no_world_that_decides_itself() -> None:
    """The seeds a training run takes all seat every faction, and feed it.

    This calls what the runner calls, with the world the runner plays and the
    base the runner starts from. A test that judged a pool of its own would
    not say what a run draws.
    """
    pool = viable_seeds(POOL_WORLD, POOL_SIZE, POOL_START)

    assert ONE_SEAT_SEED not in pool
    for seed in pool:
        reports = founding(POOL_WORLD, seed)
        assert seats_every_faction(reports, POOL_WORLD.faction_count), (
            f"seed {seed} entered the pool with the report {reports}"
        )


def test_the_old_filter_puts_the_deciding_world_back_in_the_pool(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """The test above is able to fail.

    The old filter accepted every seed the engine did not refuse, and the
    engine refuses only a world that seats nobody. This puts that filter
    back through one name and asserts that the seed returns.
    """
    monkeypatch.setattr(
        env_module, "seats_every_faction", lambda reports, factions: True
    )
    pool = viable_seeds(POOL_WORLD, POOL_SIZE, POOL_START)

    assert ONE_SEAT_SEED in pool, (
        "the old filter did not take the seed back, so the test above proves nothing"
    )


def test_the_pool_holds_the_same_seeds_for_the_same_start_and_count() -> None:
    """One start and one count give one pool, and a prefix of a longer one.

    A run states its seeds by a start and a count, and the reproducibility of
    that run rests on the answer being the same every time. A filter that
    read anything outside the founding report would break it.
    """
    first = viable_seeds(POOL_WORLD, 40, POOL_START)
    second = viable_seeds(POOL_WORLD, 40, POOL_START)
    longer = viable_seeds(POOL_WORLD, 60, POOL_START)

    assert first == second
    assert longer[:40] == first


def run_on_one_seed(
    out: Path, seed: int
) -> tuple[list[np.ndarray], list[dict[str, Any]]]:
    """Train two generations on one seed, and return each stored centre.

    The run stores its centre after every generation. The caller stops after
    the first generation, reads the file, then resumes for the second, so it
    holds the centre from both sides of one update. The pool holds one seed,
    so both generations play the same world.
    """
    centres: list[np.ndarray] = []
    rows: list[dict[str, Any]] = []
    for generations in (1, 2):
        result = train(
            "t",
            SMALL,
            WEIGHTING,
            TrainConfig(
                generations=generations,
                population=4,
                seeds_per_generation=1,
                workers=2,
                seed=0,
            ),
            out,
            [seed],
            validation=[],
            resume=generations > 1,
        )
        rows.append(
            {
                "history": list(result["history"]),
                "degenerate_generations": list(result["degenerate_generations"]),
            }
        )
        stored, _ = load_policy(out / "t-latest.npz")
        centres.append(np.asarray(stored.flat()))
    return centres, rows


def test_the_run_says_out_loud_that_a_generation_carried_no_information(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    """The event reaches the log, and not only the report.

    A reader watches the log while a run works. A mean that equals the best
    reads like a population that agreed, so the run has to name the
    generation in words.

    **The trainer is what must report the configuration, and not the search.**
    A run that gives each candidate fewer worlds than its sigma needs, or that
    is too short for the alignment of its steps, has to say so on its first
    lines. This run gives one world and asks for one generation, so both notes
    fire. A search that held those figures behind a function nobody calls
    would pass every test of its own and would ship inert.[^1]

    References
    ----------
    [^1]: Recurring defect shapes, shape 3.
    ``.agents/rules/recurring-defects.md``
    """
    train(
        "t",
        SMALL,
        WEIGHTING,
        TrainConfig(
            generations=1, population=4, seeds_per_generation=1, workers=2, seed=0
        ),
        tmp_path,
        [SMALL_ONE_SEAT_SEED],
        validation=[],
    )

    printed = capsys.readouterr().out
    assert "carried no information" in printed, printed
    assert "The centre does not move" in printed, printed
    assert "under-sampled for its sigma" in printed, printed
    assert "mostly wander" in printed, printed
    assert "aligns" in printed, printed
    assert "agreed" in printed, printed


def test_a_generation_that_carried_no_information_moves_no_centre(
    tmp_path: Path,
) -> None:
    """A world that decides itself leaves the weights where they were.

    The pool holds one seed, and that world seats one faction of three. Every
    candidate of the generation therefore scores the same number. The centre
    must stand still, and the run must say so.
    """
    centres, rows = run_on_one_seed(tmp_path, SMALL_ONE_SEAT_SEED)

    # The first run scored generation zero. The second resumed and scored
    # generation one, so its history holds that generation alone.
    spreads = [row["history"][-1]["spread"] for row in rows]
    assert spreads == [0.0, 0.0], (
        f"the fixture supplied a spread, so it did not reach the case: {rows}"
    )
    assert [row["history"][-1]["degenerate"] for row in rows] == [1.0, 1.0]
    assert [row["degenerate_generations"] for row in rows] == [[0], [1]]
    assert np.array_equal(centres[0], centres[1])


def index_order_ranks(scores: np.ndarray) -> np.ndarray:
    """Rank the scores the way a stable sort ranked them before this change.

    **This is the defect, restated so that a test can put it back.** A stable
    sort gives a tied score the position it occupies, so an equal generation
    ranks by candidate index. Every plus half then ranks below its own minus
    half by the same amount.
    """
    order = np.argsort(np.argsort(scores, kind="stable"), kind="stable")
    return order / (len(scores) - 1) - 0.5


def test_the_old_trainer_moves_the_centre_from_an_equal_generation(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """The test above is able to fail.

    The old trainer ranked every generation, whatever it scored, and it ranked
    a tied score by candidate index. **Both defects must go back**, because
    either one alone leaves the centre where it was. A ranking that gives a
    tied score the mean of its positions gives every candidate of an equal
    generation one rank, so every pair weight is zero and the search reads no
    direction to step along.

    The two together move the centre. The index order gives every plus half a
    lower rank than its own minus half, and the guard lets that ranking
    through.
    """
    monkeypatch.setattr(search_module, "carries_information", lambda spread: True)
    monkeypatch.setattr(search_module, "rank_shape", index_order_ranks)
    monkeypatch.setattr(search_module, "generation_agreement", lambda ranks: 1.0)
    centres, rows = run_on_one_seed(tmp_path, SMALL_ONE_SEAT_SEED)

    assert [row["history"][-1]["spread"] for row in rows] == [0.0, 0.0]
    assert not np.array_equal(centres[0], centres[1]), (
        "the old trainer left the centre alone, so the test above proves nothing"
    )
