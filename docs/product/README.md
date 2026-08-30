# Product Requirement Records

This directory holds the product need. One file is one need. A file moves
between four directories as the need progresses.

This project has three systems, and each answers one question.

| System | Question |
|---|---|
| This directory | What does the project need, and for whom? |
| The decision records | How is the engine built to meet the need? |
| The backlog | What work builds it? |

A product requirement record states a need. It does not state a structure.
A record that names a data structure holds an architectural decision, and
that decision belongs in a decision record.[^1]

## The directories

| Directory | Status | Meaning |
|---|---|---|
| `idea/` | `Idea` | The number is reserved. The need is not worked out. |
| `shaped/` | `Shaped` | The product review is complete. Not committed to. |
| `accepted/` | `Accepted` | The project commits to the need. Cite it. |
| `shipped/` | `Shipped` or `Dropped` | Finished, or declined. Kept for the record. |

A record moves with `git mv`. The file name never changes, so the history
follows the file.

## Naming and numbering

A file is named `prd-NNNN-<slug>.md`, where `NNNN` is a four-digit number.

**Allocate the number in the registry before you write the file.** Add the
row with status `Idea` and an empty file column. Then write the file.[^2]

The registry is the allocator. Three number collisions happened during the
research phase, because each writer chose its own number. A single
allocator removes that failure.

A number is never reused.

## Who sets the status

**An author may set `Shaped`. Only a reviewer may set anything beyond it.**

A file that exists is a fact, so its author states it. Acceptance is a
judgement about value, so a reviewer states it. This split matches the
decision record registry.

## The gate between `idea/` and `shaped/`

This is the load-bearing part of the system. A record in `idea/` may be one
sentence. A record in `shaped/` must answer all six questions below, and
each answer needs its own section.

1. **Who is this for.** Name one audience. The engine serves a developer
   who builds a strategy game, a modeller who needs a large agent count,
   and a researcher who must reproduce a run. A record that serves all
   three usually serves none.
2. **What the person cannot do today.** State the behaviour. Do not state
   a feature.
3. **What good looks like.** Write a statement a reader can check. Write
   "a faction sees only the tiles its own units observe". Do not write
   "the fog of war feels correct".
4. **What this does not do.** State the bound. A record without a bound
   grows without limit.
5. **What it costs at the target scale.** The target is 16.7 million tiles
   and one million units. A need no algorithm can meet at that scale is a
   different need. Learn that here, not in a backlog item.
6. **Which blockers govern it.** A value behind an open blocker is
   expressed parametrically. Cite the blocker. Do not invent the value.

If you cannot answer these, the record stays in `idea/`. Say what is
missing.

## The link to work

A refined backlog item names the record it serves, next to the decision
records that govern it.[^3]

```
implements: [ADR-0001 D4]
serves:     [PRD-0001]
```

**A decision record cites no product requirement record.** A decision
record answers to a constraint, not to a feature. A product direction
changes more often than a constraint does, and a decision record must not
hold material that changes.[^4]

The backlog item is therefore the only join between the product need and
the architecture. This keeps the decision records stable while the product
moves.

## What does not belong here

| The statement | Its home |
|---|---|
| A need, an audience, a bound | This directory |
| How the engine meets the need | A decision record |
| The work that builds it | A backlog item |
| A number that changes | A register |

## The checks

Two scripts enforce these rules, and the check command runs both.

`check-prd-registry.sh` fails when a file has no registry row, when a row
past `Idea` names no file, when a status disagrees with its directory, or
when a number occurs twice.

`check-prd-scope.sh` fails when a record in `shaped/` or beyond is missing
one of the six gate sections, when a gate section is empty, or when a
record body cites a decision record. A decision record citation inside a
product record shows that the record states a structure.

## References

[^1]: ADR Registry. `docs/adrs/REGISTRY.md`
[^2]: Product registry. `docs/product/REGISTRY.md`
[^3]: Backlog guide. `docs/backlog/README.md`
[^4]: Decision record scope rule. `.claude/rules/adr-scope.md`
