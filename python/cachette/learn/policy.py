"""A linear policy over the observation, and the file that stores it.

The policy is learner-side arithmetic. **It is not simulated state and it
never enters the world**, so it may hold a floating point number.[^1] The
engine stores integers, and the boundary is the action integer this module
returns.

# The features squash a wide range into a narrow one

The observation holds a tick beside a raw Q16.16 store total beside a count
of cells. The ranges differ by many orders of magnitude, so a linear policy
over the raw array would be driven by one field. The encoder therefore takes
the signed logarithm of each position, which keeps the sign and the order and
throws away the scale.

# The mask decides before the weights do

The policy scores every row of the action table, then it removes the rows the
engine says are illegal, then it takes the highest of what is left. Row zero
is the no-op and it is always legal, so the choice is never empty.[^2]

# References

[^1]: ADR-0002, state holds no floating point number, decision D1.
``docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md``
[^2]: ADR-0154, the observation and the action of a faction are
schema-declared bounded tables, decision D5.
``docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md``
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING, Protocol

import numpy as np

from .layout import ObservationLayout

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Mapping

    from cachette._core import ActionSchema, ObservationSchema

    from .structured import StructuredPolicy

# The divisor that brings the signed logarithm into roughly one unit. A store
# total of a raw Q16.16 quantity reaches about twenty in the logarithm, so
# this puts the widest field near one.
FEATURE_SCALE = 20.0


def encode(observation: np.ndarray) -> np.ndarray:
    """Turn one observation array into the feature vector of the policy.

    The result holds one entry for each position of the observation, and one
    trailing entry of one for the bias.
    """
    values = observation.astype(np.float64)
    squashed = np.sign(values) * np.log1p(np.abs(values)) / FEATURE_SCALE
    return np.concatenate([squashed, np.ones(1)])


def encode_many(observations: np.ndarray) -> np.ndarray:
    """Encode a stack of observations, one for each row."""
    values = observations.astype(np.float64)
    squashed = np.sign(values) * np.log1p(np.abs(values)) / FEATURE_SCALE
    ones = np.ones((values.shape[0], 1))
    return np.concatenate([squashed, ones], axis=1)


class PolicyFitError(ValueError):
    """A stored policy does not fit the world a caller asked it to play.

    A weight file is a function of one observation layout and one action
    layout. Both layouts are functions of the world parameters, so a file
    written against one world states nothing about another.[^1]

    **The lengths alone do not separate two worlds.** The observation length
    counts the cells of the block lattice, and a block is a fixed number of
    tiles on a side. Every world from one block to two blocks on each axis
    therefore holds the same cell count and the same observation length. A
    policy trained on the smallest of those loads on the largest, reads an
    array of the length it expects, and plays a world it never saw. Nothing
    raises, because nothing has a shape to disagree about.

    The fit therefore carries the world extent and the faction count beside
    the two lengths and the two versions.

    References
    ----------
    [^1]: ADR-0154, the observation and the action of a faction are
    schema-declared bounded tables, decision D2.
    ``docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md``
    """


@dataclass(frozen=True)
class PolicyFit:
    """What one weight file was trained against.

    Every entry is a function of the world parameters and never of the
    population.[^1] A file states its fit, and a caller that plays the file
    states the fit of its own world. The two must agree.

    The two version entries come from the engine schemas and never from a
    constant in this package. A version written by hand is a second
    declaration of a number the engine owns, and nothing fails when the two
    disagree.[^2]

    References
    ----------
    [^1]: ADR-0154, the observation and the action of a faction are
    schema-declared bounded tables, decision D2.
    ``docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md``

    [^2]: Recurring defect shapes, shape 1.
    ``.agents/rules/recurring-defects.md``
    """

    observation_version: int
    action_version: int
    observation_length: int
    action_length: int
    width: int
    height: int
    faction_count: int

    # The keys a weight file stores the fit under. The names are the ones
    # the trainer already wrote, so a file written before this type existed
    # still reads back as a fit.
    KEYS = (
        "observation_version",
        "action_version",
        "observation_length",
        "action_length",
        "width",
        "height",
        "faction_count",
    )

    @classmethod
    def of_env(cls, env: EnvLike) -> PolicyFit:
        """Return the fit of the world one environment builds."""
        config = env.config
        return cls(
            observation_version=int(env.observation_version),
            action_version=int(env.action_version),
            observation_length=int(env.observation_length),
            action_length=int(env.action_length),
            width=int(config.width),
            height=int(config.height),
            faction_count=int(config.faction_count),
        )

    @classmethod
    def of_world(cls, world: WorldLike) -> PolicyFit:
        """Return the fit of one world, read from the schemas it publishes.

        A caller that holds a world and no environment reads the fit here.
        The demonstration is such a caller: it builds a world of its own and
        seats a stored policy on one faction of it.

        **The two versions and the two lengths come from the engine
        schemas**, in the way they do for an environment. Nothing here holds
        a constant, so a change in the engine reaches both callers at once.
        """
        observation = world.observation_schema()
        action = world.action_schema()
        return cls(
            observation_version=int(observation["version"]),
            action_version=int(action["version"]),
            observation_length=int(observation["length"]),
            action_length=int(action["length"]),
            width=int(world.width),
            height=int(world.height),
            faction_count=int(world.faction_count),
        )

    @classmethod
    def read(cls, meta: Mapping[str, object]) -> PolicyFit | None:
        """Return the fit a weight file states, or nothing when it states none.

        A file written before this package stored a fit names some of the
        keys and not others. Such a file cannot be placed, so this returns
        nothing and the caller refuses it.
        """
        values: dict[str, int] = {}
        for key in cls.KEYS:
            value = meta.get(key)
            if not isinstance(value, (int, float)) or isinstance(value, bool):
                return None
            values[key] = int(value)
        return cls(**values)

    def as_meta(self) -> dict[str, int]:
        """Return the fit as the entries a weight file stores."""
        return {key: int(getattr(self, key)) for key in self.KEYS}

    def describe(self) -> str:
        """Return one line that names every entry of the fit."""
        return ", ".join(f"{key}={getattr(self, key)}" for key in self.KEYS)

    def check(self, wanted: PolicyFit, path: Path | None = None) -> None:
        """Refuse when this fit is not the fit a caller asked for.

        Raises ``PolicyFitError`` naming both sides, so a reader sees which
        entry differs without opening the file.
        """
        if self == wanted:
            return
        differ = [
            f"{key}: the file says {getattr(self, key)} "
            f"and the world says {getattr(wanted, key)}"
            for key in self.KEYS
            if getattr(self, key) != getattr(wanted, key)
        ]
        where = f" at {path}" if path is not None else ""
        message = (
            f"the stored policy{where} does not fit this world. "
            + "; ".join(differ)
            + f". The file states {self.describe()}. "
            f"The world states {wanted.describe()}. "
            "Train a policy against this world, or play the policy on the "
            "world it was trained against."
        )
        raise PolicyFitError(message)


class EnvLike(Protocol):
    """What a fit reads from an environment.

    The fit needs the two schema versions, the two lengths and the world
    parameters. Naming them here keeps this module free of an import from
    the environment, which imports this one.
    """

    observation_version: int
    action_version: int
    observation_length: int
    action_length: int

    @property
    def config(self) -> ConfigLike:
        """The configuration the environment runs."""


class ConfigLike(Protocol):
    """The world parameters a fit reads from an environment configuration.

    **The three are read-only, because the fit only reads them.** A protocol
    that declares a plain attribute asks for one that can be written, and a
    frozen configuration cannot answer that. Declaring what this actually
    needs lets a frozen dataclass satisfy it.
    """

    @property
    def width(self) -> int:
        """How many tiles the world holds across."""

    @property
    def height(self) -> int:
        """How many tiles the world holds down."""

    @property
    def faction_count(self) -> int:
        """How many factions play the world."""


class WorldLike(Protocol):
    """What a fit reads from a world it did not build.

    The two schemas state the layout, and the three parameters state the
    world. Naming them here keeps this module free of an import from the
    engine binding, which the type checker reads from a stub.

    **The three parameters are read-only, because the fit only reads them.**
    A protocol that declares a plain attribute asks for one that can be
    written, and the world publishes them as properties.
    """

    @property
    def width(self) -> int:
        """How many tiles the world holds across."""

    @property
    def height(self) -> int:
        """How many tiles the world holds down."""

    @property
    def faction_count(self) -> int:
        """How many factions play the world."""

    def observation_schema(self) -> ObservationSchema:
        """Give back the layout of the observation array."""

    def action_schema(self) -> ActionSchema:
        """Give back the layout of the action table."""


class Policy(Protocol):
    """What a caller needs of a policy to play it on a batch of worlds.

    The training loop and the holdout measurement both score a whole batch at
    once, so the one thing either of them asks of a policy is a choice for
    every row. The linear policy and the two baselines all answer it, and a
    signature that named one of them would refuse the other two.
    """

    def choose_many(self, observations: np.ndarray, masks: np.ndarray) -> list[int]:
        """Return one action integer for each row of a stack of observations."""


class LinearPolicy:
    """One weight matrix over the features, and one score for each action."""

    # **A positive scaling of the weights leaves every choice where it was.**
    # One matrix scores every action row as a weighted sum over one feature
    # vector, so scaling the matrix scales every score by one factor. The row
    # that scores highest stays the row that scores highest. The bias is a
    # trailing feature of value one, and its weight scales with every other
    # weight, so it carries no exception.
    #
    # The search reads this and holds the centre of this kind at unit length.
    CHOICE_SURVIVES_SCALING = True

    def __init__(self, weights: np.ndarray) -> None:
        """Take the weight matrix. Its shape is (actions, features)."""
        self.weights = np.asarray(weights, dtype=np.float64)

    @classmethod
    def zeros(cls, action_length: int, observation_length: int) -> LinearPolicy:
        """Build the untrained policy. Every score is zero.

        A zero policy takes the first legal row of the table at every
        decision, which is the no-op. It is the baseline every trained model
        is measured against.
        """
        return cls(np.zeros((action_length, observation_length + 1)))

    @property
    def shape(self) -> tuple[int, int]:
        """The shape of the weight matrix."""
        actions, features = self.weights.shape
        return actions, features

    def choose(self, observation: np.ndarray, mask: np.ndarray) -> int:
        """Return the action integer of the highest-scoring legal row."""
        scores = self.weights @ encode(observation)
        scores = np.where(mask > 0, scores, -np.inf)
        return int(np.argmax(scores))

    def choose_many(self, observations: np.ndarray, masks: np.ndarray) -> list[int]:
        """Return one action for each row of a stack of observations."""
        scores = encode_many(observations) @ self.weights.T
        scores = np.where(masks > 0, scores, -np.inf)
        return [int(value) for value in np.argmax(scores, axis=1)]

    def flat(self) -> np.ndarray:
        """Return every trainable weight as one vector."""
        return self.weights.reshape(-1)

    def rebuild(self, flat: np.ndarray) -> LinearPolicy:
        """Return a policy of this shape with the given weights."""
        return LinearPolicy(flat.reshape(self.weights.shape))

    def save(self, path: Path, meta: Mapping[str, object]) -> None:
        """Write the weights and what they were trained against.

        The action version and the observation version go into the file. A
        new verb or a new bound moves every row of the action table, so a
        file written under one version means something else under the next.
        """
        path.parent.mkdir(parents=True, exist_ok=True)
        # numpy declares ``allow_pickle`` as a keyword before its own
        # ``**kwds``, so a mapping keyed on ``str`` can never unpack cleanly.
        np.savez(
            path,
            weights=self.weights,
            kind=np.array("linear"),
            **{key: np.array(value) for key, value in meta.items()},  # type: ignore[arg-type]
        )

    @classmethod
    def load(cls, path: Path) -> tuple[LinearPolicy, dict[str, object]]:
        """Read a weight file, and return the policy and what it names."""
        stored = np.load(path, allow_pickle=False)
        meta = {key: stored[key].tolist() for key in stored.files if key != "weights"}
        return cls(stored["weights"]), meta


def load_policy(
    path: Path,
    wanted: PolicyFit | None = None,
    layout: ObservationLayout | None = None,
) -> tuple[LinearPolicy | StructuredPolicy, dict[str, object]]:
    """Read a weight file, and return the policy it holds and what it names.

    The file states its own kind. A file that names none holds a linear
    policy, which is what the runs of the first night wrote.

    **Pass the fit of the world the policy will play.** The reader then
    refuses a file that was trained against another world, and it names both
    sides in the message. A caller that passes nothing takes whatever the
    file holds, which is correct for a reader that only reports what a file
    says.

    **Pass the layout when the policy is a structured one.** The two schema
    versions catch a change the engine made to the observation. They do not
    catch a change a caller made to the geometry it states over one version,
    and a stack of 151 cells by 25 channels holds the same positions as a
    stack of 25 cells by 151 channels. The layout separates the two.

    Raises ``PolicyFitError`` when a fit is asked for and the file does not
    match it, when a fit is asked for and the file states none, and when a
    layout is asked for and the file reads another one.

    **A file that names a kind this package no longer builds is refused by
    name.** The project deleted one kind, and a file written under it holds
    its two layers and no single weight matrix. Reading it as a linear policy
    raised an error about a missing archive entry, which named the storage
    and not the cause.[^1]

    References
    ----------
    [^1]: Findings register, FND-686. ``docs/FINDINGS.md``
    """
    stored = np.load(path, allow_pickle=False)
    kind = str(stored["kind"]) if "kind" in stored.files else "linear"
    skip = {"weights", "kind", "flat", "architecture"}
    meta = {
        key: stored[key].tolist()
        for key in stored.files
        if key not in skip and not key.startswith("layout_")
    }
    meta["kind"] = kind
    if wanted is not None:
        held = PolicyFit.read(meta)
        if held is None:
            message = (
                f"the stored policy at {path} states no fit, so nothing can "
                "place it. The file must name every one of "
                f"{', '.join(PolicyFit.KEYS)}. The world states "
                f"{wanted.describe()}. Train a policy against this world."
            )
            raise PolicyFitError(message)
        held.check(wanted, path)
    if kind == "structured":
        # The structured policy reads this module for the encoder and the
        # fit, so the import sits here and the two modules do not form a
        # cycle at import time.
        from .structured import StructuredPolicy

        policy = StructuredPolicy.restore(stored)
        if layout is not None:
            policy.check_layout(layout, path)
        return policy, meta
    if "weights" not in stored.files:
        message = (
            f"the stored policy at {path} names the kind {kind!r}, which this "
            "package does not build. The kinds it builds are 'linear' and "
            "'structured'. Train a policy against this world."
        )
        raise PolicyFitError(message)
    return LinearPolicy(stored["weights"]), meta


class RandomPolicy:
    """A policy that takes one legal row at random.

    This is the second baseline. The untrained linear policy scores every row
    at zero and therefore always takes the no-op, which measures a faction
    that does nothing. A random policy measures a faction that acts without
    reading the world, and it is the harder of the two to beat.
    """

    def __init__(self, seed: int = 0) -> None:
        """Build the policy over one stream of draws."""
        self._rng = np.random.default_rng(seed)

    def choose_many(self, observations: np.ndarray, masks: np.ndarray) -> list[int]:
        """Return one legal action for each row, drawn uniformly."""
        del observations
        chosen = []
        for mask in masks:
            legal = np.flatnonzero(mask)
            chosen.append(int(self._rng.choice(legal)))
        return chosen
