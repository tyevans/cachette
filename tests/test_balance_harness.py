"""The balance harness runs a seed set to game end and reports on the set.

Every test drives the harness through its command line entry point, on a
small world with a short tick limit, so the tests are about the harness and
not about the game. No test asserts on wall time.

References
----------
Testing Rules, sections 1, 3 and 5. ``.agents/rules/testing.md``

Balance register. ``docs/reference/balance.md``
"""

from __future__ import annotations

import json
import pathlib
import re

import pytest

from cachette.balance import (
    END_ROW,
    SEAT_ROW,
    SEED_SET_ROW,
    SHARE_ROWS,
    WIN_PATH_ROW,
    RegisterError,
    default_seeds,
    main,
    parse_register,
    read_register,
)

REPOSITORY = pathlib.Path(__file__).resolve().parent.parent
REGISTER = REPOSITORY / "docs" / "reference" / "balance.md"

# A world small enough that two games cost less than the test collection.
SMALL = ["--extent", "32", "--factions", "3", "--tick-limit", "12"]
SEEDS = ["--seeds", "1", "2"]

REPORT_KEYS = {
    "extent",
    "faction_count",
    "tick_limit",
    "seeds",
    "thresholds",
    "games",
    "path_shares",
    "seat_shares",
    "reached_tick_limit",
    "zero_in_every_game",
    "statements",
}
GAME_KEYS = {"seed", "winner", "path", "tick", "reached_tick_limit", "scores", "census"}


def run(
    tmp_path: pathlib.Path, *extra: str, name: str = "out.json"
) -> tuple[int, dict[str, object]]:
    """Run the harness against the real register and read its JSON back."""
    out = tmp_path / name
    code = main([*SMALL, *SEEDS, "--table", str(REGISTER), "--json", str(out), *extra])
    return code, json.loads(out.read_text(encoding="utf-8"))


def set_row(text: str, row: str, value: str) -> str:
    """Return the register text with one share row's Set cell replaced."""
    pattern = re.compile(
        rf"^(\| {re.escape(row)}[^|]*\| [^|]*\| )([^|]*)(\|.*)$", re.MULTILINE
    )
    replaced, count = pattern.subn(rf"\g<1>{value} \g<3>", text)
    assert count == 1, f"the row {row!r} was not found once"
    return replaced


def test_the_parser_reads_the_four_share_rows_of_the_real_register() -> None:
    """The real register is the one declaration site, and the parser finds every row."""
    thresholds = read_register(REGISTER)
    assert set(thresholds.as_dict()) == set(SHARE_ROWS)
    # Every share is unset while the rules of the downstream game are open.
    assert all(value is None for value in thresholds.as_dict().values())


def test_the_parser_rejects_a_register_without_the_rows() -> None:
    """A register that lacks a row is an error, not a silent pass."""
    with pytest.raises(RegisterError):
        parse_register("# Nothing here\n")
    text = REGISTER.read_text(encoding="utf-8").replace(
        "| Seat share,", "| Chair share,"
    )
    with pytest.raises(RegisterError, match="Seat share"):
        parse_register(text)


def test_the_parser_reads_a_set_percent() -> None:
    """A Set cell that holds an integer percent is read as that percent."""
    text = set_row(REGISTER.read_text(encoding="utf-8"), WIN_PATH_ROW, "60")
    assert parse_register(text).win_path == 60
    assert parse_register(set_row(text, END_ROW, "75%")).end == 75


def test_the_default_seed_set_is_derived_from_the_base_seed() -> None:
    """The set is fixed, its members are distinct, and each fits in 64 bits."""
    seeds = default_seeds()
    assert seeds == default_seeds()
    assert len(set(seeds)) == len(seeds)
    assert all(0 <= seed < 1 << 64 for seed in seeds)


def test_the_harness_runs_to_completion_and_writes_the_report(
    tmp_path: pathlib.Path, capsys: pytest.CaptureFixture[str]
) -> None:
    """One run writes the JSON with every required key, and prints the table."""
    code, report = run(tmp_path)
    assert code == 0
    assert set(report) == REPORT_KEYS
    games = report["games"]
    assert isinstance(games, list)
    assert [game["seed"] for game in games] == [1, 2]
    for game in games:
        assert set(game) == GAME_KEYS
        assert game["path"] == "territory"
        assert game["tick"] == 12
        assert len(game["scores"]) == 3
        assert game["census"]["game_ended"] == 1
    statements = report["statements"]
    assert isinstance(statements, list)
    assert [statement["row"] for statement in statements] == list(SHARE_ROWS)
    out = capsys.readouterr().out
    assert "statement 1, Win-path share (unset): unset: reporting only" in out
    assert "statement 4, Seed set (unset): unset: reporting only" in out
    assert "territory 2/2" in out


def test_the_unset_register_reports_only_and_exits_zero(tmp_path: pathlib.Path) -> None:
    """While every share is unset, no statement can fail."""
    code, report = run(tmp_path)
    assert code == 0
    statements = report["statements"]
    assert isinstance(statements, list)
    assert all(
        statement["verdict"] == "unset: reporting only" for statement in statements
    )
    assert all(statement["failing_seeds"] == [] for statement in statements)


def test_the_same_seed_set_gives_the_same_json_at_two_thread_counts(
    tmp_path: pathlib.Path,
) -> None:
    """The report holds nothing about the thread count, and the game does not either."""
    run(tmp_path, "--threads", "1", name="one.json")
    run(tmp_path, "--threads", "2", name="two.json")
    one = (tmp_path / "one.json").read_bytes()
    two = (tmp_path / "two.json").read_bytes()
    assert one == two
    # The comparison can fail: a different seed set gives a different report.
    out = tmp_path / "other.json"
    main([*SMALL, "--seeds", "1", "3", "--table", str(REGISTER), "--json", str(out)])
    assert out.read_bytes() != one


def test_a_set_threshold_the_run_misses_exits_nonzero(
    tmp_path: pathlib.Path, capsys: pytest.CaptureFixture[str]
) -> None:
    """With the seat share below what a seat wins, statement 2 fails and names seeds."""
    table = tmp_path / "balance.md"
    # Two seeds and one path: every seat share and the path share are 0/2, 1/2
    # or 2/2, so a share of 49 percent is missed whenever one seat wins both,
    # and 40 percent is missed whenever any seat wins at all.
    table.write_text(
        set_row(REGISTER.read_text(encoding="utf-8"), SEAT_ROW, "40"), encoding="utf-8"
    )
    out = tmp_path / "out.json"
    code = main([*SMALL, *SEEDS, "--table", str(table), "--json", str(out)])
    assert code == 1
    report = json.loads(out.read_text(encoding="utf-8"))
    seat = report["statements"][1]
    assert seat["row"] == SEAT_ROW
    assert seat["threshold"] == 40
    assert seat["verdict"] == "fail"
    assert seat["failing_seeds"]
    assert set(seat["failing_seeds"]) <= {1, 2}
    text = capsys.readouterr().out
    assert "statement 2, Seat share (40%): fail" in text
    assert "failing seeds:" in text


def test_a_set_threshold_the_run_meets_exits_zero(tmp_path: pathlib.Path) -> None:
    """With the path share set at the whole set, the one path passes."""
    table = tmp_path / "balance.md"
    table.write_text(
        set_row(REGISTER.read_text(encoding="utf-8"), WIN_PATH_ROW, "100"),
        encoding="utf-8",
    )
    out = tmp_path / "out.json"
    code = main([*SMALL, *SEEDS, "--table", str(table), "--json", str(out)])
    assert code == 0
    report = json.loads(out.read_text(encoding="utf-8"))
    assert report["statements"][0]["verdict"] == "pass"


def test_the_end_share_fails_when_every_game_reaches_the_limit(
    tmp_path: pathlib.Path,
) -> None:
    """The territory path fires at the tick limit, so a floor above zero is missed."""
    table = tmp_path / "balance.md"
    table.write_text(
        set_row(REGISTER.read_text(encoding="utf-8"), END_ROW, "1"), encoding="utf-8"
    )
    out = tmp_path / "out.json"
    code = main([*SMALL, *SEEDS, "--table", str(table), "--json", str(out)])
    assert code == 1
    report = json.loads(out.read_text(encoding="utf-8"))
    end = report["statements"][2]
    assert end["verdict"] == "fail"
    assert end["failing_seeds"] == [1, 2]
    assert report["reached_tick_limit"] == {"won": 2, "of": 2}


def test_the_seed_set_row_turns_the_census_statement_into_a_check(
    tmp_path: pathlib.Path,
) -> None:
    """A short game leaves census rows at zero, and a set row makes that a failure."""
    table = tmp_path / "balance.md"
    table.write_text(
        set_row(REGISTER.read_text(encoding="utf-8"), SEED_SET_ROW, "2"),
        encoding="utf-8",
    )
    out = tmp_path / "out.json"
    code = main([*SMALL, *SEEDS, "--table", str(table), "--json", str(out)])
    report = json.loads(out.read_text(encoding="utf-8"))
    zero = report["statements"][3]
    assert zero["row"] == SEED_SET_ROW
    # The controller emits nothing after the game end, so its command count
    # is zero at the end of every game, and the list is never empty.
    assert "controller_commands" in report["zero_in_every_game"]
    assert code == 1
    assert zero["verdict"] == "fail"
    assert zero["failing_seeds"] == [1, 2]
