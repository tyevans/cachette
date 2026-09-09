---
id: 0057
title: More than one way to win decides a game
status: Shaped
created: 2026-09-08
---

# PRD-0057 — More than one way to win decides a game

## Who this is for

A developer who builds a strategy game on this engine. The developer must know
that the game rewards a plan. A player who chooses a way to win must be able to
reach it. That player must also be able to lose to a player who chose a
different way.

A researcher who trains a player against the engine needs this second. The need
differs between the two readers, and the difference is worth stating.

A developer wants each way to win to be a plan. A plan is a way to win that a
player selects at the start and steers toward for a whole game. A game with one
live way to win offers no selection, so it offers no plan.

A researcher wants an arena that separates one player from another. A game that
one way to win decides ranks every player on one axis. Two players that learned
different games then rate the same, and the arena hides what each one learned.

The game fails both readers in the same way. A game with one live way to win and
three that decide nothing is a game with one way to win and three decorations.

## What the person cannot do today

A developer cannot tell whether a way to win is a plan.

The engine holds four ways to win today: domination, territory, wonder and
renown. A run ends by one of them, and a reader states which one ended it.
Nothing states whether a faction could have chosen another one and won.

Four behaviours show the cost.

**A way to win can be out of reach by arithmetic.** One way asks a faction to
reach a target. One source raises the quantity toward that target, and a whole
game supplies less than the target asks. Nothing fails, because a way to win
that nobody can reach looks the same as a way to win that nobody chose.

**A faction cannot read its true distance to a way to win.** One reading
measures that distance against a requirement that the win does not use, so the
reading cannot reach the winning value. A player that reads it sees no progress
while it makes progress. That player cannot learn to steer toward the path, and
no test says so.

**A weak player wins by default.** One way to win ranks the factions at the tick
limit, so it decides a game that nobody won. A faction that makes no plan takes
it. The game cannot separate a plan from the absence of one.

**A developer cannot repeat an unplanned way to win.** A player found one way to
win that nothing asked it to pursue, and it won every game by that way alone.
That result shows that the game holds more than one plan. The developer cannot
tell whether the game will produce it again.

## What good looks like

Each statement below can be checked. A reader checks each one against the report
of a measured set of games that rotates every seat.[^1]

- Every way to win ends at least one game in the set. The report names any way
  to win that ends none.
- No way to win ends more than a stated share of the games that the set
  decides. The share is a rule of the game, and a register holds it.[^2]
- Each way to win is reachable by arithmetic. For each one, the report states
  the most that a whole game can supply toward its requirement. Each of those
  figures is above the requirement it serves.
- The distance a faction reads toward a way to win reaches the winning value on
  the tick that the win fires. The way the engine measures a distance does not
  hold that distance away from the winning value.
- A player that pursues one way to win alone wins games by that way.
- The winning way depends on who played. The report states the winning way of
  each player separately, so a reader can see two players win by two different
  ways in the same set.
- No way to win is the outcome of making no act. A set of factions that make no
  act ends every game with no winner, and the report separates a game a faction
  won from a game that reached the tick limit.

**An equal share for each way is not the target, and this record does not ask
for one.** Four ways that each decide a quarter of the games can come from
noise. Noise is the opposite of a plan. A game where each way is equally likely,
and where no player can steer toward one, is worse for both readers than a game
where one way is common and a player can choose another and still win. The
statements above therefore bound the share of the most common way and ask that
the winning way follow the player, the seat and the world. They do not ask that
the four shares match.

## What this does not do

- It does not make the four ways equally likely. The section above says why.
- It does not state a share, a target, a rate or a tick limit. A blocker
  governs each of those values.[^3]
- It does not add a way to win, and it does not remove one. How many ways exist
  is a rule of the game.
- It does not decide how a faction chooses a way to win. It does not decide how
  a player is rewarded for reaching one. Both are mechanisms, and a mechanism
  belongs in a decision record and in a backlog item.
- It does not ask for a training method. The need holds whether or not anybody
  ever trains a player against this engine.
- It does not rank players. Whether a player is skilled is a separate reading.
- It does not build the report. A separate need holds the command that plays a
  fixed set of seeds and reports on the set.[^1]
- It does not tune the game. It says which ways to win are live. The tuning is
  the work that follows the report.
- It does not state how many players the measured set must hold.

## What it costs at the target scale

The cost driver is the number of games, times the length of one game. Nothing
in this record adds to the cost of a step.

Three properties follow. A solution must have all three.

- **The requirement of each way to win scales with what the world supplies.** A
  fixed count is not a plan at two sizes. A count that a small world cannot
  reach in a whole game, and that a large world reaches early, gives a dead way
  to win at one size and a trivial one at the other. At 16.7 million tiles and
  one million units, the same rule must leave each way to win live.
- **The distance a faction reads is a fixed-size answer for each faction and
  each way to win.** It does not grow with the population. A reading that walks
  one million units cannot run on every tick that a player decides on.
- **The measured set costs a whole game for each seed.** It is long by design,
  and it is not a merge gate.

No cost figure appears here. One blocker governs every cost figure this project
holds, and it says which figures are measured and which are derived.[^4]

## Which blockers govern this

**One blocker holds the rules of the downstream game.**[^3] Every share, every
target, every rate and the tick limit are rules of that game. This record states
none of them. A register holds each provisional value under that row, with the
reading that produced it.[^2]

**One blocker governs one of the four ways to win directly.**[^5] Nobody has
said what raises and lowers renown. One source raises it, nothing lowers it, and
both the source rate and the target are provisional. Whether that way to win is
reachable therefore rests on two values that the row holds open.

**One blocker governs every cost figure.**[^4] Every cost statement above
states a shape and not a number.

**One blocker governs the scale at which each way to win must stay live.**[^6]
Nobody has said what world size, unit count and faction count the downstream
game runs at. This engine's own target is decided, and the scale section states
the need against that target.

Two blockers opened on 8 September 2026, and neither governs this record. One
asks how the attack column and the armour column of a unit type combine into one
strength.[^7] One asks whether a settlement holds people apart from units.[^8]
Both govern what a player reads about its army and about its people. Neither is
a requirement of any way to win.

This record depends on factions playing a game to an end, and on the report of a
measured set of seeds. Both needs are accepted.[^9] [^1]

## References

[^1]: PRD-0053, a game is balanced across seeds. `docs/product/accepted/prd-0053-a-game-is-balanced-across-seeds.md`
[^2]: Balance register, the win-path share and the renown rows. `docs/reference/balance.md`
[^3]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^4]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^5]: Blockers register, BLK-150. `docs/BLOCKERS.md`
[^6]: Blockers register, BLK-051. `docs/BLOCKERS.md`
[^7]: Blockers register, BLK-158. `docs/BLOCKERS.md`
[^8]: Blockers register, BLK-159. `docs/BLOCKERS.md`
[^9]: PRD-0048, a developer watches factions play a game to an end. `docs/product/accepted/prd-0048-a-developer-watches-factions-play-a-game-to-an-end.md`
