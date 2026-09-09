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
    """How long the training runs and how wide each generation is.

    **Sigma is a relative size and never a length.** Each perturbation has
    unit length before the search scales it, and the search scales it by the
    length of the centre. So sigma is the fraction of the centre that one
    candidate moves. The search states the length, and this states the
    fraction. **This is the only declaration of the fraction**, and every
    caller that wants the default asks this field for it.

    A measurement fixed the value. It varied sigma over four settings against
    one trained centre, and 0.5 had the largest signal, the best ratio of
    signal to noise, and the highest rank agreement between two disjoint
    world sets. It also needed the fewest worlds for each candidate. A larger
    sigma found worse candidates outright, and the value the project ran
    before this needed more worlds than the run gave it.[^1]

    The learning rate is the largest fraction of the centre that one
    generation moves. A generation reaches it only when the ranking splits
    every antithetic pair to the ends of the order, and the search moves a
    lesser fraction for a generation whose candidates agreed less.[^2]

    The worker count is how many engine threads one batch of worlds steps
    with. **It reaches the passes that a queue does not split**, which are
    the validation pass, the held-out pass and the reference sample. A worker
    process of the queue plays one episode of one world, so it holds the
    engine to one thread whatever this field says.

    References
    ----------
    [^1]: Report on how sigma trades against the worlds each candidate plays,
    section 7.
    ``docs/research/how-sigma-trades-against-worlds-for-each-candidate.md``

    [^2]: The search, the step and the agreement of a generation.
    ``python/cachette/learn/search.py``
    """

    generations: int = 12
    population: int = 16
    seeds_per_generation: int = 4
    sigma: float = 0.5
    learning_rate: float = 0.3
    workers: int = 4
    # How many worker processes hold the queue of one run. One process scores
    # every generation in the process that asked for it, and starts nothing.
    #
    # **One task is one episode, so this names the machine and not the shape
    # of the work.** A run gives it the cores it rents. The command line takes
    # the core count by default, because a number that nobody chooses is the
    # number that cannot be wrong.
    pool: int = 1
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
