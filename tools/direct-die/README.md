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

Use the loop to make four candidates and a fault list. Use a person to
choose. Do not leave the loop running unattended and expect the score to
climb.
