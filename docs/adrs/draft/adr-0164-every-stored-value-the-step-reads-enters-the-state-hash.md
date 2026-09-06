# ADR-0164: Every stored value the step reads enters the state hash, and only a derived value stays out

## Context

The engine hashes its whole state each frame. A test compares that value
against a stored file, and the comparison is one of the two tests that protect
determinism.[^1]

The word "whole" is the problem. The hash is a function that a person writes,
field by field. Nothing derives the field list from the type, so a field that
the author did not write is silently outside the hash. The golden file cannot
report the gap, because it compares one run against a file that the same
function produced.

The world holds three kinds of field. The first is stored state that a pass
reads and writes: a tile stock, a unit column, a faction relation. The second
is a derived projection, which a rebuild computes again from level 0 and which
states no fact of its own.[^2] The third is a parameter: a schedule, a rule
set, a weight table, a bound. A caller writes a parameter through a verb, and a
pass reads it on every tick.

The project had no rule for the third kind, and it guessed twice. The
depletion ledger wrote its entries and not the recovery rules beside them, so
two worlds with the same takes and different recovery periods hashed the same
and diverged on the next tick that aged an entry.[^3] The choice pass read four
parameters that the hash did not cover, and the field comment argued that the
outcome column carried the effect and that the cause needed no coverage.[^4]

Both arguments sound reasonable when read one at a time. That is the danger.
An author writes a new pass, adds a parameter for it, and reasons that the
outcome is already covered. Nothing fails.

## Decision

### D1. A stored value that any pass or verb reads enters the state hash

The state hash covers every field of the world that a later frame reads,
whatever writes it and however rarely it changes. A rule set, a schedule, a
table, a bound and a flag are each such a field.

A reviewer finds a violation by asking one question of each field: does
anything read this after the frame that wrote it? If the answer is yes, the
hash must write it.

### D2. A derived projection stays out, and its source enters instead

A value that a rebuild computes again from level 0 states no fact of its own,
so it does not enter.[^2] The inputs of the derivation enter, and they are
enough: two worlds that agree in the inputs agree in the projection.

A per-frame log stays out for the same reason. It reports the frame that just
ran, and the state it reports is already in the hash.

### D3. The outcome does not excuse the cause

A parameter that decides what a pass writes enters the hash, even when the
column the pass writes is already in it.

The column carries the effect. A hash of the effect alone reports a changed
parameter only after the difference has reached a stored value, which is one or
more ticks after the change. A learner or a replay that trusts the hash then
reads two worlds as one.

### D4. A test names the value, and the golden file does not

Each value that D1 covers gets a test that builds two worlds, changes that one
value through the public interface, and asserts that the hashes differ.

The golden file notices that something changed. It cannot say which input the
hash stopped depending on, so it is not this test.[^5]

## Consequences

**Every new parameter costs a hash line and a test.** A pass that gains a
schedule or a table is not finished when it runs.

**A value moved into the hash moves every golden file.** The hash chains frame
to frame, so the change is not local to the frame that reads the value. Such a
change must be taken on its own, and the new files must be read before they are
committed.

**A field that only a caller reads still enters.** The rule asks whether
anything reads the field, and a verb is a reader. This is wider than the rule
the project applied before, and it is wider on purpose: the narrower question
is the one that was answered wrongly twice.

**The rule cannot be checked by a script.** Deciding whether a field is stored
or derived needs a reader. The project pays for this with the per-value tests
that D4 requires.

## References

[^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^2]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^3]: Findings register, FND-480. `docs/FINDINGS.md`
[^4]: Findings register, FND-537. `docs/FINDINGS.md`
[^5]: Testing rules, section 2. `.agents/rules/testing.md`
