"""Train a policy, and measure what it learned.

This module holds the loop of a run and nothing else. It asks a search for a
population, asks the play to score it, hands the scores back to the search,
writes the centre to disk, validates it every few generations, and records
each generation. The search, the play and the records each live in their own
module.

# The seed set moves between generations

A generation plays one set of seeds, taken from the pool in a fixed order, so
a repeat of a run plays the same worlds. The set moves at the next generation,
so a policy cannot learn one map.

**The mean of a generation is not a learning curve.** The seed set moves, so a
mean that rises may only mean that the new worlds are easier. Only the
held-out measurement is evidence.

# The win share selects, and the shaped return trains

The shaped return is the training signal. It is dense, and it is what the
search ranks. **It is not a measure of play.** A run that kept the centre
with the highest mean shaped return published four policies that take one unit
and wander, and the win share of every validation pass of that run was already
computed and thrown away.[^2]

This loop therefore selects on the win share and breaks a tie on the mean
shaped return. Nothing about the training signal changes.

# The held-out pass runs at an interval, not only at the end

The validation seeds choose the centre, so a validation figure is a selection
maximum and never an unbiased measurement. The held-out seeds choose nothing.

A run under a wall clock cap can end at any generation, so a held-out pass
that ran only after the loop returned left no honest figure behind at all.
This loop takes a held-out interval in the way it takes a validation
interval.[^2]

# The latest centre and the best centre are two different files

The latest centre is the resume point. The run writes it after every
generation, whatever it scored. The best centre is what a reader loads to play
or to measure, and it moves only when a validation pass finds something
better.

Conflating them cost this project twice. Writing only the best meant that a
run with no validation seeds wrote nothing until it ended, so an early stop
lost everything. Writing only the latest meant that the first full run stored
a centre taken from inside a collapsed region.

# The yardstick is measured in the single-seat world

A candidate in a league meets another candidate and one built-in controller.
The centre on the validation seeds meets two built-in controllers, which is
the game the run is judged on. A run that measured its centre inside its own
league would move the opponent and the policy together, and no number of that
run could be compared with a number of another.

# The learner-side arithmetic is float, and the engine's is not

The weights, the scores and the update are floating point. None of them enters
the world. The engine holds integers, and the only thing this module sends it
is one action integer.

# References

[^1]: ADR-0194, a generation is scored in shards and combined in candidate
order. ``docs/adrs/draft/adr-0194-a-generation-is-scored-in-shards.md``

[^2]: What is wrong with training and evaluation, items 1 and 2.
``docs/research/what-is-wrong-with-training-and-evaluation.md``
"""

from __future__ import annotations

import json
import math
import time
from contextlib import AbstractContextManager, nullcontext
from dataclasses import asdict, dataclass, field, replace
from typing import TYPE_CHECKING, NotRequired, TypedDict

import numpy as np

from .config import TrainConfig
from .env import Env, EnvConfig, viable_seeds
from .normalize import reference_normalizer
from .policy import (
    FeatureNormalizer,
    LinearPolicy,
    Policy,
    PolicyFit,
    PolicyFitError,
    load_policy,
)
from .presets import ObjectiveSchedule
from .record import (
    EpisodeRecord,
    GenerationRecord,
    PopulationRecord,
    ValidationScore,
    most_common_share,
    preference_varies,
)
from .reward import Scoring, Weighting
from .rollout import (
    HEARTBEAT_SECONDS,
    Generation,
    one_scoring,
    run_population,
    score_generation,
)
from .search import (
    EvolutionStrategy,
    Optimiser,
    Trainable,
    Update,
    carries_information,
    configuration_notes,
    generation_noise,
    pair_candidates,
    rank_shape,
    shell_policy,
    unit,
)
from .shard import ShardPool, run_sharded_generation
from .sizing import measures_holdout, validates
from .structured import layout_of

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Mapping, Sequence
    from pathlib import Path


class TrainResult(TypedDict):
    """What one training run reports when it ends.

    **The shape is declared here and nowhere else.** A caller that read this
    from a mapping of loose values would state the shape a second time, and
    the two statements would part company at the first change.

    The history entry holds one summary row for each generation. The
    generations entry holds the whole record of each one, including the score
    of every candidate and the reading of every episode, so that a later
    reader can ask a question this run did not ask.

    **Every entry names which quantity it holds.** The selection entries come
    from the validation seeds, which choose the centre, so they are a maximum
    over the passes of the run. The held-out entries come from seeds that
    never influenced the choice. A result that gave one number for both let a
    selection maximum be read as an unbiased measurement.[^1]

    The held-out entries are the newest periodic pass of the run. They are
    absent for a run with no held-out seeds. The holdout entry is a separate
    thing: it is the four-way comparison the report writer adds after the run
    ends, and it is absent while the run lives.

    References
    ----------
    [^1]: What is wrong with training and evaluation, items 1 and 2.
    ``docs/research/what-is-wrong-with-training-and-evaluation.md``
    """

    name: str
    kind: str
    history: list[dict[str, float | None]]
    generations: list[dict[str, object]]
    weights: str
    latest_weights: str
    parameters: int
    best_generation: int
    best_selection_won: float | None
    best_selection_return: float | None
    validation_seeds: list[int]
    holdout_seeds: list[int]
    held_out_won: float | None
    held_out_return: float | None
    held_out_generation: int
    degenerate_generations: list[int]
    holdout: NotRequired[dict[str, dict[str, float]]]


def _figure_meta(prefix: str, score: ValidationScore | None) -> dict[str, object]:
    """Return the entries a weight file states for one measured figure.

    **The prefix says what the figure is.** A name that begins with
    ``selection`` comes from the seeds that chose the centre. A name that
    begins with ``held_out`` comes from seeds that chose nothing. A reader
    that compares the two compares different quantities, so the file must not
    let one be read as the other.[^1]

    A figure that nothing measured reads back as the quiet value. That is the
    state a run with no validation seeds is in.

    References
    ----------
    [^1]: What is wrong with training and evaluation, items 1 and 2.
    ``docs/research/what-is-wrong-with-training-and-evaluation.md``
    """
    quiet = float("nan")
    if score is None:
        return {
            f"{prefix}_won": quiet,
            f"{prefix}_return": quiet,
            f"{prefix}_episodes": 0,
        }
    return {
        f"{prefix}_won": float(score.won),
        f"{prefix}_return": float(score.mean),
        f"{prefix}_episodes": int(score.episodes),
    }


def _stored_figure(meta: Mapping[str, object], prefix: str) -> ValidationScore | None:
    """Read back one measured figure of a weight file, or nothing.

    **A quiet value means that nothing measured the figure**, so it reads
    back as nothing rather than as a score. A resumed run that took the quiet
    value as it stands would compare every later pass against a quantity that
    no pass is greater than. The best centre would then never move again, the
    run would keep printing generations, and nothing would say that the search
    had stopped choosing.

    A file written by an older run holds neither entry, and it reads back as
    nothing for the same reason.
    """
    won = meta.get(f"{prefix}_won")
    mean = meta.get(f"{prefix}_return")
    if not isinstance(won, (int, float)) or not isinstance(mean, (int, float)):
        return None
    if not math.isfinite(won) or not math.isfinite(mean):
        return None
    episodes = meta.get(f"{prefix}_episodes")
    return ValidationScore(
        won=float(won),
        mean=float(mean),
        episodes=int(episodes) if isinstance(episodes, (int, float)) else 0,
    )


@dataclass(frozen=True)
class Checkpoint:
    """The two centres of one run on disk, and how to read them back.

    The best path is what a reader loads to play or to measure. The latest
    path is the resume point, and the run writes it after every generation.

    The normalizer entry is the feature transform of the run. It goes into
    the fit of every file this writes, and a resumed run refuses a checkpoint
    that was written under another one.
    """

    name: str
    out_dir: Path
    env_config: EnvConfig
    probe: Env
    kind: str
    normalizer: FeatureNormalizer | None = None

    @property
    def best_path(self) -> Path:
        """Where the run stores the centre that scored highest."""
        return self.out_dir / f"{self.name}.npz"

    @property
    def latest_path(self) -> Path:
        """Where the run stores the centre of the last generation."""
        return self.out_dir / f"{self.name}-latest.npz"

    def write(
        self,
        current: Trainable,
        target: Path,
        generation: int,
        spread: float,
        validated: ValidationScore | None,
        best: ValidationScore | None,
        held_out: ValidationScore | None = None,
    ) -> None:
        """Write one centre, and everything needed to reason about it later.

        The generation entry says where the centre came from, so a file found
        after a crash can be placed. The spread entry says whether the search
        still had a population to rank when it stopped, which is the signal
        that the first full run lost silently.

        **The file states which figure selected this centre and which did
        not.**[^4] The selection entries come from the validation seeds, which
        choose the centre, so they are a maximum over the passes of the run.
        The held-out entries come from seeds that never influenced the choice.

        A file used to carry one entry called the validation score, and a
        manifest built from it was published as a mean over held-out seeds.
        The two are different quantities and the label said the opposite of
        what the number was.[^3]

        A quiet value means that nothing measured that figure. A run with no
        validation seeds cannot tell one centre from another, so its
        selection entries are quiet.

        **The fit carries the action table as the engine published it.** That
        table is what lets a later reader place a row of this file by its verb
        and its candidate coordinates, so a change to one verb no longer
        retires the whole file.[^2]

        **A file names the fit and the episode shape, and nothing about the
        search that produced it.** The written entries used to carry a hidden
        width as well. Nothing read it back, so it was one number declared in
        a file and answered nowhere, which is the shape this project has paid
        for before.[^1]

        The reader builds what a file names from the keys the file holds, so
        it neither needs this key nor refuses a file that carries it. A file
        written by an older run therefore still loads.

        References
        ----------
        [^1]: Recurring defect shapes, shape 1 and shape 3.
        ``.agents/rules/recurring-defects.md``

        [^2]: ADR-0200, a stored policy names each row of the action table by
        its verb and its candidate coordinates, decision D1.
        ``docs/adrs/draft/adr-0200-a-stored-policy-names-each-action-row-by-verb-and-coordinates.md``

        [^3]: What is wrong with training and evaluation, items 1 and 2.
        ``docs/research/what-is-wrong-with-training-and-evaluation.md``

        [^4]: ADR-0202, a run selects on the win share, and
        every published figure names its seed set, decision D3.
        ``docs/adrs/draft/adr-0202-a-run-selects-on-the-win-share-and-every-published-figure-names-its-seed-set.md``
        """
        current.save(
            target,
            {
                "generation": generation,
                "spread": spread,
                **_figure_meta("selection", validated),
                **_figure_meta("best_selection", best),
                **_figure_meta("held_out", held_out),
                # **The fit states the world this file was trained against,
                # and the engine owns every number in it.** A reader refuses
                # a file whose fit is not the fit of the world it is asked
                # to play, so a policy never plays a world it never saw.
                **PolicyFit.of_env(self.probe, self.normalizer).as_meta(),
                "seat": self.env_config.seat,
                "tick_limit": self.env_config.tick_limit,
                "horizon": self.env_config.horizon,
                "decision_interval": self.env_config.decision_interval,
            },
        )

    def resume(
        self, shell: Trainable
    ) -> tuple[Trainable, int, ValidationScore | None, str | None]:
        """Read back the centre, the generation counter, the best score and a note.

        **A resumed run continues the run. It is not a fresh run wearing an
        old centre.** It takes the first three, so it neither repeats the
        generations already paid for nor overwrites a better checkpoint with
        a worse one.

        **A resumed run refuses a checkpoint from another world.** The centre
        of a run is a function of one observation layout, and a world of
        another extent can hold the same layout length while meaning
        something else by every position of it.

        It refuses a checkpoint written under another feature normalizer for
        the same reason. Every weight of the centre scores a standardized
        feature, so a centre read under another standardization means
        something else at every position.

        **A checkpoint that states no normalizer is refused here.** The reader
        takes such a file, so a policy published before the normalizer existed
        still plays. A resume must not take the same door. The file states no
        transform and this run holds one, which is one fact stored in two
        places with nothing that fails when they disagree, and the centre
        would then be read under a transform it was never trained through.
        Start a fresh run rather than resuming across that boundary.

        **A resumed run reports when it rebuilt the readout rather than
        loading it.** A change to one verb of the action table moves the rows
        of every verb above it, and the reader carries each row onto the row
        of the same identity.[^1] The note says what moved, and it is nothing
        when nothing moved. A reader who cannot tell a rebuild from a load
        cannot read the score of the first generation.

        A weight file states what it holds, and the reader gives back what
        the file held. A file written by an older run can therefore be
        missing a key, so each read names the type it needs and falls back
        to the value a fresh run would start at.

        References
        ----------
        [^1]: ADR-0200, a stored policy names each row of the action table by
        its verb and its candidate coordinates, decisions D3 and D5.
        ``docs/adrs/draft/adr-0200-a-stored-policy-names-each-action-row-by-verb-and-coordinates.md``

        [^2]: Recurring defect shapes, shape 1.
        ``.agents/rules/recurring-defects.md``
        """
        stored, meta = load_policy(
            self.latest_path,
            PolicyFit.of_env(self.probe, self.normalizer),
            layout_of(shell),
        )
        self._refuse_without_normalizer(stored)
        policy = shell.rebuild(np.asarray(stored.flat()))
        first_generation = 0
        written = meta.get("generation")
        if isinstance(written, (int, float)):
            first_generation = int(written) + 1
        best = None
        if self.best_path.exists():
            best = _stored_figure(load_policy(self.best_path)[1], "best_selection")
        note = meta.get("action_rebuild")
        return policy, first_generation, best, note if isinstance(note, str) else None

    def _refuse_without_normalizer(self, stored: Policy) -> None:
        """Refuse a checkpoint that states no feature normalizer.

        This run holds one, and the file states none. Every weight of the
        stored centre scores a plain squash, and every weight this run trains
        scores a standardized feature, so the same position means a different
        quantity on the two sides. Nothing in the fit separates them, because
        the fit accepts a file that states no normalizer on purpose.[^1] [^2]

        References
        ----------
        [^1]: Recurring defect shapes, shape 1.
        ``.agents/rules/recurring-defects.md``

        [^2]: ADR-0202, a run selects on the win share, and
        every published figure names its seed set, decision D6.
        ``docs/adrs/draft/adr-0202-a-run-selects-on-the-win-share-and-every-published-figure-names-its-seed-set.md``
        """
        if self.normalizer is None:
            return
        if getattr(stored, "normalizer", None) is not None:
            return
        message = (
            f"the checkpoint at {self.latest_path} states no feature "
            f"normalizer, and this run holds one of "
            f"{self.normalizer.describe()}. A centre trained through no "
            "transform means something else at every position under this "
            "one, so a resume would read the wrong quantity from every "
            "weight. Start a fresh run rather than resuming across that "
            "boundary."
        )
        raise PolicyFitError(message)


@dataclass
class Validator:
    """The held-out judge of a run, and the best centre it has seen.

    **The last generation is not the best generation.** An evolution strategy
    walks, and a walk can end downhill. The run therefore plays the centre on
    a validation seed set every few generations and keeps the centre that
    scored highest.

    The validation seeds belong to neither the training pool nor the held-out
    set, so keeping the best of them takes nothing from the held-out
    measurement that the report is judged on.

    **The win share selects, and the mean shaped return breaks a tie.**[^2] The
    shaped return is the training signal and it is not a measure of play. A
    run that selected on it published four policies that take one unit and
    wander, while the win share of every pass sat unread in the same
    object.[^1]

    **A validation figure is a selection maximum and never a measurement.**
    The judge also plays the held-out seeds, which chose nothing, and it
    chooses nothing from them. That pass runs at an interval, so a run that a
    wall clock cap ends still leaves an honest figure behind.

    References
    ----------
    [^1]: What is wrong with training and evaluation, items 1 and 2.
    ``docs/research/what-is-wrong-with-training-and-evaluation.md``

    [^2]: ADR-0202, a run selects on the win share, and
    every published figure names its seed set, decisions D1 and D4.
    ``docs/adrs/draft/adr-0202-a-run-selects-on-the-win-share-and-every-published-figure-names-its-seed-set.md``
    """

    name: str
    env_config: EnvConfig
    scoring: Scoring
    workers: int
    seeds: list[int]
    best_policy: Trainable
    best: ValidationScore | None = None
    best_generation: int = -1
    yardstick: ValidationScore | None = field(default=None)
    holdout_seeds: list[int] = field(default_factory=list)
    held_out: ValidationScore | None = None
    held_out_generation: int = -1

    def measure_controller(self) -> None:
        """Play the built-in controller in the learner's own seat.

        **A relative score is zero on average by construction**, so it cannot
        tell a population that improved from one that got worse together. The
        controller plays the learner's seat on the validation seeds, and the
        run reports every validation score against that number. The
        controller does not learn, so the yardstick is measured once and
        holds for the whole run.
        """
        if not self.seeds:
            return
        self.yardstick = self.score(
            self.best_policy,
            f"{self.name} yardstick",
            replace(self.env_config, controlled=False),
        )
        # **The shaped return stays the first field of this line.** Two
        # dashboards read it there by position, so the win share goes after
        # it rather than in front of it.
        print(
            f"  {self.name} controller yardstick {self.yardstick.mean:9.1f} "
            f"won {self.yardstick.won:5.2f}",
            flush=True,
        )

    def score(
        self,
        current: Policy,
        label: str,
        env_config: EnvConfig | None = None,
        seeds: Sequence[int] | None = None,
    ) -> ValidationScore:
        """Return what one policy measured on a seed set.

        **This chooses nothing.** It plays the seeds and reports. The caller
        that keeps the best centre is ``check`` below, and it is the only one
        that may move the best. A candidate must never replace the centre,
        because a candidate is the highest of many draws on a few seeds and
        the highest draw is usually the luckiest one.

        **The result carries both the win share and the mean shaped return.**
        This used to give back a bare float, and the caller that selected on
        it read the shaped mean without saying which quantity it took.[^1]

        The seeds entry names the set to play. The validation seeds are the
        default, and the held-out seeds are the other set a caller asks for.

        References
        ----------
        [^1]: What is wrong with training and evaluation, items 1 and 2.
        ``docs/research/what-is-wrong-with-training-and-evaluation.md``
        """
        played = run_population(
            env_config or self.env_config,
            self.scoring,
            [current],
            list(self.seeds if seeds is None else seeds),
            self.workers,
            label,
        )
        return ValidationScore.of_record(played)

    def check(self, current: Trainable, generation: int) -> ValidationScore | None:
        """Play the centre on the validation seeds, and keep it when it wins.

        **The win share decides, and the mean shaped return breaks a tie.**
        The shaped return trains the search and it does not measure play.
        """
        if not self.seeds:
            return None
        scored = self.score(current, f"{self.name} validation {generation:2d}")
        if scored.beats(self.best):
            self.best = scored
            self.best_policy = current
            self.best_generation = generation
        return scored

    def stored_centre(
        self, current: Trainable, generation: int
    ) -> tuple[Trainable, int]:
        """Return the centre the best file holds, and where it came from.

        **This is the one declaration of which centre a run publishes.** A run
        with no validation seeds cannot tell one centre from another, so the
        best file holds the centre of this generation. A run with them holds
        the centre of the pass that selected, whichever generation that was.

        The held-out pass and the writer of the best file both need this
        answer, and two copies would be one rule stored twice with nothing
        that fails when they disagree.[^1]

        References
        ----------
        [^1]: Recurring defect shapes, shape 1.
        ``.agents/rules/recurring-defects.md``
        """
        if not self.seeds:
            return current, generation
        return self.best_policy, self.best_generation

    def measure_holdout(
        self, current: Trainable, generation: int
    ) -> ValidationScore | None:
        """Play the published centre on the held-out seeds, and choose nothing.

        **The held-out seeds never influenced the choice of the centre**, so
        this is the only honest figure a run produces. A validation figure is
        a maximum over the passes of the run on the seeds that did the
        choosing.

        A run under a wall clock cap can end at any generation. A pass that
        ran only after the loop returned therefore left no honest figure at
        all for four published policies, and the label on the one figure they
        carried said the opposite of what it was.[^1]

        References
        ----------
        [^1]: What is wrong with training and evaluation, items 1 and 2.
        ``docs/research/what-is-wrong-with-training-and-evaluation.md``
        """
        if not self.holdout_seeds:
            return None
        centre, _ = self.stored_centre(current, generation)
        measured = self.score(
            centre,
            f"{self.name} holdout {generation:2d}",
            seeds=self.holdout_seeds,
        )
        self.held_out = measured
        self.held_out_generation = generation
        return measured


def generation_seeds(
    seed_pool: Sequence[int], generation: int, count: int
) -> list[int]:
    """Take the seeds of one generation from the pool, in a fixed order.

    A repeat of the same run takes the same worlds, because the order comes
    from the generation number and from nothing that a run advanced.
    """
    offset = generation * count
    return [seed_pool[(offset + index) % len(seed_pool)] for index in range(count)]


def train(
    name: str,
    env_config: EnvConfig,
    scoring: Scoring | ObjectiveSchedule,
    train_config: TrainConfig,
    out_dir: Path,
    seed_pool: list[int],
    kind: str = "linear",
    resume: bool = False,
    validation: list[int] | None = None,
    validate_every: int = 3,
    holdout: list[int] | None = None,
    holdout_every: int = 5,
) -> TrainResult:
    """Train one policy, and return what each generation scored.

    The kind entry names the policy the run trains. A linear policy scores
    each action row from a weighted sum of the features. A structured policy
    reads the ring stack and the entity tokens through shared weights, which
    lets it state a rule that two features must hold together.

    The scoring entry is what the seat is rewarded for. One scoring holds for
    the whole run. A schedule moves between several, either at each
    generation or at each position of the seed set, and **every candidate of
    one generation plays the same objective at the same position.** An
    evolution strategy ranks the candidates of a generation against each
    other, so two candidates scored under two objectives give a rank that
    says nothing about either policy.

    **The validation pass holds one objective for the whole run.** The run
    keeps the centre that scored highest on the validation seeds, and two
    scores taken under two objectives cannot be compared. The pass therefore
    takes the first scoring of the schedule and holds it.

    **The run derives one feature normalizer and holds it.** It plays a fixed
    reference sample of episodes before the first generation, and every
    candidate of every generation reads the result. A file this run writes
    carries the two arrays, so a reader plays the policy through the transform
    the weights were trained under.

    The validation entry names the seeds that choose the centre, and the
    validation interval says how often the run plays them. **The centre is
    chosen on the win share of that pass, and a tie falls to the mean shaped
    return.**

    The holdout entry names seeds that choose nothing, and the holdout
    interval says how often the run plays them. **A run that a wall clock cap
    ends must still leave an honest figure behind**, and a pass that ran only
    after this function returned left none for four published policies.
    """
    fixed = first_scoring(scoring)
    probe = Env(env_config, fixed)
    # **The run derives the normalizer once, before the first generation.**
    # Every candidate of every generation reads this one, in this process and
    # in a worker process, because two candidates that read two feature
    # transforms are not comparable and the rank over them says nothing.
    normalizer = reference_normalizer(env_config, fixed)
    checkpoint = Checkpoint(
        name=name,
        out_dir=out_dir,
        env_config=env_config,
        probe=probe,
        kind=kind,
        normalizer=normalizer,
    )
    shell = shell_policy(kind, probe, normalizer)
    optimiser: Optimiser = EvolutionStrategy(
        shell=shell,
        pairs=train_config.pairs,
        sigma=train_config.sigma,
        learning_rate=train_config.learning_rate,
        seed=train_config.seed,
    )
    report_configuration(name, train_config, shell.flat().size)
    policy: Trainable = shell
    first_generation = 0
    resumed_best: ValidationScore | None = None
    if resume and checkpoint.latest_path.exists():
        policy, first_generation, resumed_best, rebuilt = checkpoint.resume(policy)
        print(
            f"  {name} resumes from {checkpoint.latest_path} "
            f"at generation {first_generation}",
            flush=True,
        )
        if rebuilt is not None:
            print(f"  {name} {rebuilt}", flush=True)

    judge = Validator(
        name=name,
        env_config=env_config,
        scoring=fixed,
        workers=train_config.workers,
        seeds=list(validation or []),
        best_policy=policy,
        best=resumed_best,
        holdout_seeds=list(holdout or []),
    )
    judge.measure_controller()

    history: list[dict[str, float | None]] = []
    records: list[GenerationRecord] = []
    # The generations whose candidates all scored the same number. **A run
    # that reported only a mean and a best could not say that a generation
    # carried no information**, because a mean that equals the best reads the
    # same as a population that agreed by luck. The list names the
    # generations, so a reader of the report counts them without reading
    # every row.
    degenerate: list[int] = []
    started = time.time()

    # **The pool decides whether a generation is sharded, and the shard count
    # decides whether there is a pool.** One process opens nothing and scores
    # the generation here, which is the path every earlier run took.
    #
    # The pool stays open for the whole run, so a generation pays no process
    # start cost, and the matrix thread variables it sets hold for as long as
    # a worker might start.
    opened: AbstractContextManager[ShardPool | None]
    if train_config.shards > 1:
        opened = ShardPool(train_config.shards)
        print(
            f"  {name} scores each generation in {train_config.shards} processes "
            f"of {train_config.workers} workers",
            flush=True,
        )
    else:
        opened = nullcontext(None)

    with opened as pool:
        for generation in range(first_generation, train_config.generations):
            seeds = generation_seeds(
                seed_pool, generation, train_config.seeds_per_generation
            )
            centre = optimiser.start(policy.flat())
            label = f"{name} generation {generation:2d}"
            played = play_generation(
                env_config,
                scorings_of(scoring, generation, len(seeds)),
                train_config,
                optimiser,
                centre,
                generation,
                seeds,
                pool,
                kind,
                label,
                normalizer,
            )
            update = optimiser.update(centre, generation, played.ranked)
            policy = optimiser.rebuild(update.centre)
            if not update.informative:
                degenerate.append(generation)
                print(
                    f"  {name} generation {generation:2d} carried no information: "
                    f"{no_information_reason(update, played)} on seeds {seeds}. "
                    f"The centre does not move",
                    flush=True,
                )
            record = generation_record(generation, seeds, played, update)
            records.append(record)

            # **The schedule of a pass is declared once and read twice.** The
            # sizing module counts what a run will play, and this loop plays
            # it. Two copies of one interval would leave a plan that counted a
            # schedule the run does not run, and nothing would fail.
            validating = validates(generation, train_config.generations, validate_every)
            checked = judge.check(policy, generation) if validating else None
            if validating and checked is not None and train_config.validate_candidate:
                report_candidate(
                    name, generation, optimiser, centre, played, judge, checked
                )

            # **The held-out pass runs at its own interval.** It measures the
            # centre the run would publish, and it chooses nothing. A run that
            # a wall clock cap ends therefore leaves an honest figure behind
            # at the last interval it reached.
            measuring = measures_holdout(generation, holdout_every)
            held = judge.measure_holdout(policy, generation) if measuring else None

            store_centres(checkpoint, policy, judge, record, checked)
            history.append(
                record.summary(
                    checked,
                    judge.yardstick,
                    round(time.time() - started, 1),
                    holdout=held,
                )
            )
            print(
                f"  {name} generation {generation:2d} "
                f"mean {np.mean(record.ranked):9.1f} best {max(record.ranked):9.1f} "
                f"spread {record.spread:8.1f} "
                f"agreed {update.agreement:5.2f} "
                f"aligned {update.alignment:6.4f} "
                f"abs-spread {record.absolute_spread:8.1f} "
                f"won {record.won:5.2f} "
                f"ticks {record.ticks} "
                f"refused {record.refusal_share:5.2f} "
                + generation_figures(checked, held)
                + f"[{history[-1]['seconds']:.0f}s]",
                flush=True,
            )

    return {
        "name": name,
        "kind": kind,
        "history": history,
        "generations": [row.as_dict() for row in records],
        "weights": str(checkpoint.best_path),
        "latest_weights": str(checkpoint.latest_path),
        "parameters": int(judge.best_policy.flat().size),
        "best_generation": judge.best_generation,
        "best_selection_won": None if judge.best is None else judge.best.won,
        "best_selection_return": None if judge.best is None else judge.best.mean,
        "validation_seeds": list(validation or []),
        "holdout_seeds": list(holdout or []),
        "held_out_won": None if judge.held_out is None else judge.held_out.won,
        "held_out_return": None if judge.held_out is None else judge.held_out.mean,
        "held_out_generation": judge.held_out_generation,
        "degenerate_generations": list(degenerate),
    }


def generation_figures(
    checked: ValidationScore | None, held: ValidationScore | None
) -> str:
    """Return the measured part of one generation line, as named fields.

    **A field says which quantity it holds and which seeds gave it.** The
    validation fields come from the seeds that choose the centre, so they
    select. The held-out fields come from seeds that chose nothing.

    The two instrument fields say whether the policy answered one row at
    every decision, over the validation pass. A dash means that the
    generation took no pass, so nothing measured the field.[^1]

    A dashboard reads these fields by name and never by their order, so a
    field added here reaches a reader without a change there.

    References
    ----------
    [^1]: Findings register, FND-707. ``docs/FINDINGS.md``
    """
    fields = [
        ("valid", None if checked is None else checked.mean, "9.1f"),
        ("valid-won", None if checked is None else checked.won, "5.2f"),
        ("top-share", None if checked is None else checked.most_common_share, "5.2f"),
        ("varies", None if checked is None else checked.preference_varies, "5.2f"),
        ("holdout", None if held is None else held.mean, "9.1f"),
        ("holdout-won", None if held is None else held.won, "5.2f"),
    ]
    return "".join(
        f"{key} {'-' if value is None else format(value, shape)} "
        for key, value, shape in fields
    )


def first_scoring(scoring: Scoring | ObjectiveSchedule) -> Scoring:
    """Return the one scoring a run measures its centre against.

    A run keeps the centre that scored highest on the validation seeds. Two
    scores taken under two objectives cannot be compared, so the validation
    pass holds one objective for the whole run. A schedule gives its first
    scoring, which is the one the run started under.
    """
    if isinstance(scoring, ObjectiveSchedule):
        return scoring.scorings[0]
    return scoring


def scorings_of(
    scoring: Scoring | ObjectiveSchedule, generation: int, episodes: int
) -> Scoring | Sequence[Scoring]:
    """Return what scores each episode position of one generation.

    **The result is indexed by the seed position and never by the
    candidate.** A schedule takes the generation and the episode count, and
    it has no argument for a candidate, so a run cannot be configured to
    score two candidates of one generation under two objectives.
    """
    if isinstance(scoring, ObjectiveSchedule):
        return scoring.for_generation(generation, episodes)
    return scoring


def play_generation(
    env_config: EnvConfig,
    scoring: Scoring | Sequence[Scoring],
    train_config: TrainConfig,
    optimiser: Optimiser,
    centre: np.ndarray,
    generation: int,
    seeds: list[int],
    pool: ShardPool | None,
    kind: str,
    label: str,
    normalizer: FeatureNormalizer | None = None,
) -> Generation:
    """Score one generation, in this process or across the worker pool.

    **A worker process builds its own candidates from the centre and the
    generation number.** The centre and the normalizer cross to it, so this
    process builds the population only when it plays the population itself.

    A sharded generation scores one batch under one objective. A run that
    varies the objective by seed position therefore fails here rather than
    scoring the shards under the first entry of its schedule.
    """
    if pool is None:
        return score_generation(
            env_config,
            scoring,
            optimiser.propose(centre, generation),
            seeds,
            train_config,
            label,
        )
    return run_sharded_generation(
        env_config,
        one_scoring(scoring),
        train_config,
        seeds,
        generation,
        centre,
        pool,
        kind,
        label,
        normalizer,
    )


def report_configuration(name: str, train_config: TrainConfig, trainable: int) -> None:
    """Say what this configuration can reach, before the run spends anything.

    **A run that gives each candidate fewer worlds than its sigma needs ranks
    its candidates on noise.** A run shorter than its alignment allows wanders
    further than it climbs. Both figures follow from the configuration and
    from the trainable count of the policy, so both are free.

    The run that paid for the audit of this path was under-sampled for its
    sigma. Nothing said so, and nobody derived the number until the run had
    finished.[^1]

    References
    ----------
    [^1]: Report on how sigma trades against the worlds each candidate plays,
    section 7.
    ``docs/research/how-sigma-trades-against-worlds-for-each-candidate.md``
    """
    for note in configuration_notes(
        train_config.sigma,
        train_config.seeds_per_generation,
        train_config.pairs,
        trainable,
        train_config.generations,
    ):
        print(f"  {name} {note}", flush=True)


def no_information_reason(update: Update, played: Generation) -> str:
    """Say why one generation left the centre where it was.

    **A generation carries no information in two ways.** Every candidate may
    score the same number, which a world that decides itself produces. Or the
    candidates may score differently and still rank no better than a ranking
    of pure noise ranks, which leaves the search no direction to step along.

    The centre stands still either way, so the line names which of the two the
    generation was. A reader who could not tell them apart would read a world
    that decides itself as a search that found nothing.
    """
    if update.spread == 0.0:
        return f"every candidate scored {float(played.ranked.max()):9.1f}"
    return "the candidates agreed no better than noise agrees"


def generation_record(
    generation: int, seeds: Sequence[int], played: Generation, update: Update
) -> GenerationRecord:
    """Join what the generation scored to what the search made of it.

    The spread of the raw return is recorded beside the spread of the ranked
    score, because the two answer different questions and a run that recorded
    one of them could not be compared with the other.
    """
    return GenerationRecord(
        generation=generation,
        seeds=tuple(int(seed) for seed in seeds),
        ranked=tuple(float(value) for value in played.ranked),
        absolute=tuple(float(value) for value in played.absolute),
        spread=update.spread,
        absolute_spread=float(played.absolute.max() - played.absolute.min()),
        informative=update.informative,
        won=played.won,
        ticks=played.ticks,
        chosen=played.chosen,
        refused=played.refused,
        episodes=played.episodes,
        objectives=played.objectives,
    )


def store_centres(
    checkpoint: Checkpoint,
    policy: Trainable,
    judge: Validator,
    record: GenerationRecord,
    checked: ValidationScore | None,
) -> None:
    """Write the latest centre, and the best centre when the best moved.

    **The latest centre is written every generation, unconditionally.** No
    validation gate and no improvement gate. This is the resume point, and a
    run that stops between two validation passes must still leave one behind.

    The best centre moves only when a validation pass finds something better.
    A run with no validation seeds has no way to tell one centre from another,
    so it writes the centre of this generation to both files.

    **Each file states which figure selected the centre and which did not.**
    The selection figure comes from the validation seeds, and the held-out
    figure comes from seeds that chose nothing. The held-out figure is the
    newest periodic pass of the run.

    **The judge says which centre the best file holds.** This decides only
    whether to write it again, so the two answers cannot part company.
    """
    checkpoint.write(
        policy,
        checkpoint.latest_path,
        record.generation,
        record.spread,
        checked,
        judge.best,
        judge.held_out,
    )
    if judge.seeds and judge.best_generation != record.generation:
        return
    centre, from_generation = judge.stored_centre(policy, record.generation)
    checkpoint.write(
        centre,
        checkpoint.best_path,
        from_generation,
        record.spread,
        checked,
        judge.best,
        judge.held_out,
    )


def report_candidate(
    name: str,
    generation: int,
    optimiser: Optimiser,
    centre: np.ndarray,
    played: Generation,
    judge: Validator,
    checked: ValidationScore,
) -> ValidationScore:
    """Play the highest candidate of a generation on the validation seeds.

    **The highest candidate of a generation is the highest of many draws on a
    few seeds, so it is usually the luckiest and not the best.** The reported
    best therefore says nothing about whether the population found a policy
    the centre should move toward. Playing that candidate on the validation
    seeds says it: a candidate that holds its score there is a real gain the
    centre is not taking, and one that falls back to the score of the centre
    was luck.

    **This chooses nothing.** The stored best centre is decided by the
    validator and by nothing here.
    """
    highest = int(np.argmax(played.absolute))
    scored = judge.score(
        optimiser.propose(centre, generation)[highest],
        f"{name} candidate {generation:2d}",
    )
    print(
        f"  {name} generation {generation:2d} "
        f"candidate {highest:4d} scored "
        f"{played.absolute[highest]:9.1f} on its own seeds and "
        f"{scored.mean:9.1f} on the validation seeds, "
        f"where the centre scored {checked.mean:9.1f}",
        flush=True,
    )
    return scored


def evaluate(
    env_config: EnvConfig,
    scoring: Scoring,
    policy: Policy,
    seeds: list[int],
    workers: int,
    repeats: int = 1,
    label: str = "",
) -> dict[str, float]:
    """Play one policy on a seed set, and average what it ended with.

    The repeats entry plays the seed set more than once. The engine is
    deterministic, so a repeat only changes the answer for a policy that
    draws at random. A repeat therefore narrows the random baseline, which
    is the baseline that matters.

    The label names the pass in a progress line, for example
    ``conquer baseline``. A pass that gives one reports itself while it runs,
    and a pass that gives none stays silent. **A pass over the held-out seeds
    takes minutes, and a silent pass of that length reads as a stopped
    process.** A dashboard tells a working process from a stopped one by the
    age of its last line, so a long pass must give it one.

    The summary holds one entry for each quantity the engine publishes about
    the seat, and the refusal figures beside them. **The set of quantities
    comes from the schema of the engine and never from a tuple written here.**
    """
    played = [
        run_population(env_config, scoring, [policy], seeds, workers, label=label)
        for _ in range(max(1, repeats))
    ]
    return summarise(played)


def summarise(played: Sequence[PopulationRecord]) -> dict[str, float]:
    """Return one summary over a set of batches of one policy.

    **This is the one declaration of what a summary holds.** Two callers need
    it: the pass that plays one objective, and the pass that plays several
    objectives over one set of episodes. A second copy would be one rule
    stored twice, with nothing that fails when the copies disagree.

    The return entry is the mean over the batches, so a caller that repeated
    the seed set reads one number. Every other entry is a mean over every
    episode of every batch.

    The summary holds one entry for each quantity the engine publishes about
    the seat, and the refusal figures beside them. **The set of quantities
    comes from the schema of the engine and never from a tuple written
    here.**
    """
    rows: list[dict[str, float]] = []
    episodes: list[EpisodeRecord] = []
    values: list[float] = []
    for batch in played:
        values.append(batch.mean())
        rows.extend(batch.rows())
        episodes.extend(batch.episodes)
    summary = {"return": float(np.mean(values)), "episodes": float(len(rows))}
    for name in sorted({key for row in rows for key in row}):
        summary[name] = float(np.mean([row[name] for row in rows]))
    chosen = sum(row.chosen for row in episodes)
    refused = sum(row.refused for row in episodes)
    summary["chosen"] = float(chosen)
    summary["refused"] = float(refused)
    # What share of the actions a policy chose the engine did not carry out.
    # **A policy the engine mostly refuses is close to a no-op whatever it
    # chooses**, and no earlier figure of a run said so.
    summary["refusal_share"] = refused / chosen if chosen else 0.0
    # The two behaviour instruments, over the same episodes. **A held-out
    # figure that only reports a return cannot say that the policy answered
    # one row at every decision**, and the engine's legality answer makes a
    # fixed preference order emit many different actions.
    summary["most_common_share"] = most_common_share(episodes)
    summary["preference_varies"] = preference_varies(episodes)
    return summary


def write_report(path: Path, payload: Mapping[str, object]) -> None:
    """Write the run report as one JSON file."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, default=str), encoding="utf-8")


# The names an earlier version of this module declared. They live in the
# search, the play, the records and the run configuration now. **A caller
# keeps importing them from here**, because a rename across every call site
# buys nothing that a re-export does not.
__all__ = [
    "HEARTBEAT_SECONDS",
    "Checkpoint",
    "EnvConfig",
    "EpisodeRecord",
    "EvolutionStrategy",
    "Generation",
    "GenerationRecord",
    "LinearPolicy",
    "Optimiser",
    "Policy",
    "PolicyFit",
    "PopulationRecord",
    "Scoring",
    "TrainConfig",
    "TrainResult",
    "Trainable",
    "Update",
    "ValidationScore",
    "Validator",
    "Weighting",
    "asdict",
    "carries_information",
    "evaluate",
    "generation_figures",
    "generation_noise",
    "generation_record",
    "generation_seeds",
    "pair_candidates",
    "play_generation",
    "rank_shape",
    "report_candidate",
    "run_population",
    "score_generation",
    "shell_policy",
    "store_centres",
    "summarise",
    "train",
    "unit",
    "viable_seeds",
    "write_report",
]
