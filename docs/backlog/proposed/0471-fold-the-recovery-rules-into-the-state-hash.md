---
id: 0471
title: Fold the recovery rules into the state hash
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
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
