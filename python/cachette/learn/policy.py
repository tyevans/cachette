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

from pathlib import Path

import numpy as np

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


class LinearPolicy:
    """One weight matrix over the features, and one score for each action."""

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
        return self.weights.shape  # type: ignore[return-value]

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

    def save(self, path: Path, meta: dict[str, object]) -> None:
        """Write the weights and what they were trained against.

        The action version and the observation version go into the file. A
        new verb or a new bound moves every row of the action table, so a
        file written under one version means something else under the next.
        """
        path.parent.mkdir(parents=True, exist_ok=True)
        np.savez(
            path,
            weights=self.weights,
            kind=np.array("linear"),
            **{key: np.array(value) for key, value in meta.items()},
        )

    @classmethod
    def load(cls, path: Path) -> tuple[LinearPolicy, dict[str, object]]:
        """Read a weight file, and return the policy and what it names."""
        stored = np.load(path, allow_pickle=False)
        meta = {key: stored[key].tolist() for key in stored.files if key != "weights"}
        return cls(stored["weights"]), meta


class MLPPolicy:
    """One hidden layer over the features, and one score for each action.

    A linear policy scores each action row as a weighted sum of the features,
    so it cannot state a rule that two features must hold together. This
    policy can. The hidden layer is small, the activation is the hyperbolic
    tangent, and the whole forward pass is two matrix products.

    An evolution strategy needs no gradient, so the depth costs the trainer
    nothing but the parameter count. That count is the whole cost: a wider
    hidden layer needs more samples to find a direction in.
    """

    def __init__(self, first: np.ndarray, second: np.ndarray) -> None:
        """Take the two weight matrices, from the features to the actions."""
        self.first = np.asarray(first, dtype=np.float64)
        self.second = np.asarray(second, dtype=np.float64)

    @classmethod
    def zeros(
        cls, action_length: int, observation_length: int, hidden: int = 32
    ) -> MLPPolicy:
        """Build a policy whose second layer is zero, so every score is zero.

        **The first layer is not zero.** A network of two zero layers has a
        zero gradient in the first layer under every perturbation, so it
        would never leave the origin. The first layer therefore holds a fixed
        random projection, drawn from one fixed seed so that a repeat of a
        run builds the same starting policy.

        The second layer is zero, so this policy takes the no-op at every
        decision, in the way the untrained linear policy does.
        """
        rng = np.random.default_rng(20260907)
        scale = 1.0 / np.sqrt(observation_length + 1)
        return cls(
            rng.standard_normal((hidden, observation_length + 1)) * scale,
            np.zeros((action_length, hidden)),
        )

    @property
    def shapes(self) -> tuple[tuple[int, int], tuple[int, int]]:
        """The shape of each weight matrix."""
        return self.first.shape, self.second.shape  # type: ignore[return-value]

    def scores(self, features: np.ndarray) -> np.ndarray:
        """Return one score for each action row of each feature row."""
        hidden = np.tanh(features @ self.first.T)
        return hidden @ self.second.T

    def choose(self, observation: np.ndarray, mask: np.ndarray) -> int:
        """Return the action integer of the highest-scoring legal row."""
        scores = self.scores(encode(observation)[None, :])[0]
        scores = np.where(mask > 0, scores, -np.inf)
        return int(np.argmax(scores))

    def choose_many(self, observations: np.ndarray, masks: np.ndarray) -> list[int]:
        """Return one action for each row of a stack of observations."""
        scores = self.scores(encode_many(observations))
        scores = np.where(masks > 0, scores, -np.inf)
        return [int(value) for value in np.argmax(scores, axis=1)]

    def flat(self) -> np.ndarray:
        """Return every trainable weight as one vector.

        The first layer is a fixed projection and is not trainable, so this
        returns the second layer alone. A trainer perturbs this vector and
        rebuilds a policy from it.
        """
        return self.second.reshape(-1)

    def rebuild(self, flat: np.ndarray) -> MLPPolicy:
        """Return a policy with this projection and the given second layer."""
        return MLPPolicy(self.first, flat.reshape(self.second.shape))

    def save(self, path: Path, meta: dict[str, object]) -> None:
        """Write both layers and what they were trained against."""
        path.parent.mkdir(parents=True, exist_ok=True)
        np.savez(
            path,
            first=self.first,
            second=self.second,
            kind=np.array("mlp"),
            **{key: np.array(value) for key, value in meta.items()},
        )


def load_policy(path: Path) -> tuple[LinearPolicy | MLPPolicy, dict[str, object]]:
    """Read a weight file, and return the policy it holds and what it names.

    The file states its own kind. A file written before this module held two
    kinds names none, and it holds a linear policy.
    """
    stored = np.load(path, allow_pickle=False)
    kind = str(stored["kind"]) if "kind" in stored.files else "linear"
    skip = {"weights", "first", "second", "kind"}
    meta = {key: stored[key].tolist() for key in stored.files if key not in skip}
    meta["kind"] = kind
    if kind == "mlp":
        return MLPPolicy(stored["first"], stored["second"]), meta
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
