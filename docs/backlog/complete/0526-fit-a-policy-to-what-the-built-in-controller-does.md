---
id: 0526
title: Fit a policy to what the built-in controller does, and say whether the observation can express it
status: complete
created: 2026-09-07
implements: [ADR-0154 D6, ADR-0176 D4, ADR-0192 D1, ADR-0192 D2, ADR-0192 D3, ADR-0192 D4, ADR-0192 D5]
changes: []
creates: [ADR-0192]
serves: [PRD-0056]
blocked-by: [BLK-007]
---

## Why

**An episode yields one number, and it holds hundreds of decisions.** The
project fits a learner seat with an evolution strategy. That strategy scores a
whole episode by its return, so one episode of two and a half thousand ticks
buys one scalar. The same episode holds what the built-in controller chose at
every decision of every tick, and nothing reads it.

**Nothing has come near the built-in controller.** A run measured on the target
platform gives about nineteen thousand episodes in six hours, which is about
thirty-seven gradient steps for a policy of a few thousand parameters. No
trained policy beats the controller in the learner's own seat.

**Two explanations fit that result, and the project cannot separate them.**
Either the observation array does not hold what the controller reads, and the
ceiling is the representation, or the array is sufficient and the optimiser was
the problem. A supervised fit to controller play separates them. If a fit
cannot reproduce controller play from the array, the array is the ceiling.

**The bridge from a controller choice to an action integer already exists.** An
accepted record puts a controller choice and a learner action in one encoding,
and the engine encodes every command it plans through the action schema.[^1]
[^2] What did not exist is a way to read that log from the control plane, and a
rule for turning a window of commands into one supervised label.

## The architectural impact review

**The records that govern this work.** ADR-0154 D5 and D6 govern the legality
answer and the one-encoding command log. ADR-0176 D1, D2 and D4 govern the
mixed radix and the verb set. ADR-0144 D2 governs the verbs the controller
reaches. ADR-0006 D1 governs the plain data command row. ADR-0001 D1 and
ADR-0004 D4 govern determinism and order. PRD-0001 and PRD-0056 state the need.

**The records this work changes.** None. It contradicts none of them.

**The records this work creates.** ADR-0192. The rule for turning a window of
controller commands into one label is a decision a later contributor could
reasonably make otherwise, it fixes what every accuracy figure from such a
dataset means, and the reasoning is not visible in the recorder. The registry
row was added before the record was written.

**The blockers that hold it.** BLK-007 holds every cost figure of this project.
The work therefore states which figures were measured and on which machine, and
it takes no figure into a record.

## What the work found

**The action table already expressed every choice the engine emitted.** The
share the table could not express was zero over the whole recording. The engine
holds one case that can reach a refusal, a relation move against a faction the
world does not hold, and no world state produces it.

**The command log lost that distinction anyway.** The engine wrote the no-op
row into the action column of a choice the table refused, and the no-op is a
real action. The row now states whether its action column is an encoding, and
the stage counts the refusals. This is the redundant declaration shape at its
sharpest: one integer stood for two different facts.

**The fit does not reproduce controller play.** Both fitted shapes land within
two points of one constant answer on the held-out episodes, and both land below
it on the share of individual commands they match. A policy that plays the fit
wins far less often than the controller does on the same seeds. The findings
register holds the reading and the confound it carries.[^3]

**The two clocks do not line up, and that is the finding.** The controller
emits many commands for one learner decision. The learner sends one action for
that window. No reduction is lossless, and the record states which loss the
project takes and which it refused.

## Acceptance

- A recorder plays the built-in controller in the learner's seat and writes
  observation and label pairs, from the same reader set the learner uses.
- The observation of a window is the array the learner reads at the start of
  that window, and a test proves it by comparing against a second loop.
- A command the action table cannot express is counted and reported, never
  labelled.
- A fit produces the two policy shapes the trainer already plays, and the
  trainer loads them.
- A report gives the train and holdout accuracy against what one constant
  answer scores on the same data.
- A report gives the fitted policy's return and win share on the held-out seeds
  beside the controller's own numbers on those seeds.

## References

[^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D6. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^2]: ADR-0176, an action integer is a mixed radix over the argument positions each verb declares, decision D4. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
[^3]: Findings register, FND-643. `docs/FINDINGS.md`
