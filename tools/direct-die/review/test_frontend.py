"""Tests for the direct-die front end.

The tests build a fixture workspace in a temporary directory, then drive the
server through its public interface with the test client. No test reaches
into a private field, and no test calls the vision model.

Four groups of tests live here. The first renders every page, including the
empty case and the part-way case. The second checks which drawing wins. The
third checks the promotion and the pack export against the files on disk. The
fourth starts a real child process and watches the pages follow it.

Run them with the test runner from this directory.[^1]

## References

[^1]: The tool guide. `tools/direct-die/review/README.md`
"""

from __future__ import annotations

import json
import sys
import time
from collections.abc import Callable
from pathlib import Path

import pytest
from fastapi.testclient import TestClient

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))
# The tool package sits beside this directory. Only the exemplar test imports
# it, to prove that the real guide loader reads a promoted drawing.
if str(HERE.parent) not in sys.path:
    sys.path.insert(0, str(HERE.parent))

import exemplars as exemplar_module  # noqa: E402
import packs as pack_module  # noqa: E402
import app as app_module  # noqa: E402
from app import create_app  # noqa: E402
from make_fixtures import build_workspace  # noqa: E402
from store import SessionStore, write_json_atomically  # noqa: E402

# The sessions of the fixture workspace that the front end tests read.
CHOSEN = "cartoon/20260904-090000-forest"
LIVE = "cartoon/20260904-091000-water"
UNNAMED = "cartoon/20260904-092000"
NO_ROUND = "pencil/20260904-093000-mountain"


@pytest.fixture
def paths(tmp_path: Path) -> dict[str, Path]:
    """Write the fixture workspace, and give its four roots."""
    return build_workspace(tmp_path / "workspace")


@pytest.fixture
def client(paths: dict[str, Path]) -> TestClient:
    """Give a test client that reads the fixture workspace."""
    return TestClient(
        create_app(
            paths["sessions"], paths["styleguide"], paths["packs"], paths["runs"]
        )
    )


# -- every page renders ------------------------------------------------------


@pytest.mark.parametrize(
    "path",
    [
        "/",
        "/sessions",
        "/runs",
        "/exemplars",
        "/packs/cartoon",
        "/packs/pencil",
        f"/s/{CHOSEN}",
        f"/s/{CHOSEN}/round-01",
        f"/s/{LIVE}/round-00",
        f"/s/{UNNAMED}/round-00",
        f"/s/{NO_ROUND}",
    ],
)
def test_every_page_renders(client: TestClient, path: str) -> None:
    response = client.get(path)
    assert response.status_code == 200
    assert "Traceback" not in response.text


def test_an_empty_workspace_renders_every_page(tmp_path: Path) -> None:
    empty = TestClient(
        create_app(
            tmp_path / "sessions",
            tmp_path / "styleguide",
            tmp_path / "packs",
            tmp_path / "runs",
        )
    )
    assert "names no style yet" in empty.get("/").text
    assert "No session is on disk yet" in empty.get("/sessions").text
    assert "No run has started yet" in empty.get("/runs").text
    assert "names no style yet" in empty.get("/exemplars").text


def test_the_grid_holds_a_cell_for_every_asset_and_style(client: TestClient) -> None:
    import slugs as slug_table

    body = client.get("/").text
    for slug in slug_table.SLUGS:
        assert slug in body
    for style in ("cartoon", "pencil"):
        assert style in body


def test_the_grid_names_a_cell_that_nothing_drew(client: TestClient) -> None:
    body = client.get("/").text
    assert "nothing" in body
    assert "no drawing" in body


def test_the_grid_marks_a_cell_that_a_person_chose(client: TestClient) -> None:
    assert "chosen" in client.get("/").text


def test_a_session_with_no_round_renders_as_a_gap(client: TestClient) -> None:
    assert "holds no round yet" in client.get(f"/s/{NO_ROUND}").text


def test_a_round_shows_a_drawing_before_its_critique(client: TestClient) -> None:
    # The loop writes the SVG, the renders and the critique at different
    # moments. The page must show the picture that is there already.
    body = client.get(f"/s/{LIVE}/round-00").text
    assert "variant-a.png" in body
    assert "The critique is not on disk yet." in body


def test_the_round_page_offers_a_state_address_to_poll(client: TestClient) -> None:
    answer = client.get(f"/api/round/{LIVE}/round-00").json()
    assert set(answer) == {"fingerprint", "running", "rounds"}
    assert answer["running"] is False


def test_the_state_address_changes_when_a_file_arrives(
    client: TestClient, paths: dict[str, Path]
) -> None:
    before = client.get(f"/api/round/{LIVE}/round-00").json()["fingerprint"]
    directory = paths["sessions"] / "cartoon" / "20260904-091000-water" / "round-00"
    (directory / "variant-b.svg").write_text("<svg/>", encoding="utf-8")
    after = client.get(f"/api/round/{LIVE}/round-00").json()["fingerprint"]
    assert before != after


# -- which drawing wins ------------------------------------------------------


def test_the_asset_name_comes_from_the_session_name(paths: dict[str, Path]) -> None:
    store = SessionStore(paths["sessions"])
    session = store.load_session("cartoon", "20260904-090000-forest")
    assert pack_module.slug_of(session) == "forest"


def test_the_asset_name_falls_back_to_the_subject(paths: dict[str, Path]) -> None:
    # A run from the command line names no asset in its session identifier.
    # The subject text then gives the name.
    store = SessionStore(paths["sessions"])
    session = store.load_session("cartoon", "20260904-092000")
    assert pack_module.slug_of(session) == "hill"


def test_a_person_outranks_the_score(paths: dict[str, Path]) -> None:
    store = SessionStore(paths["sessions"])
    pick = pack_module.pick_for(store, "cartoon", "forest")
    assert pick is not None
    assert pick.source == "human"
    assert pick.letter == "b"
    assert pick.round_name == "round-01"


def test_the_score_wins_when_no_person_chose(paths: dict[str, Path]) -> None:
    store = SessionStore(paths["sessions"])
    pick = pack_module.pick_for(store, "cartoon", "hill")
    assert pick is not None
    assert pick.source == "score"
    scores = []
    for entry in store.load_session("cartoon", "20260904-092000").rounds:
        scores += [item.score for item in entry.present_variants]
    assert pick.score == max(score for score in scores if score is not None)


def test_a_new_choice_moves_the_pick(
    client: TestClient, paths: dict[str, Path]
) -> None:
    # Put the defect back: a rule that reads only the first round, or only
    # the score, gives the old answer here.
    store = SessionStore(paths["sessions"])
    assert pack_module.pick_for(store, "cartoon", "forest").letter == "b"
    client.post(
        f"/s/{CHOSEN}/round-01/feedback",
        data={"mark-d": "like", "text": "d after all"},
        follow_redirects=False,
    )
    assert pack_module.pick_for(store, "cartoon", "forest").letter == "d"


def test_an_asset_nobody_drew_has_no_pick(paths: dict[str, Path]) -> None:
    store = SessionStore(paths["sessions"])
    assert pack_module.pick_for(store, "cartoon", "wonder") is None


# -- promotion ---------------------------------------------------------------


def test_a_style_starts_with_no_exemplar(paths: dict[str, Path]) -> None:
    assert exemplar_module.list_exemplars(paths["styleguide"], "cartoon") == []


def test_a_promotion_writes_the_exemplar_where_the_guide_reads_it(
    client: TestClient, paths: dict[str, Path]
) -> None:
    response = client.post(
        "/promote",
        data={
            "style": "cartoon",
            "session_id": "20260904-090000-forest",
            "round_name": "round-01",
            "letter": "b",
            "slug": "forest",
        },
        follow_redirects=False,
    )
    assert response.status_code == 303
    target = paths["styleguide"] / "exemplars" / "cartoon" / "forest.svg"
    assert target.is_file()
    source = (
        paths["sessions"]
        / "cartoon"
        / "20260904-090000-forest"
        / "round-01"
        / "variant-b.svg"
    )
    assert target.read_text() == source.read_text()


def test_the_next_run_reads_a_promoted_exemplar(
    client: TestClient, paths: dict[str, Path]
) -> None:
    # The proof that promotion is the bootstrap step: the real guide loader
    # of the tool must attach the promoted drawing to the next critique.
    from direct_die import guide as guide_module

    before = guide_module.load("cartoon", root=paths["styleguide"])
    assert before.exemplar_names == []
    client.post(
        "/promote",
        data={
            "style": "cartoon",
            "session_id": "20260904-090000-forest",
            "round_name": "round-01",
            "letter": "b",
            "slug": "forest",
        },
        follow_redirects=False,
    )
    after = guide_module.load("cartoon", root=paths["styleguide"])
    assert after.exemplar_names == ["forest.svg"]
    assert len(after.exemplars) == 1
    assert after.version != before.version


def test_a_promotion_shows_in_the_grid_and_in_the_exemplar_page(
    client: TestClient,
) -> None:
    client.post(
        "/promote",
        data={
            "style": "cartoon",
            "session_id": "20260904-090000-forest",
            "round_name": "round-01",
            "letter": "b",
            "slug": "forest",
        },
        follow_redirects=False,
    )
    assert "promoted" in client.get("/").text
    assert "forest.svg" in client.get("/exemplars").text


def test_a_withdrawal_removes_the_exemplar(
    client: TestClient, paths: dict[str, Path]
) -> None:
    client.post(
        "/promote",
        data={
            "style": "cartoon",
            "session_id": "20260904-090000-forest",
            "round_name": "round-01",
            "letter": "b",
            "slug": "forest",
        },
        follow_redirects=False,
    )
    client.post(
        "/exemplars/cartoon/withdraw",
        data={"name": "forest.svg"},
        follow_redirects=False,
    )
    assert not (paths["styleguide"] / "exemplars" / "cartoon" / "forest.svg").exists()


def test_a_promotion_of_a_drawing_that_is_absent_is_refused(
    client: TestClient,
) -> None:
    response = client.post(
        "/promote",
        data={
            "style": "cartoon",
            "session_id": "20260904-090000-forest",
            "round_name": "round-01",
            "letter": "z",
            "slug": "forest",
        },
        follow_redirects=False,
    )
    assert response.status_code == 200
    assert "nothing to show" in response.text


def test_an_exemplar_name_outside_the_contract_is_refused(client: TestClient) -> None:
    assert client.get("/e/cartoon/..%2F..%2Fcartoon.md").status_code in (400, 404)
    assert client.get("/e/cartoon/cartoon.md").status_code == 400


# -- the pack export ---------------------------------------------------------


def test_an_export_writes_the_layout_of_the_contract(
    client: TestClient, paths: dict[str, Path]
) -> None:
    response = client.post("/packs/cartoon/export", follow_redirects=False)
    assert response.status_code == 200
    assert "wrote 3 assets" in response.text
    assert "wonder: nothing is drawn for it yet" in response.text
    directory = paths["packs"] / "cartoon"
    manifest = json.loads((directory / "pack.json").read_text())
    assert manifest["style"] == "cartoon"
    assert manifest["created"].endswith("Z")
    assert set(manifest["assets"]) == {"forest", "water", "hill"}
    entry = manifest["assets"]["forest"]
    assert set(entry) == {"svg", "png", "score", "session", "round", "variant"}
    assert entry["svg"] == "forest.svg"
    assert entry["png"] == "forest.png"
    assert entry["variant"] == "b"
    assert entry["round"] == "round-01"
    for slug in manifest["assets"]:
        assert (directory / f"{slug}.svg").is_file()
        assert (directory / f"{slug}.png").is_file()


def test_an_export_names_no_asset_that_nothing_drew(
    client: TestClient, paths: dict[str, Path]
) -> None:
    client.post("/packs/cartoon/export", follow_redirects=False)
    manifest = json.loads((paths["packs"] / "cartoon" / "pack.json").read_text())
    assert "wonder" not in manifest["assets"]
    assert not (paths["packs"] / "cartoon" / "wonder.svg").exists()


def test_an_export_of_a_style_with_nothing_drawn_gives_an_empty_pack(
    client: TestClient, paths: dict[str, Path]
) -> None:
    client.post("/packs/pencil/export", follow_redirects=False)
    manifest = json.loads((paths["packs"] / "pencil" / "pack.json").read_text())
    assert manifest["assets"] == {}


def test_a_second_export_removes_a_drawing_that_left(
    client: TestClient, paths: dict[str, Path]
) -> None:
    client.post("/packs/cartoon/export", follow_redirects=False)
    assert (paths["packs"] / "cartoon" / "hill.svg").is_file()
    # Take the hill session away, as a person who deletes a bad run does.
    import shutil

    shutil.rmtree(paths["sessions"] / "cartoon" / "20260904-092000")
    client.post("/packs/cartoon/export", follow_redirects=False)
    manifest = json.loads((paths["packs"] / "cartoon" / "pack.json").read_text())
    assert "hill" not in manifest["assets"]
    assert not (paths["packs"] / "cartoon" / "hill.svg").exists()


def test_the_pack_page_shows_the_gaps(client: TestClient) -> None:
    client.post("/packs/cartoon/export", follow_redirects=False)
    body = client.get("/packs/cartoon").text
    assert "a gap" in body
    assert "3 of 11" in body


def test_a_pack_file_outside_the_contract_is_refused(client: TestClient) -> None:
    client.post("/packs/cartoon/export", follow_redirects=False)
    assert client.get("/p/cartoon/pack.json").status_code == 200
    assert client.get("/p/cartoon/forest.svg").status_code == 200
    assert client.get("/p/cartoon/secret.txt").status_code == 400


# -- starting a run ----------------------------------------------------------
#
# These tests start a real child process. They do not call the vision model.
# A stand-in script takes the same command line and writes the same files, so
# the run manager, the job record and the pages all run for real.


def wait_until(
    condition: Callable[[], bool], tries: int = 300, pause: float = 0.05
) -> bool:
    """Wait for a condition, and report whether it came true.

    A child process writes the files that the condition reads. This waits for
    the write. It asserts nothing about how long the write took, because a
    loaded machine makes a timing assertion flaky.
    """
    for _ in range(tries):
        if condition():
            return True
        time.sleep(pause)
    return False


def fake_tool_command(
    python: str, style: str, subject: str, session_id: str, rounds: int, variants: int
) -> list[str]:
    """Build the command of the stand-in tool, in the real argument shape."""
    return [
        python,
        str(HERE / "fake_tool.py"),
        "run",
        "--asset",
        style,
        "--subject",
        subject,
        "--session",
        session_id,
        "--rounds",
        str(rounds),
        "--variants",
        str(variants),
    ]


@pytest.fixture
def running_client(
    paths: dict[str, Path], monkeypatch: pytest.MonkeyPatch
) -> TestClient:
    """Give a client whose runs call the stand-in tool."""
    monkeypatch.setenv("DIRECT_DIE_SESSIONS", str(paths["sessions"]))
    return TestClient(
        create_app(
            paths["sessions"],
            paths["styleguide"],
            paths["packs"],
            paths["runs"],
            run_command=fake_tool_command,
        )
    )


def start_one(client: TestClient, slug: str, rounds: int = 1, variants: int = 2) -> str:
    """Start one asset from the page, and give the job identifier."""
    response = client.post(
        "/start",
        data={
            "style": "cartoon",
            "scope": "one",
            "slug": slug,
            "subject": "",
            "rounds": rounds,
            "variants": variants,
        },
        follow_redirects=False,
    )
    assert response.status_code == 303
    return response.headers["location"].removeprefix("/runs/")


def test_a_start_does_not_wait_for_the_run(paths: dict[str, Path]) -> None:
    # The child waits for a file that this test writes at the end. The start
    # request therefore cannot return unless it left the child running.
    gate = paths["runs"] / "gate"
    script = (
        "import pathlib, sys, time\n"
        "target = pathlib.Path(sys.argv[1])\n"
        "while not target.exists():\n"
        "    time.sleep(0.05)\n"
    )

    def command(
        python: str,
        style: str,
        subject: str,
        session_id: str,
        rounds: int,
        variants: int,
    ) -> list[str]:
        return [python, "-c", script, str(gate)]

    client = TestClient(
        create_app(
            paths["sessions"],
            paths["styleguide"],
            paths["packs"],
            paths["runs"],
            run_command=command,
        )
    )
    job_id = start_one(client, "plain")
    assert wait_until(lambda: "running" in client.get(f"/runs/{job_id}").text)
    assert "running" in client.get("/runs").text
    # The grid marks the cell that draws now, and says what it waits for.
    grid = client.get("/").text
    assert "cell state-nothing running" in grid
    assert "waiting for the first round" in grid
    gate.parent.mkdir(parents=True, exist_ok=True)
    gate.write_text("go", encoding="utf-8")
    assert wait_until(lambda: "done" in client.get(f"/runs/{job_id}").text)


def test_a_run_writes_the_session_and_the_pages_follow_it(
    running_client: TestClient, paths: dict[str, Path]
) -> None:
    job_id = start_one(running_client, "plain", rounds=1, variants=2)
    page = running_client.get(f"/runs/{job_id}")
    assert page.status_code == 200
    assert "plain" in page.text

    assert wait_until(lambda: "done" in running_client.get(f"/runs/{job_id}").text)

    sessions = list((paths["sessions"] / "cartoon").glob("*-plain"))
    assert len(sessions) == 1
    written = sessions[0]
    assert (written / "round-00" / "variant-a.png").is_file()
    assert (written / "round-00" / "variant-b.critique.json").is_file()

    body = running_client.get(f"/s/cartoon/{written.name}/round-00").text
    assert "variant-a.png" in body
    assert "The critique is not on disk yet." not in body

    store = SessionStore(paths["sessions"])
    assert pack_module.pick_for(store, "cartoon", "plain") is not None


def test_a_second_run_continues_where_the_first_stopped(
    running_client: TestClient, paths: dict[str, Path]
) -> None:
    first = start_one(running_client, "plain", rounds=1, variants=1)
    assert wait_until(lambda: "done" in running_client.get(f"/runs/{first}").text)
    second = start_one(running_client, "plain", rounds=1, variants=1)
    assert wait_until(lambda: "done" in running_client.get(f"/runs/{second}").text)
    # Two runs of one asset make two sessions, and the grid reads both.
    sessions = sorted((paths["sessions"] / "cartoon").glob("*-plain"))
    assert len(sessions) == 2


def test_a_command_that_fails_marks_the_job_failed(paths: dict[str, Path]) -> None:
    def command(
        python: str,
        style: str,
        subject: str,
        session_id: str,
        rounds: int,
        variants: int,
    ) -> list[str]:
        return [python, "-c", "raise SystemExit(3)"]

    client = TestClient(
        create_app(
            paths["sessions"],
            paths["styleguide"],
            paths["packs"],
            paths["runs"],
            run_command=command,
        )
    )
    job_id = start_one(client, "plain")
    assert wait_until(lambda: "failed" in client.get(f"/runs/{job_id}").text)


def test_the_whole_set_starts_every_asset(paths: dict[str, Path]) -> None:
    import slugs as slug_table

    def command(
        python: str,
        style: str,
        subject: str,
        session_id: str,
        rounds: int,
        variants: int,
    ) -> list[str]:
        return [python, "-c", ""]

    client = TestClient(
        create_app(
            paths["sessions"],
            paths["styleguide"],
            paths["packs"],
            paths["runs"],
            run_command=command,
        )
    )
    response = client.post(
        "/start",
        data={"style": "cartoon", "scope": "set", "rounds": 1, "variants": 4},
        follow_redirects=False,
    )
    job_id = response.headers["location"].removeprefix("/runs/")
    record = json.loads((paths["runs"] / job_id / "job.json").read_text())
    assert [item["slug"] for item in record["items"]] == list(slug_table.SLUGS)
    assert len({item["session"] for item in record["items"]}) == len(slug_table.SLUGS)


def test_a_start_with_no_asset_and_no_subject_is_refused(
    running_client: TestClient,
) -> None:
    response = running_client.post(
        "/start",
        data={"style": "cartoon", "scope": "one", "slug": "", "subject": "  "},
        follow_redirects=False,
    )
    assert response.status_code == 200
    assert "needs an asset name or a subject" in response.text


def test_a_job_from_a_dead_server_reads_as_abandoned(
    client: TestClient, paths: dict[str, Path]
) -> None:
    # The child survives the server. The supervisor thread does not. A job
    # that names another process therefore has nothing left to end it.
    directory = paths["runs"] / "20260101-000000-cartoon"
    directory.mkdir(parents=True, exist_ok=True)
    (directory / "job.json").write_text(
        json.dumps(
            {
                "job": "20260101-000000-cartoon",
                "style": "cartoon",
                "rounds": 3,
                "variants": 4,
                "started": "2026-01-01T00:00:00Z",
                "finished": None,
                "state": "running",
                "supervisor": 999_999_999,
                "stop_requested": False,
                "items": [
                    {
                        "slug": "plain",
                        "subject": "flat open grassland",
                        "session": "20260101-000000-plain",
                        "state": "running",
                        "started": "2026-01-01T00:00:00Z",
                        "finished": None,
                        "returncode": None,
                        "log": "20260101-000000-plain.log",
                    }
                ],
            }
        ),
        encoding="utf-8",
    )
    assert "abandoned" in client.get("/runs").text
    assert (
        "supervised this job is gone"
        in client.get("/runs/20260101-000000-cartoon").text
    )


def test_a_broken_job_record_drops_out_of_the_list(
    client: TestClient, paths: dict[str, Path]
) -> None:
    directory = paths["runs"] / "20260101-000000-broken"
    directory.mkdir(parents=True, exist_ok=True)
    (directory / "job.json").write_text('{"job": ', encoding="utf-8")
    response = client.get("/runs")
    assert response.status_code == 200
    assert "Traceback" not in response.text


# -- which drawing stands for an asset ---------------------------------------


def _one_session_tree(tmp_path: Path, scores: dict, feedback: dict | None) -> Path:
    """Write one style, one session and one round, and give the sessions root.

    The fixture takes a score for each variant, so a test can put the highest
    score on the drawing that the person refused. A fixture that scores every
    variant the same cannot prove that the refusal changed the answer.
    """
    root = tmp_path / "sessions"
    directory = root / "cartoon" / "forest-20260901-1000" / "round-00"
    directory.mkdir(parents=True)
    write_json_atomically(
        root / "cartoon" / "forest-20260901-1000" / "session.json",
        {
            "asset": "forest",
            "created": "2026-09-01T10:00:00Z",
            "model": "m",
            "guide_version": "v",
        },
    )
    write_json_atomically(
        directory / "meta.json",
        {"round": 0, "prompt_summary": "a forest", "parents": {}},
    )
    for letter, score in scores.items():
        (directory / f"variant-{letter}.svg").write_text(
            '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"/>',
            encoding="utf-8",
        )
        (directory / f"variant-{letter}.png").write_bytes(b"\x89PNG\r\n\x1a\n")
        write_json_atomically(
            directory / f"variant-{letter}.critique.json",
            {"verdict": "x", "faults": [], "score": score},
        )
    if feedback is not None:
        write_json_atomically(directory / "feedback.json", feedback)
    return root


def test_the_first_of_the_order_stands_for_the_asset(tmp_path: Path) -> None:
    root = _one_session_tree(
        tmp_path,
        scores={"b": 30, "d": 90},
        feedback={
            "round": 0,
            "likes": ["b", "d"],
            "denies": [],
            "order": ["b", "d"],
            "note": "",
            "text": "",
        },
    )
    store = SessionStore(root)
    pick = pack_module.pick_for(store, "cartoon", "forest")
    assert pick is not None
    assert pick.letter == "b"
    assert pick.source == "human"


def test_a_refused_drawing_never_wins_on_score(tmp_path: Path) -> None:
    root = _one_session_tree(
        tmp_path,
        scores={"a": 95, "c": 30},
        feedback={
            "round": 0,
            "likes": [],
            "denies": ["a"],
            "order": [],
            "note": "",
            "text": "",
        },
    )
    store = SessionStore(root)
    pick = pack_module.pick_for(store, "cartoon", "forest")
    assert pick is not None
    assert pick.letter == "c"
    assert pick.source == "score"


def test_the_highest_score_still_wins_when_nobody_said_anything(tmp_path: Path) -> None:
    root = _one_session_tree(tmp_path, scores={"a": 40, "c": 80}, feedback=None)
    store = SessionStore(root)
    pick = pack_module.pick_for(store, "cartoon", "forest")
    assert pick is not None
    assert pick.letter == "c"
    assert pick.source == "score"


# -- the analysis ------------------------------------------------------------


def test_the_panel_says_so_when_no_analysis_exists(client: TestClient) -> None:
    response = client.get(f"/s/{CHOSEN}/round-00")
    assert response.status_code == 200
    assert "No analysis" in response.text


def test_the_panel_shows_the_preference_the_order_and_the_rule(
    client: TestClient,
) -> None:
    response = client.get(f"/s/{CHOSEN}/round-01")
    assert "three shapes and one flat fill" in response.text
    assert "Use three shapes or fewer inside the hexagon." in response.text
    assert 'value="d,b"' in response.text


def test_the_analysis_command_is_the_tool_command_line() -> None:
    found = app_module.analysis_command("python3", "cartoon", "forest-1", 2)
    assert found == [
        "python3", "-m", "direct_die", "analyse",
        "--asset", "cartoon", "--session", "forest-1", "--round", "2",
    ]


def test_the_analyse_route_runs_the_command(monkeypatch, client: TestClient) -> None:
    seen = {}

    class Done:
        returncode = 0
        stdout = "the analysis ranks d, b"
        stderr = ""

    def fake_run(command, **rest):
        seen["command"] = command
        seen["cwd"] = rest.get("cwd")
        return Done()

    monkeypatch.setattr(app_module.subprocess, "run", fake_run)
    response = client.post(f"/s/{CHOSEN}/round-01/analyse", follow_redirects=False)
    assert response.status_code == 303
    assert seen["command"][3] == "analyse"
    assert "--round" in seen["command"]


def test_the_analyse_route_shows_the_error_the_tool_printed(
    monkeypatch, client: TestClient
) -> None:
    class Failed:
        returncode = 4
        stdout = ""
        stderr = "analysis error: the analysis needs at least one liked drawing"

    monkeypatch.setattr(app_module.subprocess, "run", lambda command, **rest: Failed())
    response = client.post(f"/s/{CHOSEN}/round-01/analyse", follow_redirects=True)
    assert "at least one liked drawing" in response.text


def test_accepting_a_rule_appends_it_to_the_style(
    client: TestClient, paths: dict[str, Path]
) -> None:
    response = client.post(
        "/rules/cartoon/append",
        data={
            "rule": "Use three shapes or fewer inside the hexagon.",
            "back": f"/s/{CHOSEN}/round-01",
        },
        follow_redirects=False,
    )
    assert response.status_code == 303
    text = (paths["styleguide"] / "cartoon.md").read_text(encoding="utf-8")
    assert "- Use three shapes or fewer inside the hexagon." in text


def test_accepting_an_empty_rule_shows_an_error(client: TestClient) -> None:
    response = client.post(
        "/rules/cartoon/append",
        data={"rule": "   ", "back": f"/s/{CHOSEN}/round-01"},
        follow_redirects=True,
    )
    assert "empty" in response.text.lower()


def test_accepting_a_rule_for_a_style_with_no_file_shows_an_error(
    client: TestClient,
) -> None:
    response = client.post(
        "/rules/nosuchstyle/append",
        data={"rule": "Use three shapes or fewer.", "back": "/"},
        follow_redirects=True,
    )
    assert "no rules file" in response.text.lower()
