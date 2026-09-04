# Product Registry (Index)

This document is an **index**. It lists every product requirement record in
this project and assigns its number.

Rule 4 of the documentation rule exempts a table that lists files and paths
as data. The tables below are that table.[^1]

## How to use this registry

**Assign the number here before you write the record.** Add the row first,
with status `Idea` and no file. Then write the file.

This registry is the allocator. A writer that chooses its own number
collides with another writer. That happened three times during the research
phase, and it is recorded as precedent.[^2]

**Never reuse a number.** A dropped record keeps its number, because other
documents cite it.

**An author may set `Shaped`. Only a reviewer may set anything beyond it.**

## Status vocabulary

| Status | Directory | Meaning |
|---|---|---|
| `Idea` | `idea/` | The number is reserved. The need is not worked out. |
| `Shaped` | `shaped/` | The six gate answers are complete. Not committed to. |
| `Accepted` | `accepted/` | The project commits to the need. |
| `Shipped` | `shipped/` | The need is met. |
| `Dropped` | `shipped/` | Considered and declined. This row says why. |

## The records

| No. | Title | Status | Serves | File |
|---|---|---|---|---|
| 0001 | A faction sees only what its own units observe | Accepted | Game developer | `accepted/prd-0001-a-faction-sees-only-what-it-observes.md` |
| 0002 | A developer watches the world run | Shipped | Game developer | `shipped/prd-0002-a-developer-watches-the-world-run.md` |
| 0003 | A developer sees a world worth looking at | Accepted | Game developer | `accepted/prd-0003-a-developer-sees-a-world-worth-looking-at.md` |
| 0004 | The world has weather that a watcher can read | Accepted | Game developer | `accepted/prd-0004-the-world-has-weather-that-a-watcher-can-read.md` |
| 0005 | A watcher can tell what is happening and why | Shipped | Game developer | `shipped/prd-0005-a-watcher-can-tell-what-is-happening-and-why.md` |
| 0006 | A place belongs to somebody | Accepted | Game developer | `accepted/prd-0006-a-place-belongs-to-somebody.md` |
| 0007 | The world holds things worth taking | Accepted | Game developer | `accepted/prd-0007-the-world-holds-things-worth-taking.md` |
| 0008 | A unit changes the ground it stands on | Accepted | Game developer | `accepted/prd-0008-a-unit-changes-the-ground-it-stands-on.md` |
| 0009 | A unit acts on the world it can see | Accepted | Game developer | `accepted/prd-0009-a-unit-acts-on-the-world-it-can-see.md` |
| 0010 | A good moves to where it is wanted | Accepted | Game developer | `accepted/prd-0010-a-good-moves-to-where-it-is-wanted.md` |
| 0011 | A unit is born, holds a job, and dies | Accepted | Game developer | `accepted/prd-0011-a-unit-is-born-holds-a-job-and-dies.md` |
| 0012 | A world starts small and grows | Accepted | Game developer | `accepted/prd-0012-a-world-starts-small-and-grows.md` |
| 0013 | A unit consumes to continue | Accepted | Game developer | `accepted/prd-0013-a-unit-consumes-to-continue.md` |
| 0014 | Everyone needs somewhere to live | Accepted | Game developer | `accepted/prd-0014-everyone-needs-somewhere-to-live.md` |
| 0015 | A unit has parents and children | Accepted | Game developer | `accepted/prd-0015-a-unit-has-parents-and-children.md` |
| 0016 | Somebody is in charge | Accepted | Game developer | `accepted/prd-0016-somebody-is-in-charge.md` |
| 0017 | Work is assigned to the people who can do it | Accepted | Game developer | `accepted/prd-0017-work-is-assigned-to-the-people-who-can-do-it.md` |
| 0018 | A depleted deposit comes back | Shaped | Game developer | `shaped/prd-0018-a-depleted-deposit-comes-back.md` |
| 0019 | An agent can ask the running engine what it holds | Shaped | Repository agent | `shaped/prd-0019-an-agent-can-ask-the-running-engine-what-it-holds.md` |
| 0020 | A unit goes somewhere it cannot see | Shaped | Game developer | `shaped/prd-0020-a-unit-goes-somewhere-it-cannot-see.md` |
| 0021 | A developer can use the control plane without reading its source | Accepted | Game developer | `accepted/prd-0021-a-developer-can-use-the-control-plane-without-reading-its-source.md` |
| 0022 | A caller can name the people the world holds | Idea | AI researcher | `idea/prd-0022-a-caller-can-name-the-people-the-world-holds.md` |
| 0023 | An observer reads what happened near a place | Idea | AI researcher | `idea/prd-0023-an-observer-reads-what-happened-near-a-place.md` |
| 0024 | A run stays eventful for as long as it is watched | Idea | AI researcher | `idea/prd-0024-a-run-stays-eventful-for-as-long-as-it-is-watched.md` |
| 0030 | A developer builds a game the engine did not anticipate | Shaped | Game developer | `shaped/prd-0030-a-developer-builds-a-game-the-engine-did-not-anticipate.md` |
| 0031 | A god knows whose ground its people stand on | Shaped | Game developer | `shaped/prd-0031-a-god-knows-whose-ground-its-people-stand-on.md` |
| 0032 | A god knows what its ground is rich in | Shaped | Game developer | `shaped/prd-0032-a-god-knows-what-its-ground-is-rich-in.md` |
| 0034 | Two players hold each other to a future delivery | Shaped | Game developer | `shaped/prd-0034-two-players-hold-each-other-to-a-future-delivery.md` |
| 0047 | A game states its own economy | Shaped | Game developer | `shaped/prd-0047-a-game-states-its-own-economy.md` |

## What does not belong in a record

A product requirement record must not hold material that changes.

- A measured cost. Costs go to the reference budgets.
- A version or a release name.
- A value that an open blocker governs. Express it parametrically and cite
  the blocker.
- A data structure, an algorithm, or a module arrangement. Those are
  architectural decisions.[^3]

## References

[^1]: Documentation Rules. `.claude/rules/documentation.md`
[^2]: Findings register, FND-028. `docs/FINDINGS.md`
[^3]: Decision record scope rule. `.claude/rules/adr-scope.md`
