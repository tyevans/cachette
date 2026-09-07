"""The learner sees only what a player would see.

The environment drives one faction. It must read the readers that answer for
that faction, and it must read no reader that answers from the truth of the
whole world.[^1]

A check over the import list of the package would not catch this. An adapter
that holds the world object reaches every reader through that object, and no
import says so. **The check here therefore watches the calls**, and it drives
a whole episode before it judges.[^2]

References
----------
[^1]: PRD-0001, a faction sees only what it observes.
``docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md``
[^2]: Testing Rules, section 5. ``.agents/rules/testing.md``
"""

from __future__ import annotations

from typing import Any

import numpy as np
import pytest

from cachette import World
from cachette.learn import FACTION_SCOPED_READERS, Env, EnvConfig, VectorEnv, Weighting
from cachette.learn.env import viable_seeds

# A weighting with every weight set, so the reward runs. The values are the
# test's own and they state no rule of the downstream game.
WEIGHTING = Weighting(
    terms={"held_tiles": 1.0, "population": 0.5},
    won=100.0,
    lost=-100.0,
    drawn=0.0,
)

CONFIG = EnvConfig(
    width=32,
    height=32,
    faction_count=3,
    seat=0,
    tick_limit=400,
    horizon=12,
    decision_interval=4,
)


class WatchedWorld:
    """A world that records every reader the caller reaches for.

    The proxy raises the moment a caller names a reader outside the
    faction-scoped set. It raises rather than records, so the failure names
    the call that broke the rule instead of a set difference at the end.
    """

    def __init__(self, world: World) -> None:
        """Wrap one world."""
        object.__setattr__(self, "_world", world)
        object.__setattr__(self, "seen", set())

    def __getattr__(self, name: str) -> Any:  # noqa: ANN401
        """Return the attribute, and refuse a reader outside the set."""
        if name.startswith("_"):
            raise AttributeError(name)
        if name not in FACTION_SCOPED_READERS:
            message = (
                f"the environment reached for {name!r}, which is not a "
                f"faction-scoped reader. A learner sees only what a player sees."
            )
            raise AssertionError(message)
        self.seen.add(name)
        return getattr(self._world, name)


def build_watched(seed: int) -> WatchedWorld:
    """Build a seeded world under the learner's seat, and watch it."""
    world = World(
        width=CONFIG.width,
        height=CONFIG.height,
        seed=seed,
        faction_count=CONFIG.faction_count,
    )
    world.seed_world()
    world.set_win_readers_enabled(True)
    world.set_tick_limit(CONFIG.tick_limit)
    world.set_externally_controlled(CONFIG.seat, True)
    return WatchedWorld(world)


def test_the_environment_reads_no_unfogged_reader_over_a_whole_episode() -> None:
    """Drive a full episode and refuse every whole-world reader.

    The episode takes a legal action at every decision, so the legality
    answer, the observation, the verbs and the reward all run. A reader
    outside the faction-scoped set raises where it is called.
    """
    seed = viable_seeds(CONFIG, 1, 900)[0]
    watched = build_watched(seed)
    env = Env(CONFIG, WEIGHTING)
    env.adopt(watched)

    rng = np.random.default_rng(0)
    decisions = 0
    while not env.done:
        mask = env.action_mask()
        action = int(rng.choice(np.flatnonzero(mask)))
        env.step(action)
        decisions += 1

    assert decisions > 0, "the episode took no decision, so it proved nothing"
    # The proxy must have been reached at all. A test that watched a world
    # nobody read would pass while proving nothing.
    assert "faction_observation" in watched.seen
    assert "legal_actions" in watched.seen
    assert "act" in watched.seen
    assert "step" in watched.seen
    assert watched.seen <= FACTION_SCOPED_READERS


def test_the_watch_refuses_a_whole_world_reader() -> None:
    """The proxy is able to fail, so its silence in the test above means something."""
    watched = build_watched(viable_seeds(CONFIG, 1, 900)[0])
    with pytest.raises(AssertionError, match="tile_holders"):
        watched.tile_holders()
    with pytest.raises(AssertionError, match="faction_units"):
        # The trap. This reader names a faction and answers from the truth.
        watched.faction_units(0)


def test_a_reset_builds_a_new_world_and_never_reuses_the_last() -> None:
    """Two episodes of one environment do not share a world."""
    env = Env(CONFIG, WEIGHTING)
    seeds = viable_seeds(CONFIG, 2, 900)
    env.reset(seeds[0])
    first = env.world
    env.step(0)
    env.reset(seeds[1])
    assert env.world is not first
    assert env.world.tick == 0


def test_the_mask_is_never_empty_and_the_no_op_is_always_legal() -> None:
    """Row zero is legal at every decision, so a learner is never stuck."""
    env = Env(CONFIG, WEIGHTING)
    env.reset(viable_seeds(CONFIG, 1, 900)[0])
    while not env.done:
        mask = env.action_mask()
        assert mask[0] == 1
        assert mask.sum() >= 1
        env.step(0)


def test_the_vector_gives_what_the_single_environments_give() -> None:
    """Eight environments in a vector equal eight environments run alone.

    The comparison is the reward of every decision. The vector crosses the
    boundary once for the whole set, and the single environments cross once
    each, so the two runs share no code path below the action.
    """
    seeds = viable_seeds(CONFIG, 6, 900)
    plan = [
        [int(value) for value in np.random.default_rng(index).integers(0, 29, 40)]
        for index in range(len(seeds))
    ]

    alone: list[list[float]] = []
    for index, seed in enumerate(seeds):
        env = Env(CONFIG, WEIGHTING)
        env.reset(seed)
        rewards = []
        for action in plan[index]:
            if env.done:
                break
            mask = env.action_mask()
            chosen = action if mask[action] else 0
            rewards.append(env.step(chosen).reward)
        alone.append(rewards)

    vector = VectorEnv(CONFIG, WEIGHTING, count=len(seeds), workers=3)
    vector.reset(seeds)
    batched: list[list[float]] = [[] for _ in seeds]
    for turn in range(40):
        if vector.done:
            break
        masks = vector.action_masks()
        actions = []
        for index in range(len(seeds)):
            action = plan[index][turn]
            actions.append(action if masks[index][action] else 0)
        results = vector.step(actions)
        for index, result in enumerate(results):
            if not result.info.get("skipped"):
                batched[index].append(result.reward)

    assert batched == alone
