"""The direct-die review server.

A person opens this in a browser to steer one direct-die session. The pages
show the variants of a round side by side, at the display size and at the
inspection size, beside the critique that the model wrote for each one. The
person picks a letter, types free text, and submits. The server writes
`feedback.json` into the round directory. The generation loop reads it on the
next round.

The server renders plain HTML on the server. It has no build step and no
client framework. It caches nothing, because the generation loop writes into
the same directories while the person reads.

Start it with the module runner, or with the console command.[^1]

## References

[^1]: The tool guide. `tools/direct-die/review/README.md`
"""

from __future__ import annotations

import argparse
import os
import sys
from pathlib import Path

from fastapi import FastAPI, Form, Request
from fastapi.responses import FileResponse, HTMLResponse, RedirectResponse, Response
from fastapi.staticfiles import StaticFiles
from fastapi.templating import Jinja2Templates

HERE = Path(__file__).resolve().parent

# The tool is a flat directory, not an installed package. Put the directory on
# the import path so that a caller can start the server from anywhere.
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

from store import VARIANT_LETTERS, ContractError, SessionStore  # noqa: E402

# The sessions root of the on-disk contract. The generation loop writes here.
DEFAULT_SESSIONS_ROOT = HERE.parent / "sessions"

# The environment variable that points the server at a different root. The
# fixture script sets it to a scratch directory.
ROOT_VARIABLE = "DIRECT_DIE_SESSIONS"


def sessions_root() -> Path:
    """Give the sessions root that this process reads."""
    value = os.environ.get(ROOT_VARIABLE)
    return Path(value) if value else DEFAULT_SESSIONS_ROOT


def no_store(response: Response) -> Response:
    """Tell the browser to read the page again on every visit.

    The generation loop writes while the person reads, so a cached page shows
    a state that is no longer on disk.
    """
    response.headers["Cache-Control"] = "no-store, must-revalidate"
    response.headers["Pragma"] = "no-cache"
    return response


def create_app(root: Path | None = None) -> FastAPI:
    """Build the application against one sessions root."""
    store = SessionStore(root if root is not None else sessions_root())
    templates = Jinja2Templates(directory=str(HERE / "templates"))
    templates.env.globals["variant_letters"] = VARIANT_LETTERS

    app = FastAPI(title="direct-die review", docs_url=None, redoc_url=None)
    app.state.store = store
    app.mount("/static", StaticFiles(directory=str(HERE / "static")), name="static")

    def page(name: str, request: Request, **context: object) -> HTMLResponse:
        response = templates.TemplateResponse(request, name, context)
        return no_store(response)

    @app.get("/", response_class=HTMLResponse)
    def index(request: Request) -> HTMLResponse:
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
        return page(
            "round.html",
            request,
            session=session,
            round=current,
            sessions=store.list_sessions(),
            saved=request.query_params.get("saved") == "1",
        )

    @app.post("/s/{asset}/{session_id}/{round_name}/feedback")
    def submit_feedback(
        request: Request,
        asset: str,
        session_id: str,
        round_name: str,
        choice: str = Form(default=""),
        text: str = Form(default=""),
    ) -> Response:
        """Write the feedback of one round, then show the round again."""
        letter = choice if choice in VARIANT_LETTERS else None
        try:
            store.write_feedback(asset, session_id, round_name, letter, text)
        except ContractError as error:
            return page("error.html", request, message=str(error))
        target = f"/s/{asset}/{session_id}/{round_name}?saved=1"
        return no_store(RedirectResponse(target, status_code=303))

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

    return app


app = create_app()


def main() -> None:
    """Run the server for one person on the loopback address."""
    import uvicorn

    parser = argparse.ArgumentParser(description="the direct-die review server")
    parser.add_argument("--host", default="127.0.0.1", help="the address to bind")
    parser.add_argument("--port", type=int, default=8765, help="the port to bind")
    parser.add_argument(
        "--sessions",
        type=Path,
        default=None,
        help="the sessions root to read; the environment sets it otherwise",
    )
    arguments = parser.parse_args()
    if arguments.sessions is not None:
        os.environ[ROOT_VARIABLE] = str(arguments.sessions)
    uvicorn.run(
        create_app(arguments.sessions), host=arguments.host, port=arguments.port
    )


if __name__ == "__main__":
    main()
