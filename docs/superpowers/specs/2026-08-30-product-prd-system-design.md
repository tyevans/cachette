# Design: The Product Requirement Record System

Date: 2026-08-30
Status: Approved for planning

## 1. The problem

This project has two governance systems. The decision records shape the
architecture. The backlog shapes the implementation work. Neither system
holds the product need that drives both.

A need therefore has no home. It arrives in conversation, becomes a decision
record, and the reason for the record is lost. A reader of the record cannot
tell whether the project still wants the thing that the record serves.

This design adds a third system. It holds the product need. It gives the need
a number, a state, and a gate that the need must pass.

## 2. The three systems

Each system answers one question. The boundary between them is the point of
the design.

| System | Question | Location |
|---|---|---|
| Product requirement records | What does the project need, and for whom? | `docs/product/` |
| Decision records | How is the engine built to meet the need? | `docs/adrs/` |
| Backlog | What work builds it? | `docs/backlog/` |

A product requirement record states a need. It does not state a structure. A
record that names a data structure holds a decision, and that decision belongs
in a decision record.

## 3. The layout

```
docs/product/
  README.md
  REGISTRY.md
  idea/
  shaped/
  accepted/
  shipped/
```

A file is named `prd-NNNN-<slug>.md`, where `NNNN` is a four-digit number.
The prefix matches the decision record convention, because other documents
cite these files by number.

A record moves between directories with `git mv`. The file name never
changes, so the history follows the file. The number never changes.

## 4. The registry allocates the number

The registry holds one row for each record. **Add the row before you write
the file.** Set the status to `Idea` and leave the file column empty. Then
write the file.

The decision record registry uses this rule for a measured reason. Three
number collisions occurred during the research phase, because the agents that
wrote the reports chose their own numbers. A single allocator removes that
failure.

### Status vocabulary

| Status | Directory | Meaning |
|---|---|---|
| `Idea` | `idea/` | The number is reserved. The need is not worked out. |
| `Shaped` | `shaped/` | The product review is complete. Not committed to. |
| `Accepted` | `accepted/` | The project commits to the need. Cite it. |
| `Shipped` | `shipped/` | The need is met. Kept for the record. |
| `Dropped` | `shipped/` | Considered and declined. The row says why. |

**An author may set `Shaped`. Only a reviewer may set anything beyond it.**
A file that exists is a fact, so the author states it. Acceptance is a
judgement about value, so a reviewer states it. This split matches the
decision record registry.

## 5. The gate between `idea/` and `shaped/`

This is the load-bearing part of the system. A record in `idea/` may be one
sentence. A record in `shaped/` must answer all six questions below.

1. **Who is this for.** Name one audience. The engine serves an owner who
   plays a strategy game, other simulation developers, and researchers who
   study agent behaviour. A record that serves all three usually serves none.
2. **What the person cannot do today.** State the behaviour. Do not state a
   feature.
3. **What good looks like.** Write a statement that a reader can check. Write
   "a faction sees only the tiles that its own units observe". Do not write
   "the fog of war feels correct".
4. **What this does not do.** State the bound. A record without a bound grows
   without limit.
5. **What it costs at the target scale.** The target is 16.7 million tiles
   and one million units. A need that no algorithm can meet at that scale is
   a different need. Learn that here, not in a backlog item.
6. **Which blockers govern it.** A value that sits behind an open blocker is
   expressed parametrically. Cite the blocker. Do not invent the value.

If you cannot answer these, the record stays in `idea/`. Say what is missing.

## 6. The link to work

A backlog item that is refined names the product requirement record that it
serves. The backlog item format gains one field.

```
implements: [ADR-0001 D11]
serves:     [PRD-0004]
blocked-by: []
```

**A decision record cites no product requirement record.** A decision record
answers to a constraint, not to a feature. A product direction changes more
often than a constraint does. A citation from a record to a record that
changes would put changing material inside a historical document, and the
decision record registry forbids that.

The backlog item is therefore the only join between product need and
architecture. This is deliberate. It keeps the decision corpus stable while
the product moves.

## 7. What does not belong in a product requirement record

| The statement | Its home |
|---|---|
| A need, an audience, a bound | The product requirement record |
| How the engine meets the need | A decision record |
| The work that builds it | A backlog item |
| A number that changes | A register |

The three registers hold the changing values. They record open blockers,
open decisions, and corrections that the project had to make.

## 8. The checks

Two shell scripts enforce the rules. They join the two scripts that already
check the crate split and the float ban. The check command runs all of them.

**`check-prd-registry.sh`** fails when any statement below is false.

- Every file under `docs/product/` has a row in the registry.
- Every row with a status past `Idea` names a file that exists.
- The status in the row matches the directory that holds the file.
- No number occurs twice.

**`check-prd-scope.sh`** fails when any statement below is false.

- A record in `shaped/`, `accepted/` or `shipped/` holds all six gate
  sections, and no section is empty.
- No record body holds a decision record citation or a path under
  `docs/adrs/`. A reference to a decision record inside a product requirement
  record shows that the record states a structure.

Both scripts must pass against the seed content that this work delivers.

## 9. The seed content

An empty system teaches nothing. This work delivers one real record in
`shaped/`: fog of war.

That subject is a good example for four reasons. It has one named audience.
It has a success criterion that a reader can check. It has a real bound. Its
cost figure is derived and not measured, so the record shows how to cite a
figure honestly.

## 10. Testing

The two check scripts are the test. Each script runs against the seed content
and against a set of deliberately broken fixtures. A fixture holds a missing
row, a status that disagrees with its directory, a duplicate number, an empty
gate section, and a decision record citation in a body.

Each fixture must make the matching script fail. A check that cannot fail
protects nothing.

## 11. Impact review

**Governed by.** No decision record governs documentation structure. The
documentation rule governs the prose of every file that this work delivers.
The definition of done governs the work itself.

**Changes.** The backlog guide gains the `serves` field, in the item format
and in the list that the refined gate holds. `CLAUDE.md` gains a short
section that points an agent at `docs/product/`.

**Creates.** No decision record. This work adds a documentation system. It
does not decide anything about the engine.

**Blockers.** None. The seed record cites an open blocker as an example of
the correct habit.

**Precedent.** The findings register records that self-chosen numbers
collided three times. The registry allocates numbers for that reason.

**Branch.** This work sits on a branch that starts from the backlog branch,
because the `serves` field edits a file that the backlog branch introduces.
The pull request states that dependency.

## References

[^1]: Documentation Rules. `.claude/rules/documentation.md`
[^2]: Definition of Done. `.claude/rules/definition-of-done.md`
[^3]: ADR Registry. `docs/adrs/REGISTRY.md`
[^4]: Backlog guide. `docs/backlog/README.md`
[^5]: Blockers, Decisions and Findings registers. `docs/`
