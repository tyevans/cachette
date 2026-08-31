# Reviews (Index)

This directory holds the written outcome of each record review and each code
review. One file is one review.

A review that produced no file did not happen. This is not a formality.

The first attempt at the ceremony had reviewers report back by message. The
reports did arrive, and they were good: one of them found a defect in a value
type that the approver had missed. But they arrived late, after the approver
had concluded the ceremony had failed, and the message channel truncated
them. One verdict was cut off in the middle and had to be asked for again. A
review whose verdict is lost to a length limit has not been delivered.

**A reviewer writes its findings to a file here before it finishes.** The
file is the deliverable. A message is a convenience. A file does not
truncate, it does not race the reader, and it is still there in a week.

## Naming

`NNNN-<what-was-reviewed>.md`, where `NNNN` is the backlog item that the
review belongs to.

## What a review must contain

- What was reviewed, by path, and at which commit.
- Who reviewed it, and whether that reviewer wrote any of it.
- **Every objection the reviewer attempted, and why each one failed or
  held.** A review that lists no attempted objection did not happen, whatever
  its verdict says.
- The verdict for each item: `ACCEPT`, `ACCEPT WITH AMENDMENT`, or `REJECT`.
- For an amendment, the exact text to change.

## Who may review

The registry says who holds review rights and what a delegated review must
do that a second reader would do for free.[^1]

A reviewer may not review what it wrote. When that cannot be arranged, the
review says so in place of a verdict, and the record stays a draft.

## References

[^1]: ADR Registry, who reviews. `docs/adrs/REGISTRY.md`
