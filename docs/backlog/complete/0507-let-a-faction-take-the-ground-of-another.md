---
id: 0507
title: Let a faction take the ground of another
status: proposed
created: 2026-09-05
implements: []
changes: [ADR-0150, ADR-0148, ADR-0146]
creates: []
serves: [PRD-0053]
blocked-by: []
---

## Why

**Domination ends no game, and no value in the balance register can change
that.** A sweep of 32 seeds of the demonstration world, each played to 20000
ticks, ended 32 games on the wealth-or-wonder path and none on domination or
on territory. The project owner named domination and territory as the primary
win paths. The findings register holds the sweep and the chain.[^1]

**Five engine rules close the chain, and each one closes it alone.**

1. **A tile belongs to the faction of the nearest city within reach, and the
   pass reads no unit position.** The seat of a faction is the tile of its own
   founding, so its own city sits on it at distance zero and no rival city is
   nearer.[^2]
2. **No stage destroys a settlement, and no stage founds one.** The destroy
   verb has one engine caller, the rollback of a founding that failed. The
   controller has no found option. The settlement count is four at tick zero
   and four at the end of every seed.[^3]
3. **A campaign requires war, and war closes the ground the campaign must
   cross.** A campaign is raised only against a faction in the war band, and
   admission refuses a guest whose holder is below the peace edge toward it.
   The two edges are the same edge, so the objective refuses the cohort exactly
   when the cohort is raised.[^4]
4. **A campaign that reaches nothing never closes.** The close pass ends a
   campaign when the objective changes holder or when the cohort is empty. A
   faction with a live campaign is refused a new one, so each faction raises
   about one campaign for the whole run.[^5]
5. **A raise truncates the cohort to the idle units and does not wait.** The
   project order takes the same idle units, so a live cohort holds one unit and
   not the cohort size of four.

**This is one item and not five.** The first two decide whether the seat clause
can fire at all, and nothing below them matters until they are answered. A
worker that repairs the campaign alone would move a cohort onto a seat tile
that still reads its own faction as the holder.

## What is missing before this is refined

- **What takes ground.** State the rule that lets the holder of a tile change.
  A record holds the current rule, so the answer either supersedes it or adds a
  second cause beside the city distance.[^2]
- **Whether a city can fall.** If the answer to the question above is that a
  city must be destroyed, say what destroys one and what the cost is. If it is
  not, say what else moves the holder.
- **Whether a faction founds a second city.** The controller choice set has no
  found option, so held ground is fixed at tick zero by four foundings. Say
  whether a second city is part of this item or a separate one.
- **How a cohort crosses a hostile border.** The guest edge and the war edge
  are the same edge today. Say whether a cohort is exempt, whether the two
  edges separate, or whether a third rule decides.
- **When a campaign that reaches nothing ends.** State the condition. A
  deadline in ticks is one answer, and a distance is another.
- **What the cohort waits for.** Say whether a raise waits for the cohort size
  or takes what it finds, and which units it may take from.
- **Which records change.** The holding record, the game-end record and the
  relation record each hold a decision this item touches.

## Done when

Not written. Refine the item first.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-542. `docs/FINDINGS.md`
[^2]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D1. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^3]: ADR-0148, a game end is recorded once and stops the controllers, decision D3. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
[^4]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decisions D2 and D4. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
[^5]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D5. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
