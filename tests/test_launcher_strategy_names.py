"""The launcher asks the trainer which strategies a run trains.

A training run launches one trainer process for each strategy. The launcher
must know the names before it starts them, and **the trainer is the only
thing that knows them**. The table of built-in weightings is one answer. A
run that names play styles replaces that table, and a run that names a
variation other than the fixed one replaces it with a single combined
strategy whose name no caller can predict.

The launcher held two answers of its own. One counted the rows of the
trainer's source with a regular expression. One imported the table at module
scope, which is the table before the styles replace it. Both were right for a
built-in run and wrong for a style run, and nothing failed until a paid
instance launched one process for each built-in name, every one of them
raised a lookup error on its first name, and the run was torn down without a
generation.

This holds the trainer as the one declaration site.[^1]

References
----------
[^1]: Recurring defect shapes, redundant declaration sites with undocumented
precedence. ``.agents/rules/recurring-defects.md``
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LAUNCHER = ROOT / "scripts" / "graviton-train.sh"


def _names(*arguments: str) -> list[str]:
    """Run the trainer's name flag and return what it printed."""
    finished = subprocess.run(
        [sys.executable, "-m", "cachette.learn", "--print-strategies", *arguments],
        capture_output=True,
        text=True,
        check=True,
        cwd=ROOT,
    )
    return finished.stdout.split()


def test_the_flag_answers_the_built_in_table() -> None:
    """A run that names nothing trains every built-in weighting."""
    names = _names()
    assert names, "the trainer named no strategy for a run with no arguments"
    assert "conquer" in names


def test_the_flag_answers_the_named_styles() -> None:
    """A style run trains the styles it names, and nothing else.

    This is the case the launcher's own answer could not reach, because the
    styles replace the table after the arguments are read.
    """
    chosen = ["aggressive", "wonder_rush"]
    names = _names("--styles", ",".join(chosen), "--style-kind", "structured")
    assert names == chosen, f"the trainer named {names} for the styles {chosen}"


def test_the_flag_answers_a_narrowed_run() -> None:
    """A run narrowed with the only argument trains what it names."""
    assert _names("--only", "land,conquer") == ["land", "conquer"]


def test_the_launcher_holds_no_strategy_list_of_its_own() -> None:
    """The launcher asks the trainer, and holds no second answer.

    A second answer here goes stale the first time the trainer gains a way to
    name a strategy, and nothing fails when it does. This names the two
    answers the launcher held, so neither can return without a reader.
    """
    script = LAUNCHER.read_text(encoding="utf-8")
    assert "--print-strategies" in script, "the launcher does not ask the trainer"
    assert "import STRATEGIES" not in script, (
        "the launcher reads the strategy table at import, which is the table "
        "before the play styles replace it"
    )
    assert '": ($' not in script, (
        "the launcher counts the rows of the trainer's source, which no "
        "longer says how many strategies a run trains"
    )
