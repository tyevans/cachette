# Backlog Priority (Index)

This document is an **index**. It states the order in which the open backlog
items should be taken, and why each sits where it does.

**It holds no title, no status and no summary.** Those live in the item file,
and a second copy of them here would go stale the moment an item moved. This
document holds one thing the item files cannot hold: the order between them.

Rule 4 of the documentation rule exempts a table that lists files as data.[^1]

## How to use this

Take the highest item you can start. An item in `proposed/` must be refined
first, and refining it is the work.[^2]

**A number here is not a promise.** Order changes when the project learns. Move
a row and say why in the commit body.

**Every open item appears exactly once.** A check enforces it, so an item
cannot be forgotten by being left off this list.[^3]

## Now

These answer a need the project owner has stated, or they repair something the
engine gets wrong today.

| No. | Why it sits here |
|---|---|
| 0064 | Units walk at random. This is the item that makes them act, and the owner has asked for it. |
| 0056 | Nothing consumes anything, so gathering has no consequence. |
| 0057 | A shortage must end a unit, or consumption has no consequence either. |
| 0093 | The panel describes a full world. The run now founds thirty people and the panel does not say so. |
| 0085 | A faction holds ground and no watcher can see it. PRD-0006 states this and it is not met. |
| 0092 | A settlement stands on water. The owner has stated this is not wanted. |
| 0094 | One faction founds and three begin empty. This needs the owner's answer.[^4] |

## Next

These close a gap a review found, or they unblock the items above.

| No. | Why it sits here |
|---|---|
| 0069 | A tile can hold four hundred units and no watcher can see it. PRD-0002 fails on this. |
| 0070 | The viewer makes the engine wait, and a record chose that knowingly. Settle it. |
| 0059 | Housing. The population cannot be bounded by anything until a place has capacity. |
| 0060 | Population growth from the store and the housing. |
| 0071 | Passability has two declaration sites. Item 0092 must not add a third. |
| 0084 | A tile has two faction columns and one of them is not a holder. |
| 0095 | Two foundings can over-fill a tile. |
| 0080 | The settings struct prices every new parameter at twenty-five files. |
| 0098 | The gate suite has no budget and the golden test grows unwatched.[^5] |

## Later

These are real and none of them blocks anything today.

| No. | Why it sits here |
|---|---|
| 0062 | Ranked positions at a site. Wanted by job assignment. |
| 0063 | Assigning a unit to a position. Waits on 0062. |
| 0065 | Letting the job decide what a unit weighs. Waits on 0063 and 0064. |
| 0058 | Improvements. Waits on a site that produces. |
| 0067 | Descent. Waits on the character tier being used for something. |
| 0068 | A ruler and a succession. Waits on 0067. |
| 0088 | Promotion into the character tier. |
| 0097 | The layout record. Write it with the descent columns, not before.[^6] |
| 0050 | Four product collisions still carried. |
| 0099 | The faction mask union has no engine caller. |
| 0072 | The panel fit check has no production caller. |
| 0077 | The batched structural path, once its record exists. |
| 0073 | Review the renderable example again, after 0069 and 0070. |
| 0053 | Superseded in substance by the completed resource work. Close it or restate it. |
| 0036 | How a watcher reads a count of the whole world. |
| 0043 | How a level 1 cell is repaired. |
| 0041 | Read the ground once for each target. |
| 0046 | Read the ground of a new world in parallel. |
| 0039 | A rejected unit is not stuck. Waits on units having plans. |
| 0040 | Record where an out-of-frame change gets its barrier. |
| 0034 | Measure the generated terrain against a stored one. |
| 0011 | Record the terrain multiplier. |
| 0022 | Random behaviour through a keyed draw. Read against 0064 first. |
| 0021 | Audit the movement records for acceptance. |
| 0004 | Reconcile the public API examples. |
| 0005 | The cross-cutting records. |
| 0007 | The storage claims. |
| 0009 | The log claims. |
| 0010 | The Python boundary claims. |

## References

[^1]: Documentation Rules. `.claude/rules/documentation.md`
[^2]: Backlog guide. `docs/backlog/README.md`
[^3]: The priority check script. `scripts/check_priority.py`
[^4]: Blockers register, BLK-018. `docs/BLOCKERS.md`
[^5]: Decisions register, DEC-033. `docs/DECISIONS.md`
[^6]: Decisions register, DEC-032. `docs/DECISIONS.md`
