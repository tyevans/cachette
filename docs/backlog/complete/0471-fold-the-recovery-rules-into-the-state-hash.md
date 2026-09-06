---
id: 0471
title: Fold the recovery rules into the state hash
status: complete
created: 2026-09-03
implements: [ADR-0001 D4, ADR-0022 D1]
changes: []
creates: [ADR-0164]
serves: []
blocked-by: []
---

## Why

The recovery rules say how fast a depleted deposit returns. The step reads
them on every tick. They do not enter the state hash.

Two worlds that hold the same tiles and different recovery rules therefore
hash the same, and they diverge on the next tick. That is the one property the
project cannot recover once it is lost. FND-480 holds the reading.

The repair is one line in the fold of the depletion ledger. It moves every
golden file, because the hash chain changes, so it must be taken on its own
and the new files must be read before they are committed.

## Impact review

**Governed by.** ADR-0001 D4 hashes the whole world each frame and compares
the value against a stored file. ADR-0022 D1 makes level 0 the only truth, so
a derived projection states no fact and stays out of the hash.

**Changes.** None. No accepted record said that a step parameter stays out.

**Creates.** ADR-0164. The audit found seven more values of the same shape,
and two tests that stated the hole as a rule. A hash function that a person
writes field by field will grow more of them, so the rule is written down.

**Blockers.** None.

**Precedent.** FND-480 holds the reading of the recovery rules. FND-537 holds
the audit and the seven values beside them.

## Done when

- The recovery rules enter the whole-world state hash.
- Every other stored value that a pass or a verb reads enters it too, or an
  item holds the ones that were left.
- A test changes each covered value through the public interface and asserts
  that the hash differs.
- Each fold is proved by removing it again and watching a named test go red.
- The golden files are regenerated and read before they are committed.
- The world gives one answer at 1, 2 and 12 threads.

## Outcome

All six statements hold.

**The audit found eight values outside the hash, not one.** The recovery rules
were one. The choice pass reads four more on every tick: the choice schedule,
the need bucket width, the weight profile and the carry mark. A trade verb
reads the land list bound. The luxury seed flag decides what the next seed call
does, and an empty seed leaves the luxury field exactly as an unseeded world
leaves it. The delivered total is the eighth. Every one is folded in.

**Two existing tests asserted two of the holes as rules.** One said that a
weight the world could not act on moved no byte of the hash. One said that an
empty luxury seed left the hash alone. Both now state the new rule, and the
first asserts that the intents agree while the hashes differ, which is the
whole point.

**Seven of the eight folds have a test that names the value.** The eighth, the
delivered total, has none: no public path builds two worlds that differ only in
it, because a delivery moves a store at the same time. It is folded for the
reason the departed total is folded, and the report says so.

**The golden files moved.** Eleven files changed, which is every golden file
the suite holds. The tick indices are unchanged and only the hash column moved.
