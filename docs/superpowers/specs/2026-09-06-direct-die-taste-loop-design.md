# The direct-die taste loop

This design adds a taste loop to the direct-die tool. A person marks the
drawings they like and the drawings they refuse. The tool then branches from
what they liked, warns the model away from what they refused, and writes an
analysis that says what the two groups differ by.

Direct-die is a proof of concept that draws a game asset as an SVG document
and revises it against a written style guide.[^1] A web front end holds the
human half of that loop.[^2] This design changes both halves.

The tool sits outside the engine gates. It changes nothing under `crates/` and
nothing under `python/cachette/`. This work needs no decision record and no
product requirement record, because it constrains no engine decision.

## 1. Why

The front end today gives a person one radio button for each round. They pick
one variant of four, and they type free text. That is the whole signal.

Three limits follow from it.

- **A person can prefer two drawings.** One radio button makes them discard
  the second, and the next round never sees it.
- **A refusal reaches nothing.** A person who dislikes a drawing has no way to
  say so. The model draws the same fault again in the next round.
- **The score does not carry taste.** The tool records that a tile went from
  eight small shapes to three large ones, which is what the critic asked for,
  and the score moved by five points.[^1] An absolute score is a weak signal
  about a change that large.

This design gives a person three signals instead of one: a like, a refusal,
and an order between the likes.

## 2. Scope

This design covers one of four pieces of a larger request. The other three are
named here so that a reader knows what this piece does not do.

| Piece | State |
|-------|-------|
| The taste loop: like, refuse, branch, analyse | This design |
| Settings and prompt tuning in the browser | A later design |
| The composited preview, an upgrade over a terrain | A later design |
| Numbered pack snapshots that a consumer pins | A later design |

This design creates the prompt builder seam that the second piece and the
third piece both need.

## 3. The feedback file

The front end writes one file into a round directory, and the generation loop
is its only reader. The shape changes.

```json
{
  "round": 1,
  "likes": ["b", "d"],
  "denies": ["a"],
  "order": ["d", "b"],
  "note": "The standing note. Every later round of the session reads it.",
  "text": "A note for the next round only.",
  "at": "2026-09-06T21:40:00Z"
}
```

### 3.1 The fields

- **`round`** holds the index of the round as an integer. The loop refuses
  the file when the index disagrees with the directory. The front end does
  not write this field today, and the loop accepts a file without it. The
  front end now writes it, because a file that names its round is safe to
  copy and a file that does not is not.
- **`likes`** holds the variant letters that the person accepts. Order does
  not matter here.
- **`denies`** holds the variant letters that the person refuses.
- **`order`** ranks the liked letters, best first. It holds a subset of
  `likes`. The first entry is the drawing that stands for the asset.
- **`note`** is the standing note. It carries into every later round of the
  session.
- **`text`** is a note for the next round only.
- **`at`** holds an ISO 8601 time in UTC.

A letter must not appear in both `likes` and `denies`. The writer raises when
it does.

### 3.2 The old field

The field `choice` goes away. Nothing writes it again.

A reader that finds `choice` and no `likes` reads the choice as a one-entry
`likes` list and a one-entry `order` list. Every session already on disk
therefore still opens. This is a read-time rule in one function. It is not a
second write site.

### 3.3 The denylist accumulates

A refusal holds for the whole session. A variant denied in round 1 stays
denied in round 4.

The loop therefore reads the feedback of every round, not only of the last
one. It joins the `denies` lists.

A like does not accumulate. A like is a statement about the next round, and
the person restates it each round.

The standing note does not accumulate either. The loop reads the newest round
that holds a `note`, and it uses that one text. A person who writes a new note
replaces the old one.

## 4. Liked drawings become parents

### 4.1 The rule today

`run_round` picks one parent for the whole round. It gives each of the four
variant lenses that same parent. A lens is a written direction and a
temperature: follow the guide exactly, raise the contrast, use fewer shapes,
take the largest departure.

### 4.2 The rule after this change

The round builds a list of pairs. Each pair holds a parent and a lens. The
round cycles the liked parents across the variant letters.

Two likes and four variants give this assignment.

| Variant | Parent | Lens |
|---------|--------|------|
| a | the first like | follow the guide exactly |
| b | the second like | raise the contrast |
| c | the first like | use fewer shapes |
| d | the second like | the largest departure |

Each parent gets a spread of directions rather than one direction. A person
who likes one drawing gets the behaviour of today.

### 4.3 When nobody liked anything

The present rule holds. The parent is the highest scoring variant of every
round so far, and not only of the last round. A later round wins a tie.

A denied variant is never a parent, whatever it scored.

### 4.4 The round metadata

`meta.json` holds one `parent` string today. It gains a `parents` object that
names a parent for each variant letter.

```json
{
  "round": 2,
  "prompt_summary": "a dense stand of trees; revision",
  "parents": {"a": "round-01/variant-b", "b": "round-01/variant-d"}
}
```

The field `parent` goes away, for the reason that `choice` goes away. A reader
that finds `parent` and no `parents` takes that one value for every variant.

## 5. Refused drawings go into the prompt

The generation step sends text and no pictures. This is what keeps the prompt
inside the 16384 token window of the model. Only the critique step sends
pictures.[^1]

A refused drawing therefore goes into the generation prompt as its SVG source.
The model writes SVG, so it can act on SVG.

The prompt holds a section like this one, below the parent source and above
the notes.

```
DRAWINGS THE ART DIRECTOR REFUSED. Do not draw like these.
-----
<svg ...>...</svg>
-----
```

An SVG document of this tool runs about 800 tokens. The prompt takes the two
most recent refusals. The count is a constant in the prompt builder, and the
second piece of this work makes it a setting.

## 6. The analysis

### 6.1 It is a step of its own

A person ranks, then reads an analysis, then tunes the prompt, then generates.
The analysis therefore runs before a round, not inside one.

It is a command of the tool. The front end calls the tool through its command
line and never imports the tool package.[^2]

**The analysis waits, and a run does not.** A run draws up to eleven subjects
at about seventy seconds each, so a job record holds its state and no page
waits for it. The analysis is one model call. A person clicks the button and
looks at the page until the answer arrives. A job record for a call this short
would be a second state machine for no gain. The request runs the child with a
timeout and shows an error page when it fails.

```
python -m direct_die analyse --asset <style> --session <id> --round <n>
```

### 6.2 What it reads

The command reads `feedback.json` of the named round. It raises when the file
holds no like.

It shows the model the display raster of each liked drawing and of each
refused drawing, beside the rules of the style. Each raster carries a label
that says whether the art director accepted it or refused it.

With two likes and two refusals the call sends four rasters. That is the size
of one critique call, which sends two rasters and the exemplars.

### 6.3 What it writes

The command writes `analysis.json` into the round directory. The tool is the
only writer of that file.

```json
{
  "round": 1,
  "preference": "What the liked drawings share and the refused ones lack.",
  "order": ["d", "b"],
  "reasons": {"d": "why d beats b", "b": ""},
  "guide_edit": {"section": "3. Shape language", "rule": "One sentence."},
  "at": "2026-09-06T21:38:00Z"
}
```

The parser validates the shape, and asks the model once more when the answer
is malformed. This follows the critique parser, which already does this.

`order` holds every liked letter and no other letter. `guide_edit` may be
`null`, which means the model proposes no rule.

### 6.4 Why an order and not a score

The tool records that a large change to a drawing moved the absolute score by
five points.[^1] A fixed score scale with anchors improved this and did not
remove it. An order between two drawings does not have that defect, because it
compares the two drawings and not each drawing against a table.

The order does not replace the score. The score still decides which drawing
wins when no person chose.

## 7. What the person does with the analysis

The round page shows three editable things when `analysis.json` exists.

- **The preference statement.** The person edits the text and saves it. The
  save writes the text into the `note` field of `feedback.json`. It is then
  the standing note of the session.
- **The order.** The person accepts it or reorders it. The save writes the
  `order` field of `feedback.json`.
- **The proposed rule.** The person accepts it, edits it, or rejects it.
  Accepting appends the rule to the style rules file.

The analysis never writes `feedback.json`, and the front end never writes
`analysis.json`. Each file has one writer.

## 8. The rules file gains a writer

The front end README states today that the server changes no rules file.[^2]
This design changes that. Accepting a proposed rule appends one line to
`styleguide/<style>.md`.

The append is atomic, like every other write of this server. It writes a
temporary file in the same directory and renames it.

The style guide version is a digest of the rules and the exemplars, and it
goes into `session.json` when a session starts. An accepted rule therefore
changes the version, and a session that ran before the rule still names the
version it ran against. Nothing else is needed.

The README of the front end states the new contract.

## 9. Which drawing stands for an asset

The pack export decides which drawing stands for one style and one asset.[^2]
The rule reads the sessions newest first and takes the first human choice.

The rule changes in three ways.

1. A human choice is now the first entry of `order`. When `order` is empty and
   `likes` holds one letter, that letter is the choice.
2. When `likes` holds more than one letter and `order` is empty, the rule
   takes the highest scoring of the liked letters.
3. A denied letter never wins, whatever it scored, and whatever the score rule
   would say.

## 10. The parts

| Part | What changes |
|------|--------------|
| `direct_die/session.py` | Read the feedback of every round. Read the old `choice` field. |
| `direct_die/loop.py` | The parent list, the prompt builder, the refusal section, the per-variant metadata. |
| `direct_die/analyse.py` | New. The analysis prompt, the parser, and the writer. |
| `direct_die/cli.py` | The `analyse` command. |
| `review/store.py` | The new feedback shape, the writer, and the old-field reader. |
| `review/app.py` | The like and refusal form, the analysis start, and the three accept routes. |
| `review/packs.py` | The winner rule of section 9. |
| `review/runs.py` | The analysis job, beside the run job. |
| `review/templates/round.html` | The like and refusal controls, and the analysis panel. |

The prompt builder becomes one function that takes named layers: the rules,
the subject, the lens, the parent source, the refused sources, the standing
note, the round note, and the model faults. The second piece of this work
makes those layers editable. The third piece adds no layer.

## 11. Testing

The tests of the front end build a fabricated workspace in a temporary
directory and drive the server through its public interface. The run tests
start a real child process, and a stand-in script takes the same command line
as the tool and writes the same files.[^2] The new tests follow both patterns.

### 11.1 The fixture must hold the case

A fixture in which a person liked every variant cannot prove that a refusal
reaches the prompt. The prompt tests therefore run against a round with one
like and one refusal, and against a round with two likes.

The project rule is to put the defect back and watch the test stay green.[^3]
Two tests do this.

- Remove the refusal section from the prompt builder. The prompt test must
  fail.
- Give every variant the first parent, and ignore the rest. The branching test
  must fail.

### 11.2 The cases

- A feedback file that holds `choice` and no `likes` reads as one like.
- A letter in both `likes` and `denies` raises.
- A refusal in round 1 is still in the prompt of round 3.
- Two likes give two distinct parents across four variants.
- One like gives the behaviour of today.
- No like and no refusal gives the highest score of every round.
- A denied variant with the highest score does not win the asset.
- A malformed analysis answer asks once more, then raises.
- An analysis with no like raises.
- Accepting a proposed rule appends one line and changes the guide version.
- A round with no `analysis.json` renders, and the panel says so.

### 11.3 The gates

The tool is not part of any engine gate. Its own gate is `python -m pytest`
inside `tools/direct-die` and inside `tools/direct-die/review`, and the lint
that `review/ruff.toml` configures.

## 12. What this design does not do

- It does not edit the rules file freely. It appends one proposed rule.
- It does not edit the lenses or their temperatures.
- It does not composite an upgrade over a terrain.
- It does not number a pack.
- It does not change any engine code.

## References

[^1]: The direct-die tool guide. `tools/direct-die/README.md`
[^2]: The direct-die front end guide. `tools/direct-die/review/README.md`
[^3]: Testing Rules, section 2a. `.agents/rules/testing.md`
