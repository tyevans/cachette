"""Train several policies, each against a different weighting, and report.

Each strategy below names what its reward is worth. **The weights are the
strategy.** A policy trained against territory and a policy trained against
people play the same game under different scoring, and the point of the run
is to see whether they end up playing it differently.

Every weight here belongs to this run. None of them states a rule of the
downstream game, and one blocker holds that question open.[^1]

# The horizon must reach the end of the game

The engine ends a game by domination, by a wonder, by renown, or by a
comparison of held ground at the tick limit. A measurement of twenty-four
worlds at extent 48 with three factions found that a game against the
built-in controller resolves anywhere from about two hundred ticks to the
tick limit, with a middle near two thousand.[^2] A horizon shorter than that
truncates the episode, and a truncated episode reports no outcome. A policy
therefore cannot be paid for a win it never reached.

The horizon of this run covers the tick limit exactly, so every episode ends
with a win or a loss.

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
[^2]: Reinforcement learning parameters, the measured run.
``docs/reference/rl-costs.md``
"""

from __future__ import annotations

import argparse
import time
from dataclasses import asdict, replace
from pathlib import Path

from .env import Env, EnvConfig, viable_seeds
from .policy import (
    LinearPolicy,
    MLPPolicy,
    Policy,
    PolicyFit,
    RandomPolicy,
    load_policy,
)
from .reward import Weighting
from .train import TrainConfig, evaluate, train, write_report

# How many ticks one decision covers. The engine changes little in five
# ticks, and a decision costs one boundary crossing for every world of the
# batch, so a wider interval buys ticks with no loss the measurement finds.
DECISION_INTERVAL = 10

# The tick limit of one episode. The engine compares held ground at the
# limit and records a winner, so an episode that reaches the limit still
# ends won or lost.
TICK_LIMIT = 2500

# The world every strategy plays. The horizon covers the tick limit, so the
# horizon never ends an episode before the game does.
WORLD = EnvConfig(
    width=48,
    height=48,
    faction_count=3,
    seat=0,
    tick_limit=TICK_LIMIT,
    horizon=TICK_LIMIT // DECISION_INTERVAL,
    decision_interval=DECISION_INTERVAL,
)

# The same world, with the seat given back to the built-in controller. This
# is the baseline the run is judged against.
CONTROLLER_WORLD = replace(WORLD, controlled=False)


def use_decision_interval(interval: int) -> None:
    """Set how many ticks one decision covers, everywhere it is read.

    **The interval reaches three places and the horizon is derived from it.**
    The world the strategies play holds it, the controller world holds it, and
    the horizon is the tick limit divided by it. A caller that sets one and
    not the others ends an episode before the game ends, and nothing fails.
    This function is the only place that derives the horizon, so the copies
    cannot disagree.

    The learner takes one action for each decision, and the built-in
    controller issues many commands in the same span, so a shorter interval
    gives the learner more of the say.
    """
    global WORLD, CONTROLLER_WORLD, STRATEGIES
    if interval < 1:
        message = "the decision interval must be one tick or more"
        raise ValueError(message)
    horizon = TICK_LIMIT // interval
    WORLD = replace(WORLD, decision_interval=interval, horizon=horizon)
    CONTROLLER_WORLD = replace(WORLD, controlled=False)
    STRATEGIES = {
        name: (
            replace(config, decision_interval=interval, horizon=horizon),
            weighting,
            kind,
        )
        for name, (config, weighting, kind) in STRATEGIES.items()
    }

# The store total crosses as a raw Q16.16 integer, so its numbers are about
# five orders of magnitude above a tile count. This weight brings one store
# into the range of one territory.
STORE_SCALE = 1.0e-5

# What a win is worth against what the shaped terms pay over one episode. A
# territory reward of one for each tile pays a few hundred over an episode,
# so a terminal weight of this size makes the outcome the largest single
# term without drowning the shaping that leads to it.
WIN = 2000.0

STRATEGIES: dict[str, tuple[EnvConfig, Weighting, str]] = {
    # Win, and almost nothing else. The small territory term is the only
    # thing that separates two candidates that both lost, and without it the
    # first generations hold no signal at all.
    "conquer": (
        WORLD,
        Weighting(terms={"held_tiles": 0.1}, won=WIN, lost=-WIN, drawn=0.0),
        "linear",
    ),
    # The same scoring as the conquest strategy, over a policy with one
    # hidden layer. This varies the policy and holds the reward fixed, so
    # the pair measures what the depth is worth.
    "conquer-net": (
        WORLD,
        Weighting(terms={"held_tiles": 0.1}, won=WIN, lost=-WIN, drawn=0.0),
        "mlp",
    ),
    # Take ground and hold it. Nothing else scores.
    "land": (
        WORLD,
        Weighting(terms={"held_tiles": 1.0}, won=WIN, lost=-WIN, drawn=0.0),
        "linear",
    ),
    # Fill the stores. Ground scores a little, for the same reason.
    "wealth": (
        WORLD,
        Weighting(
            terms={"store_total": STORE_SCALE, "held_tiles": 0.5},
            won=WIN,
            lost=-WIN,
            drawn=0.0,
        ),
        "linear",
    ),
    # Grow the people. Ground scores a little, because a faction with no
    # ground grows nobody.
    "people": (
        WORLD,
        Weighting(
            terms={"population": 3.0, "held_tiles": 0.25},
            won=WIN,
            lost=-WIN,
            drawn=0.0,
        ),
        "linear",
    ),
}


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
    probe = Env(WORLD, STRATEGIES[names[0]][1])
    rows: dict[str, dict[str, object]] = {}

    for index, name in enumerate(names):
        env_config, weighting, kind = STRATEGIES[name]
        path = out / f"{name}.npz"
        if not path.exists():
            print(f"  {name}: no stored policy at {path}", flush=True)
            continue
        # **The stored policy must fit the world this report plays it on.**
        # The strategy table names a world for each policy, and a table that
        # moves after a run leaves files that read the right length and mean
        # something else.
        policy, meta = load_policy(path, PolicyFit.of_env(Env(env_config, weighting)))
        rows[name] = {
            "kind": str(meta["kind"]),
            **behaviour(env_config, weighting, policy, seeds, workers),
        }
        print(f"  {name}: {rows[name]['verbs']}", flush=True)
        del index, kind

    weighting = STRATEGIES[names[0]][1]
    rows["random"] = behaviour(WORLD, weighting, RandomPolicy(seed=0), seeds, workers)
    print(f"  random: {rows['random']['verbs']}", flush=True)
    rows["untrained"] = behaviour(
        WORLD,
        weighting,
        LinearPolicy.zeros(probe.action_length, probe.observation_length),
        seeds,
        workers,
    )
    write_report(out / "behaviour.json", rows)
    print(f"wrote {out / 'behaviour.json'}", flush=True)
    return 0


def main() -> int:
    """Train each named strategy, measure it against the baselines, report."""
    parser = argparse.ArgumentParser(description="Train the learner seat.")
    parser.add_argument("--out", type=Path, default=Path("runs/learn"))
    parser.add_argument("--generations", type=int, default=20)
    parser.add_argument("--population", type=int, default=24)
    parser.add_argument("--seeds", type=int, default=6)
    parser.add_argument("--workers", type=int, default=16)
    parser.add_argument("--holdout", type=int, default=24)
    parser.add_argument("--hidden", type=int, default=24)
    # Sigma is a relative size, and its working range was measured rather
    # than guessed. On the real decisions of a trained policy, a
    # perturbation of 0.25 changed 0.8 percent of the choices and one of 1.5
    # changed about a third of them.
    parser.add_argument("--sigma", type=float, default=1.5)
    parser.add_argument("--learning-rate", type=float, default=0.3)
    parser.add_argument("--validation", type=int, default=6)
    parser.add_argument("--validate-every", type=int, default=3)
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
    parser.add_argument("--only", type=str, default="")
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
    arguments = parser.parse_args()

    # The interval is set before anything reads a world, so every strategy,
    # the controller world and the report all state the same one.
    use_decision_interval(arguments.decision_interval)

    names = [name for name in arguments.only.split(",") if name] or list(STRATEGIES)
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
    pool = viable_seeds(WORLD, arguments.generations * arguments.seeds + 8, 1000)
    holdout = viable_seeds(WORLD, arguments.holdout, 50_000)
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
        "sigma": arguments.sigma,
        "learning_rate": arguments.learning_rate,
        "learner_seats": list(learner_seats),
        "relative_scoring": bool(learner_seats) and not arguments.absolute_scoring,
        "world": asdict(WORLD),
        "strategies": {},
    }
    started = time.time()

    # The schema states the lengths, and this module states none of its own.
    # A second declaration of a length that the engine already declares is
    # the defect shape this project names first.
    probe = Env(WORLD, STRATEGIES[names[0]][1])
    actions, features = probe.action_length, probe.observation_length

    def no_op(kind: str) -> Policy:
        """Return the untrained policy of one kind, which takes the no-op."""
        if kind == "mlp":
            return MLPPolicy.zeros(actions, features, arguments.hidden)
        return LinearPolicy.zeros(actions, features)

    # The controller baseline does not depend on the strategy, so the run
    # measures it once and every strategy is reported against it. The
    # weighting only scores the reading, and the reading is the same play.
    controller_weighting = STRATEGIES[names[0]][1]
    print("\n=== controller baseline ===", flush=True)
    report["controller"] = evaluate(
        CONTROLLER_WORLD,
        controller_weighting,
        no_op("linear"),
        holdout,
        arguments.workers,
    )
    print(f"  controller {report['controller']}", flush=True)
    write_report(out / "report.json", report)

    for index, name in enumerate(names):
        env_config, weighting, kind = STRATEGIES[name]
        print(f"\n=== {name} ({kind}) ===", flush=True)
        train_config = TrainConfig(
            generations=arguments.generations,
            population=arguments.population,
            seeds_per_generation=arguments.seeds,
            sigma=arguments.sigma,
            learning_rate=arguments.learning_rate,
            workers=arguments.workers,
            seed=index,
            learner_seats=learner_seats,
            relative=not arguments.absolute_scoring,
        )
        result = train(
            name,
            env_config,
            weighting,
            train_config,
            out,
            pool,
            kind=kind,
            hidden=arguments.hidden,
            resume=arguments.resume,
            validation=validation,
            validate_every=validate_every,
        )
        trained, _ = load_policy(Path(result["weights"]))
        untrained = no_op(kind)
        measured = {
            "trained": evaluate(
                env_config, weighting, trained, holdout, arguments.workers
            ),
            "untrained": evaluate(
                env_config, weighting, untrained, holdout, arguments.workers
            ),
            "random": evaluate(
                env_config,
                weighting,
                RandomPolicy(seed=index),
                holdout,
                arguments.workers,
                repeats=3,
            ),
            "controller": evaluate(
                CONTROLLER_WORLD, weighting, untrained, holdout, arguments.workers
            ),
        }
        result["holdout"] = measured
        report["strategies"][name] = result  # type: ignore[index]
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
