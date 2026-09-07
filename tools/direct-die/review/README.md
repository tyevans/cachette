# The direct-die front end

Direct-die is a proof of concept. It makes an SVG game asset by a loop: a
vision model draws four variants, critiques its own drawings against a style
guide, and draws again. This directory holds the human half of that loop. A
person opens it in a browser and drives the whole tool from one place.

The server is not part of the engine. It changes nothing under `crates/` and
nothing under `python/cachette/`. No engine gate runs it.

## The five things a person does

1. **Start.** Choose a style and an asset, or the whole set of eleven assets,
   and start a run. The page returns at once. The work runs in the background.
2. **Steer.** Read the four variants of a round side by side, read the
   critique the model wrote for each one, pick one, and type feedback. The
   server writes `feedback.json`, and the loop reads it on its next round.
3. **Promote.** Make a drawing an exemplar of its style. The critic judges
   against pictures, and an exemplar directory starts empty. This is the step
   that makes the next run better.
4. **See the packs.** Read the grid of every asset against every style. It
   says what is drawn, what is chosen, and what is missing.
5. **Export.** Write the chosen drawings of one style as a pack on disk, which
   something else can load.

## The unfinished state is the interface

One round of four variants takes about seventy seconds, and a whole set is
eleven assets. A person therefore spends most of the time looking at work that
is part way through.

No page waits for a run. Every page renders what is on disk now and marks the
rest as not yet arrived. A round with two of four variants shows two pictures
and two gaps. A variant whose picture is written and whose critique is not
shows the picture and says the critique is coming. Every page names what the
tool waits for, in words: "round 2 of 3, variant c drawing".

The grid, the run pages and the session page reload themselves while work is
in flight, and stop when the last job ends. The round page never reloads on
its own, because a person types feedback into it. It asks the server for a
short state string instead, and offers a reload when that string changes.

## A style is an asset type

The tool takes `--asset X`, reads `styleguide/X.md` for the rules, and reads
`styleguide/exemplars/X/` for the pictures. A style is therefore an asset
type. This front end calls each one a style.

The thing that the drawing shows is the asset name. The names are the
engine's own: `water`, `plain`, `forest`, `hill` and `mountain` for terrain,
and `road`, `terrace`, `wonder`, `store`, `wall` and `lodging` for upgrades. A
later consumer of a pack therefore needs no mapping table.

## Which drawing stands for an asset

The person outranks the model. The rule reads the sessions of one style and
one asset, newest session first, and takes the first human choice it finds.
When no person chose, it takes the highest score of every round of every
session, and a later round wins a tie.

The tool records no asset name in the session manifest. The server names a
session after the asset when it starts a run, so the directory name carries
the name. A run from the command line has no such name, and the rule then
reads the subject text out of the round metadata and looks for an asset name
in the words. Nothing stores the asset name twice.

## Start it

The server needs a Python interpreter with FastAPI, Jinja2 and uvicorn. The
fixture script also needs CairoSVG.

```
cd tools/direct-die/review
python make_fixtures.py /tmp/direct-die-fixtures
python app.py --sessions /tmp/direct-die-fixtures/sessions \
              --styleguide /tmp/direct-die-fixtures/styleguide \
              --packs /tmp/direct-die-fixtures/packs \
              --runs /tmp/direct-die-fixtures/runs
```

Open `http://127.0.0.1:8765` in a browser.

Run it against the real tree with no argument at all.

```
python app.py
```

It then reads four directories beside the tool: `sessions`, `styleguide`,
`packs` and `runs`. Four environment variables set the same paths:
`DIRECT_DIE_SESSIONS`, `DIRECT_DIE_STYLEGUIDE`, `DIRECT_DIE_PACKS` and
`DIRECT_DIE_RUNS`.

The server binds the loopback address and has no authentication. One person
uses it on one machine.

## How a run happens

The server calls the tool through its command line, and never imports the
tool package. The command is one line:

```
python -m direct_die run --asset <style> --subject <text> \
    --session <identifier> --rounds <n> --variants <n>
```

The server starts that command as a child process in its own session, and
returns to the browser at once. One worker thread runs the subjects of a job
one after another, because the endpoint answers one request at a time and one
machine carries the whole load.

A job record holds the state: `runs/<job>/job.json`, and one log file for each
subject beside it. Every page reads that record again.

### When the server dies mid-run

The child survives the server, because it runs in its own process session. The
supervisor thread does not survive. The job record therefore holds the
identifier of the process that supervises it, and a reader that finds a
running job under another process reports the job as abandoned.

The drawings are safe. The tool writes each file to a temporary name and
renames it, so a session directory holds whole files only. A person starts the
subject that was in flight again and keeps the rest.

## The pack on disk

```
tools/direct-die/packs/<style>/
  pack.json
  <slug>.svg
  <slug>.png        the display size render
```

`pack.json` holds `style`, `created`, and an `assets` table. Each entry holds
`svg`, `png`, `score`, `session`, `round` and `variant`.

A pack with gaps is legal and normal. The manifest names what is present and
nothing else. A second export removes a file that the manifest no longer
names, so a pack never holds a drawing that the manifest does not report.

## The on-disk contract of a session

Another agent owns the generation loop. It writes every file below except
`feedback.json`.

```
tools/direct-die/sessions/<style>/<session-id>/
  session.json                  {asset, created, model, guide_version}
  round-00/
    meta.json                   {round, prompt_summary, parent}
    variant-a.svg
    variant-a.png               the display-size render
    variant-a.large.png         the inspection-size render
    variant-a.critique.json     {verdict, faults, score}
    variant-b.*  variant-c.*  variant-d.*
    feedback.json               this server writes this file
  round-01/ ...
```

The `choice` field of the feedback holds `"a"`, `"b"`, `"c"`, `"d"`, or
`null`. The `at` field holds an ISO 8601 time in UTC.

## What this server writes

- `feedback.json` in a round directory.
- A job record and its logs under the runs directory.
- An exemplar SVG under `styleguide/exemplars/<style>/`, when a person
  promotes a drawing.
- A pack under the packs directory.

It writes nothing else. It changes no drawing, and it changes no rules file.

Every write is atomic. It writes a temporary file in the same directory, then
renames it, so a reader never sees half a file.

## A half-written directory is the normal case

The generation loop runs while a person reads the pages. A round can hold two
variants and not four. A critique can be absent. A session can hold no round.
A style can have no run at all. Every one of these renders as a gap on the
page. None of them raises.

The server caches nothing, and it tells the browser not to cache. A page
reload shows what is on disk now.

## The fixture script

The fixture script fabricates a workspace with plain SVG hexagons. It costs no
model time, so anyone can test the server with it. It writes the healthy case
and the broken cases on purpose:

- A round that holds one variant.
- A round whose variants have no critique.
- A round that holds an SVG and no render, and no `meta.json`.
- A round whose critique file holds broken JSON.
- A session that holds no round.
- A session that has no `session.json`.
- A session that names no asset, so only the subject text gives one.
- A style guide with two styles and no exemplar.
- An empty packs directory and an empty runs directory.

## The tests

```
python -m pytest
```

The tests build the fixture workspace in a temporary directory and drive the
server through its public interface. They cover the broken cases above, the
write path, the file routes, the promotion, the pack export, and a run.

The run tests start a real child process. They do not call the vision model. A
stand-in script takes the same command line as the tool and writes the same
files, so the run manager, the job record and the pages all run for real.

One test proves that a start does not wait: the child blocks on a file that
the test writes at the end, so the start request cannot return unless it left
the child running.

## The files

| Path | Contents |
|------|----------|
| `app.py` | The routes and the server entry point |
| `store.py` | The reader of the session tree, and the feedback write |
| `runs.py` | The background run manager and the job record |
| `packs.py` | Which drawing wins, and the pack export |
| `exemplars.py` | The promotion into the style guide |
| `matrix.py` | The state of one cell, and the words for a wait |
| `slugs.py` | The engine asset names, and a subject for each |
| `make_fixtures.py` | The fixture script |
| `fake_tool.py` | The stand-in tool that the run tests call |
| `test_review.py` | The tests of the round pages and the feedback write |
| `test_frontend.py` | The tests of the grid, the promotion, the export and a run |
| `templates/` | The server-rendered pages |
| `static/review.css` | One stylesheet, with no build step |
