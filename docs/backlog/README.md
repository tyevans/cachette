# Backlog

This directory holds the work queue. One file is one item. An item moves
between three directories as it progresses.

| Directory | Meaning |
|---|---|
| `proposed/` | An idea. Not yet worked out. Anyone may add one. |
| `refined/` | Ready to pick up. The impact review is done. |
| `complete/` | Finished. Kept for the record. |

An item moves by `git mv`. Its file name never changes, so its history
follows it.

## Naming

`NNNN-short-slug.md`, where `NNNN` is a four-digit number.

**Allocate the number by taking the highest across all three directories and
adding one.** There is no separate index. The directories are the index.

```
ls docs/backlog/*/ | grep -o '^[0-9]\{4\}' | sort -n | tail -1
```

A number is never reused, including by a dropped item. Delete a dropped item's
file only if it was never refined; otherwise move it to `complete/` and record
why it was dropped.

## The line between proposed and refined

This is the important part of the system, and it is not a matter of taste.

**An item is refined when its architectural impact review is complete.** That
review is section 1 of the definition of done. Doing it is what turns an idea
into work someone can pick up.

An item in `proposed/` may be one sentence.

An item in `refined/` must answer all of this:

- What is being done, and why.
- Which decision records govern it, by number and decision.
- Which records it will change, and how they are superseded.
- Which records it will create. **The registry row is allocated before the
  item is refined**, not when the work starts.
- Which blockers govern it. A value behind a blocker is expressed
  parametrically, not invented.
- Which product record it serves, if any. A product record states the need
  the work answers to.[^4] Work that serves no recorded need is allowed, but
  say so with an empty list rather than by omitting the field.
- What "done" means for this item, as statements that can be checked.

If you cannot answer these, the item is not refined. Leave it in `proposed/`
and say what is missing.

## Item format

```markdown
---
id: 0007
title: Build the thread-count equivalence harness
status: refined
created: 2026-08-30
implements: [ADR-0001 D11]
changes: []
creates: []
serves: [PRD-0001]
blocked-by: []
---

## Why

One paragraph. What problem this solves, or what it enables.

## Impact review

**Governed by.** ADR-0001 D11 requires this harness before the first solver.

**Changes.** None.

**Creates.** None.

**Blockers.** None.

**Precedent.** FND-029 records that a stale read produces a confident wrong
answer; the harness must read state after the barrier, not during it.

## Done when

- The harness runs one tick at 1, 2 and 12 threads.
- It compares the event log byte for byte.
- It fails loudly on a mismatch, naming the first differing offset.
- The whole check command runs green.

## Outcome

Filled in when the item moves to `complete/`. What was done, what changed
from the plan, and which register entries moved.
```

Front matter fields are optional except `id`, `title`, `status` and
`created`. Use an empty list rather than omitting a field when the answer is
genuinely none, so a reader can tell "none" from "not considered".

## Completing an item

1. Fill in the outcome section. Say what changed from the plan.
2. Update the registers, per section 4 of the definition of done. A correction
   goes to findings. A resolved blocker or decision has its row closed.
3. `git mv` the file to `complete/`.
4. Set `status: complete` in the front matter.

An item is not complete because the code merged. It is complete when the
record of the work is true.

## What does not belong here

- **Decisions.** A decision goes in a record under `docs/adrs/`. A backlog
  item may create one; it is not one.
- **Open questions blocking work.** Those go in the blockers register.
- **Corrections.** Those go in the findings register.

The backlog holds work. The registers hold state. The records hold decisions.

## References

[^1]: Definition of Done. `.claude/rules/definition-of-done.md`
[^2]: ADR Registry. `docs/adrs/REGISTRY.md`
[^3]: Blockers, Decisions and Findings registers. `docs/`
[^4]: Product requirement records. `docs/product/README.md`
