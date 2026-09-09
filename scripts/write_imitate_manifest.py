"""Write the manifest beside each imitation checkpoint.

A weight file that does not say what it fits is a file nobody can use later.
This script reads the report of one imitation run and writes one manifest for
each policy shape it fitted.

**The decision interval is in the manifest on purpose.** A policy asked at
another cadence acts a different number of times over a run, so it behaves
differently. A reader that plays a checkpoint takes the interval from here.
"""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

from cachette._core import World
from cachette.learn.__main__ import WORLD

REPORT = Path(sys.argv[1])
ROOT = Path(sys.argv[2])
RUN = sys.argv[3]



def _probe() -> World:
    """Build the smallest world of the trained shape, to read its declarations."""
    return World(
        width=WORLD.width,
        height=WORLD.height,
        seed=1,
        faction_count=WORLD.faction_count,
    )


def _schema() -> dict:
    """Return the observation schema the engine publishes for that shape."""
    return _probe().observation_schema()


def _action_version() -> int:
    """Return the action schema version the engine publishes."""
    return int(_probe().action_schema()["version"])


def main() -> int:
    """Write one manifest for each fitted policy shape."""
    report = json.loads(REPORT.read_text())
    for kind in ("linear",):
        name = f"imitate-obs3-{kind}.npz"
        path = ROOT / name
        fit = report["fits"][kind]
        play = report["play"][kind]
        manifest = {
            "file": name,
            "run": RUN,
            "strategy": "conquer",
            "source": "supervised fit to built-in controller play",
            "record_seeds": len(report["record_seeds"]),
            "fit_holdout_seeds": len(report["fit_holdout_seeds"]),
            "windows": report["windows"],
            "commands": report["commands"],
            "holdout_command_accuracy": fit["holdout"]["command_accuracy"],
            "holdout_window_accuracy": fit["holdout"]["window_accuracy"],
            "constant_command_accuracy": report["baseline"][
                "constant_command_accuracy"
            ],
            "constant_window_accuracy": report["baseline"]["constant_window_accuracy"],
            "play_seeds": len(report["play_seeds"]),
            "play_return": play["return"],
            "play_won": play["won"],
            "controller_return": report["play"]["controller"]["return"],
            "controller_won": report["play"]["controller"]["won"],
            "beats_controller": False,
            "observation_version": _schema()["version"],
            "action_version": _action_version(),
            "decision_interval": WORLD.decision_interval,
            "tick_limit": WORLD.tick_limit,
            "width": WORLD.width,
            "height": WORLD.height,
            "faction_count": WORLD.faction_count,
            "policy_kind": kind,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        }
        out = ROOT / f"imitate-obs3-{kind}.json"
        out.write_text(json.dumps(manifest, indent=2) + "\n")
        print(out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
