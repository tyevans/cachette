# Product Priority (Index)

This document is an **index**. It states which product needs the project works
toward next, and why each sits where it does.

**It holds no title and no status.** The registry holds those, and a second
copy here would go stale.[^1] This document holds the order.

Rule 4 of the documentation rule exempts a table that lists files as data.[^2]

## How to use this

A record here is a need, not a plan. The backlog says what work answers it.[^3]

**Every record not yet shipped appears exactly once.** A check enforces it.[^4]

**A high position is not acceptance.** Only a reviewer moves a record past
`Shaped`, and this list does not do that.[^1]

## Being built now

| No. | Why it sits here |
|---|---|
| 0012 | Eight of nine statements are met in a run. Nothing is born, so the end population is still the starting number.[^6] |
| 0007 | Tiles hold resources and units take them. Consumption is what completes it. |
| 0006 | A watcher can see the ground now. One statement still fails: two values name a tile's owner. |
| 0009 | A unit reads nothing. This is the record the project points at. |
| 0011 | Nothing is born and nothing dies. The population is still a setting. |

## Next

| No. | Why it sits here |
|---|---|
| 0013 | Consumption. Nothing makes a resource matter until something needs it. |
| 0014 | Housing. It bounds the population that 0011 will grow. |
| 0017 | Work assigned to the people who can do it. |
| 0008 | A unit changes the ground it stands on. |
| 0003 | A world worth looking at. The viewer trails the engine badly. |
| 0031 | A god knows whose ground its people stand on. **It is the central mechanic of a game the project owner named**, and the engine already holds every part of the answer: a tile carries a holder, and a unit carries a faction and a tile. What is missing is a read. The answer for the whole world is one word for each faction, so it does not follow the population and it needs no selector. It sits here rather than under `Later` because it is the cheapest stated need on this list and it ships without any new verb. |
| 0032 | A god knows what its ground is rich in. The engine now answers it, and the answer is a score that nothing consumes. It sits beside 0031 because it comes from the same game and it is the same shape of read: a fixed-size answer over the ground a faction holds. It sits below 0031 because presence gates a mechanic and variety only informs one, and because BLK-110 holds what the score should change. |
| 0034 | Two players hold each other to a future delivery. **The project owner asked for it directly**, and the engine now answers it: two players offer, counter, agree, and the world moves the goods and fails the deal that is not kept. It sits here rather than under `Later` because the need is answered and the record states what is still open about it, and above 0020 because a deal that binds a delivery is what gives a named destination a reason to exist. |
| 0020 | A unit goes somewhere it cannot see. Every option the engine has is a gradient, so a place cannot be named and a unit cannot come back from anywhere. It sits below 0008 because gathering and a store both exist and nothing carries between them, so this is the need that connects what the world already holds. |
| 0035 | A god takes the people of another god. It sits here because the engine now answers it and the record is the statement of what was answered. What is not settled is what belief costs a god, and BLK-122 holds that. It sits above 0036 because 0036 is the reading half of the same need and it cannot be judged before this one is. |
| 0036 | A god sees where it is winning people. It sits directly under 0035 because it is the observation half of the same change, and because a mechanic a player cannot see is one they cannot play. It is met by the report of a step, and what stays open is what the downstream game shows a player, which is a game rule. |

## Later

| No. | Why it sits here |
|---|---|
| 0019 | An agent can ask the running engine what it holds. It sits under `Later` because most of it is met: an agent builds a world, runs it, and reads what it holds through a protocol, and the derived answers can be checked against the parts they came from. What is not met is the last statement, that a gap reaches the next agent, and item 0211 holds that work. |
| 0021 | The control plane has no published documentation. It sits under `Later` because the selector interface and the verb interface are not written, and prose written against a surface that still moves decays faster than anybody repairs it. It sits above 0018 because nothing else on this list gives the engine a front door for a person who has not read the source. |
| 0030 | A developer builds a game the engine did not anticipate. Six of the things a named downstream game must do sit outside the control plane's command set, and a developer cannot tell which are refused by the architecture and which are merely unbuilt. It sits under `Later` because BLK-050 holds the whole list: the rules of that game are one paragraph, and what each verb does on success or failure is unstated. It sits above 0018 because the gaps it names are real whatever the game turns out to be, and because 0031 is one bounded piece of it that can ship first. |
| 0018 | A deposit never comes back. Wants 0013, because nothing drains the world until units consume. |
| 0015 | Family and descent. Wants the character tier to be doing something first. |
| 0016 | A ruler. Wants a family to succeed from. |
| 0010 | Goods moving. Nothing holds a surplus until production and consumption run. |
| 0001 | A faction sees only what it observes. Wants a reason to hide anything. |
| 0004 | Weather. Wants a world whose inhabitants care about it. |
| 0024 | A run stays eventful. It sits here because it states an outcome that 0013, 0018 and 0020 produce between them, so it cannot be worked before they are. It is the record that says whether they were enough. |
| 0022 | A caller can name the people the world holds. It sits here because it is small and blocks an audience the engine already claims: an agent cannot be attached to anybody the founding seated. |
| 0023 | An observer reads what happened near a place. It sits below 0022 because a caller with no identities has nobody to read around, and because the whole log still answers the question at the scale anybody runs today. |

## References

[^1]: Product registry. `docs/product/REGISTRY.md`
[^2]: Documentation Rules. `.claude/rules/documentation.md`
[^3]: Backlog priority. `docs/backlog/PRIORITY.md`
[^4]: The priority check script. `scripts/check_priority.py`
[^5]: Reviews, the second review of the renderable example. `docs/reviews/`
[^6]: Reviews, the founding and deposit product records. `docs/reviews/0149-the-founding-and-deposit-product-records.md`
