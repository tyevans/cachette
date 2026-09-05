"""The balance harness: run a fixed seed set to game end and report on the set.

One command plays each seed of a set to its game end, or to the tick limit,
and then checks four statements about the set.[^1]

1. No win path wins more than its share of the seeds.
2. No seat wins more than its share of the seeds.
3. The games that end before the tick limit are more than a stated share.
4. Every subsystem count is nonzero in some seed.

**The thresholds live in the balance register and nowhere else.** The harness
parses the markdown rows of the register, so the register is the one
declaration site.[^2] A share is unset until the rules of the downstream game
are written down, and that is an open blocker.[^3] While a share is unset the
harness reports the observed share and passes. When a share is set the
statement passes or fails, and the exit code says so.

**Python is a control plane here.** The harness loops over seeds and over
factions. It reads the game end record, the score per faction and the
subsystem census, and each of those is an aggregate the engine already
holds.[^4] It names no tile and no entity.

**The report is deterministic.** The JSON holds nothing about the machine, the
clock or the thread count, so one seed set gives one report at every thread
count.[^5]

This is not a merge gate. One game costs one run, and the command costs that
times the seed count.[^6]

References
----------
Design: the living world game layer, section 10.2.
``docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md``

Balance register. ``docs/reference/balance.md``

Blockers register, BLK-050. ``docs/BLOCKERS.md``

ADR-0040, Python is a control plane, not a data plane.
``docs/adrs/accepted/adr-0040-python-is-a-control-plane-not-a-data-plane.md``

ADR-0001, one binary gives one answer at any thread count.
``docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md``

PRD-0053, a game is balanced across seeds.
``docs/product/accepted/prd-0053-a-game-is-balanced-across-seeds.md``
"""

from __future__ import annotations

import argparse
import json
import pathlib
import re
import sys
from dataclasses import dataclass
from typing import TYPE_CHECKING, TypedDict

from cachette import World

if TYPE_CHECKING:
    from collections.abc import Callable, Sequence

    from cachette._core import GameEnd

# The world the harness runs when nobody says. These are the demonstration
# world, so a reader of the demonstration and a reader of this report see the
# same game.
WORLD_EXTENT = 256
FACTION_COUNT = 4

# The default seed set. Seed ``i`` is ``(BASE_SEED + i * SEED_STRIDE)`` in
# 64 bits, for ``i`` from 0 to ``SEED_COUNT - 1``. The base is the
# demonstration seed, so seed 0 is the demonstration world. The stride is an
# odd 64-bit constant, so no two seeds of the set coincide.
#
# **This set is a fixture and not a value.** Which seeds and how many is a row
# of the balance register, and that row is unset under BLK-050.
BASE_SEED = 0x0123_4567_89AB_CDEF
SEED_STRIDE = 0x9E37_79B9_7F4A_7C15
SEED_COUNT = 8
SEED_MASK = (1 << 64) - 1

# The register the thresholds come from, relative to the repository root.
REGISTER_PATH = pathlib.Path("docs") / "reference" / "balance.md"

# The section of the register that holds the shares, and the row each
# statement reads. The row name is the text of the first cell before its
# first comma.
SHARES_HEADING = "## Balance shares"
WIN_PATH_ROW = "Win-path share"
SEAT_ROW = "Seat share"
END_ROW = "End share"
SEED_SET_ROW = "Seed set"
SHARE_ROWS = (WIN_PATH_ROW, SEAT_ROW, END_ROW, SEED_SET_ROW)

# The word the register writes in the Set column while a row has no value.
UNSET = "unset"

# The three verdicts a statement can hold.
PASS = "pass"
FAIL = "fail"
REPORTING_ONLY = "unset: reporting only"


class Game(TypedDict):
    """What one seed produced."""

    seed: int
    winner: int | None
    path: str | None
    tick: int
    reached_tick_limit: bool
    scores: list[int]
    census: dict[str, int]


class Share(TypedDict):
    """A count over the seed set, exact and not rounded."""

    won: int
    of: int


class Statement(TypedDict):
    """One of the four balance statements, with its verdict."""

    row: str
    threshold: int | None
    verdict: str
    detail: str
    failing_seeds: list[int]


class Report(TypedDict):
    """The whole report, which the JSON file holds."""

    extent: int
    faction_count: int
    tick_limit: int
    seeds: list[int]
    thresholds: dict[str, int | None]
    games: list[Game]
    path_shares: dict[str, Share]
    seat_shares: dict[str, Share]
    reached_tick_limit: Share
    zero_in_every_game: list[str]
    statements: list[Statement]


@dataclass(frozen=True)
class Thresholds:
    """The four share rows, as the register holds them.

    A share is an integer percent of the seed set, or ``None`` while the row
    is unset. The seed set row is either set or unset, and its number is not
    read.
    """

    win_path: int | None
    seat: int | None
    end: int | None
    seed_set: int | None

    def as_dict(self) -> dict[str, int | None]:
        """Give the rows by name, for the report."""
        return {
            WIN_PATH_ROW: self.win_path,
            SEAT_ROW: self.seat,
            END_ROW: self.end,
            SEED_SET_ROW: self.seed_set,
        }


class RegisterError(ValueError):
    """The register does not hold what the harness needs."""


def default_seeds() -> list[int]:
    """Derive the default seed set from the base seed by the stated rule."""
    return [
        (BASE_SEED + index * SEED_STRIDE) & SEED_MASK for index in range(SEED_COUNT)
    ]


def find_register(start: pathlib.Path | None = None) -> pathlib.Path:
    """Walk up from a directory to the repository root that holds the register."""
    here = (start or pathlib.Path.cwd()).resolve()
    for directory in (here, *here.parents):
        candidate = directory / REGISTER_PATH
        if candidate.is_file():
            return candidate
    message = f"no {REGISTER_PATH} above {here}; pass --table"
    raise RegisterError(message)


def parse_set_cell(cell: str) -> int | None:
    """Read the Set column of one row.

    The cell is ``unset`` followed by anything, or an integer percent.
    """
    text = cell.strip()
    if text.lower().startswith(UNSET):
        return None
    match = re.fullmatch(r"(\d+)\s*%?", text)
    if match is None:
        message = f"a Set cell must be 'unset...' or an integer percent, not {text!r}"
        raise RegisterError(message)
    return int(match.group(1))


def parse_register(text: str) -> Thresholds:
    """Read the balance-share rows of the register text.

    The section under the shares heading holds one table. Each body row is
    ``| Value | Read by | Set | Blocker | Derivation |``. The name of a row is
    its first cell up to the first comma.
    """
    try:
        _, section = text.split(SHARES_HEADING, 1)
    except ValueError as error:
        message = f"the register has no {SHARES_HEADING!r} section"
        raise RegisterError(message) from error
    section = section.split("\n## ", 1)[0]
    found: dict[str, int | None] = {}
    for line in section.splitlines():
        if not line.startswith("|"):
            continue
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if len(cells) < 3 or cells[0] in {"Value", ""} or set(cells[0]) <= {"-"}:
            continue
        name = cells[0].split(",", 1)[0].strip()
        if name in SHARE_ROWS:
            found[name] = parse_set_cell(cells[2])
    missing = [row for row in SHARE_ROWS if row not in found]
    if missing:
        message = f"the register lacks the share rows {missing}"
        raise RegisterError(message)
    return Thresholds(
        win_path=found[WIN_PATH_ROW],
        seat=found[SEAT_ROW],
        end=found[END_ROW],
        seed_set=found[SEED_SET_ROW],
    )


def read_register(path: pathlib.Path) -> Thresholds:
    """Read the thresholds from the register file."""
    return parse_register(path.read_text(encoding="utf-8"))


def play(
    seed: int, extent: int, faction_count: int, tick_limit: int, threads: int
) -> Game:
    """Play one seed to its game end or to the tick limit, and read the aggregates."""
    world = World(extent, extent, seed=seed, faction_count=faction_count)
    world.seed_world()
    world.set_tick_limit(tick_limit)
    end: GameEnd | None = world.game_end()
    while end is None and world.tick < tick_limit:
        world.step(threads)
        end = world.game_end()
    return Game(
        seed=seed,
        winner=None if end is None else end["winner"],
        path=None if end is None else end["path"],
        tick=world.tick,
        reached_tick_limit=end is None or end["tick"] >= tick_limit,
        scores=[world.score(faction) for faction in range(faction_count)],
        census=dict(world.subsystem_census()),
    )


def _at_most(share: Share, percent: int) -> bool:
    """Whether the share is at most the percent, compared exactly."""
    return share["won"] * 100 <= percent * share["of"]


def _at_least(share: Share, percent: int) -> bool:
    """Whether the share is at least the percent, compared exactly."""
    return share["won"] * 100 >= percent * share["of"]


def _shares(
    games: list[Game], total: int, key: Callable[[Game], str | None]
) -> dict[str, Share]:
    """Count, for each value of the key over the ended games, how many games it won."""
    counts: dict[str, int] = {}
    for game in games:
        value = key(game)
        if value is not None:
            counts[value] = counts.get(value, 0) + 1
    return {name: Share(won=counts[name], of=total) for name in sorted(counts)}


def _describe(shares: dict[str, Share]) -> str:
    return ", ".join(
        f"{name} {share['won']}/{share['of']}" for name, share in shares.items()
    )


def _bounded(
    row: str,
    threshold: int | None,
    shares: dict[str, Share],
    games: list[Game],
    key: Callable[[Game], str | None],
) -> Statement:
    """Check that no share exceeds the threshold."""
    detail = _describe(shares) or "no game ended"
    if threshold is None:
        return Statement(
            row=row,
            threshold=None,
            verdict=REPORTING_ONLY,
            detail=detail,
            failing_seeds=[],
        )
    over = {name for name, share in shares.items() if not _at_most(share, threshold)}
    failing = [game["seed"] for game in games if key(game) in over]
    return Statement(
        row=row,
        threshold=threshold,
        verdict=FAIL if over else PASS,
        detail=detail,
        failing_seeds=failing,
    )


def _path(game: Game) -> str | None:
    return game["path"]


def _seat(game: Game) -> str | None:
    return None if game["winner"] is None else str(game["winner"])


def assess(
    games: list[Game],
    thresholds: Thresholds,
    extent: int,
    faction_count: int,
    tick_limit: int,
) -> Report:
    """Fold the games into shares and check the four statements."""
    total = len(games)
    path_shares = _shares(games, total, _path)
    seat_shares = _shares(games, total, _seat)
    limited = [game for game in games if game["reached_tick_limit"]]
    reached = Share(won=len(limited), of=total)
    before_limit = Share(won=total - len(limited), of=total)
    names = sorted({name for game in games for name in game["census"]})
    zero_rows = [
        name
        for name in names
        if all(game["census"].get(name, 0) == 0 for game in games)
    ]

    end_detail = (
        f"ended before the tick limit {before_limit['won']}/{before_limit['of']}"
    )
    if thresholds.end is None:
        end_verdict, end_failing = REPORTING_ONLY, []
    else:
        end_verdict = PASS if _at_least(before_limit, thresholds.end) else FAIL
        end_failing = [game["seed"] for game in limited]

    zero_detail = "zero in every game: " + (", ".join(zero_rows) or "none")
    if thresholds.seed_set is None:
        zero_verdict, zero_failing = REPORTING_ONLY, []
    else:
        zero_verdict = FAIL if zero_rows else PASS
        zero_failing = [game["seed"] for game in games] if zero_rows else []

    return Report(
        extent=extent,
        faction_count=faction_count,
        tick_limit=tick_limit,
        seeds=[game["seed"] for game in games],
        thresholds=thresholds.as_dict(),
        games=games,
        path_shares=path_shares,
        seat_shares=seat_shares,
        reached_tick_limit=reached,
        zero_in_every_game=zero_rows,
        statements=[
            _bounded(WIN_PATH_ROW, thresholds.win_path, path_shares, games, _path),
            _bounded(SEAT_ROW, thresholds.seat, seat_shares, games, _seat),
            Statement(
                row=END_ROW,
                threshold=thresholds.end,
                verdict=end_verdict,
                detail=end_detail,
                failing_seeds=end_failing,
            ),
            Statement(
                row=SEED_SET_ROW,
                threshold=thresholds.seed_set,
                verdict=zero_verdict,
                detail=zero_detail,
                failing_seeds=zero_failing,
            ),
        ],
    )


def run(
    seeds: Sequence[int],
    extent: int,
    faction_count: int,
    tick_limit: int,
    threads: int,
    thresholds: Thresholds,
) -> Report:
    """Play every seed and assess the set."""
    games = [play(seed, extent, faction_count, tick_limit, threads) for seed in seeds]
    return assess(games, thresholds, extent, faction_count, tick_limit)


def render(report: Report) -> str:
    """Write the report as one table and four statement lines."""
    lines = [
        f"balance: extent {report['extent']}, {report['faction_count']} factions, "
        f"tick limit {report['tick_limit']}, {len(report['seeds'])} seeds",
        "",
        f"{'seed':>20}  {'winner':>6}  {'path':<10}  {'tick':>6}  census",
    ]
    for game in report["games"]:
        winner = "-" if game["winner"] is None else str(game["winner"])
        path = game["path"] or "-"
        census = " ".join(f"{name}={count}" for name, count in game["census"].items())
        lines.append(
            f"{game['seed']:>20}  {winner:>6}  {path:<10}  {game['tick']:>6}  {census}"
        )
    lines.append("")
    for number, statement in enumerate(report["statements"], start=1):
        threshold = (
            UNSET if statement["threshold"] is None else f"{statement['threshold']}%"
        )
        lines.append(
            f"statement {number}, {statement['row']} ({threshold}): "
            f"{statement['verdict']}; {statement['detail']}"
        )
        if statement["failing_seeds"]:
            lines.append(
                "  failing seeds: "
                + ", ".join(str(seed) for seed in statement["failing_seeds"])
            )
    return "\n".join(lines) + "\n"


def to_json(report: Report) -> str:
    """Serialise the report with sorted keys, so two equal reports are equal bytes."""
    return json.dumps(report, indent=2, sort_keys=True) + "\n"


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    """Read the command line."""
    parser = argparse.ArgumentParser(
        prog="python -m cachette.balance",
        description="Run a seed set to game end and check the four balance statements.",
    )
    parser.add_argument(
        "--seeds",
        type=int,
        nargs="+",
        default=None,
        help="the seed set; the default is derived from the base seed",
    )
    parser.add_argument(
        "--extent",
        type=int,
        default=WORLD_EXTENT,
        help="the world width and height in tiles",
    )
    parser.add_argument(
        "--factions", type=int, default=FACTION_COUNT, help="the faction count"
    )
    parser.add_argument(
        "--tick-limit",
        type=int,
        default=None,
        help="the tick limit; the default is the engine default",
    )
    parser.add_argument(
        "--threads", type=int, default=1, help="the thread count of each step"
    )
    parser.add_argument(
        "--table",
        type=pathlib.Path,
        default=None,
        help="the balance register; the default is found above the working directory",
    )
    parser.add_argument(
        "--json",
        type=pathlib.Path,
        default=pathlib.Path("target/balance.json"),
        help="where to write the JSON report",
    )
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    """Run the harness. Exit 0 when no statement fails, and 1 otherwise."""
    args = parse_args(argv)
    seeds: list[int] = list(args.seeds) if args.seeds is not None else default_seeds()
    register: pathlib.Path = args.table if args.table is not None else find_register()
    thresholds = read_register(register)
    tick_limit: int = (
        args.tick_limit if args.tick_limit is not None else World(1, 1).tick_limit
    )
    report = run(
        seeds, args.extent, args.factions, tick_limit, args.threads, thresholds
    )
    output: pathlib.Path = args.json
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(to_json(report), encoding="utf-8")
    sys.stdout.write(render(report))
    sys.stdout.write(f"register {register}; threads {args.threads}; json {output}\n")
    failed = any(statement["verdict"] == FAIL for statement in report["statements"])
    return 1 if failed else 0
