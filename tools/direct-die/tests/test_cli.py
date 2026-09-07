"""Tests for the command line of the tool.

The front end calls the tool this way and no other way, so the exit codes and
the argument names are a contract.
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from direct_die import analyse, cli  # noqa: E402


def test_the_analyse_command_calls_the_runner(monkeypatch, capsys):
    seen = {}

    def fake_run(asset, session_id, index, **rest):
        seen["asset"] = asset
        seen["session"] = session_id
        seen["round"] = index
        return {"preference": "flat fills read better", "order": ["d", "b"]}

    monkeypatch.setattr(analyse, "run", fake_run)
    code = cli.main(["analyse", "--asset", "cartoon", "--session", "s1", "--round", "2"])
    assert code == 0
    assert seen == {"asset": "cartoon", "session": "s1", "round": 2}
    assert "flat fills read better" in capsys.readouterr().out


def test_the_analyse_command_reports_an_analysis_error(monkeypatch, capsys):
    def fake_run(*args, **rest):
        raise analyse.AnalysisError("the analysis needs at least one liked drawing")

    monkeypatch.setattr(analyse, "run", fake_run)
    code = cli.main(["analyse", "--asset", "cartoon", "--session", "s1", "--round", "0"])
    assert code == 4
    assert "at least one liked drawing" in capsys.readouterr().err


def test_the_analyse_command_needs_a_session_and_a_round():
    with pytest.raises(SystemExit):
        cli.main(["analyse", "--asset", "cartoon"])
