---
id: 0030
title: A developer builds a game the engine did not anticipate
status: Shaped
created: 2026-09-03
---

# PRD-0030 — A developer builds a game the engine did not anticipate

## Who this is for

A developer who builds a strategy game on this engine, and whose game is not
the game the engine was written against.

One real project is that developer. In it a god directs a congregation, and the
god is played by a person or by a language model. The project owner named what
a god will want to do: move units somewhere, explore, build things, gather
units in a place, attack another god's units, and convert people.

The audience is not the modeller and not the researcher. Both reach the engine
through the same package, and both gain from this. Neither states the need as
sharply, because neither arrives with a list of things their world must do.

## What the person cannot do today

**A developer can only ask the engine for the things it already does.**

The control plane offers a small set of commands. It creates units at addresses
the caller names. It removes them. It tells a set of units to collect one kind
of resource. It founds a settlement, and it founds a starting group. It changes
what a settlement prefers, and it sets how often one schedule runs.

Six things on the developer's list are outside that set. A unit cannot be sent
to a place. Nothing is hidden, so nothing can be explored. A unit cannot be
told to build, although the engine below the boundary can. Existing units
cannot be assembled anywhere. Two sides that meet do nothing. A unit's side
never changes.

**The developer cannot tell which of these is refused and which is merely
absent.** The engine has strong rules about what a control plane may do, and a
developer who meets a missing command reasonably reads it as a rule. Some of
these are the architecture. Most are not. Nothing tells the developer which.

This has three costs.

**A developer designs around walls that are not there.** A game shaped to avoid
a rule that does not exist is a smaller game than the engine could carry.

**A developer asks for the wrong repair.** A missing command and a missing
model need different work, and a developer who cannot tell them apart asks for
the cheap one and gets the expensive one.

**A developer abandons the engine at the second program.** The first program
runs a world. The second asks the world to do the thing the game is about.

## What good looks like

Each statement below can be checked.

- A developer writes down what their game needs a set of units to do, and for
  each thing the project can say whether the engine does it, configures it, or
  cannot do it yet.
- A developer sends a set of units toward a place they choose, and the units
  move toward it without the developer naming a route.
- A developer tells a set of units to build, and a watcher sees the ground
  change.
- A developer describes a set of units by a property rather than by holding
  every identity, and gives that set one order.
- A developer adds a new kind of thing to build by supplying data, and writes
  no engine code.
- A developer who asks for something the engine refuses meets an error that
  says why it is refused, and not silence.
- No command a developer sends makes the engine visit entities from the control
  plane, and none of them costs the developer a loop.
- The same orders on the same world give the same result at every thread count.

## What this does not do

**It does not promise every verb a game could want.** The need is that the
project can answer for each one, and that the answers stop being "no" for
reasons that are only history. A record that promised an unbounded vocabulary
would have no bound at all.

**It does not let a developer write behaviour that runs inside a step.** A
developer supplies data and sends commands. A developer never supplies a
function that the engine calls while the world runs. That is the one refusal
this record keeps, and it keeps it because the project cannot recover
determinism after it is lost.

**It does not let two units in one place be sent different ways.** The engine
decides movement for a block of ground, so a developer splits a crowd by where
it stands and not by naming individuals. This record does not remove that.

**It does not give each unit its own preferences.** Every unit alive shares one
set of weights, so two units in one place with the same need choose alike. That
is a separate need with a separate answer, and folding it in here would let
this record claim a gap it does not close.

**It does not carry a conversation between sides.** A message between two
players is the game's business, and the engine holds no channel for one.

**It does not state how the engine answers any of it.** Whether a thing is a
new command, a new column or a new table is an architectural question, and this
record states none of it.

## What it costs at the target scale

The engine holds far more tiles and units than a script can visit, and the
scale constants table holds the figures.[^1]

**The bound that matters is which term each answer follows.** An answer
computed for each unit costs the population. An answer computed for each block
of ground costs the block count, and the block count does not change when units
are born. Every command this record asks for must take the second shape, and
the project already states that preference in general terms.[^2]

**The cost of a command must not follow the size of the set it names.** A
developer who orders one unit and a developer who orders a million must pay for
the description and not for the population.

**Two of the six things on the list add state to every unit, and that is the
cost this record cannot avoid.** A contest between two sides needs something on
a unit to contest. A unit that has a type needs a place to keep it. Each is a
small addition to a large array, so each costs the population once and
permanently.

**No figure is stated here.** One blocker governs every cost figure in this
project, and it says which figures are measured and which are derived.[^3] The
statements above argue about which term a cost follows. They are not results.

## Which blockers govern this

**One blocker governs every cost claim above.**[^3]

**One blocker holds the whole list.** The rules of the game this need came from
are one paragraph.[^4] Six verbs are named and one rule is stated in full. What
each verb does when it succeeds, what it does when it fails, and what the world
looks like afterwards are unstated. Work continues, because the shape of each
answer does not depend on the detail, and because the engine gaps are real
whatever the game turns out to be.

**One blocker holds the scale.** Nobody has said how large the downstream game
runs.[^5] The whole bargain about loops is priced at one million units. A game
that runs ten thousand pays a different price, and several conclusions about
what is worth building change with it.

**One blocker holds the fighting.** Nobody has run two sides into contact, so
nobody knows whether a fight at this engine's granularity looks like a
fight.[^6] It governs the fifth item on the list and nothing else.

**One blocker holds a smaller question that touches building.** Whether a built
thing changes hands when the ground under it does is unanswered.[^7] A
developer who builds on ground that changes hands meets it at once.

**Nothing here waits on a question about the control plane rules.** Those are
decided. What is open is which of the six things are inside them, and this
record exists so that the project answers that rather than the developer
guessing.

## References

[^1]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^2]: Project orientation, the design principles. `CLAUDE.md`
[^3]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^4]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^5]: Blockers register, BLK-051. `docs/BLOCKERS.md`
[^6]: Blockers register, BLK-052. `docs/BLOCKERS.md`
[^7]: Blockers register, BLK-036. `docs/BLOCKERS.md`
