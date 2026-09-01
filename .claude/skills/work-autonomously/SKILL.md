---
name: work-autonomously
description: Use when asked to work autonomously, to keep building, or to run parallel agents on this repository. Drives the product, decision, backlog and register systems together and dispatches isolated workers.
---

# Work Autonomously

You are the dispatcher. You allocate, you merge, and you report. Workers build.

Read `CLAUDE.md` and every file in `.claude/rules/` before dispatching. They
bind you and they bind every worker you dispatch.

## The four systems, and the order between them

1. **Product.** A need, for one audience. `docs/product/`. A record reaches
   `shaped/` only when it answers the six gate questions. It never states a
   structure.
2. **Architecture.** A binding constraint. `docs/adrs/`. Apply the
   three-condition test before writing one. Most decisions need no record, and
   a record for a subsystem that does not exist is the failure this project
   keeps finding.
3. **Backlog.** The work. `docs/backlog/`. An item is refined when its impact
   review is done, not when it sounds ready.
4. **Registers.** Findings, blockers and decisions. A correction the project
   had to make belongs in the findings register, or the next worker repeats it.

A backlog item is the only join between a need and a constraint. Keep it so.

## Allocate every number yourself, before dispatch

Give each worker its own range for backlog items, records, findings, decisions
and blockers. **Never let two workers read a next-number line.** Three
registers cannot hold a reserved range at all, so the reservation lives only in
your prompt and you must expect to renumber at merge. When you renumber, sweep
the whole tree for citations that moved and put the search command in the
commit body.

## Dispatch along these axes

- **Isolation.** One worktree for each worker. Scratch files stay inside it.
- **File disjointness.** Name the files each worker owns. The world step
  function is the bottleneck; only one worker changes it at a time.
- **Dependency.** Say what each item unblocks. Storage precedes the systems
  that read it.
- **Scope.** State what is out of scope, so a worker stops rather than spreads.
- **Evidence.** Require the worker to break each rule it tests, watch the test
  fail, restore the source, and report which defects were caught. A green suite
  is not evidence.
- **Honesty.** Require a report of what was left undone, and require any figure
  to name the machine that produced it.

## Merge

You merge; workers do not. Merge one branch at a time and rebase the rest.

**Never merge a golden file.** Regenerate it from the merged source and verify
at more than one thread count. Both sides of the conflict are stale.

Run the whole check command before merging, and wait for continuous
integration. Do not hand over a red pipeline.

## What a good run refuses to do

Ship a record nothing needed. Mark a need met that you did not check by running
it. Claim a measurement nobody took. Build ahead of a stated need. Report a
count in a record instead of a commit message.
