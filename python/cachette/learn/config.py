"""What one training run is, as one frozen record of its arguments.

A training run reads these numbers, and so does every worker process that
scores part of a generation for it. **The declaration sits in one module,
because several modules read it and a worker process carries it across.** A
second copy of any of these numbers would be a value declared twice, with
nothing that fails when the copies disagree.

The trainer holds the loop and the checkpoints. The optimiser holds the search.
The play holds the batch. Each of them reads the fields it needs from here.
"""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class TrainConfig:
    """How long the training runs and how wide each generation is."""

    generations: int = 12
    population: int = 16
    seeds_per_generation: int = 4
    # Sigma is a relative size. The centre has unit length and each
    # perturbation has unit length, so sigma is the fraction of the centre
    # that one candidate moves. A measurement on real decisions of a trained
    # policy fixed the working range, and it is far above the value a run
    # would reach by analogy with a gradient method.
    sigma: float = 1.5
    # The fraction of the centre that one generation moves.
    learning_rate: float = 0.3
    # How many engine workers one process gives its batch. **This is a per
    # process count, whatever the shard count is.** A run of five processes
    # with this at twelve asks for sixty workers on the machine.
    workers: int = 4
    # How many worker processes score one generation. One process scores the
    # whole generation in the process that asked for it, and starts nothing.
    # **The caller states this. Nothing derives it from the core count**, and
    # a second declaration site that silently disagreed with the worker count
    # is the defect shape this project names first.
    shards: int = 1
    seed: int = 0
    # The seats a candidate may take. An empty list puts one candidate in one
    # world, in the seat the environment names, and every other seat keeps the
    # built-in controller. Two or more seats put that many candidates in one
    # world, and the trainer then scores each of them against the others.
    learner_seats: tuple[int, ...] = ()
    # Whether a seated generation ranks the margin against the other seats of
    # one world, or the raw return. This has no meaning without seated play.
    relative: bool = True
    # Whether a validating generation also plays the highest candidate on the
    # validation worlds. The centre is what the run keeps, so this measures
    # nothing the run acts on. It answers one question: whether the highest
    # of many draws on a few worlds is a real gain or the luckiest draw. That
    # question has been answered, so the pass is off unless a caller asks.
    validate_candidate: bool = False

    @property
    def pairs(self) -> int:
        """How many antithetic pairs one generation holds.

        A pair is one perturbation tried in both directions, so a population
        of an even size holds half as many pairs as candidates.
        """
        return self.population // 2


__all__ = ["TrainConfig"]
