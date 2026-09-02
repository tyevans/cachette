---
id: 0168
title: Let the control plane read a household
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: [PRD-0015]
blocked-by: [0161]
---

## Why

The core derives a household from the dwelling slot, and a watcher inside Rust
can read one.[^1] Python cannot. The binding exposes no settlement, so the
control plane holds no way to name a dwelling and could not call the read even
if the read were bound.

A binding written today would be a capability that nothing invokes. That is a
defect shape the register names, and the rule is to bind nothing before a
caller exists.[^2]

## What the work does

1. The control plane names a dwelling.
2. The binding returns the members as one column, not one call per unit.
   Python never loops over entities.[^3]
3. A dwelling with no residents returns an empty column. A dead identity
   raises.

## What is missing before this is refined

- **How the control plane names a dwelling.** Item 0161 holds the selector
  work, and DEC-063 names the destination.[^4] This item takes whatever
  identity that item settles on.
- **The column type.** The event log columns already return arrays through
  the binding. Whether a household column reuses that shape is not worked
  out.

## Done when

Filled in when the item is refined.

## References

[^1]: Backlog item 0103. `docs/backlog/complete/0103-derive-a-household-from-the-dwelling-slot.md`
[^2]: Recurring Defect Shapes, section 3. `.claude/rules/recurring-defects.md`
[^3]: Project orientation, the design principles. `CLAUDE.md`
[^4]: Backlog item 0161. `docs/backlog/proposed/0161-let-a-selector-say-where-to-act.md`
