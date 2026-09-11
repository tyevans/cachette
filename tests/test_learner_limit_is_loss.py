"""Under the limit rule, only a win before the tick limit is a win.

The engine ends a game when a reader fires. The territory reader fires only at
the tick limit, and it names the faction with the most held ground there. A
run that counts that ending as a win pays a policy for letting the clock run
out. The limit rule counts it as a loss instead, and it counts a draw as a
loss as well.

**The rule must reach every reader of an episode together.** The reward pays
the terminal weight of the outcome, the win share counts the outcomes, and the
controller bar is a win share of the built-in controller in the same seat. A
rule that reached one of them and not another would publish a bar that the
reward does not train toward.

# The fixtures reach each ending

A fixture that never reaches an ending cannot fail on it.[^1] Each seed below
was chosen by a search over the seeds 0 to 159 under the world of this module,
with the built-in controller in seat 0.

- Seed 4 ends by domination at tick 90, and seat 0 wins. It is the win before
  the limit.
- Seed 0 ends on territory at tick 400, which is the limit, and seat 0 wins.
  It is the game that reaches the limit with the seat in the lead.

A draw needs no seed. The territory reader names a winner in almost every game
that reaches the limit, so a draw fixture adopts a world whose readers are off.
That world runs to the limit with no end record, which is exactly the state
the engine reports as a draw.

**The seeds are fixtures and they decay.** Which game ends where is a property
of the engine. Each test asserts the ending it needs before it asserts the
rule, so a seed that moved fails with a message that names the fixture.

# References

[^1]: Testing Rules, a fixture supplies the input. ``.agents/rules/testing.md``
"""

from __future__ import annotations

import subprocess
import sys
from dataclasses import replace
from pathlib import Path

import pytest

from cachette._core import World
from cachette.learn import __main__ as runner
from cachette.learn.__main__ import use_play_styles, use_world, world_lines
from cachette.learn.baseline import BaselineCache
from cachette.learn.env import Env, EnvConfig
from cachette.learn.policy import LinearPolicy, PolicyFitError, load_policy
from cachette.learn.record import ValidationScore
from cachette.learn.reward import Weighting
from cachette.learn.rollout import run_population
from cachette.learn.train import TrainConfig, evaluate, train

ROOT = Path(__file__).resolve().parent.parent

TICK_LIMIT = 400
INTERVAL = 5

# The world of the endings, with the built-in controller in seat 0. The horizon
# covers the tick limit exactly, so no episode truncates before the limit.
CONTROLLER = EnvConfig(
    width=24,
    height=24,
    faction_count=3,
    seat=0,
    tick_limit=TICK_LIMIT,
    horizon=TICK_LIMIT // INTERVAL,
    decision_interval=INTERVAL,
    controlled=False,
)
CONTROLLER_LIMIT = replace(CONTROLLER, limit_is_loss=True)

EARLY_SEED = 4
LIMIT_SEED = 0

WIN = 100.0
LOSS = 10.0

# A weighting with no shaped term, so the return of an episode is its terminal
# weight and nothing else. The draw weight is zero, as in every strategy row.
TERMINAL = Weighting(terms={}, won=WIN, lost=-LOSS, drawn=0.0)


def play(config: EnvConfig, seed: int) -> tuple[Env, float]:
    """Play one episode to its end, and return the environment and the return.

    The seat takes the no-op row on every decision. A world whose seat the
    built-in controller holds reads no action at all.
    """
    env = Env(config, TERMINAL)
    env.reset(seed)
    paid = 0.0
    while not env.done:
        paid += env.step(0).reward
    return env, paid


def require_ending(env: Env, path: str, winner: int, at_limit: bool) -> None:
    """Assert that a fixture reached the ending a test needs.

    A seed that moved to another ending must fail here, with the name of the
    fixture, and never pass a test that it no longer reaches.
    """
    end = env.world.game_end()
    assert end is not None, "the fixture holds no end record"
    assert end["path"] == path, f"the fixture ended on {end['path']}, not {path}"
    assert end["winner"] == winner, f"the fixture named faction {end['winner']}"
    if at_limit:
        assert end["tick"] == TICK_LIMIT, f"the fixture ended at tick {end['tick']}"
    else:
        assert end["tick"] < TICK_LIMIT, f"the fixture ended at tick {end['tick']}"


@pytest.fixture(autouse=True)
def _restore_the_world() -> object:
    """Put the module world and the strategy table back after each test."""
    held = runner.WORLD
    held_table = runner.STRATEGIES
    yield
    use_world(
        held.width,
        held.faction_count,
        held.tick_limit,
        held.decision_interval,
        held.limit_is_loss,
    )
    runner.STRATEGIES = held_table


@pytest.mark.parametrize("config", [CONTROLLER, CONTROLLER_LIMIT])
def test_a_win_before_the_limit_is_a_win_under_both_rules(config: EnvConfig) -> None:
    """A reader that ends the game early decides it, whichever rule holds."""
    env, paid = play(config, EARLY_SEED)
    require_ending(env, "domination", winner=0, at_limit=False)
    assert env.outcome == "won"
    assert paid == WIN


def test_a_win_at_the_limit_is_a_loss_under_the_limit_rule() -> None:
    """The seat leads the ground at the limit, and only the old rule pays it."""
    old, old_paid = play(CONTROLLER, LIMIT_SEED)
    require_ending(old, "territory", winner=0, at_limit=True)
    assert old.outcome == "won"
    assert old_paid == WIN

    limit, limit_paid = play(CONTROLLER_LIMIT, LIMIT_SEED)
    require_ending(limit, "territory", winner=0, at_limit=True)
    assert limit.outcome == "lost"
    assert limit_paid == -LOSS


def _drawn_episode(config: EnvConfig) -> tuple[Env, float]:
    """Play a world with its readers off to the limit, through the seat.

    The world runs to the tick limit and holds no end record, which is the
    state the engine reports as a draw. The environment takes the world
    through the door a caller uses to hand it a world of its own.
    """
    env = Env(config, TERMINAL)
    world = World(width=24, height=24, seed=LIMIT_SEED, faction_count=3)
    world.seed_world()
    world.set_win_readers_enabled(False)
    world.set_tick_limit(TICK_LIMIT)
    world.set_externally_controlled(0, True)
    env.adopt(world)
    paid = 0.0
    while not env.done:
        paid += env.step(0).reward
    assert world.game_end() is None, "the draw fixture holds an end record"
    assert world.tick >= TICK_LIMIT, f"the draw fixture stopped at {world.tick}"
    return env, paid


def test_a_draw_pays_nothing_under_the_old_rule_and_a_loss_under_the_limit_rule() -> (
    None
):
    """A clock that ran out with no winner is a loss under the limit rule."""
    seated = replace(CONTROLLER, controlled=True)
    old, old_paid = _drawn_episode(seated)
    assert old.outcome == "drawn"
    assert old_paid == 0.0

    limit, limit_paid = _drawn_episode(replace(seated, limit_is_loss=True))
    assert limit.outcome == "lost"
    assert limit_paid == -LOSS


def test_the_win_share_counts_only_the_wins_before_the_limit() -> None:
    """The instruments a run prints read the same rule the reward pays.

    The held-out and the controller figures come from the summary of a pass.
    The training, the validation and the yardstick figures come from the
    record of a batch. Both must drop the win at the limit.
    """
    seeds = [EARLY_SEED, LIMIT_SEED]
    probe = Env(CONTROLLER, TERMINAL)
    policy = LinearPolicy.zeros(probe.action_length, probe.observation_length)

    old = evaluate(CONTROLLER, TERMINAL, policy, seeds, 1)
    limit = evaluate(CONTROLLER_LIMIT, TERMINAL, policy, seeds, 1)
    assert old["won"] == 1.0
    assert limit["won"] == 0.5
    assert limit["lost"] == 0.5
    assert limit["return"] == (WIN - LOSS) / 2

    batch = run_population(CONTROLLER_LIMIT, TERMINAL, [policy], seeds, 1)
    assert batch.won == 0.5
    assert ValidationScore.of_record(batch).won == 0.5


def test_the_baseline_cache_key_names_the_rule(tmp_path: Path) -> None:
    """A bar measured under one rule never answers for the other.

    The controller world of the run carries the rule, so the key of the bar
    the run asks for differs between the two settings.
    """
    cache = BaselineCache(directory=tmp_path, engine_key="an-engine-build")
    probe = Env(CONTROLLER, TERMINAL)
    version = probe.observation_version
    seeds = [EARLY_SEED, LIMIT_SEED]

    use_world(24, 3, TICK_LIMIT, INTERVAL)
    old = cache.inputs(runner.CONTROLLER_WORLD, TERMINAL, seeds, version)
    use_world(24, 3, TICK_LIMIT, INTERVAL, limit_is_loss=True)
    limit = cache.inputs(runner.CONTROLLER_WORLD, TERMINAL, seeds, version)
    assert old is not None and limit is not None
    assert cache.path_of(old) != cache.path_of(limit)

    cache.write(old, {"return": WIN, "won": 1.0})
    assert cache.read(limit) is None, "a bar of the old rule answered the new one"


def test_both_strategy_tables_play_the_world_of_the_rule() -> None:
    """The built-in weightings and the play styles both carry the rule."""
    use_world(24, 3, TICK_LIMIT, INTERVAL, limit_is_loss=True)
    assert runner.WORLD.limit_is_loss
    assert runner.CONTROLLER_WORLD.limit_is_loss
    assert all(row[0].limit_is_loss for row in runner.STRATEGIES.values())

    use_play_styles(["aggressive"], "fixed", None, "linear")
    assert all(row[0].limit_is_loss for row in runner.STRATEGIES.values())


def test_the_rule_refuses_a_limit_the_interval_does_not_divide() -> None:
    """An episode that the horizon ends before the limit would pay nothing.

    The rule asks for a loss at the limit, and an unfinished episode pays
    neither a loss nor a win. The world therefore refuses the pair.
    """
    with pytest.raises(ValueError, match="whole number of decisions"):
        use_world(24, 3, TICK_LIMIT + 5, 10, limit_is_loss=True)
    use_world(24, 3, TICK_LIMIT + 5, 10)


# The world a resumed run trains in. It is small and short, because the resume
# is refused before the first generation plays.
SEATED = EnvConfig(
    width=24,
    height=24,
    faction_count=3,
    seat=0,
    tick_limit=200,
    horizon=20,
    decision_interval=10,
)

ONE_GENERATION = TrainConfig(
    generations=1, population=4, seeds_per_generation=1, workers=2, pool=1, seed=0
)


@pytest.mark.parametrize("written", [False, True])
def test_a_resume_refuses_a_checkpoint_from_the_other_rule(
    tmp_path: Path, written: bool
) -> None:
    """A centre trained under one rule does not continue under the other."""
    under = replace(SEATED, limit_is_loss=written)
    train("t", under, TERMINAL, ONE_GENERATION, tmp_path, [900])
    _, meta = load_policy(tmp_path / "t-latest.npz")
    assert meta["limit_is_loss"] is written

    other = replace(SEATED, limit_is_loss=not written)
    with pytest.raises(PolicyFitError, match="tick limit as a loss"):
        train(
            "t",
            other,
            TERMINAL,
            replace(ONE_GENERATION, generations=2),
            tmp_path,
            [900],
            resume=True,
        )


def _printed(*arguments: str) -> dict[str, str]:
    """Run the trainer with one print flag, and read the rows it printed."""
    finished = subprocess.run(
        [sys.executable, "-m", "cachette.learn", *arguments],
        capture_output=True,
        text=True,
        check=True,
        cwd=ROOT,
    )
    rows = {}
    for line in finished.stdout.splitlines():
        name, _, value = line.partition("\t")
        rows[name] = value
    return rows


def test_the_world_and_the_plan_state_the_rule() -> None:
    """A launcher and a person both read the rule before a run starts."""
    assert "limit_is_loss\t0" in world_lines(CONTROLLER).splitlines()
    assert "limit_is_loss\t1" in world_lines(CONTROLLER_LIMIT).splitlines()

    shape = ("--world-extent", "48", "--tick-limit", "2500")
    assert _printed("--print-world", *shape)["limit_is_loss"] == "0"
    assert _printed("--print-world", *shape, "--limit-is-loss")["limit_is_loss"] == "1"
    plan = _printed("--print-plan", "--cores", "4", *shape, "--limit-is-loss")
    assert plan["limit_is_loss"] == "1"
