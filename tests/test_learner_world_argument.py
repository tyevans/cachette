"""A training run states its world on the command line, and the launcher reads it.

Every reinforcement learning run this project has done trained on a world of
extent 48 by 48 with three factions. The extent was a module constant, so a
run could not state another one, and a launcher that sizes a probe from the
world had nothing to ask.

Three things must hold together.

**The world reaches the trainer.** One entry point sets the extent, the
faction count, the tick limit and the decision interval, and it derives the
horizon from the last two. A horizon that does not cover the tick limit ends
an episode before the game ends, and a truncated episode reports no outcome.

**The world reaches the strategy table.** A level weight of the table divides
by the horizon, so a run that changed the tick limit and kept the table would
pay a fraction of its shaping, and nothing would fail.

**The world reaches the launcher.** The launcher sizes its throughput probe
from the world the run plays. It held two stale copies of the strategy list
once, and a paid instance died on both, so it must hold no copy of the world
either.[^1] [^2]

The observation and the action table do not grow with the world, and one test
below holds that. It is the property that makes a larger world cheap for the
policy: the same weights fit every extent.[^3]

References
----------
[^1]: Findings register, FND-693. ``docs/FINDINGS.md``
[^2]: Recurring defect shapes, redundant declaration sites with undocumented
precedence. ``.agents/rules/recurring-defects.md``
[^3]: ADR-0195, the observation of a faction is a fixed-width scale-free table
in an egocentric frame.
``docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md``
"""

from __future__ import annotations

import importlib.util
import subprocess
import sys
from pathlib import Path
from types import ModuleType

import pytest

from cachette.learn import __main__ as runner
from cachette.learn.__main__ import strategy_table, use_world, world_lines
from cachette.learn.env import Env
from cachette.learn.presets import ObjectiveSchedule
from cachette.learn.reward import Scoring, Weighting
from cachette.learn.train import first_scoring

ROOT = Path(__file__).resolve().parent.parent
LAUNCHER = ROOT / "scripts" / "graviton-train.sh"

# The scoring a probe world runs under. It weights nothing, because a probe
# reads the layout and plays nothing.
PROBE_WEIGHTING = Weighting(terms={}, won=0.0, lost=0.0, drawn=0.0)

# The extents the layout test reads. The lowest is the world every run so far
# trained on, and the highest is far above it, so a length that followed the
# world would differ between two rows.
EXTENTS = (48, 96, 256)


def _load(name: str) -> ModuleType:
    """Import a script by path, because `scripts/` is not a package."""
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / f"{name}.py")
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


probe = _load("world_scale")


def _level(scoring: Scoring | ObjectiveSchedule, field: str) -> float:
    """Return the level weight of one field, and refuse an unset weight."""
    weight = _weighting(scoring).levels.get(field)
    assert weight is not None, f"the weighting states no level weight for {field}"
    return float(weight)


def _weighting(scoring: Scoring | ObjectiveSchedule) -> Weighting:
    """Return the weighting of one row, and refuse a row that holds none.

    Every row of the strategy table states a weighting. A row that stated an
    objective vector would hold no level weight, so the assertions below
    would read nothing rather than fail.
    """
    weighting = first_scoring(scoring)
    assert isinstance(weighting, Weighting), (
        "every row of the strategy table states a weighting"
    )
    return weighting


@pytest.fixture(autouse=True)
def _restore_the_world() -> object:
    """Put the module world back after a test that replaced it.

    The world and the strategy table are module state, so a test that sets one
    changes what every later test reads. This restores both from the world the
    module held before the test.
    """
    held = runner.WORLD
    yield
    use_world(held.width, held.faction_count, held.tick_limit, held.decision_interval)


def _printed(*arguments: str) -> dict[str, int]:
    """Run the world flag of the trainer and read what it printed."""
    finished = subprocess.run(
        [sys.executable, "-m", "cachette.learn", "--print-world", *arguments],
        capture_output=True,
        text=True,
        check=True,
        cwd=ROOT,
    )
    fields = {}
    for line in finished.stdout.splitlines():
        name, _, value = line.partition("\t")
        fields[name] = int(value)
    return fields


def test_the_flag_answers_the_default_world() -> None:
    """A run that names nothing plays the world the module declares."""
    fields = _printed()
    assert fields["width"] == runner.WORLD.width
    assert fields["height"] == runner.WORLD.height
    assert fields["factions"] == runner.WORLD.faction_count
    assert fields["tick_limit"] == runner.WORLD.tick_limit
    assert fields["decision_interval"] == runner.WORLD.decision_interval
    assert fields["horizon"] == runner.WORLD.horizon


def test_the_flag_answers_a_world_the_run_named() -> None:
    """Every part of the world reaches the printed answer.

    This is the case a launcher reads. A launcher that could not see one of
    these would size its probe against a world nobody trains in.
    """
    fields = _printed(
        "--world-extent",
        "192",
        "--factions",
        "4",
        "--tick-limit",
        "6000",
        "--decision-interval",
        "20",
    )
    assert fields["width"] == 192
    assert fields["height"] == 192
    assert fields["factions"] == 4
    assert fields["tick_limit"] == 6000
    assert fields["decision_interval"] == 20
    assert fields["horizon"] == 300


def test_the_horizon_covers_the_tick_limit() -> None:
    """The horizon is the tick limit divided by the interval, and no less.

    A horizon below that truncates the episode, and a truncated episode
    reports no outcome at all, so a policy cannot be paid for a win it never
    reached.
    """
    use_world(96, 3, 7000, 10)
    assert (
        runner.WORLD.horizon * runner.WORLD.decision_interval >= runner.WORLD.tick_limit
    )


def test_a_new_world_rebuilds_the_strategy_table() -> None:
    """A level weight follows the horizon of the world the run plays.

    A level weight is paid on every decision, so the table divides the win by
    the horizon. A table that kept its weights over a new tick limit would pay
    a fraction of its shaping, and nothing would fail.
    """
    use_world(96, 3, 5000, 10)
    world, scoring, _ = runner.STRATEGIES["land"]
    assert world.width == 96
    assert world.tick_limit == 5000
    assert world.horizon == 500
    expected = strategy_table(world)["land"][1]
    assert _weighting(scoring).levels == _weighting(expected).levels


def test_the_level_weight_falls_as_the_horizon_grows() -> None:
    """A longer game pays less for each decision, and the same in total.

    This is the reason the table is a function of the world. Two tick limits
    that differ by a factor of two give level weights that differ by the same
    factor, so the whole episode pays the same shaping either way.
    """
    use_world(48, 3, 2500, 10)
    short = _level(runner.STRATEGIES["land"][1], "held_tiles")
    use_world(48, 3, 5000, 10)
    long = _level(runner.STRATEGIES["land"][1], "held_tiles")
    assert long == pytest.approx(short / 2.0)


def test_the_world_refuses_a_value_no_game_can_play() -> None:
    """Each bound of the world states itself rather than failing later."""
    with pytest.raises(ValueError, match="extent"):
        use_world(0, 3, 2500, 10)
    with pytest.raises(ValueError, match="factions"):
        use_world(48, 1, 2500, 10)
    with pytest.raises(ValueError, match="tick limit"):
        use_world(48, 3, 5, 10)
    with pytest.raises(ValueError, match="decision interval"):
        use_world(48, 3, 2500, 0)


def test_the_printed_world_names_every_field() -> None:
    """The launcher reads a field by name, so every field carries one."""
    lines = dict(line.split("\t") for line in world_lines(runner.WORLD).splitlines())
    assert set(lines) == {
        "width",
        "height",
        "factions",
        "tick_limit",
        "decision_interval",
        "horizon",
    }


def test_the_observation_and_the_action_table_do_not_grow() -> None:
    """One policy fits every extent, because neither length follows the world.

    This drives the engine at three extents and reads the two lengths out of
    the schema of each world. A length that followed the tile count would
    differ between the rows, and every policy trained at one extent would be
    retired by a change to the extent.
    """
    use_world(48, 3, 2500, 10)
    scoring = _weighting(runner.STRATEGIES["land"][1])
    lengths = set()
    for extent in EXTENTS:
        use_world(extent, 3, 2500, 10)
        world = runner.STRATEGIES["land"][0]
        env = Env(world, scoring)
        lengths.add((env.observation_length, env.action_length))
    assert len(lengths) == 1, f"the lengths follow the extent: {sorted(lengths)}"


def test_the_ring_bound_gains_one_ring_for_each_doubling() -> None:
    """The frame gains exactly one ring when the extent doubles.

    The ring index of a tile is the bit length of its hex distance from the
    centre. A doubled distance has a bit length one greater, so a doubled
    extent buys one ring and no more. This is the whole of the relation
    between the world size and the spatial input of a policy.
    """
    assert probe.ring_bound(48, 48) == 8
    assert probe.ring_bound(96, 96) == 9
    assert probe.ring_bound(192, 192) == 10
    assert probe.ring_bound(384, 384) == 11


def test_the_training_world_leaves_most_of_the_frame_empty() -> None:
    """A played world fills no more rings than its extent can hold.

    The bound comes from the geometry and the reach comes from the engine, so
    this drives a world and reads the channel that says how much of a cell
    lies inside the world. The reach is at or below the bound, because the
    frame is egocentric and a faction is not in a corner.
    """
    use_world(48, 3, 2500, 10)
    env = Env(runner.WORLD, PROBE_WEIGHTING)
    env.reset(1)
    filled = probe.rings_filled(env)
    counts = probe.ring_cell_counts(env)
    assert filled <= probe.ring_bound(runner.WORLD.width, runner.WORLD.height)
    assert filled < len(counts), (
        "the training world fills the whole frame, so no ring is empty and "
        "the case this probe measures no longer exists"
    )


def test_the_recommended_limit_covers_the_stated_share() -> None:
    """The recommendation rounds the quantile of the end tick upward.

    A limit below the quantile truncates a game that had not resolved, so the
    recommendation never rounds down.
    """
    ends = [100, 200, 300, 2600]
    assert probe.recommended_limit(ends) >= probe.quantile(
        [float(tick) for tick in ends], probe.LIMIT_QUANTILE
    )
    assert probe.recommended_limit(ends) % probe.LIMIT_STEP == 0
    assert probe.recommended_limit([]) == 0


def test_the_launcher_holds_no_world_of_its_own() -> None:
    """The launcher asks the trainer for the world, and copies none of it.

    A copy here goes stale the first time a run states another extent, and
    nothing fails when it does: the probe reports a figure for a world nobody
    trained in, and the figure reads like a training cost.
    """
    script = LAUNCHER.read_text(encoding="utf-8")
    assert "--print-world" in script, "the launcher does not ask the trainer"
    for passed in ("--width", "--height", "--factions", "--decision-interval"):
        assert f'{passed} "$probe_' in script, (
            f"the launcher does not pass {passed} to the throughput probe, so "
            "the probe measures its own default world"
        )
    assert "--width 48" not in script
    assert "--factions 3" not in script
