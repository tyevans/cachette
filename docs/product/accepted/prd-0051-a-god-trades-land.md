---
id: 0051
title: A god trades land
status: Accepted
created: 2026-09-05
---

# PRD-0051 — A god trades land

## Who this is for

A developer who builds a game in which a god directs a congregation. The god
is a person or a language model, and it acts through the control plane.

A modeller needs this second. A modeller wants a border that moves by
agreement and not only by force. A study can then tell the two apart.

## What the person cannot do today

A god cannot give ground away.

Ground changes hands today in one way. Units of one faction stand on it, and
the holder follows the units. A god that wants to cede a valley to a
neighbour has to march its people out. Then it hopes the neighbour marches in.

This has three costs.

A god cannot buy peace. The oldest deal between two powers is ground for
quiet, and the engine cannot express it.

A god cannot sell what it holds. Ground is the one thing every god has, and
it is the one thing no deal can name. Every deal is goods for goods.

A border cannot move without a fight. Every change of holder looks like a
conquest, because every change of holder is one.

## What good looks like

Each statement below can be checked.

- A god offers a bounded set of ground it holds as its side of a deal. The
  offer names the ground, and the deal is otherwise the deal that exists
  today.
- The engine refuses an offer of ground the god does not hold, in whole or in
  part. One refusal leaves the world as it was.
- The ground changes holder when the other side delivers in full. Nothing
  carries it. The change happens at once.
- A god may ask for ground as well as offer it. Either side of a deal may be
  ground.
- A watcher reads the holder of a traded tile before the deal and after it.
  The two readings differ.
- The deal moves no unit. A unit on traded ground stays where it stands, on
  ground another god now holds.
- The engine refuses an offer of ground that carries a built thing. The
  refusal holds while the question of what happens to that thing is
  open.[^1] When the question closes, the refusal goes.
- The same seed with the same deals gives the same holders, at every thread
  count, on every run.

## What this does not do

- It does not decide what happens to a built thing on traded ground. The
  project owner holds that question, and it is open. This record expresses
  the need with that answer as a parameter.
- It does not move anybody. A unit on traded ground stays where it is. What a
  guest may do on ground another god holds is a separate need.
- It does not price ground. What a tile is worth is a rule of the downstream
  game, and nobody has stated it.
- It does not let a god take ground. Conquest exists today and is not a deal.
- It does not decide how ground is named in a deal. That is an architectural
  question, and it belongs in a decision record.
- It does not give away ground a god does not hold. A promise of ground to be
  conquered is a separate need.
- It does not say what a god may offer apart from ground. A step in standing
  as one side of a deal is a separate need.[^2]

## What it costs at the target scale

The cost driver is the size of the ground offered. It is not the size of the
world and not the size of the population.

A deal names a bounded set of ground. The engine checks that the god holds
each tile of it. On delivery it changes the holder of each tile. Both acts
cost the set and nothing else.

Three properties follow. A solution must have all three.

- What an offer costs to check grows with the ground offered, and the ground
  offered has a ceiling.
- What a delivery costs grows with the same ceiling. No unit walks and no
  carrier moves.
- The reading of how much ground each faction holds changes by the size of
  the set. No pass over the world recounts it.

No cost figure appears here. One blocker governs every cost figure this
project holds, and it says which figures are measured and which are
derived.[^3]

## Which blockers govern this

- **One blocker governs every cost figure here.**[^3] Every cost statement
  above states a shape and not a number.
- **One blocker holds whether a built thing changes hands with the ground under
  it.**[^1] The project owner holds it. Until it closes, a deal that names
  ground with a built thing on it is refused. This record states no answer.
- **One blocker holds the rules of the downstream game.**[^4] How much ground
  one deal may name is a rule of that game. So is what ground is worth. This
  record states neither.

This record depends on a place belonging to somebody, and on two players
dealing with each other. Both exist.[^5] [^6]

## References

[^1]: Blockers register, BLK-036. `docs/BLOCKERS.md`
[^2]: PRD-0049, a god declares war and makes peace. `docs/product/accepted/prd-0049-a-god-declares-war-and-makes-peace.md`
[^3]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^4]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^5]: PRD-0006, a place belongs to somebody. `docs/product/accepted/prd-0006-a-place-belongs-to-somebody.md`
[^6]: PRD-0034, two players hold each other to a future delivery. `docs/product/shaped/prd-0034-two-players-hold-each-other-to-a-future-delivery.md`
