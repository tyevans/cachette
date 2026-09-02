---
id: 0047
title: Review ADR-0067 for acceptance
status: complete
created: 2026-08-31
implements: []
changes: [ADR-0067, ADR-0070]
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

ADR-0067 states that the viewer reads the world and never writes to it. It is
a draft, so nothing may cite it as binding.

Two records rest on it. ADR-0070 cites three of its decisions as the
boundaries it extends, and the review of ADR-0070 found the record sound and
recommended against accepting it, because an accepted record whose foundation
binds nothing can be made false by an edit to a file nobody is required to
leave alone.[^1] ADR-0068 D4 cites it once for the same reason.

The viewer crate exists and is substantial. The record can be read against the
code rather than against the intent, which is the first thing a review here
must do.

## Impact review

**Governed by.** The registry states who reviews and what a delegated review
must do that a second reader would do for free: read the record against the
code, be an agent that did not write it, and state what it tried to reject.

**Changes.** ADR-0067 moves from `Draft` to `Accepted`, or the review returns
it with objections. The registry row and the file location change with it, and
every citation of the draft path moves. The citation check finds them.

**Creates.** None.

**Blockers.** None. ADR-0067 states no cost figure.

**Precedent.** A review of ADR-0070 already exists and holds the shape.[^1] It
also holds one thing this review must resolve rather than repeat: whether
ADR-0067 D2 and ADR-0070 D1 state one constraint or two. That review concluded
two, from the side of ADR-0070, and this one must reach the same conclusion
from the other side or say why not.

## Done when

- An agent that did not write ADR-0067 has read all five of its decisions
  against the viewer crate.
- The review states what it tried to reject, and why each rejection failed.
- D4 is checked against the demonstration binary, because it ties the drawing
  rate to the simulation rate and names what would supersede it.
- D3 is checked against the tree: no value that has been a floating point
  number reaches the engine, in any form.
- The registry row holds the outcome, and ADR-0070 is accepted in the same
  change if ADR-0067 is.

## Outcome

ADR-0067 is accepted with D1 amended, and ADR-0070 is accepted unchanged in
the same change, because the review that recommended it named this record as
the only thing holding it.[^2]

**The finding: D1 and D4 contradicted each other.** D1 said the viewer holds a
shared reference and never advances a tick. D5 said the viewer is a crate. D4
said one loop steps the engine and then draws, and that loop is in that crate.
The demonstration binary therefore did every one of the three things D1
forbids, and the record's own consequences named that binary and described it
stepping.

The claim was never in doubt. The subject was: D1 was about the path from the
world to the picture and was written as though that path were the whole crate.
A constraint a reviewer must read past is not a constraint, so the record
needed an amendment even though the code needed none. FND-056 holds the
shape.[^3]

**D1's subject is now the drawing and the reporting.** The constraint on that
path is unchanged and the compiler still enforces it. A sentence says plainly
that the program owning the loop is not bound by it, and why.

**The dependency did not hold this record.** ADR-0067's registry row names
ADR-0036, which is `Proposed` and has no file. The review of ADR-0070 refused
acceptance for exactly that shape, so this had to be resolved rather than
assumed. ADR-0067 cites ADR-0036 once and cites it for its absence: the
alternative design needs a snapshot mechanism that no record holds. Nothing
here is built on a decision of ADR-0036, because ADR-0036 has no decisions.
The binding content rests on ADR-0001, ADR-0002 and ADR-0017, all accepted.

**The other four decisions hold against the code without a change.** No engine
field is named for a display. No value that has been a float reaches an engine
call. One loop steps and then draws. The core's manifest does not name the
viewer.

Five objections were attempted and all five failed. The strongest was that the
record should say what the viewer may spend, and it fails because that is
ADR-0070's subject and filling it here would give two records one claim.

**Eleven files cited the two records by their draft paths and every one moved
with them.** The search command is in the commit body.

## References

[^1]: Review 0035, the head-up display record. `docs/reviews/0035-the-head-up-display-record.md`
[^2]: Review 0047, the viewer boundary record. `docs/reviews/0047-the-viewer-boundary-record.md`
[^3]: Findings register, FND-056. `docs/FINDINGS.md`
