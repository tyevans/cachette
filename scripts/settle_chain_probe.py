"""Measure each link of the chain that ends in a founded city.

No trained policy of this project has founded a city. The settle verb was
never offered as legal in a recorded sample of decisions, and every
checkpoint sits below the built-in controller on the settlement count. This
tool asks which link of the chain refuses, and it asks the engine rather than
a reader of the engine.

# The chain has four links

A faction founds a city through one path, and each step of it gates the step
after it.

1. The queue verb takes a unit type. One row of the action table therefore
   names the settler, and the row is legal while the faction owns a site
   whose queue has room.
2. The verb pushes the order into the queue of the site the engine picks. The
   push is refused when the site belongs to another faction, when the queue
   already holds its bound, or when the number names no unit type.
3. The production pass advances the front entry of every queue. A finished
   entry needs the work, the goods in the store and one resident of the site.
4. The settle verb founds from a settler that stands on ground a city may
   take, and it walks the rest at the place the survey names. The verb
   refuses held ground, ground that carries a settlement, ground that admits
   nobody, and a place inside the founding distance of a settlement that
   stands.

# Every row of the table is resolved by identity

The action table has been renumbered once, so this tool holds no row number.
It reads the schema the engine publishes, finds the block of the queue verb by
name, and takes the row that names the settler by the stride of the unit type
position.[^1] It finds the settler by the settle column of the unit type
table, and never by a type index.[^2]

# The controller is the positive control

The built-in controller founds cities and no policy does. This tool therefore
plays the controller in the same seat on the same seeds. **A link the
controller also fails is an engine defect. A link only a policy fails is a
choice the policy never makes.**

The third arm is a scripted prober. It takes the settler row whenever the row
is legal and it takes the settle row whenever that row is legal. It is the arm
that proves links two, three and four can pass at all, because a policy that
never queues a settler exercises none of them.

# A refusal is silent, so the tool reads the engine reader

The action verb answers one byte and names no reason. This tool therefore
reads the queue of every site the seat owns before and after the push, so a
push that changed nothing is visible. It reads the settle refusal of every
settler through the engine reader, so the reason comes from the rule the verb
applies and not from a copy of that rule.[^3]

# What the numbers mean

A share is over the decisions of the episode. A count is over the episode. The
report gives the mean of each over the seeds, and the count of seeds on which
the arm founded at least one city.

# References

[^1]: ADR-0176, an action integer is a mixed radix over the argument positions
each verb declares, decision D1.
``docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md``
[^2]: ADR-0145, a unit type is a row of capability columns, and zero means
cannot, decision D2.
``docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md``
[^3]: Recurring defect shapes, shape 1. ``.agents/rules/recurring-defects.md``
"""

from __future__ import annotations

import argparse
import json
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import TYPE_CHECKING

import numpy as np

from cachette import ViewError, World
from cachette.learn.env import Env, EnvConfig, viable_seeds
from cachette.learn.policy import PolicyFit, load_policy
from cachette.learn.reward import Weighting

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Sequence

    from cachette.learn.policy import ActionTable, Policy

# The verb the chain starts at, and the verb it ends at. The names come from
# the schema the engine publishes, and the tool fails by name when the engine
# stops publishing one.
QUEUE_VERB = "queue"
SETTLE_VERB = "settle"

# The candidate the queue verb declares for its argument position.
UNIT_TYPE_CANDIDATE = "unit_type"

# Where the seed search starts. It is the start the behaviour report of the
# training run reads from, so the two measurements share worlds.
SEED_START = 50_000

# The weighting the arms play under. Every terminal weight is set, because a
# reward refuses to run while a weight a caller asked for is unset. No shaped
# term is asked for, so no reading of this tool depends on a weight.
PROBE_WEIGHTING = Weighting(terms={}, won=1.0, lost=-1.0, drawn=0.0)


class ProbeError(RuntimeError):
    """The engine published a schema this tool cannot read."""


def probe_world(config: EnvConfig) -> World:
    """Build one seeded world of the shape the run plays.

    The unit type table is a property of the world parameters alone, so one
    world answers for every arm and every seed.
    """
    world = World(
        width=config.width,
        height=config.height,
        seed=0,
        faction_count=config.faction_count,
    )
    world.seed_world()
    return world


def settler_type_of(world: World) -> int:
    """Return the unit type row that founds a city.

    The answer is the lowest row whose settle column is above zero. **The
    column is the rule and a type index is not**, so a table that moves the
    settler still answers here.

    Raises ``ProbeError`` when no row of the table founds a city.
    """
    column = np.asarray(world.unit_type_table()["settle_group"])
    rows = np.flatnonzero(column > 0)
    if rows.size == 0:
        message = "no row of the unit type table holds a settle column above zero"
        raise ProbeError(message)
    return int(rows[0])


def queue_row_of(table: ActionTable, settler: int) -> int:
    """Return the action row that queues the settler.

    The row comes from the block of the queue verb and the stride of its unit
    type position. Nothing here holds a row number.

    Raises ``ProbeError`` when the table holds no queue verb, when the verb
    declares no unit type position, or when the settler is outside the bound
    of that position.
    """
    block = table.named(QUEUE_VERB)
    if block is None:
        held = [verb.name for verb in table.verbs]
        message = f"the table holds no {QUEUE_VERB!r} verb. It holds {held}."
        raise ProbeError(message)
    candidates = block.candidates()
    if UNIT_TYPE_CANDIDATE not in candidates:
        message = (
            f"the {QUEUE_VERB!r} verb declares no {UNIT_TYPE_CANDIDATE!r} "
            f"position. It declares {list(candidates)}."
        )
        raise ProbeError(message)
    place = candidates.index(UNIT_TYPE_CANDIDATE)
    coordinates = [0] * len(candidates)
    coordinates[place] = settler
    if not block.holds(coordinates):
        bound = block.positions[place].bound
        message = f"the settler is row {settler} and the position bound is {bound}"
        raise ProbeError(message)
    return int(block.row_of(coordinates))


def settle_row_of(table: ActionTable) -> int:
    """Return the action row that founds a city.

    Raises ``ProbeError`` when the table holds no settle verb.
    """
    block = table.named(SETTLE_VERB)
    if block is None:
        held = [verb.name for verb in table.verbs]
        message = f"the table holds no {SETTLE_VERB!r} verb. It holds {held}."
        raise ProbeError(message)
    return int(block.first)


@dataclass
class Tally:
    """What one episode of one arm reached at each link of the chain.

    Every entry is a count over the episode. A share is derived from the
    decision count when the report is built, so nothing here divides.
    """

    decisions: int = 0
    queue_legal: int = 0
    queue_taken: int = 0
    queue_applied: int = 0
    queue_reached_a_site: int = 0
    settle_legal: int = 0
    settle_taken: int = 0
    settle_applied: int = 0
    settler_entries: int = 0
    settlers_made: int = 0
    settlers_highest: int = 0
    foundings: int = 0
    settlements_at_the_end: int = 0
    refusals: dict[str, int] = field(default_factory=dict)

    def count_refusal(self, reason: str) -> None:
        """Add one to the count of one named settle refusal."""
        self.refusals[reason] = self.refusals.get(reason, 0) + 1


class Watcher:
    """Reads the seat of one world, and states what each link reached.

    The watcher holds the identities of the sites it has seen founded, because
    the engine publishes no list of the settlements of a faction. It reads the
    founding log after every write, because the log carries one tick and the
    step empties it.
    """

    def __init__(self, world: World, seat: int, settler: int) -> None:
        """Watch one seat of one world, and read the sites it owns."""
        self._world = world
        self._seat = seat
        self._settler = settler
        self._sites: set[int] = set()
        self._founded: set[int] = set()
        self._seeded: set[int] = set()
        self._settlers: set[int] = set()
        self.collect()
        # **The seeding founds one city for each faction, and that founding
        # is not the seat's own.** The question is whether the seat founded a
        # city while it played, so the seats of the world stay out of the
        # count. The log still holds those rows until the first tick empties
        # it, so the reader keeps their identities and never counts them.
        self._seeded = set(self._founded)
        self._founded.clear()

    def collect(self) -> None:
        """Read the founding log, and keep the identity of each new city.

        The log holds the foundings of the current tick alone, and the step
        empties it. A caller therefore reads it after each action and after
        each tick, because both write it.

        **The reader keeps identities and never adds to a count.** Two reads
        of one tick see the same row twice, and a count would then hold one
        founding as two.
        """
        log = self._world.log("settlement_founded")
        for site, faction in zip(log["settlement"], log["faction"], strict=True):
            self._sites.add(int(site))
            if int(faction) == self._seat and int(site) not in self._seeded:
                self._founded.add(int(site))

    @property
    def foundings(self) -> int:
        """How many cities the seat has founded since the episode started."""
        return len(self._founded)

    def my_sites(self) -> list[int]:
        """Return the sites the seat owns now, in identity order.

        A site the seat lost, or a site that no longer stands, is dropped. The
        engine answers the faction of a site, so ownership comes from the
        engine and not from the tick the site was founded on.
        """
        mine = []
        for site in sorted(self._sites):
            try:
                economy = self._world.site_economy(site)
            except ViewError:
                continue
            if int(economy["faction"]) == self._seat:
                mine.append(site)
        return mine

    def queued_settlers(self) -> int:
        """Return the settler entries standing in the queues of the seat."""
        total = 0
        for site in self.my_sites():
            try:
                queue = self._world.site_queue(site)
            except ViewError:
                continue
            total += int(
                np.count_nonzero(np.asarray(queue["unit_type"]) == self._settler)
            )
        return total

    def settlers(self) -> list[int]:
        """Return the live settlers of the seat, in identity order."""
        units = self._world.faction_units(self._seat)["unit"]
        live = [
            int(unit)
            for unit in units
            if self._world.unit_type(int(unit)) == self._settler
        ]
        self._settlers.update(live)
        return sorted(live)

    @property
    def settlers_made(self) -> int:
        """How many distinct settlers the seat has held this episode."""
        return len(self._settlers)

    def settle_refusals(self) -> list[str]:
        """Name why the settle verb refuses each settler of the seat.

        The reason comes from the engine reader, so it is the rule the verb
        applies. A settler the verb would accept is named by the accepted
        token, which the engine also publishes.
        """
        return list(self._world.settle_refusals(self._seat))


def play(
    env: Env,
    seed: int,
    policy: Policy | None,
    queue_row: int,
    settle_row: int,
    settler: int,
    scripted: bool,
) -> Tally:
    """Play one episode and tally what each link of the chain reached.

    The policy entry is the thing that chooses a row. It is nothing for the
    arm that leaves the seat to the built-in controller, and it is nothing for
    the scripted arm, which takes the two rows of the chain by rule.

    **This drives the action and the ticks apart**, in the way the batch does.
    An action founds a city between two ticks, and the step empties the
    founding log, so a caller that only read the log after a tick would miss
    every founding an action made.
    """
    observation = env.reset(seed)
    world = env.world
    watch = Watcher(world, env.seat, settler)
    tally = Tally()
    while not env.done:
        mask = env.action_mask()
        queue_legal = bool(mask[queue_row])
        settle_legal = bool(mask[settle_row])
        tally.queue_legal += int(queue_legal)
        tally.settle_legal += int(settle_legal)
        for reason in watch.settle_refusals():
            tally.count_refusal(reason)

        action = 0
        if scripted:
            if settle_legal:
                action = settle_row
            elif queue_legal:
                action = queue_row
        elif policy is not None:
            # **The policy protocol answers a batch and never one row.** The
            # training loop and the held-out measurement both score a whole
            # batch, so a caller of one row sends a batch of one.
            batch = policy.choose_many(observation[np.newaxis, :], mask[np.newaxis, :])
            action = int(batch[0])

        before = watch.queued_settlers()
        applied = env.apply(action)
        watch.collect()
        after = watch.queued_settlers()
        if action == queue_row:
            tally.queue_taken += 1
            tally.queue_applied += int(bool(applied))
            tally.queue_reached_a_site += int(after > before)
        if action == settle_row:
            tally.settle_taken += 1
            tally.settle_applied += int(bool(applied))

        for _ in range(env.config.decision_interval):
            world.step(env.config.threads)
            watch.collect()
        result = env.settle(applied)
        observation = result.observation
        tally.decisions += 1
        tally.settler_entries = max(tally.settler_entries, watch.queued_settlers())
        tally.settlers_highest = max(tally.settlers_highest, len(watch.settlers()))

    tally.settlers_made = watch.settlers_made
    tally.foundings = watch.foundings
    tally.settlements_at_the_end = len(watch.my_sites())
    return tally


@dataclass(frozen=True)
class Arm:
    """One thing that plays the seat, and how the loop drives it."""

    name: str
    config: EnvConfig
    policy: Policy | None
    scripted: bool


def build_arms(config: EnvConfig, weights: Sequence[Path]) -> list[Arm]:
    """Return the arms of the run, in the order the report prints them.

    The controller arm leaves the seat to the built-in controller, which is
    the positive control. The scripted arm takes the two rows of the chain by
    rule. Each weight file gives one policy arm.
    """
    probe = Env(config, PROBE_WEIGHTING)
    fit = PolicyFit.of_env(probe)
    arms = [
        Arm(
            name="controller",
            config=EnvConfig(**{**config.__dict__, "controlled": False}),
            policy=None,
            scripted=False,
        ),
        Arm(name="scripted", config=config, policy=None, scripted=True),
    ]
    for path in weights:
        policy, _ = load_policy(path, fit)
        arms.append(Arm(name=path.stem, config=config, policy=policy, scripted=False))
    return arms


@dataclass(frozen=True)
class Report:
    """What one arm reached at each link, over every seed it played.

    A share is over the decisions of every episode. A mean is over the
    episodes. The refusal entry holds the count of each named settle refusal
    over every decision of every episode.
    """

    arm: str
    seeds: int
    decisions: int
    queue_legal_share: float
    queue_taken_share: float
    queue_applied: int
    queue_reached_a_site: int
    settle_legal_share: float
    settle_taken_share: float
    settle_applied: int
    settler_entries_highest: int
    settlers_made_mean: float
    settlers_highest: int
    foundings_mean: float
    seeds_that_founded: int
    settlements_mean: float
    settle_refusals: dict[str, int]

    def as_dict(self) -> dict[str, object]:
        """Return this report as plain values, for a report file."""
        return asdict(self)


def report_of(name: str, tallies: Sequence[Tally]) -> Report:
    """Return the mean of each link over the seeds of one arm."""
    decisions = float(sum(tally.decisions for tally in tallies)) or 1.0
    seeds = float(len(tallies)) or 1.0
    refusals: dict[str, int] = {}
    for tally in tallies:
        for reason, count in tally.refusals.items():
            refusals[reason] = refusals.get(reason, 0) + count
    return Report(
        arm=name,
        seeds=len(tallies),
        decisions=int(decisions),
        queue_legal_share=sum(tally.queue_legal for tally in tallies) / decisions,
        queue_taken_share=sum(tally.queue_taken for tally in tallies) / decisions,
        queue_applied=sum(tally.queue_applied for tally in tallies),
        queue_reached_a_site=sum(tally.queue_reached_a_site for tally in tallies),
        settle_legal_share=sum(tally.settle_legal for tally in tallies) / decisions,
        settle_taken_share=sum(tally.settle_taken for tally in tallies) / decisions,
        settle_applied=sum(tally.settle_applied for tally in tallies),
        settler_entries_highest=max(tally.settler_entries for tally in tallies),
        settlers_made_mean=sum(tally.settlers_made for tally in tallies) / seeds,
        settlers_highest=max(tally.settlers_highest for tally in tallies),
        foundings_mean=sum(tally.foundings for tally in tallies) / seeds,
        seeds_that_founded=sum(1 for tally in tallies if tally.foundings > 0),
        settlements_mean=sum(tally.settlements_at_the_end for tally in tallies) / seeds,
        settle_refusals=dict(sorted(refusals.items())),
    )


def print_report(rows: Sequence[Report]) -> None:
    """Print one line for each link of the chain, for each arm."""
    heading = (
        f"{'arm':34} {'q_legal':>8} {'q_taken':>8} {'q_reach':>8} "
        f"{'s_legal':>8} {'s_taken':>8} {'settlers':>9} {'found':>7} {'sites':>7}"
    )
    print(heading)
    print("-" * len(heading))
    for row in rows:
        print(
            f"{row.arm:34} "
            f"{row.queue_legal_share:8.3f} "
            f"{row.queue_taken_share:8.3f} "
            f"{row.queue_reached_a_site:8d} "
            f"{row.settle_legal_share:8.3f} "
            f"{row.settle_taken_share:8.3f} "
            f"{row.settlers_made_mean:9.2f} "
            f"{row.foundings_mean:7.2f} "
            f"{row.settlements_mean:7.2f}"
        )
    print()
    for row in rows:
        print(f"{row.arm}: settle refusals {row.settle_refusals}")


def main(argv: Sequence[str] | None = None) -> int:
    """Run the probe and print the report."""
    parser = argparse.ArgumentParser(description=__doc__ and __doc__.splitlines()[0])
    parser.add_argument("--extent", type=int, default=48)
    parser.add_argument("--factions", type=int, default=3)
    parser.add_argument("--seeds", type=int, default=8)
    parser.add_argument("--horizon", type=int, default=120)
    parser.add_argument("--tick-limit", type=int, default=600)
    parser.add_argument("--decision-interval", type=int, default=5)
    parser.add_argument(
        "--policy",
        action="append",
        default=[],
        type=Path,
        help="a stored weight file to play, repeated for each arm",
    )
    parser.add_argument("--out", type=Path, default=None)
    args = parser.parse_args(argv)

    config = EnvConfig(
        width=args.extent,
        height=args.extent,
        faction_count=args.factions,
        tick_limit=args.tick_limit,
        horizon=args.horizon,
        decision_interval=args.decision_interval,
    )
    seeds = viable_seeds(config, args.seeds, SEED_START)
    arms = build_arms(config, args.policy)
    settler = settler_type_of(probe_world(config))

    rows = []
    for arm in arms:
        env = Env(arm.config, PROBE_WEIGHTING)
        queue_row = queue_row_of(env.action_table, settler)
        settle_row = settle_row_of(env.action_table)
        tallies = [
            play(env, seed, arm.policy, queue_row, settle_row, settler, arm.scripted)
            for seed in seeds
        ]
        rows.append(report_of(arm.name, tallies))

    print_report(rows)
    if args.out is not None:
        report = [row.as_dict() for row in rows]
        args.out.write_text(json.dumps(report, indent=2), encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
