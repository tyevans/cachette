---
id: 0189
title: Require a test that something reads the value
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: [DEC-074]
---

## Why

The engine writes a value into state, and no stage reads that value to decide
anything. The option column is the instance: movement reads whether a unit
chose and not what it chose.[^1] The influence field and the tile stub value
are two more.

**The rules the project holds do not catch this.** One says not to declare a
capability before something calls it. One says that when the engine is
obligated to invoke a thing, the test starts at the engine. Both look for an
absent caller, and this defect has one.[^2]

So do the repairs that suggest themselves first. A rule that a person must be
able to run the feature passes, because the demonstration runs the choice pass
on every tick. A check that reports a public verb with no caller passes,
because the pass and the column both have callers. A rule that a backlog item
names its caller before it is refined passes, because the item named the right
caller.

## What the work does

1. The testing rule gains a section: for each value the work writes into
   state, name the stage that reads it to decide something, and write a test
   that changes the value and asserts that the decision changes.
2. The definition of done gains one line in the impact review that asks the
   same question before the work starts.[^3]
3. The rule states the falsification the project already trusts: pin the value
   to a constant and run the suite. A suite that stays green proves that
   nothing reads it.[^4]

The rule is the same discipline the testing rule already states for a keyed
draw, turned round. That rule says to test what the value depends on. This
says to test what depends on the value.

## What is missing before this is refined

- The decision. DEC-074 holds four options and recommends this one.[^5]
- Whether a mechanical companion is taken beside the rule, and which. The
  decision row states the cost of each.
- Whether the rule applies to a value the state hash reads, since the hash
  reads every column and is not a decision.

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-180. `docs/FINDINGS.md`
[^2]: Findings register, FND-181. `docs/FINDINGS.md`
[^3]: Definition of Done, section 3. `.claude/rules/definition-of-done.md`
[^4]: Testing Rules, section 2a. `.claude/rules/testing.md`
[^5]: Decisions register, DEC-074. `docs/DECISIONS.md`
