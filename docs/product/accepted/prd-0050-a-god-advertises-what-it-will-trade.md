---
id: 0050
title: A god advertises what it will trade
status: Accepted
created: 2026-09-05
---

# PRD-0050 — A god advertises what it will trade

## Who this is for

A developer who builds a game in which a god directs a congregation. The god
is a person or a language model, and it acts through the control plane.

A modeller needs this second. A modeller wants to see a market form between
populations that never spoke. A posted want is the smallest market there is.

## What the person cannot do today

A god cannot find a trading partner.

Two gods can deal with each other today. One offers, the other counters, and
the two agree or not. The offer must name the other god. A god that has a
surplus and no idea who wants it has nobody to offer it to.

This has three costs.

A god trades blind. It offers what it has to whoever it guesses, and most
guesses are refused. A refused offer costs a turn and teaches the god nothing.

A god cannot be found. A god that needs grain has no way to say so to
everybody at once. It can only ask each god in turn.

A faction that acts on its own cannot trade at all. A faction that chooses
for itself needs a fact to choose from. No fact says who wants what.

## What good looks like

Each statement below can be checked.

- A god posts what it offers and what it wants, in one call, without naming a
  partner.
- A post names a good and a quantity. It says whether the good is offered or
  wanted, and what the god asks in return.
- Any god reads the posts of any other god. Reading is free. It moves no
  standing and costs the reader nothing.
- A post is bounded. A god holds a bounded number of posts, and a post past
  the bound does not raise the count.
- A faction that acts on its own writes its own posts from what its sites
  hold.[^1] It reads the posts of others before it offers a deal.
- A faction that acts on its own offers a deal only where its own surplus
  meets a posted want.
- A post makes no promise. A god that posts and then refuses the offer that
  answers it is permitted.
- The same seed gives the same posts, at every thread count, on every run.

## What this does not do

- It does not make a deal. A post is an invitation. The deal is a separate
  act that exists today.
- It does not set a price. What a good is worth is a rule of the downstream
  game, and nobody has stated it.
- It does not match posts for anybody. A god that wants a match reads the
  posts and offers. The engine does not offer on its behalf.
- It does not decide how the posts are stored. That is an architectural
  question, and it belongs in a decision record.
- It does not let a god post on behalf of another god.
- It does not hide a post. Every god reads every post. Secrecy is a separate
  need.

## What it costs at the target scale

The cost driver is the number of factions and the bound on posts. It is not
the number of units and not the number of tiles.

A god holds a fixed number of posts. The number of gods has a ceiling. The
whole market is therefore bounded by two constants, and nothing about it grows
with the population.

Three properties follow. A solution must have all three.

- What the world remembers about posts grows with the faction ceiling times
  the post bound and with nothing else.
- Reading the posts of one god costs the post bound. It starts no pass over
  the world.
- A faction that writes its own posts reads a summary of its own sites. The
  engine already keeps that summary. The faction does not walk its units.

No cost figure appears here. One blocker governs every cost figure this
project holds, and it says which figures are measured and which are
derived.[^2]

## Which blockers govern this

- **One blocker governs every cost figure here.**[^2] Every cost statement
  above states a shape and not a number.
- **One blocker holds the rules of the downstream game.**[^3] How many posts a
  god holds and what a surplus is are rules of that game. So is how often a
  faction rewrites its posts. The engine holds a value for each, and the
  blocker says that no owner chose them. This record states none of them.

This record depends on two players dealing with each other, and on a site
holding a store. Both exist.[^4] [^5]

## References

[^1]: PRD-0048, a developer watches factions play a game to an end. `docs/product/accepted/prd-0048-a-developer-watches-factions-play-a-game-to-an-end.md`
[^2]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^3]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^4]: PRD-0034, two players hold each other to a future delivery. `docs/product/shaped/prd-0034-two-players-hold-each-other-to-a-future-delivery.md`
[^5]: PRD-0010, a good moves to where it is wanted. `docs/product/accepted/prd-0010-a-good-moves-to-where-it-is-wanted.md`
