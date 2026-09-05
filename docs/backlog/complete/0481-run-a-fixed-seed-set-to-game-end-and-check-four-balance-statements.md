---
id: 0481
title: Run a fixed seed set to game end and check four balance statements
status: complete
created: 2026-09-05
implements: [ADR-0148, ADR-0040 D1, ADR-0040 D2, ADR-0001 D4]
changes: []
creates: []
serves: [PRD-0053]
blocked-by: [BLK-007, BLK-050]
---

## Why

**Nothing says whether the game is fair.** One seed may hand every game to one
seat or to one path, and no run would say so. This item is pass 10 of the
living world game layer.[^1]

A recipe `just balance` runs a fixed seed set to game end. It checks four
statements against thresholds in the balance register.[^2]

1. No win path wins more than its share of the seeds.
2. No seat wins more than its share of the seeds.
3. Every game ends before the tick limit in more than a stated share of the
   seeds.
4. Every subsystem count is nonzero in every seed.

The harness is long, so it is not a merge gate. It runs in the slow test
recipe, on the schedule that recipe runs, and before any commit that changes
a balance value. Its output names the seed set and every failing seed.

**This pass builds the harness and does not set a value.** Every share and
floor of the balance register is a rule of the downstream game, and BLK-050
holds that nobody has written those rules down.[^5] The harness reads each
share from the register. While a share is unset the harness reports the
observed share and passes. When a share is set the harness passes or fails
against it. The observed shares go into the derivation column, with the date,
the seed count and the machine, and the value column stays unset.

**This pass does not touch `fn step` in `world.rs`.** It touches no Rust at
all. One win path exists today, so statement 1 reports one path at the whole
share until pass 8 lands the others.

## Impact review

**Governed by.** ADR-0148 holds that a game end is recorded once and that the
record carries the winner, the path and the tick. The harness reads that record
and nothing else about the end. ADR-0040 D1 and D2 hold that Python is a
control plane: it loops over seeds and over factions, and it reads the score
and the census, which are aggregates the engine already holds. It names no
tile and no entity. ADR-0001 D4 holds that one binary gives one answer at any
thread count. The harness takes a thread count, and a test runs one seed set
at two thread counts and compares the reports byte for byte.

**Changes.** None.

**Creates.** None. The harness is a reader of a register and a runner of the
engine. The register holds the values and the record holds the game end.

**Blockers.** BLK-007 governs the cost of one run and so the tick limit and
the end share.[^5] BLK-050 governs every share and every floor: the win-path
share, the seat share, the end share and the seed set.[^6] The harness reads
each as a parameter and invents none. A run on a development machine is a
run on a development machine, and the report says so.

**The seed set.** The default set is derived from one base seed by one stated
rule in the code, so a reader can regenerate it, and `--seeds` overrides it.
Which seeds and how many is a register value under BLK-050, so the default is
a fixture and not a value. The seed that reaches an extreme, one faction with
no units, is the fixture of the census gate of item 0472 and not of this
harness.

**A path that cannot fire.** Statement 1 reports the share of every path that
won at least once. A path that never wins has a share of zero and the report
says which paths won. Renown is behind BLK-150, and the register row for the
renown target is unset.

**Precedent.** FND-174 records that neither determinism test defends a whole
game.[^3] The harness runs whole games and a test compares two thread counts
over them. FND-051 records that a fixture chosen for realism hides the defect
it should show.[^4] The failure test therefore sets a threshold that the run
must miss, through a copy of the register, and asserts the exit code.
FND-320 records that nothing regenerates the type stub.[^8] The harness adds
no reader, so the stub does not change.

**Serves.** PRD-0053, a game is balanced across seeds.[^7]

## Done when

- `python -m cachette.balance` runs a seed set to game end or to the tick
  limit, and writes one table to standard output and one JSON file. The JSON
  holds, per seed, the winner, the path, the tick and the census, and then
  the four statements.
- The thresholds come from the balance register, parsed from the markdown
  rows. One declaration site. A test asserts that the parser reads the real
  file and finds the four balance-share rows.
- While a threshold is unset the harness prints `unset: reporting only` for
  that statement and exits 0. When a threshold is set, the statement passes
  or fails and the exit code says so. A test proves each exit path through a
  copy of the register.
- The same seed set at 1 and 2 threads gives byte-identical JSON, and a
  different seed set gives different JSON, so the test can fail.
- Python loops over seeds and factions only. A whole-tree search of the
  module shows no call that names an entity.
- `just balance` exists, `just check` does not call it, and the recipe
  comment says when it runs.
- The harness ran once on the demonstration extent on a development machine,
  the whole output is in the commit body and the report, and the observed
  shares are in the derivation column of the register with the date, the
  seed count and the machine. The value column stays unset under BLK-050.
- No test asserts on wall time.

## Outcome

The harness landed as one Python package with a command line entry point and
one recipe, `just balance`. It reads the four share rows of the balance
register from the markdown table, so the register is the one declaration site.
Every share is unset, so every statement reports and passes. No value column
changed. The observed shares are in the derivation column.

The harness ran once on 2026-09-05 on one development machine, ty001-ubuntu,
an x86-64 laptop. This is one run on a development machine and it says
nothing about the target platform. The output follows.

```
balance: extent 256, 4 factions, tick limit 2000, 8 seeds

                seed  winner  path          tick  census
   81985529216486895       3  territory     2000  units=246 settlements=4 seats_filled=0 characters=235 upgrades_complete=4218 luxury_tiles=8 storms_raised=1 contracts=0 controller_commands=0 controller_refused=0 game_ended=1
11482700348539685380       0  territory     2000  units=193 settlements=4 seats_filled=0 characters=226 upgrades_complete=3784 luxury_tiles=8 storms_raised=1 contracts=0 controller_commands=0 controller_refused=0 game_ended=1
 4436671094153332249       2  territory     2000  units=196 settlements=4 seats_filled=0 characters=163 upgrades_complete=2469 luxury_tiles=8 storms_raised=1 contracts=0 controller_commands=0 controller_refused=0 game_ended=1
15837385913476530734       0  territory     2000  units=200 settlements=4 seats_filled=0 characters=246 upgrades_complete=4439 luxury_tiles=8 storms_raised=1 contracts=0 controller_commands=0 controller_refused=0 game_ended=1
 8791356659090177603       1  territory     2000  units=216 settlements=4 seats_filled=0 characters=228 upgrades_complete=4585 luxury_tiles=8 storms_raised=1 contracts=0 controller_commands=0 controller_refused=0 game_ended=1
 1745327404703824472       1  territory     2000  units=186 settlements=4 seats_filled=8 characters=173 upgrades_complete=2682 luxury_tiles=8 storms_raised=1 contracts=0 controller_commands=0 controller_refused=0 game_ended=1
13146042224027022957       1  territory     2000  units=233 settlements=4 seats_filled=0 characters=215 upgrades_complete=3780 luxury_tiles=8 storms_raised=1 contracts=0 controller_commands=0 controller_refused=0 game_ended=1
 6100012969640669826       2  territory     2000  units=213 settlements=4 seats_filled=0 characters=234 upgrades_complete=4142 luxury_tiles=8 storms_raised=1 contracts=0 controller_commands=0 controller_refused=0 game_ended=1

statement 1, Win-path share (unset): unset: reporting only; territory 8/8
statement 2, Seat share (unset): unset: reporting only; 0 2/8, 1 3/8, 2 2/8, 3 1/8
statement 3, End share (unset): unset: reporting only; ended before the tick limit 0/8
statement 4, Seed set (unset): unset: reporting only; zero in every game: contracts, controller_commands, controller_refused
```

Two things the run showed, and this item did not act on. The seat count
`seats_filled` was zero at the end of seven games and eight in one, so the
seats empty during a long game. The controller counts are per tick, so they
are zero at every game end by construction of the game end record, and the
census statement will name them in every run until a reader counts them over
the whole game.

## References

[^1]: Design: the living world game layer, sections 10.2 and 13. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^2]: Balance register. `docs/reference/balance.md`
[^3]: Findings register, FND-174. `docs/FINDINGS.md`
[^4]: Testing Rules, section 2a. `.agents/rules/testing.md`
[^5]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^6]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^7]: PRD-0053, a game is balanced across seeds. `docs/product/accepted/prd-0053-a-game-is-balanced-across-seeds.md`
[^8]: Findings register, FND-320. `docs/FINDINGS.md`
