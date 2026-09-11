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

References
----------
[^1]: Testing Rules, drive the real caller. ``.agents/rules/testing.md``
[^2]: The training launcher. ``scripts/graviton-train.sh``
"""

from __future__ import annotations

import os
import stat
import subprocess
import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]
LAUNCHER = ROOT / "scripts" / "graviton-train.sh"

# The arguments of the run that failed on the instance. It trains the readout
# alone of one structured strategy, and every other row of the table is linear.
ONE_BOX_ARGUMENTS = (
    "--only wealth-structured --world-extent 48 --tick-limit 2500 "
    "--population 128 --seeds 12 --validation 24 --train-readout-only "
    "--limit-is-loss --generations 20"
)

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

STAND_IN_TRAINER = '''
"""Run the trainer as `python -m cachette.learn` does, up to the seed search."""

import sys

from cachette.learn import __main__ as trainer


class ReachedTheSeedSearch(Exception):
    """The training run passed every check of its arguments."""


def stop_at_the_seed_search(*arguments: object, **keywords: object) -> list[int]:
    """Stop the run where it would begin to build worlds."""
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


def _tool(directory: Path, name: str, text: str) -> None:
    """Write one stand-in tool and make it executable."""
    path = directory / name
    path.write_text(text, encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IXUSR)


def _stand_ins(scratch: Path) -> dict[str, str]:
    """Put the stand-ins on a path, and return the environment that reaches them."""
    tools = scratch / "bin"
    tools.mkdir()
    _tool(tools, "uv", STAND_IN_UV)
    _tool(tools, "aws", STAND_IN_AWS)
    _tool(tools, "sudo", QUIET_TOOL)
    _tool(tools, "tar", QUIET_TOOL)
    _tool(tools, "curl", QUIET_TOOL)
    _tool(tools, "nproc", CORE_COUNT_TOOL)
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


def _failures(calls: list[tuple[int, str]]) -> list[str]:
    """Return each call that did not succeed, with its status."""
    return [f"exit {status}: {arguments}" for status, arguments in calls if status]


def instance_script() -> str:
    """Return the script the launcher writes for the instance, from the launcher."""
    text = LAUNCHER.read_text(encoding="utf-8")
    start = text.index(INSTANCE_OPENING) + len(INSTANCE_OPENING)
    return text[start : text.index(INSTANCE_CLOSING, start)]


def run_the_dry_path(scratch: Path, arguments: str) -> subprocess.CompletedProcess[str]:
    """Run the dry path of the launcher, which asks the trainer before it rents."""
    environment = _stand_ins(scratch)
    environment.update(
        CACHETTE_TRAIN_ARGS=arguments,
        CACHETTE_TRAIN_OUT=str(scratch / "out"),
        STAND_IN_SIDE="local",
    )
    return subprocess.run(
        ["bash", str(LAUNCHER), "--dry-run"],
        capture_output=True,
        text=True,
        check=False,
        env=environment,
    )


def run_the_instance_script(
    scratch: Path, arguments: str
) -> subprocess.CompletedProcess[str]:
    """Run the instance script of the launcher in a home of its own.

    **The script writes its marker and its probe table under `/tmp`.** Two
    runs of this test at once would then share those files, so the text moves
    them into the scratch directory. Nothing else in the script changes.
    """
    environment = _stand_ins(scratch)
    environment.update(
        TRAIN_ARGS=arguments,
        PRICE="0.5",
        PROBE_WORLDS="144",
        WALL_MINUTES="360",
        CACHETTE_ENGINE_KEY="",
        PROBE_ONLY="0",
        STAND_IN_SIDE="instance",
    )
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
