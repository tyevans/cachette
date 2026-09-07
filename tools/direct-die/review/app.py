"""The direct-die front end.

A person opens this in a browser and drives the whole tool from it. They
start a run, watch it draw, steer a round, promote a drawing to an exemplar
of its style, read the grid of every asset against every style, and export a
style as a pack that something else can load.

The pages render plain HTML on the server. There is no build step and no
client framework. A small script reloads a page while work runs.

## The unfinished state is the interface

One round of four variants takes about seventy seconds, and a whole asset
set is eleven subjects. A person therefore spends most of the time looking
at work that is part way through. No page waits for a run. Every page shows
what is on disk now and marks the rest as not yet arrived. A round with two
of four variants shows two pictures and two gaps.

The round page never reloads on its own, because a person types feedback
into it. It polls a small state address instead, and offers a reload when
the disk changes.

## What this server writes

It writes `feedback.json` into a round directory. It writes a job record
under the runs directory. It writes an exemplar into the style guide when a
person promotes a drawing. It writes a pack under the packs directory. It
writes nothing else, and it changes no drawing.

Start it with the module runner, or with the console command.[^1]

## References

[^1]: The tool guide. `tools/direct-die/review/README.md`
"""

from __future__ import annotations

import argparse
import os
import sys
from collections.abc import Callable
from pathlib import Path

from fastapi import FastAPI, Form, Request
from fastapi.responses import (
    FileResponse,
    HTMLResponse,
    JSONResponse,
    RedirectResponse,
    Response,
)
from fastapi.staticfiles import StaticFiles
from fastapi.templating import Jinja2Templates

HERE = Path(__file__).resolve().parent

# The tool is a flat directory, not an installed package. Put the directory on
# the import path so that a caller can start the server from anywhere.
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

import exemplars as exemplar_module  # noqa: E402
import matrix as matrix_module  # noqa: E402
import packs as pack_module  # noqa: E402
import slugs as slug_table  # noqa: E402
from runs import RunManager, default_command  # noqa: E402
from store import (  # noqa: E402
    VARIANT_LETTERS,
    ContractError,
    Round,
    SessionStore,
)

# The tool directory. The server calls the tool from here, and every default
# directory below sits inside it.
TOOL_DIRECTORY = HERE.parent

# The four roots that the server reads. The generation loop writes the first.
DEFAULT_SESSIONS_ROOT = TOOL_DIRECTORY / "sessions"
DEFAULT_STYLEGUIDE_ROOT = TOOL_DIRECTORY / "styleguide"
DEFAULT_PACKS_ROOT = TOOL_DIRECTORY / "packs"
DEFAULT_RUNS_ROOT = TOOL_DIRECTORY / "runs"

# The environment variables that point the server at other roots. The tests
# and the fixture script set them to a scratch directory.
ROOT_VARIABLE = "DIRECT_DIE_SESSIONS"
STYLEGUIDE_VARIABLE = "DIRECT_DIE_STYLEGUIDE"
PACKS_VARIABLE = "DIRECT_DIE_PACKS"
RUNS_VARIABLE = "DIRECT_DIE_RUNS"

# How many rounds and variants the start form offers.
ROUND_CHOICES = (1, 2, 3, 4, 5, 6)
VARIANT_CHOICES = (1, 2, 3, 4)


def _root(variable: str, fallback: Path) -> Path:
    value = os.environ.get(variable)
    return Path(value) if value else fallback


def sessions_root() -> Path:
    """Give the sessions root that this process reads."""
    return _root(ROOT_VARIABLE, DEFAULT_SESSIONS_ROOT)


def no_store(response: Response) -> Response:
    """Tell the browser to read the page again on every visit.

    The generation loop writes while the person reads, so a cached page shows
    a state that is no longer on disk.
    """
    response.headers["Cache-Control"] = "no-store, must-revalidate"
    response.headers["Pragma"] = "no-cache"
    return response


def round_fingerprint(current: Round) -> str:
    """Give a short string that changes when a round gains a file.

    The round page compares this against the one it holds. A change means
    the loop wrote a drawing or a critique, and the person can reload.
    """
    parts = []
    for variant in current.variants:
        parts.append(
            f"{variant.letter}"
            f"{1 if variant.svg else 0}"
            f"{1 if variant.display_png else 0}"
            f"{1 if variant.large_png else 0}"
            f"{1 if variant.critique is not None else 0}"
        )
    parts.append("f" if current.feedback is not None else "-")
    return ".".join(parts)


def create_app(
    root: Path | None = None,
    styleguide: Path | None = None,
    packs_root: Path | None = None,
    runs_root: Path | None = None,
    run_command: Callable[..., list[str]] = default_command,
    tool_directory: Path | None = None,
) -> FastAPI:
    """Build the application against one set of directories."""
    store = SessionStore(root if root is not None else sessions_root())
    styleguide_root = (
        styleguide
        if styleguide is not None
        else _root(STYLEGUIDE_VARIABLE, DEFAULT_STYLEGUIDE_ROOT)
    )
    packs_directory = (
        packs_root
        if packs_root is not None
        else _root(PACKS_VARIABLE, DEFAULT_PACKS_ROOT)
    )
    runs_directory = (
        runs_root if runs_root is not None else _root(RUNS_VARIABLE, DEFAULT_RUNS_ROOT)
    )
    runner = RunManager(
        runs_root=runs_directory,
        sessions_root=store.root,
        tool_directory=tool_directory if tool_directory is not None else TOOL_DIRECTORY,
        command=run_command,
    )

    templates = Jinja2Templates(directory=str(HERE / "templates"))
    templates.env.globals["variant_letters"] = VARIANT_LETTERS
    templates.env.globals["round_choices"] = ROUND_CHOICES
    templates.env.globals["variant_choices"] = VARIANT_CHOICES
    templates.env.globals["all_slugs"] = slug_table.SLUGS

    app = FastAPI(title="direct-die", docs_url=None, redoc_url=None)
    app.state.store = store
    app.state.runner = runner
    app.state.styleguide_root = styleguide_root
    app.state.packs_root = packs_directory
    app.mount("/static", StaticFiles(directory=str(HERE / "static")), name="static")

    def page(name: str, request: Request, **context: object) -> HTMLResponse:
        context.setdefault("active_jobs", runner.active_jobs())
        context.setdefault("auto_reload", False)
        response = templates.TemplateResponse(request, name, context)
        return no_store(response)

    def styles() -> list[str]:
        return matrix_module.known_styles(styleguide_root, store)

    # -- the grid of every asset against every style -----------------------

    @app.get("/", response_class=HTMLResponse)
    def index(request: Request) -> HTMLResponse:
        """Show the grid: what is drawn, what is chosen, and what is missing."""
        names = styles()
        rows = matrix_module.build_matrix(
            store, names, runner, styleguide_root, packs_directory
        )
        packs = {name: pack_module.pack_assets(packs_directory, name) for name in names}
        return page(
            "matrix.html",
            request,
            styles=names,
            rows=rows,
            packs=packs,
            ladder=matrix_module.LADDER,
            auto_reload=True,
        )

    # -- starting work -----------------------------------------------------

    @app.post("/start")
    def start(
        request: Request,
        style: str = Form(...),
        scope: str = Form(default="one"),
        slug: str = Form(default=""),
        subject: str = Form(default=""),
        rounds: int = Form(default=3),
        variants: int = Form(default=4),
    ) -> Response:
        """Start a run in the background, and show the job at once.

        The call returns before the tool draws anything. One round takes
        about seventy seconds, so nothing may wait for it.
        """
        rounds = max(1, min(rounds, max(ROUND_CHOICES)))
        variants = max(1, min(variants, max(VARIANT_CHOICES)))
        if scope == "set":
            requests = [
                (name, slug_table.subject_for(name)) for name in slug_table.SLUGS
            ]
        elif slug and slug_table.is_slug(slug):
            requests = [(slug, subject.strip() or slug_table.subject_for(slug))]
        elif subject.strip():
            requests = [(None, subject.strip())]
        else:
            return page(
                "error.html",
                request,
                message="A run needs an asset name or a subject.",
            )
        try:
            job = runner.start_job(style, requests, rounds, variants)
        except (ValueError, OSError) as error:
            return page("error.html", request, message=str(error))
        return no_store(RedirectResponse(f"/runs/{job.job_id}", status_code=303))

    @app.get("/runs", response_class=HTMLResponse)
    def runs_page(request: Request) -> HTMLResponse:
        """List every job, newest first."""
        return page(
            "runs.html",
            request,
            jobs=runner.jobs(),
            styles=styles(),
            auto_reload=True,
        )

    @app.get("/runs/{job_id}", response_class=HTMLResponse)
    def run_page(request: Request, job_id: str) -> HTMLResponse:
        """Show one job: each subject, its state, and what it waits for."""
        try:
            job = runner.load(job_id)
        except ContractError as error:
            return page("error.html", request, message=str(error))
        if job is None:
            return page("error.html", request, message=f"no such run: {job_id}")
        lines = []
        for item in job.items:
            session = store.load_session(job.style, item.session_id)
            waiting = (
                matrix_module.waiting_words(session, job.rounds, job.variants)
                if item.state == "running"
                else ""
            )
            lines.append(
                {
                    "item": item,
                    "session": session,
                    "waiting": waiting,
                    "log": runner.log_tail(job.job_id, item.log, 14),
                }
            )
        return page("run.html", request, job=job, lines=lines, auto_reload=True)

    @app.post("/runs/{job_id}/stop")
    def stop_run(request: Request, job_id: str) -> Response:
        """Ask one job to stop after the subject that runs now."""
        try:
            runner.request_stop(job_id)
        except ContractError as error:
            return page("error.html", request, message=str(error))
        return no_store(RedirectResponse(f"/runs/{job_id}", status_code=303))

    # -- the sessions, and one round --------------------------------------

    @app.get("/sessions", response_class=HTMLResponse)
    def sessions_page(request: Request) -> HTMLResponse:
        """Show every session, and the rounds each one holds."""
        return page("index.html", request, sessions=store.list_sessions())

    @app.get("/s/{asset}/{session_id}", response_class=HTMLResponse)
    def session_page(request: Request, asset: str, session_id: str) -> HTMLResponse:
        """Show one session, and send the person to its latest round."""
        try:
            if not store.session_directory(asset, session_id).is_dir():
                raise ContractError(f"no such session: {asset}/{session_id}")
            session = store.load_session(asset, session_id)
        except ContractError as error:
            return page("error.html", request, message=str(error))
        return page(
            "session.html",
            request,
            session=session,
            sessions=store.list_sessions(),
            live=runner.item_for_session(asset, session_id),
            auto_reload=True,
        )

    @app.get("/s/{asset}/{session_id}/{round_name}", response_class=HTMLResponse)
    def round_page(
        request: Request, asset: str, session_id: str, round_name: str
    ) -> HTMLResponse:
        """Show one round: the variants, the critiques, and the form."""
        try:
            session = store.load_session(asset, session_id)
            current = store.load_round(asset, session_id, round_name)
        except ContractError as error:
            return page("error.html", request, message=str(error))
        if not store.round_directory(asset, session_id, round_name).is_dir():
            return page(
                "error.html",
                request,
                message=f"no such round: {asset}/{session_id}/{round_name}",
            )
        live = runner.item_for_session(asset, session_id)
        waiting = ""
        if live is not None:
            waiting = matrix_module.waiting_words(
                session, live[0].rounds, live[0].variants
            )
        return page(
            "round.html",
            request,
            session=session,
            round=current,
            sessions=store.list_sessions(),
            saved=request.query_params.get("saved") == "1",
            promoted=request.query_params.get("promoted"),
            slug=pack_module.slug_of(session),
            live=live,
            waiting=waiting,
            fingerprint=round_fingerprint(current),
        )

    @app.post("/s/{asset}/{session_id}/{round_name}/feedback")
    async def submit_feedback(
        request: Request, asset: str, session_id: str, round_name: str
    ) -> Response:
        """Write the feedback of one round, then show the round again.

        Each variant carries one radio group named `mark-<letter>`, whose
        value is `like`, `deny` or `none`. A drawing is liked, refused, or
        neither, and those are exclusive, so a radio group is the honest
        control.

        The order field holds the liked letters, best first, separated by
        commas. A blank order takes the letters in the order of the page.
        """
        form = await request.form()
        likes: list[str] = []
        denies: list[str] = []
        for letter in VARIANT_LETTERS:
            mark = form.get(f"mark-{letter}")
            if mark == "like":
                likes.append(letter)
            elif mark == "deny":
                denies.append(letter)

        ranked = [
            item.strip()
            for item in str(form.get("order") or "").split(",")
            if item.strip() in likes
        ]
        ranked = list(dict.fromkeys(ranked))
        ranked.extend(letter for letter in likes if letter not in ranked)

        try:
            store.write_feedback(
                asset,
                session_id,
                round_name,
                likes,
                denies,
                ranked,
                str(form.get("note") or ""),
                str(form.get("text") or ""),
            )
        except ContractError as error:
            return page("error.html", request, message=str(error))
        target = f"/s/{asset}/{session_id}/{round_name}?saved=1"
        return no_store(RedirectResponse(target, status_code=303))

    @app.get("/api/round/{asset}/{session_id}/{round_name}")
    def round_state(asset: str, session_id: str, round_name: str) -> Response:
        """Give a short state of one round, for the page to poll.

        The page holds the string that it rendered. A different string means
        the loop wrote a drawing or a critique since then.
        """
        try:
            current = store.load_round(asset, session_id, round_name)
        except ContractError as error:
            return no_store(JSONResponse({"error": str(error)}, status_code=400))
        live = runner.item_for_session(asset, session_id)
        return no_store(
            JSONResponse(
                {
                    "fingerprint": round_fingerprint(current),
                    "running": live is not None,
                    "rounds": len(store.round_names(asset, session_id)),
                }
            )
        )

    @app.get("/f/{asset}/{session_id}/{round_name}/{file_name}")
    def variant_file(
        asset: str, session_id: str, round_name: str, file_name: str
    ) -> Response:
        """Serve one render or one SVG out of a round directory."""
        try:
            path = store.asset_file(asset, session_id, round_name, file_name)
        except ContractError as error:
            return no_store(Response(str(error), status_code=400))
        if path is None:
            return no_store(Response("no such file", status_code=404))
        return no_store(FileResponse(path))

    # -- promotion ---------------------------------------------------------

    @app.post("/promote")
    def promote(
        request: Request,
        style: str = Form(...),
        session_id: str = Form(...),
        round_name: str = Form(...),
        letter: str = Form(...),
        slug: str = Form(...),
    ) -> Response:
        """Make one drawing an exemplar of its style.

        The next run reads the exemplar directory again, so the drawing
        becomes part of the guide with no other step.
        """
        try:
            source = store.asset_file(
                style, session_id, round_name, f"variant-{letter}.svg"
            )
            if source is None:
                raise ContractError("that drawing has no SVG on disk")
            exemplar_module.promote(styleguide_root, style, slug, source)
        except (ContractError, ValueError, OSError) as error:
            return page("error.html", request, message=str(error))
        target = f"/s/{style}/{session_id}/{round_name}?promoted={slug}"
        return no_store(RedirectResponse(target, status_code=303))

    @app.get("/exemplars", response_class=HTMLResponse)
    def exemplars_page(request: Request) -> HTMLResponse:
        """Show the accepted pictures of every style."""
        names = styles()
        table = {
            name: exemplar_module.list_exemplars(styleguide_root, name)
            for name in names
        }
        return page("exemplars.html", request, styles=names, exemplars=table)

    @app.post("/exemplars/{style}/withdraw")
    def withdraw_exemplar(
        request: Request, style: str, name: str = Form(...)
    ) -> Response:
        """Remove one exemplar from a style."""
        try:
            exemplar_module.withdraw(styleguide_root, style, name)
        except (ContractError, ValueError, OSError) as error:
            return page("error.html", request, message=str(error))
        return no_store(RedirectResponse("/exemplars", status_code=303))

    @app.get("/e/{style}/{file_name}")
    def exemplar_file(style: str, file_name: str) -> Response:
        """Serve one exemplar picture."""
        try:
            path = exemplar_module.exemplar_file(styleguide_root, style, file_name)
        except (ContractError, ValueError) as error:
            return no_store(Response(str(error), status_code=400))
        if path is None:
            return no_store(Response("no such exemplar", status_code=404))
        return no_store(FileResponse(path))

    # -- the packs ---------------------------------------------------------

    def render_pack(
        request: Request, style: str, report: pack_module.ExportReport | None
    ) -> HTMLResponse:
        """Render one pack page, with the report of an export that just ran."""
        manifest = pack_module.read_pack(packs_directory, style)
        sessions = store.list_sessions()
        picks = {
            slug: pack_module.pick_for(store, style, slug, sessions)
            for slug in slug_table.SLUGS
        }
        assets = pack_module.pack_assets(packs_directory, style)
        return page(
            "pack.html",
            request,
            style=style,
            manifest=manifest,
            assets=assets,
            picks=picks,
            directory=pack_module.pack_directory(packs_directory, style),
            report=report,
        )

    @app.get("/packs/{style}", response_class=HTMLResponse)
    def pack_page(request: Request, style: str) -> HTMLResponse:
        """Show one pack: what it holds, and what it still lacks."""
        try:
            return render_pack(request, style, None)
        except ContractError as error:
            return page("error.html", request, message=str(error))

    @app.post("/packs/{style}/export", response_class=HTMLResponse)
    def export(request: Request, style: str) -> Response:
        """Write the chosen drawings of one style as a pack, and report.

        The answer is the pack page with the report of this export. The
        report names each asset that the pack could not hold, and why.
        """
        try:
            report = pack_module.export_pack(store, packs_directory, style)
            return render_pack(request, style, report)
        except (ContractError, ValueError, OSError) as error:
            return page("error.html", request, message=str(error))

    @app.get("/p/{style}/{file_name}")
    def pack_file(style: str, file_name: str) -> Response:
        """Serve one file out of a pack directory."""
        try:
            path = pack_module.pack_file(packs_directory, style, file_name)
        except (ContractError, ValueError) as error:
            return no_store(Response(str(error), status_code=400))
        if path is None:
            return no_store(Response("no such file", status_code=404))
        return no_store(FileResponse(path))

    return app


app = create_app()


def main() -> None:
    """Run the server for one person on the loopback address."""
    import uvicorn

    parser = argparse.ArgumentParser(description="the direct-die front end")
    parser.add_argument("--host", default="127.0.0.1", help="the address to bind")
    parser.add_argument("--port", type=int, default=8765, help="the port to bind")
    parser.add_argument(
        "--sessions",
        type=Path,
        default=None,
        help="the sessions root to read; the environment sets it otherwise",
    )
    parser.add_argument(
        "--styleguide",
        type=Path,
        default=None,
        help="the style guide root that holds the rules and the exemplars",
    )
    parser.add_argument(
        "--packs", type=Path, default=None, help="the directory that holds the packs"
    )
    parser.add_argument(
        "--runs", type=Path, default=None, help="the directory that holds the jobs"
    )
    arguments = parser.parse_args()
    for value, variable in (
        (arguments.sessions, ROOT_VARIABLE),
        (arguments.styleguide, STYLEGUIDE_VARIABLE),
        (arguments.packs, PACKS_VARIABLE),
        (arguments.runs, RUNS_VARIABLE),
    ):
        if value is not None:
            os.environ[variable] = str(value)
    uvicorn.run(
        create_app(
            arguments.sessions,
            arguments.styleguide,
            arguments.packs,
            arguments.runs,
        ),
        host=arguments.host,
        port=arguments.port,
    )


if __name__ == "__main__":
    main()
