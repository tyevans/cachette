# Roadmap (index)

This document is an index. It names the milestones that carry Cachette from a
renderable example to an autonomous society, and it says what each milestone
must deliver. It holds no design. A milestone becomes work through a product
requirement record, then through decision records, then through backlog
items.[^1] [^2] [^3]

## The goal

A rich simulated world, where units live, work, trade, fight and belong to a
kingdom, and where a person who watches it can tell a story about what
happened. Emergence is the target. A scripted narrative is not.

Two properties govern every milestone. The simulation stays deterministic at
any thread count.[^4] The state holds no floating point number.[^5]

## The milestones

A milestone is complete when its product record moves to `shipped/`.

| # | Milestone | The world gains |
|---|---|---|
| M0 | The renderable example | A window, a scrolling world, soldiers that move |
| M1 | Terrain | A generated world of tile kinds, height and passability |
| M2 | Weather | A field over the terrain that changes with the season |
| M3 | Kingdoms | A territory, a border, and a claim on a tile |
| M4 | Resources and gathering | A deposit on a tile, and a unit that takes from it |
| M5 | Workers and improvements | A unit that builds, and a tile that carries the work |
| M6 | Warrior behaviour | A soldier that answers the world rather than a draw |
| M7 | Trading | A price, a route, and a good that moves between kingdoms |
| M8 | Lives and employment | A unit that is born, takes a job, ages and dies |
| M9 | The HUD | A viewer that says what is happening and why |

A cross-cutting track runs beside them: the cost of a step. The first
measurement is in the tree, and it says the derived budget does not hold at
the target scale.[^6]

## How a milestone runs

1. **Shape.** A product record answers the six questions and moves to
   `shaped/`.
2. **Decide.** An impact review names the records that govern the work. Work
   that creates a decision writes the record before the code.
3. **Refine.** Backlog items enter `refined/` with their governing records
   named.
4. **Build.** Each item becomes a branch and a pull request.
5. **Close.** The registers are updated and the product record ships.

## How the work runs in parallel

A milestone is one movement of work, and one agent owns it end to end. Two
milestones run at the same time only when they touch different files. The
rule that decides this is simple: two agents never hold the same source file.

An agent works in its own worktree and delivers a pull request. The
integrator merges. An agent never merges its own work.

## References

[^1]: Product requirement records. `docs/product/README.md`
[^2]: ADR Registry. `docs/adrs/REGISTRY.md`
[^3]: Backlog guide. `docs/backlog/README.md`
[^4]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^5]: ADR-0002, state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^6]: Sprint sequence. `docs/backlog/SPRINTS.md`
