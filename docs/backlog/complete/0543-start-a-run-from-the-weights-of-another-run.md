---
id: 0543
title: Start a run from the weights of another run
status: complete
created: 2026-09-11
implements: [ADR-0200 D1, ADR-0200 D3, ADR-0200 D4, ADR-0200 D5, ADR-0202 D1, ADR-0202 D3, ADR-0202 D6]
changes: []
creates: []
serves: [PRD-0056]
blocked-by: []
---

## Why

The owner wants to train one strategy under its own reward from the weights
that a run under another reward produced. The product record states the need
of a person who trains a policy for one faction.[^1]

Two things stopped this. The resume flag of the trainer continues the
generation counter and the best score of the run it resumes.[^2] A start under
a new reward is a new run, so it must start at generation 0 with no best
score. The launcher also copies only the tracked files to the instance, so
nothing could put a weight file of an earlier run on the machine.[^3]

## Impact review

**Governed by.** ADR-0200 D1 to D5 govern what a checkpoint stores, and how a
reader rebuilds the readout by the identity of each row.[^4] A start reads
through the same reader and writes through the same checkpoint, so it takes
those rules without a copy.

ADR-0202 D3 makes every stored figure name its seed set. D6 refuses a resume
of a centre from another feature transform.[^5] A start applies the D6 refusal
through the same method as the resume. A started run holds no best score, so
D1 selects its first centre from its own validation pass.

**Changes.** None. The start is a second caller of the refusals that the
resume already made.

**Creates.** None. The rule that a start from a file is a new run is cheap to
change, and the docstrings of the trainer state the reason. It therefore needs
no record.[^6]

**Blockers.** None. The work states no cost figure.

**Precedent.** FND-763 records a flag refusal that stopped a paid launch,
because no test drove the calls that the launcher makes.[^7] The tests of this
item drive the launcher. Recurring defect shape 1 is the risk of a second list
of refusals.[^8] The resume and the start therefore read a centre through one
method.

## Done when

- A run with the start flag begins from the stored centre, at generation 0,
  with no best score.
- Every checkpoint of that run names the source file, and the strategy and
  the generation that the source states.
- A restart of that run keeps naming the source.
- A start refuses a file of another world, another readout setting, another
  limit rule or another policy kind, and a file with no normalizer.
- The command line refuses a start beside the resume flag, with more than one
  strategy, and for a file it cannot read. It refuses before anything plays.
- The launcher refuses a missing start file, and a start or a resume in the
  run arguments, before it rents.
- The trainer on the instance receives the file on the first attempt only. A
  restart resumes.
- Each defect put back turns a named test red.

## Outcome

One commit holds the code and the tests.[^9]

**The trainer.** The trainer takes `--start-from PATH`. The run takes the
centre of the file and nothing else of its source run. It starts at
generation 0 with no best score, and it writes its files under its own
strategy name. The first line it prints names the file.

Every checkpoint now names the strategy of its run under the key `strategy`.
A checkpoint of a started run also names the source file under
`started_from`. It names the strategy and the generation of the source under
`started_from_strategy` and `started_from_generation`, when the source states
them. A resume reads the three entries back and writes them again.

**One method reads a centre.** The resume and the start both call
`Checkpoint.read_centre`. The refusals of another world, another normalizer,
no normalizer, another readout setting and another limit rule moved there.
The refusal of another policy kind joined them, because a start across two
strategies can meet a centre of the other kind.

**The command line.** The trainer refuses a start beside `--resume`, and a
start with more than one strategy. It reads the file through the same method
before anything plays. It runs this check on the train path and on the plan
path. The launcher asks for the plan before it rents, so a file of another
world fails before the rental.

**The launcher.** The launcher takes `CACHETTE_TRAIN_START_FROM`. Before it
rents, it refuses a missing file, and a file name that a command line splits.
It also refuses `--start-from` or `--resume` in `CACHETTE_TRAIN_ARGS`, and
each abbreviation of them. The plan call carries the file, and the preview
names it. After the tracked files, the launcher copies the file to a
directory of its own in the home of the instance.

The first attempt of the trainer receives `--start-from`. A restart after a
fault receives `--resume`. A restart receives the file again only when no
resume point exists yet, because a resume with no resume point starts from
the seeded draw.

**How to launch.** Name one strategy. The launcher refuses the run otherwise.

```sh
CACHETTE_TRAIN_START_FROM=runs/graviton/RUN/learn/wonder-structured-latest.npz \
CACHETTE_TRAIN_ARGS="--only renown-structured --world-extent 48 \
--tick-limit 2500 --population 128 --seeds 12 --validation 24 \
--train-readout-only --limit-is-loss --generations 20" \
scripts/graviton-train.sh --dry-run
```

Read the preview, then run the same command without `--dry-run`.

**What changed from the plan.** The plan asked for the refusals of the
resume. The kind refusal is new, and the resume takes it too. The key
`strategy` is new, because no checkpoint stated its strategy before. Without
it the provenance could name no source strategy. The rule for a restart
before the first resume point is new. The plan call now shows the reason of
a refusal, and FND-768 records why.[^10]

**What was left undone.**

- The command line check holds no feature normalizer, because the trainer
  derives it from played episodes. A file with no normalizer, or with another
  one, therefore fails on the instance. It fails after the reference sample
  and before generation 0, and the launcher has already rented the machine.
- On the instance the combined log starts with the throughput probe. The
  start line is the first line of the trainer, and not the first line of that
  file.
- The run report does not name the start file. The checkpoints do.
- A run that starts from a started run names its own source only, and not the
  source of that source.

**The defects put back.** Each row put one defect into the source and ran the
named tests or the probe. Each turned red, and the source was restored.

| Defect put back | Red |
|---|---|
| A start continues the counter of its source | the stored centre test |
| A start keeps the best score of its source | the stored centre test |
| The trainer prints no start line | the stored centre test |
| A written file names no start point | the file test, the restart test |
| A resume drops the start point | the restart test |
| Every checkpoint names a start point | the seeded draw control |
| A start reads the file past the shared refusals | all five refusal cases, the command line world case |
| The kind refusal is gone | the kind refusal case |
| The trainer takes a resume and a start together | the resume and start test |
| The trainer reads the file after the first episodes | the missing, empty and text cases |
| The train path of the command line does not refuse | the three flag cases, the three file cases |
| The command line takes more than one strategy | two command line cases, the dry path strategy test |
| The plan path does not refuse | the dry path strategy test |
| The command line prints no start line | both cases of the first line test |
| The plan call omits the file | the dry path plan test, the dry path strategy test |
| The plan call hides the reason of a refusal | the dry path strategy test |
| The preview does not name the file | the dry path plan test |
| The launcher does not refuse before the rental | the missing file test, four argument cases, one probe case |
| The first attempt does not receive the file | the first attempt test |
| A restart receives the file again | the first attempt test |
| A restart with no resume point resumes | the restart before a resume point test |
| A run with no start file receives one | the one box instance test |
| The refusal matches on four characters | the probe case for a style run |
| The refusal fires with no start file | the probe case for no start file |
| A file name that a command line splits passes | the probe case for a split name |

**Registers.** FND-768 is new. No blocker and no decision opened or closed.

**Gates.** The item ran only its own tests, the probe, and the linter and
the formatter on the files it touched. The dispatcher runs the whole check
command on the settled tree. This item does not state that result.

## References

[^1]: PRD-0056, a learner plays one faction against the controllers. `docs/product/accepted/prd-0056-a-learner-plays-one-faction-against-the-controllers.md`
[^2]: The trainer, the checkpoint of a run and its resume. `python/cachette/learn/train.py`
[^3]: The training launcher, the copy of the tracked files. `scripts/graviton-train.sh`
[^4]: ADR-0200, a stored policy names each action row by verb and coordinates, decisions D1 to D5. `docs/adrs/draft/adr-0200-a-stored-policy-names-each-action-row-by-verb-and-coordinates.md`
[^5]: ADR-0202, a run selects on the win share and every published figure names its seed set, decisions D1, D3 and D6. `docs/adrs/draft/adr-0202-a-run-selects-on-the-win-share-and-every-published-figure-names-its-seed-set.md`
[^6]: Decision Record Scope, section 1. `.agents/rules/adr-scope.md`
[^7]: Findings register, FND-763. `docs/FINDINGS.md`
[^8]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
[^9]: The commit "Start a training run from the weights of another run", on the branch of this item.
[^10]: Findings register, FND-768. `docs/FINDINGS.md`
