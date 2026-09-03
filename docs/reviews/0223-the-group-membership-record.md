# Review 0223: The group membership record

## What was reviewed

| Item | Value |
|---|---|
| `docs/adrs/draft/adr-0065-a-group-is-a-site-membership-not-a-region.md` | `Draft` at review, and `Draft` after it |
| Commit | `d124bfa`, the head of the review branch |
| Code read | the position table in full, the terrain capacity table, the upgrade capacity composition, the world invariant check and its position pass, and the site position tests |

The reviewer did not write this record and has no record of its own that cites
it.

**The reviewer compiled nothing.** Other workers hold the machine, and one of
them holds the terrain module, so section 1 states the commit its reading was
taken at.

## Verdict

**Accept with amendment.** D1's central claim is right, implemented and worth
binding: a group is a stored membership and never a spatial test. D2 is right
and its check is wired into the world's invariant pass.

Two statements are false, and one of them is bold and inside a decision.
Section 5 gives exact text for both, and section 6 files the open choice that
the second one uncovers.

The status stays `Draft`. It would have stayed there anyway: the record is
cited by path ten times, in the world module and in a Python test, and document
work may not move it.

## 1. The objection that held first: the two bounds are not one bound

D3 says, in bold:

> A membership held at a place cannot be larger than what can stand in that
> place. The ground of a tile states how many units it admits. That table is
> the one declaration of the bound. The width of a member row is folded from
> the same table, so raising a capacity raises the row width. **The two bounds
> are one bound.**

**The terrain module holds two capacity constants and the fold walks one of
them.** The fold that gives the row width walks the terrain kinds. Beside it in
the same module sits the crossing capacity of a made way, which the fold does
not see, and whose own comment says no terrain kind carries it.

**Admission composes both and takes the larger.** The composition reads the
ground and the finished upgrade, and a finished road returns the crossing
capacity.

**The position table reads the ground alone**, and clamps to the row width, and
says in its own comment that the answer comes from the terrain capacity table
and from nowhere else.

So on a tile with a finished road, more units may stand than the membership can
hold. The two bounds are two bounds, and the record states the opposite as its
emphasis.

**This is already recorded, and the record is the second one to make the
claim.** A finding holds the evidence and states that the first record to claim
the universal was the upgrade record, which a review corrected.[^1] An open
decision row holds the three options and recommends one.[^2] This record was
written against a belief the register had already refuted.

## 2. The objection that held second: the register resolved the military case the other way

D1 says:

> The civilian case and the military case are one claim. A workforce is a
> membership held by a site. A formation is a membership held by a command
> node. The register already resolved the military case this way.

**The register resolved it the other way round.** The blocker is resolved and
its text says formation membership is an ownership column plus a reverse index.
An ownership column is stored on the member: the unit carries the identity of
its group. A reverse index is derived from that column.

This record's D1 stores the membership on the group: the group holds a row of
entries and each entry names a member. That is the opposite direction, and the
code implements this record's direction, not the register's.

**The record quotes the register correctly and then mischaracterises it.** Its
own context states the register's answer accurately, in the register's words.
Two paragraphs later D1 calls that the same answer. It is not.

**The difference is not cosmetic, and this record needs its own direction.**
D1's best idea is that an empty entry is a state rather than an absence,
because a group that stored only its filled entries could not say what it is
short of. A reverse index derived from an ownership column cannot hold a
vacancy: nothing owns the empty seat, so nothing puts it in the index. The
shape the register names cannot express the thing this record is for.

So the record is not a second declaration of half a rule, which is what it
claims to be. It is a change of answer, and stating it as an agreement hides
the choice from the next reader. Section 6 files that choice.

## 3. What holds, and holds well

**D1's core claim.** No group is defined by a spatial test. The reasoning is
the one that matters: a region changes what it contains between frames, so a
command sent to a region changes its own recipient set while it runs. A
contributor would reach for the region because it costs no storage and reads
like the way a person describes a crowd, and nothing in the code says why it
was refused.

**D1's identity clause, against the code.** A position holds a unit by its
whole identity. The stored field is the identity in bits and not a slot index,
zero means nobody, and every read resolves through the arena so the generation
is checked. The module's own documentation gives the same reason the record
does.

**D2, and it is wired.** The position table has a check that fails when an entry
names a unit the arena no longer holds, and the world's invariant check calls it.
The release of dead members runs inside the step rather than on the resize
interval, which is what D2 requires and what makes the interval safe to lengthen.

**D3's split, against the code.** Each kind takes the truncated proportion and
the remainder goes one entry at a time in ascending kind order. The module says
so and does so. The parts sum to the whole and no tie needs a draw, which is
what keeps it out of the ordering rules.

**D3's schedule as a parameter.** The world carries the interval and no kernel
holds it as a constant. The binding exposes it. The record's claim that a caller
changes how often a group reconsiders without touching the engine is true.

## 4. The objections that failed

**Does the record state a figure?** **No**, and it is careful about it. The
bound, the interval and the width are all named as quantities that live
elsewhere.

**Does D3 bound a formation?** D3 bounds a membership held at a place, by the
ground of that place. A formation is not held at a place, so D3 gives it no
bound. **The objection fails as an objection to D3**, because D3's heading says
"bounded by the ground" and a reader can see the scope. It is worth naming in
the amendment anyway, because D1 says the two kinds take one shape and D3 then
bounds one of them.

**Does the control plane paragraph in D3 contradict the review of the control
plane record?** That review found that the set-valued command carries one value
for the whole set, so a caller wanting a different value for each member loops.
**The objection fails against this record.** D3 claims only that the command
names no member, and that is true: it states what a place wants and the engine
turns it into entries. The per-member gap belongs to the command shape and an
item holds it.[^3]

**Does the record overlap the residence record, which stores the site on the
unit?** **The objection fails.** Where a unit lives and what work it holds are
different relations, and the register's reasoning for the residence column is
its own. It is worth naming in the decision row, because the project now holds
both directions and neither record says why they differ.

## 5. The amendment

The reviewer did not edit the record. The text below is a proposal for the
author.

**First**, replace the bolded claim in D3:

> That table is the one declaration of the bound. The width of a member row is
> folded from the same table, so raising a capacity raises the row width. **The
> two bounds are one bound.**

with:

> The width of a member row is folded from the terrain capacity table, so
> raising the capacity of a kind raises the row width and neither number is a
> copy of the other. **That fold is not the whole bound today.** The module
> holds a second capacity for the crossing a made way gives, the fold does not
> walk it, and admission composes both and takes the larger. So a tile with a
> finished road admits more units than the membership can hold. A finding holds
> the evidence and an open row holds the choice, and this record binds the
> membership to whatever the bound becomes rather than asserting what it is.

**Second**, replace the last sentence of D1's paragraph on the two cases:

> The register already resolved the military case this way.

with:

> The register resolved the military case as an ownership column on the member
> plus a derived reverse index. This record chooses the other direction, and
> the reason is the empty entry: a reverse index derived from an ownership
> column cannot hold a vacancy, because nothing owns an empty seat, and a group
> that cannot say what it lacks cannot ask for a member. An open row carries
> the choice, and the register entry is superseded by whatever closes it.

**Third**, add one sentence to D3, after its first paragraph:

> This bound is for a membership held at a place. A formation is held at no
> place, and what bounds one is not settled here.

The references for the finding, the open rows and the register entry go in the
reference list as footnotes.

## 6. The open choice this uncovers

The project now holds two directions for one question, in two places, with
nothing that fails when they disagree. That is the shape this record's own
context is about, and the record reproduces it while arguing against it.

A decision row is filed with both options and a recommendation.[^4] It
recommends this record's direction, on the strength of the vacancy argument,
and it says what closing it costs: the blocker's resolved text has to change,
and a resolved blocker that states a superseded answer is worse than an open
one.

## References

[^1]: Findings register, FND-193. `docs/FINDINGS.md`
[^2]: Decisions register, DEC-081. `docs/DECISIONS.md`
[^3]: Backlog item 0224. `docs/backlog/proposed/0224-answer-and-command-a-set-of-mass-tier-entities.md`
[^4]: Decisions register, DEC-091. `docs/DECISIONS.md`
