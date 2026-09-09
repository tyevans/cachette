"""Record what the built-in controller does, and fit a policy to it.

An episode scored by its return yields one number for about two hundred and
fifty decisions. The same episode holds what the built-in controller chose at
every one of those decisions. This module reads those choices and turns one
episode into many labelled samples.

**This is a diagnostic before it is a training method.** A policy that cannot
reproduce controller play from the observation array says that the array is
missing what the controller reads. A policy that can reproduce it says that
the representation is sufficient and the optimiser was the problem. Nothing
else this project can run separates those two answers.

# The controller and the learner do not act on the same clock

The controller emits several commands in one tick, one for each entry of its
draw order. The learner emits one action every few ticks. **The two do not
map one to one, and no reduction of the window is lossless.** One decision
record states how a window becomes a label, and states what it rejected.[^1]

The rule is this. Every command of the window shares the observation the
learner read at the start of the window. The label of the window is therefore
a distribution over the action table, and the fit minimises the cross entropy
against it. A window that holds no command carries the no-op, because that is
what the controller did.

# A choice the table cannot express is counted, never dropped

The engine states, for each command, whether the action table could express
the choice. This module counts the commands that it could not, and reports
the share. A silent drop would teach the fit a label the controller never
chose.[^2]

# The dataset is a function of the seeds

The recorder reads the world and writes nothing to it. The engine is
deterministic, so the same seeds give the same dataset.[^3]

# References

[^1]: ADR-0192, a window of controller commands is one label distribution
over the action table, decisions D1 to D4.
``docs/adrs/draft/adr-0192-a-window-of-controller-commands-is-one-label-distribution.md``
[^2]: Recurring defect shapes, shape 1. ``.agents/rules/recurring-defects.md``
[^3]: ADR-0001, one binary gives one answer at any thread count, decision D1.
``docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md``
"""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING

import numpy as np

from .env import Env, EnvConfig, viable_seeds
from .inspect import verb_of
from .policy import LinearPolicy, encode_many
from .reward import Scoring

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Sequence

    from cachette._core import ActionSchema, World

# How many steps the fit takes. **The count is fixed and no test of
# convergence ends it.** A stopping rule that read the loss would make the
# result depend on the arithmetic of one machine, and two machines would then
# report two policies from one dataset.
FIT_STEPS = 400

# The step size of the fit, and the decay of the momentum term.
FIT_RATE = 0.5
FIT_MOMENTUM = 0.9

# The weight of the squared-norm penalty. It keeps the fit from driving a
# rarely-seen action row to a large weight on one sample.
FIT_PENALTY = 1.0e-4


@dataclass(frozen=True)
class Dataset:
    """What the recorder read from a seed set.

    The observations hold one row for each decision window, in the order the
    recorder read them. The targets hold one row for each window and one
    column for each action row, and each row sums to one. The masks hold the
    legality answer of the seat at the start of each window. The episodes
    entry names the seed each window came from, so a caller splits by
    episode and never by window. The sizes entry says how many commands each
    window held.

    **The four counts below describe the whole recording and never a part of
    it.** A split carries them unchanged, because a share of the whole is
    what a report quotes and a part of it would state a second figure under
    the same name.
    """

    observations: np.ndarray
    targets: np.ndarray
    masks: np.ndarray
    episodes: np.ndarray
    sizes: np.ndarray
    commands: int
    unencodable: int
    silent_windows: int
    lost_windows: int

    def __len__(self) -> int:
        """Return how many decision windows the dataset holds."""
        return int(self.observations.shape[0])

    def save(self, path: Path) -> None:
        """Write the dataset to one file, so that a re-run need not record.

        The recording is the expensive half of this module. A caller that
        changes only the fit reads the file back and pays nothing.
        """
        np.savez_compressed(
            path,
            observations=self.observations,
            targets=self.targets,
            masks=self.masks,
            episodes=self.episodes,
            sizes=self.sizes,
            counts=np.asarray(
                [
                    self.commands,
                    self.unencodable,
                    self.silent_windows,
                    self.lost_windows,
                ],
                dtype=np.int64,
            ),
        )

    @staticmethod
    def load(path: Path) -> Dataset:
        """Read a dataset that a previous recording wrote."""
        with np.load(path) as held:
            counts = held["counts"]
            return Dataset(
                observations=held["observations"],
                targets=held["targets"],
                masks=held["masks"],
                episodes=held["episodes"],
                sizes=held["sizes"],
                commands=int(counts[0]),
                unencodable=int(counts[1]),
                silent_windows=int(counts[2]),
                lost_windows=int(counts[3]),
            )

    def split(self, holdout: Sequence[int]) -> tuple[Dataset, Dataset]:
        """Split by episode, into the windows outside and inside a seed set.

        **The split is by episode and never by window.** Two windows of one
        episode share a world, a map and an opponent, so a split that put one
        in each part would report a held-out accuracy that the training part
        already knew.
        """
        held = np.isin(self.episodes, np.asarray(list(holdout), dtype=np.int64))
        return self._take(~held), self._take(held)

    def _take(self, keep: np.ndarray) -> Dataset:
        """Return the part of this dataset the mask keeps."""
        return Dataset(
            observations=self.observations[keep],
            targets=self.targets[keep],
            masks=self.masks[keep],
            episodes=self.episodes[keep],
            sizes=self.sizes[keep],
            commands=self.commands,
            unencodable=self.unencodable,
            silent_windows=self.silent_windows,
            lost_windows=self.lost_windows,
        )

    def label_counts(self) -> np.ndarray:
        """Return the weight each action row carries over the whole set."""
        return self.targets.sum(axis=0)

    def baseline(self) -> dict[str, float]:
        """Return what one constant answer scores on this set.

        **Read every accuracy against this.** The label of a window is a
        distribution, and a window that holds many commands spreads it. A
        policy that names the most common row of the whole set already
        scores the first figure here, and it reads nothing at all.

        The distinct entry is the mean number of different action rows one
        window holds. A window that holds one row can be answered exactly. A
        window that holds many cannot, whatever the policy reads.
        """
        weight = self.label_counts()
        total = float(weight.sum()) or 1.0
        best = int(np.argmax(weight))
        hit = self.targets[:, best]
        windows = max(len(self), 1)
        return {
            "constant_row": float(best),
            "constant_command_accuracy": float(weight[best] / total),
            "constant_window_accuracy": float((hit > 0.0).mean()),
            "distinct_rows_per_window": float(
                np.count_nonzero(self.targets, axis=1).sum() / windows
            ),
            "commands_per_window": float(self.sizes.sum() / windows),
        }

    def no_op_share(self) -> float:
        """Return the share of the label weight that sits on the no-op row."""
        total = float(self.targets.sum())
        if total == 0.0:
            return 0.0
        return float(self.targets[:, 0].sum() / total)


class _WindowReader:
    """Gathers the commands of one seat over the ticks of one window.

    The engine empties the command log at the start of each tick, so a
    reader must run once for each tick. This holds what it read.
    """

    def __init__(self, seat: int) -> None:
        """Build the reader of one seat, with an empty window."""
        self._seat = seat
        self.taken: list[np.ndarray] = []
        self.refused = 0

    def __call__(self, world: World) -> None:
        """Read the commands of one tick, at the frame barrier."""
        rows = world.controller_actions(self._seat)
        encoded = np.asarray(rows["encoded"])
        action = np.asarray(rows["action"], dtype=np.int64)
        self.taken.append(action[encoded > 0])
        self.refused += int(np.count_nonzero(encoded == 0))

    def actions(self) -> np.ndarray:
        """Return every command of the window the table could express."""
        if not self.taken:
            return np.zeros(0, dtype=np.int64)
        return np.concatenate(self.taken)


def record(
    config: EnvConfig,
    scoring: Scoring,
    seeds: Sequence[int],
    progress: bool = False,
) -> Dataset:
    """Play the built-in controller in the learner's seat, and record it.

    The configuration must leave the seat to the built-in controller. A
    configuration that takes the seat records nothing, because the controller
    then emits no command for it.

    The observation of a window is the array the learner reads at the start
    of that window, and the labels are the commands the controller emitted
    over the ticks of the window. **The two come from one loop**, so the
    dataset cannot state an observation the learner would never have seen.
    """
    if config.controlled:
        message = (
            "the recorder needs the seat under the built-in controller. "
            "Build the configuration with controlled=False."
        )
        raise ValueError(message)

    env = Env(config, scoring)
    seat = config.seat
    observations: list[np.ndarray] = []
    masks: list[np.ndarray] = []
    targets: list[np.ndarray] = []
    episodes: list[int] = []
    sizes: list[int] = []
    commands = 0
    unencodable = 0
    silent = 0
    lost = 0

    for index, seed in enumerate(seeds):
        observation = env.reset(int(seed))
        while not env.done:
            mask = env.action_mask()
            reader = _WindowReader(seat)
            result = env.step(0, on_tick=reader)
            taken = reader.actions()
            commands += int(taken.size) + reader.refused
            unencodable += reader.refused
            row = np.zeros(env.action_length)
            if taken.size:
                counts = np.bincount(taken, minlength=env.action_length)
                row = counts.astype(np.float64) / float(taken.size)
            elif reader.refused == 0:
                # The controller emitted nothing at all over the window, so
                # the no-op is what it did.
                row[0] = 1.0
                silent += 1
            else:
                # Every command of the window fell outside the table. The
                # window states no label, and it is counted rather than
                # labelled with the no-op.
                lost += 1
                observation = result.observation
                continue
            observations.append(observation)
            masks.append(mask)
            targets.append(row)
            episodes.append(int(seed))
            sizes.append(int(taken.size))
            observation = result.observation
        if progress:
            print(
                f"  recorded seed {seed} "
                f"({index + 1}/{len(seeds)}), "
                f"windows {len(observations)}, commands {commands}",
                flush=True,
            )

    return Dataset(
        observations=np.stack(observations),
        targets=np.stack(targets),
        masks=np.stack(masks),
        episodes=np.asarray(episodes, dtype=np.int64),
        sizes=np.asarray(sizes, dtype=np.int64),
        commands=commands,
        unencodable=unencodable,
        silent_windows=silent,
        lost_windows=lost,
    )


def fit_weights(features: np.ndarray, targets: np.ndarray) -> np.ndarray:
    """Fit one weight matrix by cross entropy against the label rows.

    The features hold one row for each window. The targets hold one row for
    each window and one column for each action row, and each row sums to one.
    The result has one row for each action row and one column for each
    feature.

    **The step count is fixed.** Nothing here reads the loss to decide when
    to stop, so two machines fit the same dataset to the same weights up to
    the arithmetic of the products themselves.
    """
    samples = features.shape[0]
    weights = np.zeros((targets.shape[1], features.shape[1]))
    velocity = np.zeros_like(weights)
    for _ in range(FIT_STEPS):
        scores = features @ weights.T
        scores -= scores.max(axis=1, keepdims=True)
        exponent = np.exp(scores)
        probability = exponent / exponent.sum(axis=1, keepdims=True)
        gradient = (probability - targets).T @ features / samples
        gradient += FIT_PENALTY * weights
        velocity = FIT_MOMENTUM * velocity - FIT_RATE * gradient
        weights = weights + velocity
    return weights


def fit_linear(data: Dataset) -> LinearPolicy:
    """Fit the linear policy shape to a recorded dataset."""
    features = encode_many(data.observations)
    return LinearPolicy(fit_weights(features, data.targets))


def score(policy: LinearPolicy, data: Dataset) -> dict[str, float]:
    """Report how often the policy names a command the controller gave.

    The command accuracy is the chance that the policy names the command a
    reader would draw at random from the window. The window accuracy is the
    share of windows where the policy names any command the controller gave
    in that window. **The second is the higher of the two whenever a window
    holds more than one command**, and a reader that quotes one must say
    which.

    The mask is the legality answer of the seat at the window, so the score
    measures what the policy would have played and not what it scored.
    """
    chosen = np.asarray(
        policy.choose_many(data.observations, data.masks), dtype=np.int64
    )
    rows = np.arange(chosen.size)
    hit = data.targets[rows, chosen]
    return {
        "windows": float(chosen.size),
        "command_accuracy": float(hit.mean()) if chosen.size else 0.0,
        "window_accuracy": float((hit > 0.0).mean()) if chosen.size else 0.0,
        "no_op_share": data.no_op_share(),
        "no_op_chosen": float((chosen == 0).mean()) if chosen.size else 0.0,
    }


def verb_mix(schema: ActionSchema, weights: np.ndarray) -> dict[str, float]:
    """Return the share of the label weight each verb carries."""
    counts: dict[str, float] = {}
    for action, weight in enumerate(weights):
        if weight > 0.0:
            name = verb_of(schema, action)
            counts[name] = counts.get(name, 0.0) + float(weight)
    total = sum(counts.values()) or 1.0
    ordered = sorted(counts.items(), key=lambda row: (-row[1], row[0]))
    return {name: round(value / total, 4) for name, value in ordered}


def write_json(path: Path, payload: object) -> None:
    """Write one report as a JSON file."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, default=str), encoding="utf-8")


def main() -> int:
    """Record the controller, fit the linear policy shape, and measure it.

    The run plays the built-in controller in the learner's seat over a set of
    recording seeds, fits a policy to what it did, then plays that policy on
    the same held-out seeds the training runner uses. The report names the
    controller's own numbers on those seeds beside the policy's, because a
    number without that comparison says nothing.
    """
    import argparse

    from .__main__ import CONTROLLER_WORLD, STRATEGIES, WORLD
    from .policy import PolicyFit
    from .train import evaluate, first_scoring

    parser = argparse.ArgumentParser(
        description="Fit a policy to what the built-in controller does."
    )
    parser.add_argument("--out", type=Path, default=Path("runs/imitate"))
    parser.add_argument("--record-seeds", type=int, default=24)
    parser.add_argument("--record-start", type=int, default=1000)
    parser.add_argument("--fit-holdout", type=int, default=6)
    parser.add_argument("--holdout", type=int, default=24)
    parser.add_argument("--workers", type=int, default=16)
    parser.add_argument(
        "--play",
        action="store_true",
        help="play the fitted policies on the held-out seeds and report",
    )
    arguments = parser.parse_args()

    scoring = first_scoring(STRATEGIES["conquer"][1])
    seeds = viable_seeds(
        CONTROLLER_WORLD, arguments.record_seeds, arguments.record_start
    )
    arguments.out.mkdir(parents=True, exist_ok=True)
    store = arguments.out / "dataset.npz"
    if store.exists():
        print(f"reading the recording from {store}", flush=True)
        data = Dataset.load(store)
    else:
        print(f"recording {len(seeds)} episodes from seed {seeds[0]}", flush=True)
        data = record(CONTROLLER_WORLD, scoring, seeds, progress=True)
        data.save(store)
    fitting, held = data.split(seeds[-arguments.fit_holdout :])
    probe = Env(WORLD, scoring)
    probe.reset(int(seeds[0]))
    schema = probe.world.action_schema()

    report: dict[str, object] = {
        "record_seeds": list(seeds),
        "fit_holdout_seeds": list(seeds[-arguments.fit_holdout :]),
        "windows": len(data),
        "commands": data.commands,
        "unencodable": data.unencodable,
        "unencodable_share": data.unencodable / max(data.commands, 1),
        "silent_windows": data.silent_windows,
        "lost_windows": data.lost_windows,
        "baseline": data.baseline(),
        "label_verbs": verb_mix(schema, data.label_counts()),
        "world": {
            "width": WORLD.width,
            "height": WORLD.height,
            "faction_count": WORLD.faction_count,
            "tick_limit": WORLD.tick_limit,
            "decision_interval": WORLD.decision_interval,
        },
        "fits": {},
    }
    print(f"  windows {len(data)}, commands {data.commands}", flush=True)
    print(f"  baseline {data.baseline()}", flush=True)

    # **This pass fits the linear shape alone.** A supervised fit needs a
    # convex problem, and the structured policy is not one. A fit of it needs
    # a gradient method, which this module does not hold.
    fits: dict[str, LinearPolicy] = {"linear": fit_linear(fitting)}
    wanted = PolicyFit.of_env(probe)
    for name, policy in fits.items():
        path = arguments.out / f"imitate-{name}.npz"
        policy.save(path, wanted.as_meta())
        row = {
            "train": score(policy, fitting),
            "holdout": score(policy, held),
            "weights": str(path),
        }
        report["fits"][name] = row  # type: ignore[index]
        print(f"  {name} train {row['train']}", flush=True)
        print(f"  {name} holdout {row['holdout']}", flush=True)
    write_json(arguments.out / "imitate.json", report)

    if arguments.play:
        played = viable_seeds(WORLD, arguments.holdout, 50_000)
        report["play_seeds"] = list(played)
        play: dict[str, dict[str, float]] = {
            "controller": evaluate(
                CONTROLLER_WORLD,
                scoring,
                LinearPolicy.zeros(probe.action_length, probe.observation_length),
                played,
                arguments.workers,
            ),
            **{
                name: evaluate(WORLD, scoring, policy, played, arguments.workers)
                for name, policy in fits.items()
            },
        }
        report["play"] = play
        for label, summary in play.items():
            print(
                f"  {label:11s} return {summary['return']:10.1f} "
                f"won {summary['won']:5.2f} lost {summary['lost']:5.2f}",
                flush=True,
            )
        write_json(arguments.out / "imitate.json", report)
    print(f"wrote {arguments.out / 'imitate.json'}", flush=True)
    return 0


__all__ = [
    "Dataset",
    "fit_linear",
    "fit_weights",
    "main",
    "record",
    "score",
    "verb_mix",
    "write_json",
]


if __name__ == "__main__":  # pragma: no cover - the module entry point
    raise SystemExit(main())
