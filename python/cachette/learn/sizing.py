"""Say what a training run will play, and whether it fits its wall clock.

A paid run rents one machine and bills for every minute it lives. The owner
of that run needs two answers before the machine exists.

1. **How much of the run measures rather than trains?** A validation pass, a
   held-out pass and a baseline pass all play episodes, and none of them moves
   a centre. A launcher that called them nearly free spent more than a third
   of a paid run on them and said so nowhere.
2. **Does the configuration finish inside the wall clock cap?** One run asked
   for twenty generations and the four published centres came from generations
   3, 7, 7 and 9. The cap ended it, and nothing had said that it would.

This module answers both from the arguments alone, so a launcher prints the
answer before it creates anything.

# One strategy holds a share of the machine

One process trains every strategy of a run, and one queue of worker processes
holds their episodes. The strategies submit into that queue at the same time,
so each of them receives about the cores divided by the strategy count: a run
of four strategies on sixty-four cores works out at sixteen for each.

**The reasoning that sized a run assumed the whole machine for one strategy.**
Every estimate of time here therefore takes the share one strategy receives
and never the cores of the instance. The queue figures take the cores, because
one queue feeds every core from the episodes of every strategy.

# The estimate is a floor, and it is derived

The rate for each worker comes from what the project's own training runs
reach on a sixty-four core machine of the target platform: eighteen thousand
to twenty-one thousand simulated ticks a second for the whole machine. This
model takes the low end and divides by the cores, so a strategy holding a
quarter of the machine is estimated at a quarter of the low end.

**An earlier version of this model read a register row of one process holding
twelve workers, and that row names no extent.** It gave one hundred and three
ticks a second for each worker, which is a third of what the runs reach, so
the estimate refused configurations that finish. A tick of one world size is
not a tick of another, and a rate row without an extent is not a rate.[^1]

The episode length is bounded at the tick limit, which is the longest an
episode can run.

**An estimate from this module is not a measurement.** One blocker states
which cost figures of this project are measured and which are derived.[^2]

References
----------
[^1]: Target platform costs, one process against five.
``docs/reference/graviton-costs.md``

[^2]: Blockers register, BLK-007. ``docs/BLOCKERS.md``
"""

from __future__ import annotations

from dataclasses import dataclass

# The ticks a second this model gives one engine worker.
#
# **The rate depends on the extent, and the register row behind this figure
# states none.** That row is a first-week measurement and it is stale.
#
# A sixty-four core machine of the target platform reaches eighteen to
# twenty-one thousand simulated ticks a second across the whole machine **at
# extent 48**, which is about 281 for each core. A development box measured
# 756 ticks a second at extent 48 against 439 at extent 128, so the ratio
# between the two extents is 1.72 and the figure for extent 128 is about 163
# for each core. Both are above the figure here.
#
# **So the rate is not what made a run of twenty generations deliver nine.**
# This model is conservative on the rate and conservative on the episode
# length as well, because it bounds an episode at the tick limit while an
# episode of the audited world ended between tick 1271 and 3311. Two
# conservative inputs gave an optimistic answer, so the shortfall came from
# elsewhere: four processes of sixteen workers oversubscribe sixty-four cores
# once the trainer processes are counted, and a measurement share of 0.567
# went uncharged.
#
# A caller that knows its extent passes the rate for that extent. The
# throughput probe of the launcher measures the real one a minute into a run
# and the estimate is reprinted against it.
#
# **This is the one declaration of the rate.** A caller that measured its own
# throughput passes it in rather than writing a second one, and the launcher
# reprints the estimate against the rate its throughput probe measures.
TICKS_A_SECOND_FOR_EACH_WORKER = 1231.0 / 12.0

# The shortest and the longest episode of the audited training world, in
# ticks. Both come from the audit of one paid run, which recorded the end tick
# of every episode of a generation.
#
# **The spread is what decides whether a queue keeps a machine busy.** One
# task is one episode, so a worker takes the next episode when it finishes
# one, and the only wait left is the last episode of a generation. A worker
# that plays few episodes cannot cover that wait with the rest of its queue.
EPISODE_TICKS_SHORTEST = 1271
EPISODE_TICKS_LONGEST = 3311

# How many times longer the longest episode of a generation runs than the
# shortest. **This is the one declaration of the spread**, and the threshold
# below derives from it rather than from a chosen number.
EPISODE_LENGTH_SPREAD = EPISODE_TICKS_LONGEST / EPISODE_TICKS_SHORTEST

# How many times the held-out pass plays the random policy at the end of a
# strategy. The engine is deterministic, so only a policy that draws at random
# gains anything from a repeat, and the trainer repeats that one alone.
RANDOM_REPEATS = 3

# How many held-out passes one strategy plays when it ends: the trained
# centre, the untrained centre, and the random policy repeated.
#
# The controller row of the same table reads the cache that the baseline pass
# filled before any trainer started, so it plays nothing.
FINAL_PASSES = 2 + RANDOM_REPEATS


def validates(generation: int, generations: int, validate_every: int) -> bool:
    """Say whether the run plays the validation seeds after this generation.

    **This is the one declaration of the validation schedule.** The training
    loop reads it and this module counts with it, so a run cannot play a
    schedule that its own plan did not count.

    The last generation always validates, because a run that ended between
    two intervals would publish a centre that nothing selected.
    """
    if validate_every <= 0:
        return generation == generations - 1
    return (
        generation == generations - 1
        or generation % validate_every == validate_every - 1
    )


def measures_holdout(generation: int, holdout_every: int) -> bool:
    """Say whether the run plays the held-out seeds after this generation.

    **This is the one declaration of the held-out schedule.** An interval of
    zero turns the periodic pass off, which leaves only the pass a strategy
    plays when it ends.
    """
    return holdout_every > 0 and generation % holdout_every == holdout_every - 1


@dataclass(frozen=True)
class RunShape:
    """The arguments of a run that decide how many episodes it plays."""

    generations: int
    population: int
    seeds: int
    validation: int
    validate_every: int
    holdout: int
    holdout_every: int
    tick_limit: int
    strategies: int = 1


@dataclass(frozen=True)
class Episodes:
    """How many episodes one strategy of a run plays, by what plays them.

    Every count is for one strategy. The strategies of a run play at the same
    time on their own share of the machine, so a total over the run answers a
    question about money and never about time.
    """

    training: int
    yardstick: int
    validation: int
    holdout_during: int
    holdout_final: int
    # The share of the controller baseline pass that this strategy carries.
    # **One pass answers for every strategy.** It plays the held-out seeds
    # once, before any trainer starts, and it scores that one batch under
    # every objective the run trains. A count of the whole pass for each
    # strategy would charge a run of four strategies four times for it.
    baseline: int

    @property
    def measurement(self) -> int:
        """Return the episodes that measure rather than train.

        The yardstick, the validation passes, the held-out passes and the
        baseline pass all play episodes and none of them moves a centre.
        """
        return (
            self.yardstick
            + self.validation
            + self.holdout_during
            + self.holdout_final
            + self.baseline
        )

    @property
    def total(self) -> int:
        """Return every episode one strategy plays."""
        return self.training + self.measurement

    @property
    def measurement_share(self) -> float:
        """Return the part of the episodes that measures, from 0 to 1."""
        return 0.0 if not self.total else self.measurement / self.total


def episodes(shape: RunShape, generations: int | None = None) -> Episodes:
    """Return what one strategy plays, over this many generations.

    The generations entry answers what a run that stopped early would have
    played. It defaults to the generations the shape asks for.
    """
    span = shape.generations if generations is None else generations
    span = max(0, min(span, shape.generations))
    validating = sum(
        1
        for generation in range(span)
        if validates(generation, span, shape.validate_every)
    )
    measuring = sum(
        1
        for generation in range(span)
        if measures_holdout(generation, shape.holdout_every)
    )
    return Episodes(
        training=span * shape.population * shape.seeds,
        yardstick=shape.validation if span else 0,
        validation=validating * shape.validation,
        holdout_during=measuring * shape.holdout,
        holdout_final=FINAL_PASSES * shape.holdout,
        baseline=shape.holdout // max(1, shape.strategies),
    )


def workers_for_each_strategy(cores: int, strategies: int) -> int:
    """Return the share of the machine one strategy receives.

    Every strategy of a run trains at the same time, and they submit their
    episodes into one queue. **A strategy therefore never holds the machine.**
    Every estimate of the time a generation takes reads this and never the
    core count of the instance.

    A pass that a queue does not split steps one batch of worlds with this
    many engine threads, for the same reason: the strategies play those
    passes at the same time.
    """
    if strategies <= 0:
        return max(1, cores)
    return max(1, cores // strategies)


def seconds_for(
    shape: RunShape,
    workers: int,
    generations: int | None = None,
    rate_for_each_worker: float = TICKS_A_SECOND_FOR_EACH_WORKER,
) -> float:
    """Return the seconds one strategy needs, at the workers it receives.

    The episode is bounded at the tick limit, which is the longest one can
    run, and the rate is the measured rate for each worker. The strategies of
    a run play at the same time, so this is the wall clock of the run and not
    a part of it.
    """
    rate = rate_for_each_worker * max(1, workers)
    if rate <= 0.0:
        return 0.0
    return episodes(shape, generations).total * shape.tick_limit / rate


def generations_inside(
    shape: RunShape,
    workers: int,
    seconds: float,
    rate_for_each_worker: float = TICKS_A_SECOND_FOR_EACH_WORKER,
) -> int:
    """Return the generations a strategy finishes before this wall clock.

    **A run that cannot finish must say so before it starts.** This is the
    figure that says it: a configuration asking for twenty generations that
    answers nine here has already told the owner what the cap will do to it.
    """
    reached = 0
    for span in range(1, shape.generations + 1):
        if seconds_for(shape, workers, span, rate_for_each_worker) > seconds:
            break
        reached = span
    return reached


@dataclass(frozen=True)
class Plan:
    """What a run will play, and whether its wall clock holds it."""

    shape: RunShape
    cores: int
    workers: int
    seconds: float
    cap_seconds: float
    generations_reached: int

    @property
    def fits(self) -> bool:
        """Say whether the run finishes every generation inside the cap."""
        return self.cap_seconds <= 0.0 or self.seconds <= self.cap_seconds

    @property
    def generation_episodes(self) -> int:
        """How many episodes one generation of one strategy plays.

        A candidate plays one world for each seed, and one world is one
        episode. One task is one episode, so this is also the task count of
        one generation, and it reads no worker count.
        """
        return self.shape.population * self.shape.seeds

    @property
    def queued_episodes(self) -> int:
        """How many episodes one generation of every strategy plays.

        **One queue holds the episodes of every strategy of a run.** A
        strategy needs the generation before its own and needs no episode of
        another strategy, so the strategies do not wait for each other and
        the queue holds this many episodes while every one of them is working.
        """
        return self.generation_episodes * max(1, self.shape.strategies)

    @property
    def episodes_for_each_worker(self) -> float:
        """How many episodes of one generation one worker plays.

        This reads the cores of the machine, because one queue feeds them
        all. A machine that divides evenly between the strategies reaches the
        same figure from one strategy over the workers it holds.
        """
        return self.queued_episodes / max(1, self.cores)

    @property
    def run_episodes_for_each_worker(self) -> float:
        """How many episodes of the whole run one worker plays.

        This counts the measurement passes as well as the training
        generations, so it is what one core plays over the life of the
        machine. **A measurement pass does not pass through the queue.** It
        steps one batch of worlds in the process of its own strategy, and it
        holds the machine for as long as it runs whatever the queue holds.
        """
        played = episodes(self.shape)
        return played.total * max(1, self.shape.strategies) / max(1, self.cores)

    @property
    def episode_tail_share(self) -> float:
        """The share of a generation that the last episode can hold open.

        A worker plays its episodes one after another, so the wait at the end
        of a generation is at most one episode out of the episodes a worker
        plays. That is this share.
        """
        depth = self.episodes_for_each_worker
        return 1.0 if depth <= 0.0 else min(1.0, 1.0 / depth)

    @property
    def tail_dominates(self) -> bool:
        """Say whether the tail of one episode dominates a generation.

        **The threshold is the measured spread of the episode lengths and not
        a chosen number.** An episode of the audited world runs between 1,271
        and 3,311 ticks, so the longest is 2.6 times the shortest. A worker
        that plays fewer episodes than that ratio cannot cover a worker that
        drew the longest episode: it empties its own queue and idles while
        that one episode finishes. A worker that plays more than the ratio
        covers it with the episodes it has left.
        """
        return self.episodes_for_each_worker < EPISODE_LENGTH_SPREAD


def plan_of(
    shape: RunShape,
    cores: int,
    cap_seconds: float,
    rate_for_each_worker: float = TICKS_A_SECOND_FOR_EACH_WORKER,
) -> Plan:
    """Return the plan of a run on a machine of this many cores."""
    workers = workers_for_each_strategy(cores, shape.strategies)
    seconds = seconds_for(shape, workers, None, rate_for_each_worker)
    reached = (
        shape.generations
        if cap_seconds <= 0.0
        else generations_inside(shape, workers, cap_seconds, rate_for_each_worker)
    )
    return Plan(
        shape=shape,
        cores=cores,
        workers=workers,
        seconds=seconds,
        cap_seconds=cap_seconds,
        generations_reached=reached,
    )


def plan_lines(plan: Plan) -> str:
    """Return the plan as one name and one value on each line.

    A launcher reads these by name, in the way it reads the world the trainer
    prints. The names never move, so a launcher that gains a field keeps
    reading the ones it already read.
    """
    played = episodes(plan.shape)
    shape = plan.shape
    rows: list[tuple[str, str]] = [
        ("strategies", str(shape.strategies)),
        ("cores", str(plan.cores)),
        ("workers_each", str(plan.workers)),
        ("population", str(shape.population)),
        ("seeds", str(shape.seeds)),
        ("generation_episodes", str(plan.generation_episodes)),
        ("queued_episodes", str(plan.queued_episodes)),
        ("episodes_each_worker", f"{plan.episodes_for_each_worker:.2f}"),
        ("run_episodes_each_worker", f"{plan.run_episodes_for_each_worker:.2f}"),
        ("episode_tail_share", f"{plan.episode_tail_share:.3f}"),
        ("episode_tail_spread", f"{EPISODE_LENGTH_SPREAD:.2f}"),
        ("episode_tail_dominates", "yes" if plan.tail_dominates else "no"),
        ("validation_seeds", str(shape.validation)),
        ("validate_every", str(shape.validate_every)),
        ("holdout_seeds", str(shape.holdout)),
        ("holdout_every", str(shape.holdout_every)),
        ("training_episodes", str(played.training)),
        ("yardstick_episodes", str(played.yardstick)),
        ("validation_episodes", str(played.validation)),
        ("holdout_episodes", str(played.holdout_during)),
        ("holdout_final_episodes", str(played.holdout_final)),
        ("baseline_episodes", str(played.baseline)),
        ("measurement_episodes", str(played.measurement)),
        ("total_episodes", str(played.total)),
        ("measurement_share", f"{played.measurement_share:.3f}"),
        ("estimated_seconds", f"{plan.seconds:.0f}"),
        ("estimated_minutes", f"{plan.seconds / 60.0:.0f}"),
        ("cap_minutes", f"{plan.cap_seconds / 60.0:.0f}"),
        ("generations_asked", str(shape.generations)),
        ("generations_reached", str(plan.generations_reached)),
        ("generations_total", str(shape.generations * max(1, shape.strategies))),
        ("fits", "yes" if plan.fits else "no"),
    ]
    return "\n".join(f"{name}\t{value}" for name, value in rows)


def plan_notes(plan: Plan) -> list[str]:
    """Return the sentences a reader of the plan must not miss.

    A row of the plan states a number and a launcher reads it by name. A note
    states what a number means for this configuration, in the log a person
    reads while the machine bills.

    **This holds no judgement of its own.** Each note reads a field of the
    plan, so a note cannot say something the rows deny.
    """
    notes: list[str] = []
    if plan.tail_dominates:
        notes.append(
            f"one queue holds {plan.queued_episodes} episodes for each "
            f"generation of the run, which is "
            f"{plan.episodes_for_each_worker:.1f} for each of the {plan.cores} "
            f"cores. The longest episode of a generation runs "
            f"{EPISODE_LENGTH_SPREAD:.1f} times as long as the shortest, so a "
            f"worker with fewer episodes than that cannot cover the last one, "
            f"and the tail of one episode holds "
            f"{plan.episode_tail_share * 100.0:.0f} percent of a generation "
            f"open. Raise the population, the seeds or the strategies, or "
            f"rent fewer cores."
        )
    if not plan.fits:
        notes.append(
            f"the wall clock cap ends this run at generation "
            f"{plan.generations_reached} of {plan.shape.generations}"
        )
    return notes
