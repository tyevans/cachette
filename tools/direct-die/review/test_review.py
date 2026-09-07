"""Tests for the direct-die review server.

The tests build the fixture tree in a temporary directory, then drive the
server through its public interface with the test client. No test reaches
into a private field.

Most of the tests exercise a half-written directory, because the generation
loop runs while a person reads the pages. A page must render a gap, not
raise.

Run them with the test runner from this directory.[^1]

## References

[^1]: The tool guide. `tools/direct-die/review/README.md`
"""

from __future__ import annotations

import datetime
import json
import sys
from pathlib import Path

import pytest
from fastapi.testclient import TestClient

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

from app import create_app  # noqa: E402
from make_fixtures import build  # noqa: E402
from store import ContractError, SessionStore  # noqa: E402

HEALTHY = "hex-forest/sess-20260901-1000"
PARTIAL = "hex-mountain/sess-20260902-0930"
EMPTY = "hex-water/sess-20260903-0800"
NO_MANIFEST = "unit-scout/sess-20260903-1100"


@pytest.fixture(scope="module")
def root(tmp_path_factory: pytest.TempPathFactory) -> Path:
    """Write the fixture tree once, and give its root."""
    return build(tmp_path_factory.mktemp("sessions") / "tree")


@pytest.fixture
def client(root: Path) -> TestClient:
    """Give a test client that reads the fixture tree."""
    return TestClient(create_app(root))


# -- the pages render --------------------------------------------------------


def test_the_index_lists_every_session(client: TestClient) -> None:
    response = client.get("/sessions")
    assert response.status_code == 200
    for asset in ("hex-forest", "hex-mountain", "hex-water", "unit-scout"):
        assert asset in response.text


@pytest.mark.parametrize(
    "path",
    [
        "/",
        "/sessions",
        f"/s/{HEALTHY}",
        f"/s/{HEALTHY}/round-00",
        f"/s/{HEALTHY}/round-01",
        f"/s/{HEALTHY}/round-02",
        f"/s/{PARTIAL}",
        f"/s/{PARTIAL}/round-00",
        f"/s/{PARTIAL}/round-01",
        f"/s/{PARTIAL}/round-02",
        f"/s/{PARTIAL}/round-03",
        f"/s/{PARTIAL}/round-04",
        f"/s/{EMPTY}",
        f"/s/{NO_MANIFEST}",
        f"/s/{NO_MANIFEST}/round-00",
    ],
)
def test_every_page_renders(client: TestClient, path: str) -> None:
    response = client.get(path)
    assert response.status_code == 200
    assert "Traceback" not in response.text


def test_a_page_tells_the_browser_not_to_cache(client: TestClient) -> None:
    # The generation loop writes while the person reads, so a cached page
    # shows a state that is no longer on disk.
    response = client.get(f"/s/{HEALTHY}/round-00")
    assert "no-store" in response.headers["cache-control"]


# -- the half-written cases show a gap ---------------------------------------


def test_a_round_with_one_variant_shows_one_variant(client: TestClient) -> None:
    response = client.get(f"/s/{PARTIAL}/round-01")
    assert response.status_code == 200
    assert response.text.count('class="variant"') == 1


def test_a_round_with_no_critique_says_so(client: TestClient) -> None:
    response = client.get(f"/s/{PARTIAL}/round-02")
    assert "The critique is not on disk yet." in response.text


def test_a_broken_critique_file_reads_as_no_critique(client: TestClient) -> None:
    response = client.get(f"/s/{PARTIAL}/round-04")
    assert response.status_code == 200
    assert "The critique is not on disk yet." in response.text


def test_a_round_with_no_render_says_so(client: TestClient) -> None:
    response = client.get(f"/s/{PARTIAL}/round-03")
    assert "The display render is not on disk yet." in response.text
    assert "no meta.json yet" in response.text


def test_a_session_with_no_round_says_so(client: TestClient) -> None:
    response = client.get(f"/s/{EMPTY}")
    assert "holds no round yet" in response.text


def test_a_session_with_no_manifest_says_so(client: TestClient) -> None:
    response = client.get(f"/s/{NO_MANIFEST}")
    assert "no session.json yet" in response.text


def test_an_empty_root_renders(tmp_path: Path) -> None:
    empty_client = TestClient(
        create_app(
            tmp_path / "no-sessions",
            tmp_path / "no-styleguide",
            tmp_path / "no-packs",
            tmp_path / "no-runs",
        )
    )
    response = empty_client.get("/sessions")
    assert response.status_code == 200
    assert "No session is on disk yet" in response.text


# -- the critique sits beside the picture ------------------------------------


def test_the_round_page_shows_the_model_critique(client: TestClient) -> None:
    response = client.get(f"/s/{HEALTHY}/round-00")
    assert "The outline is too heavy for the display size." in response.text
    assert "keep" in response.text


def test_the_round_page_shows_both_render_sizes(client: TestClient) -> None:
    response = client.get(f"/s/{HEALTHY}/round-00")
    assert "round-00/variant-a.png" in response.text
    assert "round-00/variant-a.large.png" in response.text


def test_the_round_page_shows_the_prior_feedback(client: TestClient) -> None:
    response = client.get(f"/s/{HEALTHY}/round-00")
    assert "B reads best at tile size" in response.text
    assert "picked" in response.text


# -- the files ---------------------------------------------------------------


@pytest.mark.parametrize(
    "name", ["variant-a.png", "variant-a.large.png", "variant-a.svg"]
)
def test_a_variant_file_is_served(client: TestClient, name: str) -> None:
    response = client.get(f"/f/{HEALTHY}/round-00/{name}")
    assert response.status_code == 200
    assert len(response.content) > 0


@pytest.mark.parametrize("name", ["meta.json", "session.json", "variant-z.png"])
def test_a_file_outside_the_contract_is_refused(client: TestClient, name: str) -> None:
    response = client.get(f"/f/{HEALTHY}/round-00/{name}")
    assert response.status_code == 400


def test_an_absent_render_gives_not_found(client: TestClient) -> None:
    response = client.get(f"/f/{PARTIAL}/round-03/variant-a.png")
    assert response.status_code == 404


def test_a_traversal_reads_nothing(client: TestClient) -> None:
    for path in [
        "/s/..%2F..%2Fetc/passwd",
        f"/f/{HEALTHY}/round-00/..%2F..%2Fsession.json",
        f"/s/{HEALTHY}/..%2F..%2Fetc",
    ]:
        response = client.get(path)
        assert "root:x:" not in response.text


# -- the one write -----------------------------------------------------------


def read_feedback(root: Path, session: str, round_name: str) -> dict:
    return json.loads((root / session / round_name / "feedback.json").read_text())


def test_a_submit_writes_a_well_formed_feedback_file(
    client: TestClient, root: Path
) -> None:
    response = client.post(
        f"/s/{PARTIAL}/round-00/feedback",
        data={"mark-c": "like", "text": "C reads at tile size."},
        follow_redirects=False,
    )
    assert response.status_code == 303
    written = read_feedback(root, PARTIAL, "round-00")
    assert set(written) == {"round", "likes", "denies", "order", "note", "text", "at"}
    assert isinstance(written["round"], int)
    assert written["likes"] == ["c"]
    assert written["text"] == "C reads at tile size."
    parsed = datetime.datetime.fromisoformat(written["at"].replace("Z", "+00:00"))
    assert parsed.tzinfo is not None


def test_a_submit_can_pick_none(client: TestClient, root: Path) -> None:
    # Like a variant first, so that a server which ignores the submit cannot
    # pass this by leaving the earlier mark in place.
    client.post(
        f"/s/{PARTIAL}/round-01/feedback",
        data={"mark-a": "like", "text": "a for now"},
        follow_redirects=False,
    )
    assert read_feedback(root, PARTIAL, "round-01")["likes"] == ["a"]
    client.post(
        f"/s/{PARTIAL}/round-01/feedback",
        data={
            "mark-a": "none",
            "mark-b": "none",
            "mark-c": "none",
            "mark-d": "none",
            "text": "None of these.",
        },
        follow_redirects=False,
    )
    written = read_feedback(root, PARTIAL, "round-01")
    assert written["likes"] == []
    assert written["denies"] == []


def test_a_bad_mark_value_becomes_no_mark(client: TestClient, root: Path) -> None:
    # Like a variant first. A server that refuses the bad value and writes
    # nothing then leaves the earlier like on disk, and this test sees that.
    client.post(
        f"/s/{PARTIAL}/round-02/feedback",
        data={"mark-a": "like", "text": "a for now"},
        follow_redirects=False,
    )
    assert read_feedback(root, PARTIAL, "round-02")["likes"] == ["a"]
    client.post(
        f"/s/{PARTIAL}/round-02/feedback",
        data={"mark-a": "maybe", "text": "bad mark"},
        follow_redirects=False,
    )
    written = read_feedback(root, PARTIAL, "round-02")
    assert written["likes"] == []
    assert written["denies"] == []
    assert written["text"] == "bad mark"


def test_a_submit_to_a_missing_round_writes_nothing(
    client: TestClient, root: Path
) -> None:
    response = client.post(
        f"/s/{PARTIAL}/round-77/feedback",
        data={"mark-a": "like", "text": ""},
        follow_redirects=False,
    )
    assert response.status_code == 200
    assert "no such round" in response.text
    assert not (root / PARTIAL / "round-77").exists()


def test_the_write_leaves_no_temporary_file(client: TestClient, root: Path) -> None:
    # The write renames a temporary file in the same directory, so the loop
    # never reads half a file. Nothing may survive the rename.
    client.post(
        f"/s/{PARTIAL}/round-03/feedback",
        data={"mark-a": "like", "text": "x"},
        follow_redirects=False,
    )
    directory = root / PARTIAL / "round-03"
    assert [n.name for n in directory.glob(".feedback-*")] == []


def test_the_page_reads_the_disk_again_after_a_change(
    client: TestClient, root: Path
) -> None:
    # The generation loop writes under the person, so no page may cache.
    path = root / PARTIAL / "round-04" / "feedback.json"
    path.write_text('{"choice": "d", "text": "changed on disk", "at": "2026-01-01Z"}')
    assert "changed on disk" in client.get(f"/s/{PARTIAL}/round-04").text
    path.write_text('{"choice": "a", "text": "changed once more", "at": "2026-01-01Z"}')
    assert "changed once more" in client.get(f"/s/{PARTIAL}/round-04").text


def test_the_store_refuses_an_unsafe_name(root: Path) -> None:
    store = SessionStore(root)
    for asset in ["..", "a/b", ""]:
        with pytest.raises(ContractError):
            store.session_directory(asset, "x")


# -- the feedback shape ------------------------------------------------------


def _round_directory(root: Path, key: str, round_name: str) -> Path:
    asset, session_id = key.split("/")
    return root / asset / session_id / round_name


def test_the_writer_records_the_likes_and_the_refusals(root: Path) -> None:
    store = SessionStore(root)
    asset, session_id = HEALTHY.split("/")
    path = store.write_feedback(
        asset,
        session_id,
        "round-02",
        ["b", "d"],
        ["a"],
        ["d", "b"],
        "keep it flat",
        "darker base",
    )
    written = json.loads(path.read_text(encoding="utf-8"))
    assert written["round"] == 2
    assert written["likes"] == ["b", "d"]
    assert written["denies"] == ["a"]
    assert written["order"] == ["d", "b"]
    assert written["note"] == "keep it flat"
    assert written["text"] == "darker base"
    assert "choice" not in written


def test_the_writer_refuses_a_letter_that_is_in_both_lists(root: Path) -> None:
    store = SessionStore(root)
    asset, session_id = HEALTHY.split("/")
    with pytest.raises(ContractError):
        store.write_feedback(asset, session_id, "round-02", ["b"], ["b"], ["b"], "", "")


def test_the_writer_refuses_an_order_that_is_not_the_likes(root: Path) -> None:
    store = SessionStore(root)
    asset, session_id = HEALTHY.split("/")
    with pytest.raises(ContractError):
        store.write_feedback(
            asset, session_id, "round-02", ["b", "d"], [], ["b"], "", ""
        )


def test_the_writer_refuses_a_letter_that_is_not_a_variant(root: Path) -> None:
    store = SessionStore(root)
    asset, session_id = HEALTHY.split("/")
    with pytest.raises(ContractError):
        store.write_feedback(asset, session_id, "round-02", ["z"], [], ["z"], "", "")


def test_an_old_choice_file_reads_as_one_like(root: Path) -> None:
    store = SessionStore(root)
    asset, session_id = HEALTHY.split("/")
    current = store.load_round(asset, session_id, "round-00")
    assert current.likes == ("b",)
    assert current.winner == "b"
    assert current.denies == ()


def test_a_round_with_no_feedback_reads_as_nothing(root: Path) -> None:
    store = SessionStore(root)
    asset, session_id = EMPTY.split("/")
    session = store.load_session(asset, session_id)
    assert session.rounds == []


def test_the_round_reads_a_parents_object(root: Path) -> None:
    store = SessionStore(root)
    asset, session_id = HEALTHY.split("/")
    current = store.load_round(asset, session_id, "round-01")
    assert current.parents
    assert set(current.parents.values())


def test_a_round_with_no_analysis_gives_none(root: Path) -> None:
    store = SessionStore(root)
    asset, session_id = HEALTHY.split("/")
    assert store.load_round(asset, session_id, "round-02").analysis is None


def test_a_round_with_an_analysis_gives_it(root: Path) -> None:
    store = SessionStore(root)
    asset, session_id = HEALTHY.split("/")
    current = store.load_round(asset, session_id, "round-01")
    assert current.analysis is not None
    assert current.analysis["order"]


# -- the round form ----------------------------------------------------------


def test_the_form_writes_the_likes_and_the_refusals(
    client: TestClient, root: Path
) -> None:
    response = client.post(
        f"/s/{HEALTHY}/round-02/feedback",
        data={
            "mark-a": "deny",
            "mark-b": "like",
            "mark-c": "none",
            "mark-d": "like",
            "order": "d,b",
            "note": "keep the palette flat",
            "text": "raise the contrast",
        },
        follow_redirects=False,
    )
    assert response.status_code == 303
    asset, session_id = HEALTHY.split("/")
    written = json.loads(
        (root / asset / session_id / "round-02" / "feedback.json").read_text()
    )
    assert written["likes"] == ["b", "d"]
    assert written["denies"] == ["a"]
    assert written["order"] == ["d", "b"]
    assert written["note"] == "keep the palette flat"
    assert written["text"] == "raise the contrast"


def test_an_order_the_person_left_blank_takes_the_like_order(
    client: TestClient, root: Path
) -> None:
    client.post(
        f"/s/{HEALTHY}/round-02/feedback",
        data={"mark-b": "like", "mark-d": "like", "order": "", "note": "", "text": ""},
        follow_redirects=False,
    )
    asset, session_id = HEALTHY.split("/")
    written = json.loads(
        (root / asset / session_id / "round-02" / "feedback.json").read_text()
    )
    assert written["order"] == ["b", "d"]


def test_an_order_that_names_a_refused_letter_is_dropped(
    client: TestClient, root: Path
) -> None:
    client.post(
        f"/s/{HEALTHY}/round-02/feedback",
        data={
            "mark-a": "deny",
            "mark-b": "like",
            "order": "a,b",
            "note": "",
            "text": "",
        },
        follow_redirects=False,
    )
    asset, session_id = HEALTHY.split("/")
    written = json.loads(
        (root / asset / session_id / "round-02" / "feedback.json").read_text()
    )
    assert written["order"] == ["b"]
    assert written["denies"] == ["a"]


def test_a_form_that_marks_nothing_writes_two_empty_lists(
    client: TestClient, root: Path
) -> None:
    client.post(
        f"/s/{HEALTHY}/round-02/feedback",
        data={"order": "", "note": "", "text": "nothing yet"},
        follow_redirects=False,
    )
    asset, session_id = HEALTHY.split("/")
    written = json.loads(
        (root / asset / session_id / "round-02" / "feedback.json").read_text()
    )
    assert written["likes"] == []
    assert written["denies"] == []
    assert written["text"] == "nothing yet"


def test_the_round_page_shows_the_three_marks_for_each_variant(
    client: TestClient,
) -> None:
    response = client.get(f"/s/{HEALTHY}/round-01")
    assert response.status_code == 200
    for letter in ("a", "b", "c", "d"):
        assert f'name="mark-{letter}"' in response.text
    assert 'name="note"' in response.text
    assert 'name="order"' in response.text


def test_the_round_page_marks_the_saved_like_and_refusal(client: TestClient) -> None:
    response = client.get(f"/s/{HEALTHY}/round-01")
    assert "refused" in response.text.lower()
    assert "liked" in response.text.lower()


def test_the_round_page_names_the_parent_of_each_variant(client: TestClient) -> None:
    response = client.get(f"/s/{HEALTHY}/round-01")
    assert "round-00/variant-b" in response.text
    assert "round-00/variant-d" in response.text
