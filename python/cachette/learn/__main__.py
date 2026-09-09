"""Train several policies, each against a different weighting, and report.

Each strategy below names what its reward is worth. **The weights are the
strategy.** A policy trained against territory and a policy trained against
people play the same game under different scoring, and the point of the run
is to see whether they end up playing it differently.

Every weight here belongs to this run. None of them states a rule of the
downstream game, and one blocker holds that question open.[^1]

# The horizon must reach the end of the game

The engine ends a game by domination, by a wonder, by renown, or by a
comparison of held ground at the tick limit. A measurement over held-out
seeds ran the population script against the built-in controller and read the
end tick of each game.[^2] **The endings fall in two clusters.** Most games
resolve early, and the rest run to the tick limit with nothing in between.
The register holds the figures, because a figure in a docstring decays and a
register row does not.[^3]

**The second cluster is the limit and not the play.** A probe replayed the
same worlds under a tick limit far above the one a run uses, and the games
that had reached the limit resolved well past it. A limit that is too low
therefore reports a cluster of indecisive games that were not indecisive. The
share of games that resolve under the limit of this run falls as the extent
rises. A finding holds the measurement, and the register holds the limit and
the resolve share of each extent.[^4] [^5]

A horizon shorter than the tick limit truncates the episode, and a truncated
episode reports no outcome. A policy therefore cannot be paid for a win it
never reached.

The horizon of this run covers the tick limit exactly, so every episode ends
with a win or a loss.

# The world is an argument of the run, and the launcher reads it

A run states its extent, its faction count and its tick limit on the command
line. One function sets all of them and derives the horizon, and it rebuilds
the strategy table, because a level weight divides by the horizon.

The launcher asks this module for the world through one flag, so it holds no
copy. It held two copies of the strategy list once, and a paid instance died
on both.[^6]

# A league run scores a candidate against the seats it played

The run puts one candidate in one world by default. A seat list turns that
into a league: two or more candidates take seats of the same world, and each
is scored by its return minus the mean return of the other learner seats of
that world. Two candidates in one game share the map, the weather and the
opponents, so the margin between them holds almost none of the variance that
either return holds on its own.

**One seat always stays with the built-in controller.** The runner refuses a
seat list that fills every seat, because that removes the opponent the run is
measured against and pins the win rate at one over the faction count.

# Three baselines, and the controller is the one that matters

The untrained policy takes the no-op at every decision, so it measures a
faction that does nothing. The random policy takes one legal row at random,
so it measures a faction that acts without reading the world. The controller
baseline gives the learner's own seat back to the built-in controller, so it
measures the opponent the learner trains against.

**The controller baseline is the real measure.** A policy that beats the
first two and loses to the third has learned to act, not to play.

# References

[^1]: Blockers register, BLK-050. ``docs/BLOCKERS.md``
[^2]: The instrumental population script, which measures the end tick of each
game. ``scripts/instrumental_population.py``
[^3]: Findings register, FND-689. ``docs/FINDINGS.md``
[^4]: Findings register, FND-695. ``docs/FINDINGS.md``
[^5]: Reinforcement learning parameters, the world a training run plays.
``docs/reference/rl-costs.md``
[^6]: Findings register, FND-693. ``docs/FINDINGS.md``
"""

from __future__ import annotations

import argparse
import time
from dataclasses import asdict, replace
from pathlib import Path

from .baseline import available_workers, controller_baseline, controller_baselines
from .env import Env, EnvConfig, viable_seeds
from .policy import (
    LinearPolicy,
    Policy,
    PolicyFit,
    RandomPolicy,
    load_policy,
)
from .presets import ObjectiveSchedule, load_library, schedule_of
from .reward import Scoring, Weighting
from .structured import STRUCTURED_KIND, StructuredPolicy
from .train import TrainConfig, evaluate, first_scoring, train, write_report

# How many ticks one decision covers. The engine changes little in five
# ticks, and a decision costs one boundary crossing for every world of the
# batch, so a wider interval buys ticks with no loss the measurement finds.
DECISION_INTERVAL = 10

# The tick limit of one episode. The engine compares held ground at the
# limit and records a winner, so an episode that reaches the limit still
# ends won or lost.
#
# **This is the world a run trains in, and it is declared here only.** The
# launcher asks the trainer for it and holds no copy, because a launcher that
# held a copy measured a world nobody trained in. A test that wants a cheaper
# world replaces these fields rather than reading a second constant.
TICK_LIMIT = 6000

# The extent of the world every strategy plays, in columns and in rows. The
# world is square, and a run states one number for both sides.
#
# A measurement of four extents chose 128. Eight of sixteen episodes at 48
# ended with the seat holding no settlement at all, and none did at 256, and
# a larger world resolves sooner rather than later.
WORLD_EXTENT = 128

# How many factions play one game, counting the learner seat.
FACTION_COUNT = 3

# The world every strategy plays. The horizon covers the tick limit, so the
# horizon never ends an episode before the game does.
WORLD = EnvConfig(
    width=WORLD_EXTENT,
    height=WORLD_EXTENT,
    faction_count=FACTION_COUNT,
    seat=0,
    tick_limit=TICK_LIMIT,
    horizon=TICK_LIMIT // DECISION_INTERVAL,
    decision_interval=DECISION_INTERVAL,
)

# The same world, with the seat given back to the built-in controller. This
# is the baseline the run is judged against.
CONTROLLER_WORLD = replace(WORLD, controlled=False)


def use_world(
    extent: int,
    faction_count: int,
    tick_limit: int,
    interval: int,
) -> None:
    """Set the world every strategy plays, everywhere the run reads it.

    **Five values reach one world and the horizon is derived from four of
    them.** The world the strategies play holds them, the controller world
    holds the same ones, the horizon is the tick limit divided by the
    interval, and the level weights of the strategy table divide by the
    horizon. A caller that sets one and not the others ends an episode before
    the game ends, and nothing fails. This function is the only place that
    derives the horizon, so the copies cannot disagree.

    **This rebuilds the strategy table rather than replacing the world of
    each row.** A level weight is paid on every decision, so a wider interval
    gives fewer decisions and each one must pay more. A table that kept its
    weights and took a new horizon would pay a fraction of its shaping, and
    nothing would fail.

    The learner takes one action for each decision, and the built-in
    controller issues many commands in the same span, so a shorter interval
    gives the learner more of the say.

    **The observation and the action table do not grow with the extent.** The
    observation of a faction is a fixed-width table over an egocentric frame,
    and the action table names verbs and candidate positions rather than
    tiles, so a policy trained in one world fits another. A probe measured
    both lengths over five extents and read one pair.[^1] The tick cost, the
    tile count the pyramid summarises and the tick a game resolves at all do
    grow, and the probe measured each.

    References
    ----------
    [^1]: The world scale probe. ``scripts/world_scale.py``
    """
    global WORLD, CONTROLLER_WORLD, STRATEGIES
    if interval < 1:
        message = "the decision interval must be one tick or more"
        raise ValueError(message)
    if extent < 1:
        message = "the world extent must be one column and one row or more"
        raise ValueError(message)
    if faction_count < 2:
        message = "a game needs two factions or more"
        raise ValueError(message)
    if tick_limit < interval:
        message = "the tick limit must cover one decision or more"
        raise ValueError(message)
    horizon = tick_limit // interval
    WORLD = replace(
        WORLD,
        width=extent,
        height=extent,
        faction_count=faction_count,
        tick_limit=tick_limit,
        decision_interval=interval,
        horizon=horizon,
    )
    CONTROLLER_WORLD = replace(WORLD, controlled=False)
    STRATEGIES = strategy_table(WORLD)


def world_lines(world: EnvConfig) -> str:
    """Return the world as one name and one value for each line.

    **A launcher reads this rather than holding a world of its own.** The
    launcher sizes its throughput probe from the world the run plays, and a
    probe that measured a different extent would describe a world nobody
    trained in. The launcher held two stale copies of the strategy list once,
    and the run failed after it paid for the instance, so this flag exists to
    keep the same shape from returning over the world.[^1]

    The format is one name, one tab and one value, so a shell reads a field by
    name rather than by position.

    References
    ----------
    [^1]: Findings register, FND-693. ``docs/FINDINGS.md``
    """
    fields = {
        "width": world.width,
        "height": world.height,
        "factions": world.faction_count,
        "tick_limit": world.tick_limit,
        "decision_interval": world.decision_interval,
        "horizon": world.horizon,
    }
    return "\n".join(f"{name}\t{value}" for name, value in fields.items())


def use_decision_interval(interval: int) -> None:
    """Set how many ticks one decision covers, and hold the rest of the world.

    This is the entry point a caller that changes only the interval calls. It
    derives nothing of its own, so the interval and the horizon cannot
    disagree between the two entry points.
    """
    use_world(WORLD.width, WORLD.faction_count, WORLD.tick_limit, interval)


def use_play_styles(
    names: list[str],
    variation: str,
    library_path: Path | None,
    kind: str,
) -> None:
    """Replace the strategy table with the named play styles.

    A play style weights the objective vector of the run, and the table of
    styles is data a researcher edits.[^1] This reads that table, binds every
    objective to the world the strategies play, and gives one strategy for
    each style.

    A variation other than the fixed one gives one strategy that cycles
    through every named style. **Every candidate of one generation then plays
    the same objective**, because a schedule answers for a generation and for
    a seed position and has no argument for a candidate.

    References
    ----------
    [^1]: Report 42, what a policy should be able to see, section 10.3.
    ``docs/research/reports/42-what-a-policy-should-be-able-to-see.md``
    """
    global STRATEGIES
    library = load_library(library_path)
    chosen = names or list(library.names)
    # The catalogue comes from the schema of a probe world, and the probe
    # needs a scoring that states nothing. A weighting with no term and no
    # outcome weight scores every reading at zero, which is what a probe
    # wants: it reads the layout and plays nothing.
    probe = Env(WORLD, Weighting(terms={}, won=0.0, lost=0.0, drawn=0.0))
    if variation == "fixed":
        STRATEGIES = {
            name: (WORLD, library.scoring(name, probe.signals), kind) for name in chosen
        }
        return
    STRATEGIES = {
        "-".join(chosen): (
            WORLD,
            schedule_of(library, chosen, probe.signals, variation),
            kind,
        )
    }


# What a win is worth against what the shaped terms pay over one episode. A
# level term of one weight unit pays at most one on each decision, so it pays
# at most the horizon over a whole episode. This terminal weight is therefore
# above what any one shaped term can pay, and it stays the largest single
# term without drowning the shaping that leads to it.
WIN = 2000.0

# A loss must cost less than a win pays, by the ratio the chance line sets.
# A seat of a symmetric game of three factions takes one third of the wins
# whatever the players do. With a symmetric pair an attempt at a win scores
# a third of the win less two thirds of the loss, which is negative for every
# win rate below one half, so the objective ranks a draw above the attempt and
# the policy learns to survive. This ratio makes the attempt worth making
# above a win rate of about one part in eleven.[^1]
#
# [^1]: Findings register, FND-679. `docs/FINDINGS.md`
LOSS = WIN / 10.0

# What the time left on the clock pays on a win. A win with the whole tick
# limit still to run pays this much above the win weight, and a win at the
# tick limit pays nothing above it. The territory path decides a game at the
# limit, so a game that runs out the clock still ends won or lost.
#
# **This weight makes an early win the best outcome and changes no ordering.**
# The smallest win still pays the win weight, which is ten times what a loss
# costs, so no win ranks below a loss. The loss weight therefore does not move
# for this term. Both the loss weight and this weight push a policy away from
# playing for the clock, and this one pushes on the win side alone, so neither
# overtakes the other.[^1] [^2]
#
# Only the two conquest strategies carry it. Their trained policies sit still
# and let the clock run out, which is the behaviour this term ranks last.
#
# [^1]: Findings register, FND-679. `docs/FINDINGS.md`
# [^2]: Findings register, FND-692. `docs/FINDINGS.md`
EARLY = WIN / 2.0

# How many settlements the founding ladder must keep worth founding. The
# rise of the settlement level for one more settlement falls as the count
# rises, so the ladder ratio below answers for a settlement count and no
# more.
SETTLEMENT_TARGET = 8

# What the readiness to found a settlement pays, against what a settlement
# pays. The engine sets one field when the settle verb is legal for the
# faction now, which means a settler stands on ground the faction may build
# on. Founding the settlement spends the settler and clears the field.
#
# **A readiness weight above this ratio pays a faction to hold the settler
# and never found.** The bound is the rise of the settlement level for one
# more settlement at the target count above. One test derives that bound from
# the schema of the world and fails when this ratio passes it, so the two
# cannot disagree.
FOUND_READY = 0.003


StrategyTable = dict[str, tuple[EnvConfig, "Scoring | ObjectiveSchedule", str]]

# The signals each strategy below reads as a level, named once. A weighting
# names a field of the observation schema, and the engine owns that name.
_TILES = "held_tiles"
_SETTLEMENTS = "settlements"
_MAY_FOUND = "may_found"
_STORE = "store_total"
_PEOPLE = "population"


def strategy_table(world: EnvConfig) -> StrategyTable:
    """Return one strategy for each weighting this run trains against.

    **Every shaped weight below reads a level and not a change.** An
    evolution strategy sums the reward of every decision of the episode with
    no discount, so a sum of changes collapses to the last reading less the
    first. Every shaped weight of this table read a change once, so the
    whole table trained against a terminal reward under weights that read as
    dense.[^1] [^2]

    A level weight reads the published value of one field divided by the unit
    the engine published for it, so every level lies between minus one and
    one. The engine compresses each count before it publishes one, so a
    weight means the same thing over a tile total and over a store total.
    That was not true of the change form: a weight there multiplied the raw
    published value, and one weight of the table carried a factor of a
    hundred thousand that the compression had already removed.[^2]

    The world argument gives the horizon, which is how many decisions a level
    weight is paid on. **The weights are therefore a function of the world
    and not a constant.** A caller that sets the decision interval changes
    the horizon, and this rebuilds the table against it.

    References
    ----------
    [^1]: Findings register, FND-679. `docs/FINDINGS.md`
    [^2]: Findings register, FND-700. `docs/FINDINGS.md`
    """
    level = WIN / max(world.horizon, 1)
    found_ready = level * FOUND_READY

    # Win, and almost nothing else. The small territory term is the only
    # thing that separates two candidates that both lost, and without it the
    # first generations hold no signal at all.
    #
    # **The settlement term is the one shaped term of this row that a policy
    # reaches in several steps.** A faction founds a city by queueing a
    # settler, waiting for it, and settling with it, and no other term of the
    # table pays anything at any step of that chain. A city is how the engine
    # grows a faction, so the chain leads to the win this row is about.
    conquest = Weighting(
        levels={
            _TILES: level * 0.1,
            _SETTLEMENTS: level * 0.5,
            _MAY_FOUND: found_ready * 0.5,
        },
        won=WIN,
        lost=-LOSS,
        drawn=0.0,
        won_early=EARLY,
    )
    # Take ground, and plant seats on it. The two terms carry one weight
    # each, because one more settlement and one more tile both add the same
    # amount to their own compressed magnitude at the first step. A
    # settlement is then worth many tiles at the margin, because the tile
    # count is far higher and the compression flattens with the count.
    ground = Weighting(
        levels={
            _TILES: level * 1.0,
            _SETTLEMENTS: level * 1.0,
            _MAY_FOUND: found_ready,
        },
        won=WIN,
        lost=-LOSS,
        drawn=0.0,
    )
    # Fill the stores. Ground scores a little, because a faction with no
    # ground fills nothing.
    riches = Weighting(
        levels={_STORE: level * 1.0, _TILES: level * 0.5},
        won=WIN,
        lost=-LOSS,
        drawn=0.0,
    )
    # Grow the people. Ground scores a little, because a faction with no
    # ground grows nobody.
    #
    # **The population field and the live unit field publish one number.**
    # Every decision of every episode a measurement played read the same
    # value in both, so this row rewards the unit count under the name of the
    # people. The row keeps its name, because a stored policy carries it.[^1]
    #
    # [^1]: Findings register, FND-702. `docs/FINDINGS.md`
    people = Weighting(
        levels={_PEOPLE: level * 3.0, _TILES: level * 0.25},
        won=WIN,
        lost=-LOSS,
        drawn=0.0,
    )
    return {
        "conquer": (world, conquest, "linear"),
        # The same scoring as the conquest strategy, over the structured
        # policy. This varies the policy and holds the reward fixed, so the
        # pair measures what the structure is worth.
        "conquer-structured": (world, conquest, STRUCTURED_KIND),
        "land": (world, ground, "linear"),
        # The same scoring as the ground strategy, over the structured
        # policy. **The pair measures the structure against a dense score.**
        # The conquest pair measures it against a nearly ternary one, and
        # that pair went flat after five generations while the ground
        # strategy was still rising at sixty-seven. Neither pair alone says
        # whether the policy or the density carried it.[^1]
        #
        # [^1]: Findings register, FND-650. `docs/FINDINGS.md`
        "land-structured": (world, ground, STRUCTURED_KIND),
        "wealth": (world, riches, "linear"),
        # The same scoring as the wealth strategy, over the structured policy.
        "wealth-structured": (world, riches, STRUCTURED_KIND),
        "people": (world, people, "linear"),
        # The same scoring as the people strategy, over the structured policy.
        "people-structured": (world, people, STRUCTURED_KIND),
    }


STRATEGIES: StrategyTable = strategy_table(WORLD)


def report_behaviour(names: list[str], out: Path, holdout: int, workers: int) -> int:
    """Say what each stored policy does, verb by verb, and write it out.

    A score says that a policy is better. It does not say what the policy
    does. This pass plays each stored policy on the held-out seeds and counts
    which verb it chose, so a reader sees the behaviour beside the number.

    The two acting baselines run through the same counter, so a reader
    compares the verb mix of a trained policy against the verb mix of a
    faction that acts at random.
    """
    from .inspect import behaviour

    seeds = viable_seeds(WORLD, holdout, 50_000)
    probe = Env(WORLD, first_scoring(STRATEGIES[names[0]][1]))
    rows: dict[str, dict[str, object]] = {}

    for index, name in enumerate(names):
        env_config, scoring, kind = STRATEGIES[name]
        path = out / f"{name}.npz"
        if not path.exists():
            print(f"  {name}: no stored policy at {path}", flush=True)
            continue
        # **The stored policy must fit the world this report plays it on.**
        # The strategy table names a world for each policy, and a table that
        # moves after a run leaves files that read the right length and mean
        # something else.
        fixed = first_scoring(scoring)
        policy, meta = load_policy(path, PolicyFit.of_env(Env(env_config, fixed)))
        rows[name] = {
            "kind": str(meta["kind"]),
            **behaviour(env_config, fixed, policy, seeds, workers),
        }
        print(f"  {name}: {rows[name]['verbs']}", flush=True)
        del index, kind

    scoring = first_scoring(STRATEGIES[names[0]][1])
    rows["random"] = behaviour(WORLD, scoring, RandomPolicy(seed=0), seeds, workers)
    print(f"  random: {rows['random']['verbs']}", flush=True)
    rows["untrained"] = behaviour(
        WORLD,
        scoring,
        LinearPolicy.zeros(probe.action_length, probe.observation_length),
        seeds,
        workers,
    )
    write_report(out / "behaviour.json", rows)
    print(f"wrote {out / 'behaviour.json'}", flush=True)
    return 0


def fill_baseline_cache(
    names: list[str], holdout: list[int], workers: int, probe: Env
) -> int:
    """Measure the controller baseline of each named strategy into the cache.

    **The launcher starts one trainer process for each strategy, and every one
    of them needs this number.** Six processes that each measure it play the
    same worlds six times, and each of them holds a sixth of the cores while
    it does. This pass runs once, before any trainer starts, and it may hold
    every core.

    **One set of games answers for every strategy.** The games come from the
    world and the seed set, and the objective weights their readings. The
    strategies that miss therefore play one batch, and the pass scores that
    batch once for each of them. A seventh strategy then costs the arithmetic
    of one more scorer.

    Two strategies that hold the same objective share one number. The second
    of them reads what the first wrote, so this pass needs no list of its own
    of which objectives differ.

    **Every strategy plays the same world here.** The strategy table names a
    world for each strategy, and a table that gave two strategies two
    different worlds could not share one batch between them. This refuses
    such a table rather than reporting one world under the name of another.
    """
    world = replace(STRATEGIES[names[0]][0], controlled=False)
    other = [
        name
        for name in names
        if replace(STRATEGIES[name][0], controlled=False) != world
    ]
    if other:
        message = (
            f"{other} name another world than {names[0]!r}, and one batch "
            "plays one world. Measure them in separate passes."
        )
        raise ValueError(message)
    measured = controller_baselines(
        world,
        {name: first_scoring(STRATEGIES[name][1]) for name in names},
        LinearPolicy.zeros(probe.action_length, probe.observation_length),
        holdout,
        workers,
        probe.observation_version,
        "baseline",
    )
    for name in names:
        summary, source = measured[name]
        print(
            f"  {name} controller {source} return {summary['return']:10.1f} "
            f"won {summary['won']:5.2f}",
            flush=True,
        )
    return 0


def main() -> int:
    """Train each named strategy, measure it against the baselines, report."""
    parser = argparse.ArgumentParser(description="Train the learner seat.")
    parser.add_argument("--out", type=Path, default=Path("runs/learn"))
    parser.add_argument("--generations", type=int, default=20)
    parser.add_argument("--population", type=int, default=24)
    parser.add_argument("--seeds", type=int, default=6)
    parser.add_argument(
        "--workers",
        type=int,
        default=16,
        help=(
            "how many engine workers one process gives its batch. This is a "
            "per process count, so a run of five shards with twelve workers "
            "asks for sixty workers on the machine"
        ),
    )
    # **The pass that fills the baseline cache runs alone, so it takes the
    # whole machine.** Every trainer of the run waits for it, and it took the
    # per process worker count of a trainer instead. That held the pass over
    # the whole held-out seed set to a tenth of a rented machine of sixty
    # four cores, and nothing failed. Zero asks the machine what it has.
    parser.add_argument(
        "--baseline-workers",
        type=int,
        default=0,
        help=(
            "how many engine workers the pass that fills the baseline cache "
            "gives its batch. Zero takes every core the machine offers, "
            "because that pass runs alone and every trainer waits for it"
        ),
    )
    parser.add_argument(
        "--shards",
        type=int,
        default=1,
        help=(
            "how many worker processes score one generation. One process "
            "scores it here and starts nothing. The candidates are split "
            "across the processes and the scores are combined in candidate "
            "order, so the weights do not depend on this number"
        ),
    )
    # **The holdout decides whether a run achieved anything, so its size
    # sets what the run can claim.** A win share is a proportion, and the
    # error of a proportion near one third over n worlds is the square root
    # of 0.333 times 0.667 over n. Over twenty-four worlds that is 9.6
    # points, so two policies must differ by about nineteen points before
    # the holdout can separate them. The gap between a trained policy and
    # the built-in controller was measured at about seventeen, and a report
    # of twenty-four worlds therefore called four different policies the
    # same thing while reading as though it had measured them.
    #
    # Two hundred and fifty-six worlds bring the error to 2.9 points. The
    # holdout runs once, at the end, so the whole cost is a few minutes
    # against a run of hours.
    parser.add_argument("--holdout", type=int, default=256)
    parser.add_argument(
        "--sigma",
        type=float,
        default=TrainConfig.sigma,
        help=(
            "the fraction of the centre that one candidate moves. The "
            "trainer configuration holds the only declaration of the "
            "default, and this argument reads it rather than restating it. A "
            "measurement fixed the value against the signal it buys and the "
            "worlds it needs. The run reports on its first lines whether the "
            "world count covers the sigma it was given"
        ),
    )
    parser.add_argument("--learning-rate", type=float, default=0.3)
    parser.add_argument("--validation", type=int, default=6)
    parser.add_argument("--validate-every", type=int, default=3)
    # **The held-out pass must leave a figure behind before a run ends.** The
    # validation seeds choose the centre, so a validation figure is a maximum
    # over the passes of the run on the seeds that did the choosing. The
    # held-out seeds choose nothing, and that pass used to run only after a
    # strategy returned. A wall clock cap ended a paid run before any
    # strategy returned, so no held-out figure existed for any of the four
    # policies it published, and the index called the selection figure held
    # out.
    #
    # The interval works in the way the validation interval does. Zero turns
    # the periodic pass off, which leaves only the pass at the end.
    parser.add_argument(
        "--holdout-every",
        type=int,
        default=5,
        help=(
            "how many generations pass between two held-out measurements of "
            "the best centre. The held-out seeds choose nothing, so this is "
            "the only honest figure a run leaves behind before it ends. Zero "
            "turns the periodic pass off"
        ),
    )
    parser.add_argument(
        "--decision-interval",
        type=int,
        default=DECISION_INTERVAL,
        help=(
            "how many ticks one decision covers. The learner takes one action "
            "for each decision, and the built-in controller issues many "
            "commands in the same span, so a shorter interval gives the "
            f"learner more of the say. Default {DECISION_INTERVAL}"
        ),
    )
    # **The world extent reaches the launcher as well as the trainer.** The
    # launcher sizes its throughput probe from the world the run plays, and it
    # asks the trainer for that world rather than holding a copy. A launcher
    # that held a copy measured the wrong world, and that shape has already
    # cost this project a paid instance once, over the strategy list.
    parser.add_argument(
        "--world-extent",
        type=int,
        default=WORLD_EXTENT,
        help=(
            "how many columns and rows the world holds. The world is square, "
            "so one number states both sides. A doubling of this buys one "
            "more ring of the observation frame and about four times the "
            f"tile count. Default {WORLD_EXTENT}"
        ),
    )
    parser.add_argument(
        "--factions",
        type=int,
        default=FACTION_COUNT,
        help=(
            "how many factions play one game, counting the learner seat. The "
            "win share of a policy that plays no better than chance is one "
            f"over this number. Default {FACTION_COUNT}"
        ),
    )
    parser.add_argument(
        "--tick-limit",
        type=int,
        default=TICK_LIMIT,
        help=(
            "how many ticks one episode runs before the engine compares held "
            "ground and names a winner. A larger world takes longer to "
            "resolve, so a limit that does not move with the extent ends "
            f"every game at the limit. Default {TICK_LIMIT}"
        ),
    )
    # **The candidate pass answers one question, and it has answered it.**
    # The highest candidate of a generation is the highest of many draws on
    # a few worlds, so it is usually the luckiest and not the best. Playing
    # it on the validation worlds says which. It was measured three times
    # and gave the same answer each time: the candidate scores no better
    # than the centre, and once it scored well below it. The pass costs one
    # validation set for each validating generation, so it is off unless a
    # caller asks for it again.
    parser.add_argument(
        "--validate-candidate",
        action="store_true",
        help=(
            "play the highest candidate of a generation on the validation "
            "worlds as well as the centre. Off by default"
        ),
    )
    parser.add_argument("--only", type=str, default="")
    parser.add_argument(
        "--print-strategies",
        action="store_true",
        help=(
            "print the names this run would train, one line separated "
            "by spaces, and exit without training. A launcher asks for "
            "the names through this flag, so no launcher holds a list "
            "of its own"
        ),
    )
    parser.add_argument(
        "--print-world",
        action="store_true",
        help=(
            "print the world this run would play, as one name and one value "
            "for each line, and exit without training. A launcher asks for "
            "the world through this flag, so no launcher holds a world of "
            "its own"
        ),
    )
    parser.add_argument(
        "--styles",
        type=str,
        default="",
        help=(
            "train against the play styles of the style table rather than "
            "against the built-in weightings, named as a comma separated "
            "list. An empty value with --style-variation set takes every "
            "style of the table"
        ),
    )
    parser.add_argument(
        "--style-variation",
        type=str,
        default="fixed",
        help=(
            "how the objective moves inside one run: fixed holds one style, "
            "generation moves to the next style at each generation, and "
            "episode gives each seed position its own style. A run cannot "
            "vary the objective between the candidates of one generation, "
            "because an evolution strategy ranks them against each other"
        ),
    )
    parser.add_argument(
        "--style-table",
        type=Path,
        default=None,
        help="read the play styles from this file rather than the shipped one",
    )
    parser.add_argument(
        "--style-kind",
        type=str,
        default="linear",
        choices=("linear", STRUCTURED_KIND),
        help="which policy each play style trains",
    )
    parser.add_argument(
        "--league",
        type=str,
        default="",
        help=(
            "seat two or more candidates in each world, named as a comma "
            "separated seat list such as 0,1. A candidate is then scored by "
            "its margin against the other seats of its own world, and its "
            "seat turns by one position at each seed"
        ),
    )
    parser.add_argument(
        "--absolute-scoring",
        action="store_true",
        help=(
            "rank a seated generation by the raw return rather than by the "
            "margin against the other seats of one world"
        ),
    )
    parser.add_argument(
        "--resume",
        action="store_true",
        help="start each strategy from the weights already stored under --out",
    )
    parser.add_argument(
        "--behaviour",
        action="store_true",
        help="read the stored policies and report what they do, and train nothing",
    )
    parser.add_argument(
        "--baseline-only",
        action="store_true",
        help=(
            "measure the controller baseline of each named strategy into the "
            "cache, and train nothing. The launcher runs this once with every "
            "core before it starts one trainer for each strategy, so the "
            "trainers read the number rather than measure it"
        ),
    )
    arguments = parser.parse_args()

    # The world is set before anything reads one, so every strategy, the
    # controller world and the report all state the same one.
    use_world(
        arguments.world_extent,
        arguments.factions,
        arguments.tick_limit,
        arguments.decision_interval,
    )

    # The play styles replace the strategy table, so they are chosen before
    # anything reads the table. A run that names none keeps the built-in
    # weightings.
    styles = [name for name in arguments.styles.split(",") if name]
    if styles or arguments.style_variation != "fixed":
        use_play_styles(
            styles,
            arguments.style_variation,
            arguments.style_table,
            arguments.style_kind,
        )

    names = [name for name in arguments.only.split(",") if name] or list(STRATEGIES)

    if arguments.print_strategies:
        print(" ".join(names))
        return 0
    if arguments.print_world:
        print(world_lines(WORLD))
        return 0
    learner_seats = tuple(
        int(seat) for seat in arguments.league.split(",") if seat.strip()
    )
    # **A relative score cannot say whether the population improved**, so a
    # league run measures the centre against the built-in controller on every
    # generation rather than every third one.
    validate_every = 1 if learner_seats else arguments.validate_every
    out = arguments.out
    out.mkdir(parents=True, exist_ok=True)
    if arguments.behaviour:
        return report_behaviour(names, out, arguments.holdout, arguments.workers)

    # The training pool and the holdout share no seed, so a reported figure
    # comes from a world the policy never trained on.
    holdout = viable_seeds(WORLD, arguments.holdout, 50_000)

    # **The pass that only fills the cache trains nothing, so it takes no
    # training pool.** The pool holds one set of seeds for each generation,
    # and finding them builds a world for each candidate seed. This pass runs
    # before every trainer of a run starts, and every second it takes is a
    # second the training does not get.
    if arguments.baseline_only:
        return fill_baseline_cache(
            names,
            holdout,
            arguments.baseline_workers or available_workers(),
            Env(WORLD, first_scoring(STRATEGIES[names[0]][1])),
        )

    pool = viable_seeds(WORLD, arguments.generations * arguments.seeds + 8, 1000)
    # The validation seeds pick the checkpoint. They share nothing with the
    # training pool and nothing with the held-out set, so the figure the
    # report is judged on never chose the policy it reports.
    validation = viable_seeds(WORLD, arguments.validation, 20_000)
    print(
        f"training seeds {len(pool)}, validation seeds {validation}, "
        f"holdout seeds {len(holdout)}",
        flush=True,
    )

    report: dict[str, object] = {
        "holdout": holdout,
        "generations": arguments.generations,
        "population": arguments.population,
        "seeds_per_generation": arguments.seeds,
        "tick_limit": WORLD.tick_limit,
        "horizon": WORLD.horizon,
        "decision_interval": WORLD.decision_interval,
        "validation": validation,
        "holdout_every": arguments.holdout_every,
        "sigma": arguments.sigma,
        "learning_rate": arguments.learning_rate,
        "learner_seats": list(learner_seats),
        "workers": arguments.workers,
        "shards": arguments.shards,
        "relative_scoring": bool(learner_seats) and not arguments.absolute_scoring,
        "world": asdict(WORLD),
        "strategies": {},
    }
    started = time.time()

    # The schema states the lengths, and this module states none of its own.
    # A second declaration of a length that the engine already declares is
    # the defect shape this project names first.
    probe = Env(WORLD, first_scoring(STRATEGIES[names[0]][1]))
    actions, features = probe.action_length, probe.observation_length

    def no_op(kind: str) -> Policy:
        """Return the untrained policy of one kind, which takes the no-op.

        The structured kind reads the whole layout, so it takes the signal
        catalogue of the probe rather than the two lengths.
        """
        if kind == STRUCTURED_KIND:
            return StructuredPolicy.of_catalogue(actions, probe.signals)
        return LinearPolicy.zeros(actions, features)

    # **The controller baseline is one number for the whole run, and one
    # process measured it once for every strategy it trained.** The play is a
    # function of the engine, the world, the seeds and the objective, and of
    # nothing a run trains. The launcher starts one process for each
    # strategy, so a run of six strategies paid for the same number twelve
    # times: once before each strategy and once after it. A cache holds it
    # now, keyed on every input, and a process that finds it pays nothing.
    #
    # The weighting reaches the key because the reading is weighted. The play
    # does not change with the weighting, and the return does.
    controller_scoring = first_scoring(STRATEGIES[names[0]][1])
    print("\n=== controller baseline ===", flush=True)
    report["controller"], source = controller_baseline(
        CONTROLLER_WORLD,
        controller_scoring,
        no_op("linear"),
        holdout,
        arguments.workers,
        probe.observation_version,
        f"{names[0]} baseline",
    )
    # **The line below is the interface the dashboard reads.** Two readers
    # match it by shape, so the source of the number goes on its own line
    # rather than inside this one.
    print(f"  controller {report['controller']}", flush=True)
    print(f"  the controller baseline was {source}", flush=True)
    write_report(out / "report.json", report)

    for index, name in enumerate(names):
        env_config, scoring, kind = STRATEGIES[name]
        print(f"\n=== {name} ({kind}) ===", flush=True)
        train_config = TrainConfig(
            generations=arguments.generations,
            population=arguments.population,
            seeds_per_generation=arguments.seeds,
            sigma=arguments.sigma,
            learning_rate=arguments.learning_rate,
            workers=arguments.workers,
            shards=arguments.shards,
            seed=index,
            learner_seats=learner_seats,
            relative=not arguments.absolute_scoring,
            validate_candidate=arguments.validate_candidate,
        )
        result = train(
            name,
            env_config,
            scoring,
            train_config,
            out,
            pool,
            kind=kind,
            resume=arguments.resume,
            validation=validation,
            validate_every=validate_every,
            holdout=holdout,
            holdout_every=arguments.holdout_every,
        )
        trained, _ = load_policy(Path(result["weights"]))
        untrained = no_op(kind)
        # **The holdout measurement holds one objective for the whole
        # strategy.** A schedule moves the objective between generations, and
        # two numbers taken under two objectives cannot be compared. The pass
        # therefore takes the first scoring, which is what the run started
        # under and what the validation pass held.
        fixed = first_scoring(scoring)
        measured = {
            "trained": evaluate(env_config, fixed, trained, holdout, arguments.workers),
            "untrained": evaluate(
                env_config, fixed, untrained, holdout, arguments.workers
            ),
            "random": evaluate(
                env_config,
                fixed,
                RandomPolicy(seed=index),
                holdout,
                arguments.workers,
                repeats=3,
            ),
            "controller": controller_baseline(
                CONTROLLER_WORLD,
                fixed,
                untrained,
                holdout,
                arguments.workers,
                probe.observation_version,
                f"{name} baseline",
            )[0],
        }
        result["holdout"] = measured
        report["strategies"][name] = result  # type: ignore[index]
        # **A generation that carried no information hides inside a mean and a
        # best.** Every candidate scored the same number, so the mean equals
        # the best, and that reads like a population which agreed. The trainer
        # names each one as it happens, and this line names them again beside
        # the held-out figures, where a reader who reads only the end of a
        # strategy still meets them.
        wasted = result["degenerate_generations"]
        if wasted:
            print(
                f"  {len(wasted)} of {arguments.generations} generations carried "
                f"no information and moved no centre: {wasted}",
                flush=True,
            )
        for label in ("trained", "untrained", "random", "controller"):
            row = measured[label]
            print(
                f"  {label:11s} return {row['return']:10.1f} "
                f"tiles {row['held_tiles']:7.1f} won {row['won']:5.2f} "
                f"lost {row['lost']:5.2f}",
                flush=True,
            )
        write_report(out / "report.json", report)

    report["seconds"] = round(time.time() - started, 1)
    write_report(out / "report.json", report)
    print(f"\nwrote {out / 'report.json'} in {report['seconds']}s", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
