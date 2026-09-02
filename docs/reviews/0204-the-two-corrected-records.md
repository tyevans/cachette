# Review 0204: The two corrected records

## What was reviewed

| Item | Value |
|---|---|
| `docs/adrs/draft/adr-0088-a-tile-field-is-a-generated-base-and-a-stored-change.md` | `Draft` at review, and `Draft` after it |
| `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md` | `Draft` at review, and `Draft` after it |
| Commit | `64e1d96`, the head of `main` at the time of the review |
| Correction reviewed | `da4a5d6`, which repairs the two claims that review 0199 returned |
| Code read | the terrain module, the upgrade module, the position table, the founding, the world step and its admission pass, the tile value field, the pyramid, the head-up display and the drawing pass |

The reviewer wrote neither record and neither correction. The reviewer wrote
review 0199, which returned both, and so knew what it was looking for. That is
a weaker position than a fresh reader, and section 4 records what the reviewer
did about it: it tested the author's argument against the accepted record
rather than against its own earlier wording.

**The reviewer compiled nothing.** Other workers hold the machine.

## Verdict

| Record | Verdict |
|---|---|
| ADR-0088 | Accept. The bullet now says what the code does, and every other decision held at the previous review |
| ADR-0090 | Accept. The key is now enforcement, which is true and which an accepted record already fixes |

**Neither record was moved to `Accepted` in the registry, and the reason is
mechanical.** Section 5 holds it. Accepting a record moves its file from
`draft/` to `accepted/`, and every citation that names the old path then names
a path that does not resolve. Nineteen of them are in source comments under
`crates/`, and this work is forbidden to touch that tree. The verdict above is
the reviewer's judgement. The status change is one `git mv` and one sweep, and
whoever may edit `crates/` can make both.

One registry repair was made, because it needs no move: row 0090 did not list
ADR-0074 among its dependencies, and decision D3 now rests on it.

## 1. ADR-0088, the corrected bullet

The bullet now reads:

> - Building the field visits no tile and allocates nothing for it. The seed
>   and the extent are the whole of a new field. Building a world still visits
>   each tile, because the first level of the pyramid sums the value of every
>   tile. The consequences below name that reader and the item that holds its
>   removal.

**It says what the code does.** The field constructor is a `const fn` that
holds the seed, the extent and two empty vectors, so building the field visits
no tile and allocates nothing. Building a world closes by filling the first
level of the pyramid, and that fill sums the tile value of every tile of every
block. The build test asserts one visit for each tile rather than none, and the
register already states plainly that a world is still not built without a pass
over every tile.[^1]

**The record no longer contradicts itself.** Its consequences say the first
level of the pyramid sweeps the field, and name the item that holds the removal
of that pass. The bullet now points at that paragraph instead of denying it.

**The prose rule is met.** The four sentences run 11, 11, 19 and 13 words,
each states one idea, each is active, and the bold span is gone. The bullet
states no count, which matters: the test asserts one visit for each tile today,
and an item that removes the remaining pass would make a number in the record
false.

The reviewer re-read D2, D3 and D4 and found nothing that the correction
disturbed.

## 2. ADR-0090, the corrected key

The key now reads:

> **One function reads both tables, and every caller that enforces the capacity
> of a tile calls it.**

followed by a paragraph that says a caller which asks a different question
reads the table its question is about, names the two kinds of such caller, and
cites the record that makes admission the only enforcer.

### 2.1 The author's argument is correct, and the reviewer's text was not

The author declined the reviewer's replacement text, which read "every caller
that asks how many units may stand on a tile". Its reason is that the founding
asks exactly that question and reads the ground alone, so the narrowed
universal would still have been false.

**The reviewer tested this and the author is right.** The founding fills each
tile of its disc up to the capacity of that tile's ground, and it reads the
terrain and nothing else. That is the question "how many units may stand on
this tile", answered from one of the two tables. The reviewer's wording would
have moved the falsehood rather than removed it, and review 0199 would then
have returned a record into a second wrong state.

### 2.2 The enforcement key holds, and an accepted record fixes it

The key is not the author's judgement. An accepted record already states it in
its own decision: admission grants no intent that raises a tile above the
capacity of its ground, and that record calls admission the only
enforcer.[^2]

The same record makes the other half explicit. A spawn places a unit without
reading the tile capacity, and a caller that wants a tile filled to its
capacity counts its own placements, asking the terrain rather than the
world.[^3] [^4] The founding is named in that decision by its behaviour. So
the two callers the corrected paragraph describes are not exceptions this
record invents. They are the shape an accepted record already chose.

**The claim is therefore true, and it is checkable.** The composition has two
callers: the public reader that reports what a tile holds, and the admission
pass. Admission is the only enforcer. Every enforcer calls it.

### 2.3 The record correctly does not list the call sites

The corrected paragraph describes two kinds of caller and names no file. That
is right, and it would have been wrong to do otherwise: the scope rule bans a
file table and a survivor list from a record, because the next change to the
tree makes them false.[^5] The commit message holds the table of three callers
and the search that produced it, which is where that material stays true.[^6]

### 2.4 Where the key has no hole, and where the system does

The reviewer went looking for a third enforcer and found none. The invariant
that bounds the positions of a site refuses a state, which is the closest thing
to a second enforcement in the tree. It bounds the work a site opens and not
the units that stand on a tile, and the corrected paragraph names it as a
different question. Under the accepted record that reserves enforcement to
admission, that is the correct reading.[^2]

**The hole is in the system and not in the key.** A fourth caller answers the
capacity question from the ground alone, and review 0199 did not find it: the
drawing pass counts a painted tile as at its capacity when the units on it
reach the capacity of its ground, and it paints an over-full marker above that.
On a tile with a finished made way, admission admits units past that number, so
a watcher would read a correctly filled tile as over-full.

The viewer enforces nothing, because it never writes to the world, so ADR-0090
D3 stays true.[^7] What this changes is the register: the finding named three
callers inside the core, and the fourth is the one a person sees. The finding,
the open row and the item that holds the work all now name it.[^8] [^9] [^10]

## 3. The registry repair

Row 0090 lists ADR-0012, ADR-0015, ADR-0056, ADR-0066, ADR-0068 and ADR-0072
as its dependencies. Decision D3 now rests on ADR-0074 D2 for its key, and the
record cited ADR-0074 D3 before the correction as well. The row named neither.

The reviewer added ADR-0074 to that row. **Review 0199 missed this**, and it is
the kind of thing the dependency column exists for: a reader who changes
ADR-0074 must be able to find every record that would move with it.

The record file was not touched.

## 4. Every objection the reviewer attempted

**Objection 1: the enforcement key is a word the author chose to make its own
text true.** **Failed.** It is the word an accepted record uses, in a decision
titled for it. The reviewer read that record before reading the correction
again, so that the test was against the project's rule rather than against the
reviewer's earlier wording.

**Objection 2: the position invariant is a second enforcer.** **Failed.** It
bounds the positions a site opens, which is work rather than standing room, and
the corrected paragraph names it as a different question. The accepted record
reserves enforcement of a tile capacity to admission.

**Objection 3: the corrected paragraph names two callers and the author
verified three.** **Failed.** The paragraph leads with the general rule and
gives examples. Naming all three would be the survivor list that the scope rule
bans, and the commit message already holds the table.

**Objection 4: the drawing pass is a third enforcer, so the key is false.**
**Failed, and it found something.** The viewer reads the ground alone and would
report a roaded tile as over-full, but it writes nothing to the world, so it
enforces nothing. The registers gained the caller.

**Objection 5: ADR-0088's bullet still promises something the build does not
do.** **Failed.** It now separates building the field from building a world,
and both halves match the test and the finding.

**Objection 6: ADR-0088's bullet states a count.** **Failed.** It says the
build visits each tile and gives no number, so an item that removes the
remaining pass makes the sentence obsolete rather than false.

**Objection 7: the correction introduced a footnote defect.** **Failed.** Two
footnotes were added at 13 and 14 where they first occur, and the labels below
them shifted so the order still ascends. The footnote check lists neither
record.

**Objection 8: row 0090 does not name the record its key rests on.** **Held.**
Section 3.

## 5. Why neither status changed

The registry says a record file moves between directories as its status
changes, and the layout table gives `draft/` to records under review and
`accepted/` to accepted records.[^11]

A citation of a record names the path, and the path holds the directory. The
citation check fails when a footnote names a `docs/` path that does not resolve
on disk, and it reads source comments as well as documents.[^12] Moving
ADR-0090 therefore breaks nineteen citations in source comments under
`crates/`, and moving ADR-0088 breaks one. This work is forbidden to edit that
tree, and other workers hold those files.

So the acceptance is a `git mv`, a sweep of every citation of the old path, and
one edit to the status and the source column of each registry row. The reviewer
can do the last of those and not the first two. **A partial acceptance is the
worse outcome**: a record that reads `Accepted` while sitting in `draft/`, with
nineteen source comments calling a binding record a draft, is one fact in two
places with nothing that fails when the copies disagree.

The reviewer records the verdicts and leaves the rows. Both records are ready,
and the priority index now says so in their rows.

**This is a defect in the citation convention rather than a property of these
two records.** It falls on every acceptance, it grows with how well a record is
cited, and nothing reports it until the check runs. The register holds the
finding and the choice.[^13] [^14]

## 6. What only a run can settle

- **Every test named in this review passes.** Each was read, not run.
- **A watcher sees the over-full marker on a roaded tile.** The reviewer
  derived this from the drawing pass and the capacity constants. No run
  produced it, and no run reaches it today, because no engine rule issues a
  build order.

## 7. Checks run

Five document checks were run. All five pass. They compile nothing.

| Check | Result |
|---|---|
| `scripts/check_adrs.py` | 0 failures |
| `scripts/check_footnotes.py` | 0 failures |
| `scripts/check_priority.py` | 0 failures |
| `scripts/check-citations.sh` | 0 failures |
| `scripts/check_conflict_markers.py` | 0 failures |

## References

[^1]: Findings register, FND-162. `docs/FINDINGS.md`
[^2]: ADR-0074, a spawn may over-fill a tile, and only admission enforces the capacity, decision D2. `docs/adrs/accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
[^3]: ADR-0074, a spawn may over-fill a tile, and only admission enforces the capacity, decision D1. `docs/adrs/accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
[^4]: ADR-0074, a spawn may over-fill a tile, and only admission enforces the capacity, decision D4. `docs/adrs/accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
[^5]: Decision Record Scope, section 4.3. `.claude/rules/adr-scope.md`
[^6]: Commit Message Rules. `.claude/rules/commits.md`
[^7]: ADR-0067, the viewer reads the world and never writes to it. `docs/adrs/REGISTRY.md`
[^8]: Findings register, FND-193. `docs/FINDINGS.md`
[^9]: Decisions register, DEC-081. `docs/DECISIONS.md`
[^10]: Backlog item 0200, give one answer to how many units a tile holds. `docs/backlog/proposed/0200-give-one-answer-to-how-many-units-a-tile-holds.md`
[^11]: ADR Registry, the layout. `docs/adrs/REGISTRY.md`
[^12]: The citation check script. `scripts/check_citations.py`
[^13]: Findings register, FND-197. `docs/FINDINGS.md`
[^14]: Decisions register, DEC-083. `docs/DECISIONS.md`
