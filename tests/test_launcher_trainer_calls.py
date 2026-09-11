"""Every trainer call of the training launcher accepts the arguments of a run.

The launcher calls the trainer several times. It calls it on this machine
before it rents, and on the instance from a script it writes. **The instance
script removes `--only` from the run arguments after it asks for the plan**,
because the training call adds an `--only` of its own. A call after that
removal names every row of the strategy table.

One run trained the readout alone of a structured strategy. The refusal of that
setting checked every named row on every call, so the world call on the
instance named the linear rows and failed. The instance was paid for.

The tests of the refusal drove the command line of the trainer. They never
drove the sequence of calls the launcher makes. The tests here drive the
launcher.[^1] They run its dry path, and they run the script it writes for the
instance. A stand-in on the path answers for each tool that rents or installs,
and a stand-in for `uv` sends each trainer call to the real trainer. The calls
therefore come from the launcher, so these tests follow the launcher when it
changes.[^2]

The training call stops at its first seed search, because the trainer plays
episodes after that and a test must not. Every check of the arguments comes
before that search.

**A run may start from the weights of another run.** The launcher gives the
trainer the start file on the first attempt only, and a restart after a fault
resumes from the resume point. The restart tests make the first attempt fail
as a fault in native code does, and a stand-in clock makes that attempt last
long enough to earn a restart.

References
----------
[^1]: Testing Rules, drive the real caller. ``.agents/rules/testing.md``
[^2]: The training launcher. ``scripts/graviton-train.sh``
"""

from __future__ import annotations

import os
import shutil
import stat
import subprocess
import sys
from pathlib import Path
from typing import TYPE_CHECKING

import pytest

from cachette.learn.env import Env, EnvConfig
from cachette.learn.reward import Weighting
from cachette.learn.search import shell_policy
from cachette.learn.structured import STRUCTURED_KIND
from cachette.learn.train import Checkpoint

if TYPE_CHECKING:
    from cachette.learn.policy import FeatureNormalizer

ROOT = Path(__file__).resolve().parents[1]
LAUNCHER = ROOT / "scripts" / "graviton-train.sh"

# The arguments of the run that failed on the instance. It trains the readout
# alone of one structured strategy, and every other row of the table is linear.
ONE_BOX_ARGUMENTS = (
    "--only wealth-structured --world-extent 48 --tick-limit 2500 "
    "--population 128 --seeds 12 --validation 24 --train-readout-only "
    "--limit-is-loss --generations 20"
)

# The world the arguments above name. The trainer derives the horizon from the
# tick limit and the interval, and a file of another fit would be refused by
# the check under test, so a wrong copy here fails the tests that use it.
ONE_BOX_WORLD = EnvConfig(
    width=48,
    height=48,
    faction_count=3,
    seat=0,
    tick_limit=2500,
    horizon=250,
    decision_interval=10,
    limit_is_loss=True,
)

# The name of the start file a run of one box starts from.
START_NAME = "wonder-structured-latest.npz"

# The generation a written start file states.
START_GENERATION = 7

# The status a fault in native code leaves. The stand-in trainer ends a faulted
# attempt with it.
FAULT_STATUS = 139

# The heredoc of the launcher that holds the instance script.
INSTANCE_OPENING = "<<'REMOTE'\n"
INSTANCE_CLOSING = "\nREMOTE\n"

# The flags that select a path of the trainer other than the training run.
PATH_FLAGS = ("--print-strategies", "--print-world", "--print-plan", "--baseline-only")

STAND_IN_UV = r"""#!/usr/bin/env bash
set -u
case "${1:-}" in
    run) shift ;;
    *) exit 0 ;;
esac
while [ $# -gt 0 ]; do
    case "$1" in
        --no-sync|--no-project) shift ;;
        --with) shift 2 ;;
        *) break ;;
    esac
done
if [ "${1:-}" = "python" ]; then shift; fi
while [ "${1:-}" = "-u" ]; do shift; done
if [ "${1:-}" = "-m" ] && [ "${2:-}" = "cachette.learn" ]; then
    shift 2
    "$STAND_IN_PYTHON" "$STAND_IN_TRAINER" "$@"
    status=$?
    printf '%s\t%s\t%s\n' "$STAND_IN_SIDE" "$status" "$*" >> "$STAND_IN_CALLS"
    exit "$status"
fi
if [ "${1:-}" = "scripts/train_throughput.py" ]; then
    out=""
    while [ $# -gt 0 ]; do
        if [ "$1" = "--out" ]; then out="$2"; fi
        shift
    done
    printf 'processes\tworkers\tticks_per_second_per_worker\n1\t8\t100.0\n' > "$out"
    exit 0
fi
exec "$STAND_IN_PYTHON" "$@"
"""

STAND_IN_AWS = r"""#!/usr/bin/env bash
for word in "$@"; do
    case "$word" in
        describe-spot-price-history) printf '0.5000\tus-west-2a\n'; exit 0 ;;
        describe-instance-types) printf '64\n'; exit 0 ;;
    esac
done
printf 'the stand-in for aws does not answer: %s\n' "$*" >&2
exit 1
"""

# A clock that moves a thousand seconds at each reading of the epoch, and
# gives every other reading to the real tool. The instance script restarts an
# attempt only when it lasted two minutes or more.
STAND_IN_DATE = r"""#!/usr/bin/env bash
if [ "${1:-}" = "+%s" ]; then
    now=$(( $(cat "$STAND_IN_CLOCK" 2>/dev/null || printf 0) + 1000 ))
    printf '%s\n' "$now" > "$STAND_IN_CLOCK"
    printf '%s\n' "$now"
    exit 0
fi
exec REAL_DATE "$@"
"""

STAND_IN_TRAINER = '''
"""Run the trainer as `python -m cachette.learn` does, up to the seed search."""

import os
import sys
from pathlib import Path

from cachette.learn import __main__ as trainer

# The status a fault in native code leaves, which is the case a restart is for.
FAULT_STATUS = 139


class ReachedTheSeedSearch(Exception):
    """The training run passed every check of its arguments."""


def last_value(option: str) -> str:
    """Return the last value the arguments give one option, or nothing."""
    words = sys.argv[1:]
    found = ""
    for index, word in enumerate(words[:-1]):
        if word == option:
            found = words[index + 1]
    return found


def stop_at_the_seed_search(*arguments: object, **keywords: object) -> list[int]:
    """Stop the run where it would begin to build worlds.

    A caller that names a fault file fails the first run that reaches here, as
    a fault in native code fails a long run. When the caller asks, it leaves
    the resume point of the strategy first, as a run that finished a
    generation does.
    """
    fault = os.environ.get("STAND_IN_FAULT_ONCE", "")
    if fault and not Path(fault).exists():
        Path(fault).touch()
        if os.environ.get("STAND_IN_LEAVE_RESUME_POINT") == "1":
            out = Path(last_value("--out"))
            out.mkdir(parents=True, exist_ok=True)
            (out / f"{last_value('--only')}-latest.npz").touch()
        raise SystemExit(FAULT_STATUS)
    raise ReachedTheSeedSearch


trainer.viable_seeds = stop_at_the_seed_search
sys.argv = ["cachette.learn", *sys.argv[1:]]
try:
    status = trainer.main()
except ReachedTheSeedSearch:
    status = 0
raise SystemExit(status)
'''

QUIET_TOOL = "#!/usr/bin/env bash\nexit 0\n"
CORE_COUNT_TOOL = "#!/usr/bin/env bash\nprintf '64\\n'\n"


def write_start_file(
    path: Path,
    world: EnvConfig,
    kind: str,
    *,
    readout_only: bool = False,
    normalizer: FeatureNormalizer | None = None,
) -> Path:
    """Write the centre of a run that another run may start from, and return it.

    The centre is the shell of the kind, and the real checkpoint writes it, so
    the file holds every entry a run writes. **The file states no normalizer
    unless a caller gives one.** The checks before the seed search hold none,
    because the trainer derives the normalizer from played episodes.
    """
    probe = Env(world, Weighting(terms={}, won=0.0, lost=0.0, drawn=0.0))
    checkpoint = Checkpoint(
        name="wonder",
        out_dir=path.parent,
        env_config=world,
        probe=probe,
        kind=kind,
        normalizer=normalizer,
        readout_only=readout_only,
    )
    path.parent.mkdir(parents=True, exist_ok=True)
    checkpoint.write(
        shell_policy(kind, probe, normalizer), path, START_GENERATION, 0.0, None, None
    )
    return path


def _tool(directory: Path, name: str, text: str) -> None:
    """Write one stand-in tool and make it executable."""
    path = directory / name
    path.write_text(text, encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IXUSR)


def _stand_ins(scratch: Path, clock: bool = False) -> dict[str, str]:
    """Put the stand-ins on a path, and return the environment that reaches them.

    The clock entry adds a clock that moves a thousand seconds at each reading,
    and a sleep that returns at once. A test of a restart needs both.
    """
    tools = scratch / "bin"
    tools.mkdir()
    real_date = shutil.which("date")
    _tool(tools, "uv", STAND_IN_UV)
    _tool(tools, "aws", STAND_IN_AWS)
    _tool(tools, "sudo", QUIET_TOOL)
    _tool(tools, "tar", QUIET_TOOL)
    _tool(tools, "curl", QUIET_TOOL)
    _tool(tools, "nproc", CORE_COUNT_TOOL)
    if clock and real_date is not None:
        _tool(tools, "date", STAND_IN_DATE.replace("REAL_DATE", real_date))
        _tool(tools, "sleep", QUIET_TOOL)
    trainer = scratch / "trainer.py"
    trainer.write_text(STAND_IN_TRAINER, encoding="utf-8")
    home = scratch / "home"
    (home / "wheelhouse").mkdir(parents=True)
    (home / "wheelhouse" / "cachette-0.0.0-py3-none-any.whl").touch()
    return {
        **os.environ,
        "PATH": f"{tools}{os.pathsep}{os.environ['PATH']}",
        "HOME": str(home),
        "STAND_IN_PYTHON": sys.executable,
        "STAND_IN_TRAINER": str(trainer),
        "STAND_IN_CALLS": str(scratch / "calls.tsv"),
        "STAND_IN_CLOCK": str(scratch / "clock"),
    }


def _calls(scratch: Path, side: str) -> list[tuple[int, str]]:
    """Return the status and the arguments of each trainer call from one side."""
    log = scratch / "calls.tsv"
    if not log.exists():
        return []
    rows = [
        line.split("\t", 2) for line in log.read_text(encoding="utf-8").splitlines()
    ]
    return [
        (int(status), arguments) for where, status, arguments in rows if where == side
    ]


def _path_of(arguments: str) -> str:
    """Name the path of the trainer that one call selects."""
    words = arguments.split()
    return next((flag for flag in PATH_FLAGS if flag in words), "training")


def _value_of(arguments: str, option: str) -> str:
    """Return the value one call gives an option, or nothing."""
    words = arguments.split()
    if option not in words[:-1]:
        return ""
    return words[words.index(option) + 1]


def _failures(calls: list[tuple[int, str]]) -> list[str]:
    """Return each call that did not succeed, with its status."""
    return [f"exit {status}: {arguments}" for status, arguments in calls if status]


def instance_script() -> str:
    """Return the script the launcher writes for the instance, from the launcher."""
    text = LAUNCHER.read_text(encoding="utf-8")
    start = text.index(INSTANCE_OPENING) + len(INSTANCE_OPENING)
    return text[start : text.index(INSTANCE_CLOSING, start)]


def run_the_dry_path(
    scratch: Path, arguments: str, start_from: Path | None = None
) -> subprocess.CompletedProcess[str]:
    """Run the dry path of the launcher, which asks the trainer before it rents.

    The start entry sets the start file of the run, as a person sets it.
    """
    environment = _stand_ins(scratch)
    environment.update(
        CACHETTE_TRAIN_ARGS=arguments,
        CACHETTE_TRAIN_OUT=str(scratch / "out"),
        STAND_IN_SIDE="local",
    )
    environment.pop("CACHETTE_TRAIN_START_FROM", None)
    if start_from is not None:
        environment["CACHETTE_TRAIN_START_FROM"] = str(start_from)
    return subprocess.run(
        ["bash", str(LAUNCHER), "--dry-run"],
        capture_output=True,
        text=True,
        check=False,
        env=environment,
    )


def run_the_instance_script(
    scratch: Path,
    arguments: str,
    start_name: str = "",
    fault: bool = False,
    leave_resume_point: bool = False,
) -> subprocess.CompletedProcess[str]:
    """Run the instance script of the launcher in a home of its own.

    **The script writes its marker and its probe table under `/tmp`.** Two
    runs of this test at once would then share those files, so the text moves
    them into the scratch directory. Nothing else in the script changes.

    The start entry names a start file. The launcher copies such a file into
    the home of the instance, and this writes one there that fits a run of one
    box. The fault entry fails the first training attempt, and the resume
    entry makes that attempt leave a resume point first.
    """
    environment = _stand_ins(scratch, clock=fault)
    environment.update(
        TRAIN_ARGS=arguments,
        PRICE="0.5",
        PROBE_WORLDS="144",
        WALL_MINUTES="360",
        CACHETTE_ENGINE_KEY="",
        PROBE_ONLY="0",
        STAND_IN_SIDE="instance",
    )
    if start_name:
        write_start_file(
            Path(environment["HOME"]) / "start-from" / start_name,
            ONE_BOX_WORLD,
            STRUCTURED_KIND,
            readout_only=True,
        )
        environment["START_FROM_NAME"] = start_name
    if fault:
        environment["STAND_IN_FAULT_ONCE"] = str(scratch / "fault")
    if leave_resume_point:
        environment["STAND_IN_LEAVE_RESUME_POINT"] = "1"
    script = scratch / "remote.sh"
    script.write_text(
        instance_script().replace("/tmp/", f"{scratch}/"), encoding="utf-8"
    )
    return subprocess.run(
        ["bash", str(script)],
        capture_output=True,
        text=True,
        check=False,
        env=environment,
        cwd=environment["HOME"],
    )


def test_every_trainer_call_before_the_rental_accepts_a_one_box_run(
    tmp_path: Path,
) -> None:
    """The launcher asks for the world and the plan before it rents, and both answer."""
    finished = run_the_dry_path(tmp_path, ONE_BOX_ARGUMENTS)
    calls = _calls(tmp_path, "local")
    assert not _failures(calls), "\n".join(_failures(calls))
    assert finished.returncode == 0, finished.stderr[-2000:]
    assert {_path_of(arguments) for _, arguments in calls} >= {
        "--print-world",
        "--print-plan",
    }, f"the dry path made these trainer calls: {calls}"
    assert all("--start-from" not in arguments.split() for _, arguments in calls)


def test_every_trainer_call_on_the_instance_accepts_a_one_box_run(
    tmp_path: Path,
) -> None:
    """Each call of the instance script answers, after the script removes `--only`.

    **The case the defect needs is a call without `--only`.** The assertion
    below proves that the script made one, so a green result cannot come
    from a script that no longer removes the argument.
    """
    finished = run_the_instance_script(tmp_path, ONE_BOX_ARGUMENTS)
    calls = _calls(tmp_path, "instance")
    assert not _failures(calls), "\n".join(_failures(calls))
    assert finished.returncode == 0, finished.stdout[-2000:] + finished.stderr[-2000:]
    assert {_path_of(arguments) for _, arguments in calls} >= {
        "--print-strategies",
        "--print-plan",
        "--print-world",
        "training",
    }, f"the instance script made these trainer calls: {calls}"
    assert any("--only" not in arguments.split() for _, arguments in calls), (
        "no trainer call on the instance lacked --only, so this test did not "
        "reach the call that the removal of --only changes"
    )
    training = [
        arguments for _, arguments in calls if _path_of(arguments) == "training"
    ]
    assert all("--only wealth-structured" in arguments for arguments in training)
    assert all("--start-from" not in arguments.split() for _, arguments in calls)


@pytest.mark.parametrize(
    "path",
    [(), ("--baseline-only",), ("--print-plan",), ("--print-strategies",)],
    ids=["training", "baseline", "plan", "strategies"],
)
def test_a_readout_run_that_names_no_strategy_still_refuses(
    tmp_path: Path, path: tuple[str, ...]
) -> None:
    """A run with no `--only` names every row, and the linear rows refuse the flag.

    A path that trains, or that answers for a training run, must refuse. The
    refusal comes before anything plays, so the run writes nothing.

    **The call goes through the stand-in trainer.** A run that does not refuse
    then stops at its seed search and fails here. Through the real trainer it
    played a whole run, and the test did not end.
    """
    trainer = tmp_path / "trainer.py"
    trainer.write_text(STAND_IN_TRAINER, encoding="utf-8")
    out = tmp_path / "out"
    finished = subprocess.run(
        [
            sys.executable,
            str(trainer),
            *path,
            "--train-readout-only",
            "--generations",
            "1",
            "--out",
            str(out),
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    assert finished.returncode == 2, finished.stderr[-2000:]
    assert "cannot train its readout alone" in finished.stderr
    assert not out.exists()


def test_the_world_of_a_readout_run_answers_without_a_strategy() -> None:
    """The world call trains nothing, so the readout flag does not refuse it."""
    finished = subprocess.run(
        [
            sys.executable,
            "-m",
            "cachette.learn",
            "--print-world",
            "--train-readout-only",
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    assert finished.returncode == 0, finished.stderr[-2000:]
    assert "width\t" in finished.stdout


def test_the_dry_path_asks_the_plan_with_the_start_file_and_names_it(
    tmp_path: Path,
) -> None:
    """The plan call before the rental carries the file, and the preview names it.

    **The plan call is where the trainer refuses a file before the rental.** A
    launcher that left the file out of that call would let a file of another
    world reach a paid instance.
    """
    start = write_start_file(
        tmp_path / START_NAME, ONE_BOX_WORLD, STRUCTURED_KIND, readout_only=True
    )
    finished = run_the_dry_path(tmp_path, ONE_BOX_ARGUMENTS, start)
    calls = _calls(tmp_path, "local")
    assert not _failures(calls), "\n".join(_failures(calls))
    assert finished.returncode == 0, finished.stderr[-2000:]
    plans = [
        arguments for _, arguments in calls if _path_of(arguments) == "--print-plan"
    ]
    assert plans, f"the dry path made these trainer calls: {calls}"
    for arguments in plans:
        assert Path(_value_of(arguments, "--start-from")).resolve() == start.resolve()
    assert f"start from    {start}" in finished.stderr or (
        f"start from    {start.resolve()}" in finished.stderr
    ), finished.stderr[-2000:]


def test_the_dry_path_refuses_a_start_file_that_is_not_there(tmp_path: Path) -> None:
    """A missing file ends the launcher before it asks the trainer anything."""
    finished = run_the_dry_path(
        tmp_path, ONE_BOX_ARGUMENTS, tmp_path / "missing-latest.npz"
    )
    assert finished.returncode != 0
    assert "no file is there" in finished.stderr
    assert _calls(tmp_path, "local") == []


@pytest.mark.parametrize(
    "extra",
    ["--resume", "--res", "--start-from /elsewhere.npz", "--start=/elsewhere.npz"],
)
def test_the_dry_path_refuses_run_arguments_that_reach_every_attempt(
    tmp_path: Path, extra: str
) -> None:
    """A start or a resume in the run arguments would reach a restart as well.

    The trainer reads an abbreviation of an option as the option, so the
    shortened forms are refused as well.
    """
    start = write_start_file(
        tmp_path / START_NAME, ONE_BOX_WORLD, STRUCTURED_KIND, readout_only=True
    )
    finished = run_the_dry_path(tmp_path, f"{ONE_BOX_ARGUMENTS} {extra}", start)
    assert finished.returncode != 0
    assert "CACHETTE_TRAIN_START_FROM is set" in finished.stderr
    assert _calls(tmp_path, "local") == []


def test_the_dry_path_refuses_a_start_for_every_strategy(tmp_path: Path) -> None:
    """A run that names no strategy trains every row, and one centre starts one.

    **The instance names the strategies of the training call itself**, from
    the run arguments. A run with no `--only` would therefore reach the
    instance with every row. The trainer refuses it in the plan call, before
    the rental, and its reason reaches the person.
    """
    start = write_start_file(tmp_path / "wonder-latest.npz", ONE_BOX_WORLD, "linear")
    finished = run_the_dry_path(
        tmp_path, "--world-extent 48 --tick-limit 2500 --limit-is-loss", start
    )
    assert finished.returncode != 0
    assert "one centre to one strategy" in finished.stderr, finished.stderr[-2000:]
    assert "nothing has been created yet" not in finished.stderr
    plans = [
        status
        for status, arguments in _calls(tmp_path, "local")
        if _path_of(arguments) == "--print-plan"
    ]
    assert plans == [2]


def test_the_first_attempt_starts_from_the_file_and_a_restart_resumes(
    tmp_path: Path,
) -> None:
    """The file reaches the first attempt, and a restart takes the resume point.

    **A restart that started from the file again would throw away every
    generation the first attempt trained**, because the resume point then
    holds a newer centre than the file.

    The first attempt passes every check of the real trainer with the file,
    beside the readout flag and after the removal of `--only`, and then fails
    as a fault does.
    """
    finished = run_the_instance_script(
        tmp_path,
        ONE_BOX_ARGUMENTS,
        start_name=START_NAME,
        fault=True,
        leave_resume_point=True,
    )
    calls = _calls(tmp_path, "instance")
    training = [
        (status, arguments)
        for status, arguments in calls
        if _path_of(arguments) == "training"
    ]
    assert len(training) == 2, f"the instance made these training calls: {training}"
    (first_status, first), (second_status, second) = training
    assert first_status == FAULT_STATUS, first
    assert _value_of(first, "--start-from") == str(
        tmp_path / "home" / "start-from" / START_NAME
    )
    assert "--resume" not in first.split()
    assert "--only wealth-structured" in first
    assert second_status == 0, second
    assert "--resume" in second.split()
    assert "--start-from" not in second.split()
    assert finished.returncode == 0, finished.stdout[-2000:] + finished.stderr[-2000:]


def test_a_restart_before_the_first_resume_point_starts_from_the_file_again(
    tmp_path: Path,
) -> None:
    """A resume that finds no resume point starts seeded, so the file goes again.

    **A started run must never fall back to a seeded draw.** A fault before
    the first generation ends leaves no resume point, and a restart with the
    resume flag alone would then train from the seeded draw without a word.
    """
    finished = run_the_instance_script(
        tmp_path, ONE_BOX_ARGUMENTS, start_name=START_NAME, fault=True
    )
    training = [
        arguments
        for _, arguments in _calls(tmp_path, "instance")
        if _path_of(arguments) == "training"
    ]
    assert len(training) == 2, f"the instance made these training calls: {training}"
    assert all("--start-from" in arguments.split() for arguments in training)
    assert all("--resume" not in arguments.split() for arguments in training)
    assert finished.returncode == 0, finished.stdout[-2000:] + finished.stderr[-2000:]
