# The direct-die review server

Direct-die is a proof of concept. It makes an SVG game asset by a loop: a
vision model draws four variants, critiques its own drawings against a style
guide, and draws again. This directory holds the human half of that loop. A
person opens it in a browser to steer one session.

The server is not part of the engine. It changes nothing under `crates/` and
nothing under `python/cachette/`. No engine gate runs it.

## What a person does with it

1. Read the four variants of a round side by side. The page shows the
   display-size render at its own pixel size, because that is the size at
   which the asset lives on a hex map. An `Inspect` panel holds the large
   render and a link to the SVG source.
2. Read the critique that the model wrote for each variant, beside the
   picture. A person can then see where the model and their own eye disagree.
3. Pick one variant with a click, or pick none.
4. Type free text feedback on the round.
5. Submit. The server writes `feedback.json` into the round directory.
6. Move between the rounds of a session, and between sessions.

The generation loop reads `feedback.json` on its next round.

## Start it

The server needs a Python interpreter with FastAPI, Jinja2 and uvicorn. The
fixture script also needs CairoSVG.

```
cd tools/direct-die/review
python make_fixtures.py            # writes tools/direct-die/fixtures/sessions
python app.py --sessions ../fixtures/sessions
```

Open `http://127.0.0.1:8765` in a browser.

Run it against the real tree with no argument at all. It then reads
`tools/direct-die/sessions`, which the generation loop writes.

```
python app.py
```

The `DIRECT_DIE_SESSIONS` environment variable sets the same path.

The server binds the loopback address and has no authentication. One person
uses it on one machine.

## The on-disk contract

Another agent owns the generation loop. It writes every file below except
`feedback.json`. This server writes `feedback.json` and nothing else.

```
tools/direct-die/sessions/<asset-name>/<session-id>/
  session.json                  {asset, created, model, guide_version, rounds}
  round-00/
    meta.json                   {round, prompt_summary, parent}
    variant-a.svg
    variant-a.png               the display-size render
    variant-a.large.png         the inspection-size render
    variant-a.critique.json     {verdict, faults, score}
    variant-b.*  variant-c.*  variant-d.*
    feedback.json               {choice, text, at}
  round-01/ ...
```

The `choice` field holds `"a"`, `"b"`, `"c"`, `"d"`, or `null`. The `at` field
holds an ISO 8601 time in UTC.

The server writes `feedback.json` atomically. It writes a temporary file in
the same directory, then renames it. The generation loop therefore never reads
half a file.

## A half-written directory is the normal case

The generation loop runs while a person reads the pages. A round can hold two
variants and not four. A critique can be absent. A session can hold no round.
Every one of these renders as a gap on the page. None of them raises.

The server caches nothing, and it tells the browser not to cache. A page
reload shows what is on disk now.

## The fixture script

The fixture script fabricates a session tree with plain SVG hexagons. It costs
no model time, so anyone can test the server with it. It writes the healthy
case and the broken cases on purpose:

- A round that holds one variant.
- A round whose variants have no critique.
- A round that holds an SVG and no render, and no `meta.json`.
- A round whose critique file holds broken JSON.
- A session that holds no round.
- A session that has no `session.json`.

```
python make_fixtures.py /tmp/my-fixtures
```

## The tests

```
python -m pytest test_review.py
```

The tests build the fixture tree in a temporary directory and drive the server
through its public interface. They cover the broken cases above, the write
path, and the file routes.

## What the server refuses

The file route serves a variant SVG and a variant render only. It refuses any
other name in a round directory, so a request cannot read `session.json` or a
file outside the tree. A path segment that holds a separator or a dot-dot is
refused as well.

## The files

| Path | Contents |
|------|----------|
| `app.py` | The routes and the server entry point |
| `store.py` | The reader of the session tree, and the one write |
| `make_fixtures.py` | The fixture script |
| `test_review.py` | The tests |
| `templates/` | The server-rendered pages |
| `static/review.css` | One stylesheet, with no build step |
