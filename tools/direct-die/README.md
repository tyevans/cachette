# direct-die

direct-die is a proof of concept. It draws a game asset as an SVG
document, rasterises the document at two sizes, shows both rasters to a
vision model, and revises the drawing from the critique that comes back.
A living style guide holds the rules and the accepted exemplars.

This tool is a side project. It changes no part of the simulation engine
and no part of the Python control plane. It is not part of any engine
gate.

## What one round does

1. Each variant produces or revises one SVG document.
2. The tool rasterises the document at the display size and at the
   inspection size.
3. The tool shows both rasters to the model, beside the guide rules and
   the accepted exemplars, and asks for a critique.
4. The critique comes back as JSON with a verdict, a fault list, and a
   score. The tool validates it, and asks once more when the answer is
   malformed.
5. The next round revises the parent that the critique or the human
   chose.

Each round makes up to four variants, `a` to `d`. Each variant has its
own direction and its own temperature, so the four drawings differ. One
variant follows the guide exactly. One raises the contrast. One cuts the
shape count. One takes the largest departure that the rules permit.

## Two sizes, always

An asset is judged where a player sees it. A hex tile on the map is 64
pixels. A drawing that only reads at 512 pixels is of no use. The tool
therefore renders every asset twice, and it labels each raster for the
model. The size table lives in one place in the render module.

## Install

```
cd tools/direct-die
python3 -m pip install -r requirements.txt
```

The endpoint needs no key. Set `DIRECT_DIE_BASE_URL` to use a different
host.

## Use

```
python3 -m direct_die probe
python3 -m direct_die guide
python3 -m direct_die run --asset hex-tile --subject "a dense pine forest" --rounds 3
python3 -m direct_die run --asset hex-tile --subject "a dense pine forest" --session 20260906-190000 --rounds 1
python3 -m direct_die set --style cartoon --rounds 2
python3 -m direct_die set --style pencil --subjects forest,water --rounds 1
```

The `--session` option continues a session that is already on disk. It
adds rounds after the last round it finds. Use it after a person writes
feedback through the review interface.

## The score

The score runs from 0 to 100. It has fixed anchors, so a score means the
same thing in every round, and two rounds are comparable. The scale
lives in one file in the style guide directory. That file is the only
declaration of the scale. The critique prompt reads it, and the review
interface shows the score that comes back.

## The human outranks the model

The review interface writes `feedback.json` into a round directory. This
tool reads that file and never writes it. When the file names a choice,
the chosen variant becomes the parent of the next round. When the file
holds text, that text goes into the next prompt above the model
critique, and the prompt says that the human direction outranks every
model note.

The file names the round that it belongs to. This tool refuses the file
when that name disagrees with the directory, because a mismatch means
one of the two is wrong, and neither is safe to guess from.

When no feedback exists, the parent is the highest scoring variant of
every round so far, and not only of the last round. A critique names a
fault even in a good drawing, and a revision that acts on that fault can
make the drawing worse. The loop must not walk away from its best work
when that happens. A later round wins a tie, so the loop still moves.

## The layout on disk

```
sessions/<asset>/<session-id>/
  session.json
  round-00/
    meta.json
    variant-a.svg
    variant-a.png            the display size render
    variant-a.large.png      the inspection size render
    variant-a.critique.json
    feedback.json            the review interface writes this file
  round-01/
```

Every write goes to a temporary name in the same directory and then
renames, so a reader never sees half a file.

`session.json` holds the asset, the time, the model, and the guide
version. It holds no round count. The round directories are the record
of how many rounds ran. A second copy of that number would disagree with
the directories while a round is part way through, and nothing would
fail.

## The style guide

`styleguide/<asset>.md` holds the rules. `styleguide/exemplars/<asset>/`
holds the accepted exemplars. An exemplar is an SVG file or a PNG file.
The loader rasterises an SVG exemplar at the inspection size.

To add an asset type, add a rules file, add a row to the size table in
the render module, and add at least one exemplar. To accept a drawing,
copy its SVG into the exemplar directory. The guide version is a digest
of the rules and the exemplars, and it goes into `session.json`.

Keep the rules few and concrete. A vague rule gives a vague critique.

## A style is an asset type

The tool has one mechanism for a rules file, and a style needs nothing
more than that. `--asset X` and `--style X` both read
`styleguide/X.md` and `styleguide/exemplars/X/`. The size table gives a
default of 64 pixels display and 384 pixels inspection to any name that
no row holds. A style therefore needs a rules file and nothing else.

Four styles ship: `cartoon`, `pencil`, `elegant`, and `sandcastle`.

### How to add a style

1. Write `styleguide/<style>.md`. Copy the section order of a shipped
   style. State the geometry, the palette by hex value, the line
   weight, the shape language, the detail budget, the silhouette rule,
   and what the style forbids.
2. State what varies **by subject** and what stays fixed **by style**.
   Without that table the critic pulls every subject toward the nearest
   exemplar. It once told a pine forest to drop its triangles, because
   both exemplars were rounded.
3. State what survives at 64 pixels. A rule that only reads at 384
   pixels is the wrong rule for a hex tile.
4. Do not restate the score scale. That scale has one declaration site,
   and every asset type shares it.
5. Add the style to the style list in `tests/test_styles.py`. Those
   tests check that the guide loads, that it names every subject of the
   asset set, that it states what it forbids, and that it does not
   restate the score scale.
6. Add a size row in the render module only when the style needs a size
   that the default does not give.

A style that would accept the same drawing as another style is not a
second style. Give each style a rule that the others break.

### The asset set

One module declares the subjects: five terrain kinds and six upgrade
categories. The engine also has an `OPEN` upgrade category, which means
no upgrade and needs no art. A style guide names the same subjects in
its own subject table, and a test fails when one is missing. Add a
subject in the subject module, then add a row to each style guide.

### How to promote a drawing to an exemplar

An exemplar directory starts empty, and every new style has this
problem. The first run of a style therefore judges against the rules
alone. Bootstrap it this way.

1. Run one subject at one round and read the four drawings through the
   review interface.
2. Choose the drawing that shows the style best. Prefer a drawing that
   shows a rule the prose states weakly.
3. Copy its SVG into `styleguide/exemplars/<style>/<subject>.svg`. Name
   the file for the subject, because the critic reads the label.
4. Run the next subject. The critic now has a picture.

Ship an exemplar for a subject before you ask for that subject, or
expect the critic to aim at the exemplar that exists. Two rounded
exemplars pull a conifer toward a hump. Three or four exemplars that
differ from each other by subject pull nothing.

The guide version is a digest of the rules and of the exemplars, so a
new exemplar changes the version in `session.json`. A session that ran
before the exemplar is still readable, and its header says which guide
it ran against.

## Cost

The model has a 16384 token window. One critique costs about 5000 prompt
tokens with two exemplars and two candidate rasters. Use `--exemplars`
to attach fewer exemplars when the prompt gets close to the window.

The generation step sends text only. Only the critique step sends
pictures. This keeps the loop inside the window.

The model thinks before it answers unless a caller stops it. A long
thought fills the token budget and leaves the answer empty. The client
therefore stops the thinking by default.

## Known limits

The loop does not converge on its own. Three findings support this, and
each one is a reason to keep a person in the loop.

- **The critic contradicts itself between variants.** It told a tile
  with eight shapes to use fewer, and it told a tile with three shapes
  to use more. Each note is correct against the guide. Together they
  make the loop cross the same boundary in both directions.
- **The critic aims at the nearest exemplar, not at the subject.** The
  guide ships a grass exemplar and a water exemplar, and both use
  rounded shapes. The critic then told a pine forest to replace its
  triangles with rounded shapes. Ship an exemplar for the subject you
  ask for, or expect the critic to pull the drawing toward the wrong
  one.
- **A revision that acts on a fault can make the drawing worse.** One
  session reached 75, then acted on the note "add a darker base for
  depth", and fell to 45. The loop now keeps the best parent of every
  round for this reason, but the fault list still points the wrong way.
- **The score moves less than the drawing does.** In an early session
  a tile went from eight small shapes to three large ones, which is
  what the critic asked for, and the score moved by five points. A
  fixed score scale with anchors improved this. It did not remove it.

- **A score compares two rounds of one style. It does not compare two
  styles.** One bootstrap run drew the same forest in four styles at one
  round each. Two styles reached 95 and two reached 25 and 40. The
  spread measures how hard each guide is to satisfy, not how good the
  drawing is. Compare a score only inside one style.
- **A guide that is easy to satisfy stops the critic from
  discriminating.** In that same run every variant of one style scored
  95 with an empty fault list, and one of those drawings did not read as
  its subject. A guide of few mechanical rules gives the critic nothing
  to fault, so it accepts a wrong subject. Add a rule that names the
  subject silhouette when this happens.
- **The critic judges the raster, not the source.** It faulted a
  stroke-only drawing for using solid fills. The strokes were dark and
  close, so they read as solid at the display size. The note is wrong
  about the source and right about the picture.

Use the loop to make four candidates and a fault list. Use a person to
choose. Do not leave the loop running unattended and expect the score to
climb.
