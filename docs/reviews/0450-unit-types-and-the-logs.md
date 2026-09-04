# Review of items 0450 and 0451, the unit type reads and the step logs

This document reports the work of two backlog items. Item 0450 lets the Python
control plane read the type of one soldier and the shared unit type table. Item
0451 lets it read four step logs that reached no caller, and it binds the one
verb without which one of those logs could never hold an entry.[^1] [^2]

The core is a Rust crate. The control plane is a Python package that imports a
compiled extension module. The core held every read and the verb below before
this work, and no binding and no Python line called one.

**One correction to the brief that started this work.** The brief said that
setting a unit's type was unbound. It was already bound, and so was the writer
of a table row. The acceptance test that one tank beats four bowmen already
existed at the Python boundary as well. The work below is therefore the reads,
the four logs and the upkeep verb, and it repeats none of that.

## 1. The impact review

The review was made before any code was written. Both items carry it, and both
were written with the review in them before the work started.

### Records that govern the work

| Record | Decision | How the work honours it |
|---|---|---|
| ADR-0001 | D4 | The work adds no simulation code and no parallel section. Every stored golden hash is unchanged. |
| ADR-0002 | D1 | Every column crosses as an integer. Each doc comment names the columns that carry the Q16.16 scale. |
| ADR-0006 | D1, D2 | Each log holds plain data with declared padding, and a caller reads it at the frame barrier. |
| ADR-0040 | D1 | The one verb takes a set and answers once. No caller loops. |
| ADR-0043 | D1 | A write over a mass tier takes a set. A read of one soldier stays singular. |
| ADR-0044 | D1 | Every new read declares that it copies each column. |
| ADR-0046 | D1 | Every refusal raises a typed error under the one root class. |
| ADR-0085 | D1, D3 | Every identity crosses whole. Every write resolves the whole set first. |
| ADR-0107 | D2 | Every word of prose is in the Rust doc comment. The type stub gained signatures and no prose. |
| ADR-0120 | D1, D2, D3 | A type stays an index. The table stays data the world holds. The default stays row zero. |

### Decision by decision, against the implementation

**ADR-0001 D4. One binary gives one answer at any thread count.** The work adds
no simulation code. A test runs the same meeting at 1, 2 and 12 threads and
compares the faction populations and the state hash. Both determinism tests
pass, and no golden file moved. **A binding must not move a golden hash, and
none moved.** Honoured.

**ADR-0002 D1. Simulated state holds no floating point number.** The attack and
the armour cross as `numpy.int32`. The deficit and the shortfall amount cross as
`numpy.int32`. The demanded and granted amounts cross as `numpy.int64`. Each doc
comment says that the value carries the Q16.16 scale and that 65536 is one whole
unit. The deeds column crosses as `numpy.uint64` and its doc comment says it
carries no scale, because it is a whole count of stock. Honoured.

**ADR-0006 D1. An event is plain data.** No binding reinterprets an event. Each
read walks the log and copies one field into one column. Honoured.

**ADR-0006 D2. Events reach Python at the frame barrier.** Every read takes the
world lock and reads a slice the last step left. No read runs inside a step.
Honoured.

**ADR-0040 D1. The boundary carries an instruction and an answer.** The upkeep
verb takes one sequence of sites and one rate, and answers once. The reads
answer with whole columns rather than with one call for each row. Honoured.

**ADR-0043 D1. The tier of a shape decides the shape of its interface.** The
type read of one soldier is singular, and its doc comment says why: a set form
would have to fail the whole call for one stale identity, or answer with a value
that stands for nothing. The read of the tile of one soldier already follows that
rule. The upkeep verb is set-valued over sites. Honoured.

**ADR-0044 D1. What copies is declared at the call site.** Each of the five new
reads says that it copies each column. Honoured.

**ADR-0046 D1. One root type holds every error.** The type read raises the view
error for a stale identity. The upkeep verb raises the view error for a stale
site, and the verb error for a rate below zero and for a number that names no
commodity. Honoured.

**ADR-0085 D1. An entity crosses as one opaque identity.** Every unit column,
site column and character column holds a whole identity. No column holds a slot
index, and each doc comment says so. Honoured.

**ADR-0085 D3. The engine resolves the identity.** The upkeep verb resolves
every site into a list, then checks the commodity against every site, and only
then writes. One refusal leaves the world unchanged. A test proves it, and a
defect put back into that path fails the test. Honoured.

**ADR-0107 D2. The prose lives in the Rust doc comment.** Every new member
carries its prose in the bindings crate. The type stub gained five typed
dictionaries and seven signatures, and the typed dictionaries carry prose
because the compiled module does not provide them. The documentation job found
101 members with prose and none without. Honoured.

**ADR-0120 D1 and D3. A unit carries a type that indexes a table.** The type
read answers the row number and nothing else. It answers zero for a soldier that
nothing gave a type. Honoured.

**ADR-0120 D2. The table is data the world is built with.** The table read
answers the rows as data. It states the width nowhere except in the length of a
column. Honoured.

### Records changed or created

None. No record is superseded, and no record was written. The reads and the verb
state no constraint that the records above do not already state, so section 1 of
the record scope rule refuses a new record.[^3]

### Registers

- **FND-460.** The upkeep rate had no caller outside a Rust test, so the
  shortfall log could never hold an entry.
- **FND-461.** A small world feeds itself, so a log fixture built from one
  measures the fixture.
- **DEC-250.** A caller learns the width of the type table from the length of a
  column, and from nowhere else.
- **Blockers.** None opened and none closed.

## 2. The shape of the type verb

The write verb was already bound, and this work did not change it. It takes a
sequence of identities and one row number, it resolves every identity and checks
the row before it writes anything, and one refusal leaves the world unchanged.

This work added the two reads beside it.

**The type of one soldier** answers a row number for one identity. It is
singular for the reason the tile read is singular.

**The table** answers two columns, an attack and an armour, with one entry for
each row. The two arrays are the same length, and that length is the number of
types the world holds.

**The width has one declaration site that a caller can read.** A second call
that answered the number would put one value in two places with nothing failing
when they disagreed. The register holds the reasoning.

**The width is fixed and the values are configurable.** The world builds the
table with every row at zero. The already-bound row writer fills a row. A row
that nobody wrote holds zero attack and zero armour, so a unit of that row
reaches nothing and nothing reaches it. The doc comment of the table read says
all of this.

## 3. Is the tank reachable from Python?

**Yes, and it was reachable before this work.** A test that proves it already
existed at the Python boundary when the work started.

This work adds a second test that proves it at a larger count and asserts on
every step rather than on one. It writes a bowman row with an attack of one
whole casualty and no armour, and a tank row with an attack of four and an
armour of two. It gives 32 units of one faction the bowman row and one unit of
another faction the tank row. It then steps.

The tank ends four bowmen on each step and never falls. The test asserts that
the tank is alive after every step, not only at the end, because the claim is
that no number of bowmen reaches it. A control test runs the same meeting with
no row written, and nobody falls.

## 4. The logs, and their columns

Five logs cross. The trade log was already bound and this work did not change
it. The four below are new.

| Read | Columns |
|---|---|
| `starved_log_columns` | `tick`, `unit`, `deficit` |
| `shortfall_log_columns` | `tick`, `site`, `amount`, `commodity` |
| `rationed_log_columns` | `tick`, `site`, `demanded`, `granted`, `commodity` |
| `promoted_log_columns` | `tick`, `unit`, `character`, `deeds`, `faction` |

Each name is the field name of the event in the Rust source, so a reader takes a
field by its name and holds no byte offset.

**What each log records, and when the engine writes it.**

- The starved log records that a shortage ended a unit. The consumption pass
  runs on a period, and the scan of the death plane runs with it.
- The shortfall log records that a site could not pay its upkeep. The rate pass
  writes it, on the same period.
- The rationed log records that a site could not serve every cohort that drew on
  it. The consumption draw writes it, on the same period.
- The promotion log records that a soldier became a character. The promotion
  pass writes it, on its own period.

**Where a value carries the fixed-point scale.** The deficit, the shortfall
amount, the demanded amount and the granted amount all carry the Q16.16 scale,
and 65536 is one whole unit. The deeds column does not: it is a whole count of
stock. The doc comment of each read says which, because a caller who does not
know is wrong by a factor of 65536.

**The fallen log was not bound.** Another worker holds it.

## 5. The last-step lifetime

**Every one of the four passes clears its own log before it does anything, so a
log holds the last step alone and the next step destroys it.** That is stated in
each doc comment, in the same words, near the top.

No queue was invented. A caller that wants the entries of a step keeps its own
copy.

A test proves the lifetime rather than only stating it. It steps a world until
the starved log holds an entry, steps once more, and asserts that the log is
empty.

**A log is empty on a step the schedule does not name, and on a step that
recorded nothing.** A reader cannot tell the two apart, and nothing needs to.
Each doc comment says so.

## 6. The verb this work had to add

**No caller could set an upkeep rate, so the shortfall log could never hold an
entry.** A site that spends nothing can never fall short. A whole-tree search
for the writer found the core method, two Rust test files, and nothing else. The
finding records it, and the commit body holds the search command.

Binding the reader alone would have shipped a call that answers with an empty
log for ever. That is the inert capability shape with an extra step in it: the
reader is invoked, and the thing it reads is not.

The verb takes a set of sites, one rate and one commodity number. It resolves
every identity, checks the commodity against the store of every site, and only
then writes. The rate carries the Q16.16 scale, and the doc comment says so.

**The verb states no check of its own for the rate.** The engine refuses a rate
below zero and it is the one enforcement. A copy in the binding would have been
a second statement of a rule with nothing failing when the two disagreed. Section
8 below records that this was tested rather than assumed.

## 7. The defects put back

Each defect below was written into the tree, the extension was rebuilt, and the
test suite was run. Every defect was then reverted.

| Defect | Caught? | By what |
|---|---|---|
| The type write ignores the row and always writes row zero | Yes | Four tests, including the tank test |
| The table read swaps the attack column and the armour column | Yes | The table read test |
| The upkeep verb writes during the resolve, instead of after it | Yes | The refusal test for a site the world does not hold |
| The starved log is never cleared, so it goes stale | Yes | The last-step lifetime test |
| The binding drops its own check of the rate | **No** | Nothing failed |

**The last row is the honest one.** The binding held a check that refused a rate
below zero before the call reached the engine. Removing it failed no test, at
first because the engine refuses the rate as well, and then because a further
assertion could not see a difference either. The state hash of a world with an
unwritten rate table is unchanged by a refused call, so the extra check bought
nothing that a test could observe. **The check was removed rather than kept.**
One value with one enforcement site is what the defect rule asks for, and a
second site that no test can distinguish is the shape that rule names.

The refusal test kept its state hash assertion, because the assertion is correct
whether or not it can currently fail on this path.

## 8. What was not done

- **The fallen log is not bound.** Another worker holds it.
- **No verb was added for the need rule, the schedules or the production rate.**
  A caller still cannot change how fast a unit goes hungry, or how often a pass
  runs. The rationed log, the starved log and the promotion log are reachable
  without them, because the founding sets a production rate from the ground it
  surveyed.
- **The type stub is still hand-written and nothing compares it against the
  module.** A backlog item already holds the generator and the check.
- **No count getter was added for any of the four logs.** The gather log has
  one and the trade log does not, so the convention is not settled. The length
  of a column answers the same question.

## 9. The gates

Every gate below was run on the branch. Each is reported with the line that
answered.

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | passes, no output |
| `cargo clippy --workspace --all-targets -- -D warnings` | passes, no warning |
| `cargo test --workspace` | passes, every test result `ok` |
| Thread equivalence | passes, 15 tests |
| Golden state hash | passes, 2 tests, no golden file moved |
| `uv run ruff check python tests` | passes, all checks passed |
| `uv run ruff format --check python tests` | passes, 55 files already formatted |
| `uv run mypy` | passes, no issue in 27 source files |
| `uv run pytest` | passes, 182 tests |
| The record checks | pass, 0 failures across all eight |
| The record probes | fail as they must |
| The merge-defect check | passes, 0 failures |
| `just docs` | passes, 101 members with prose and 0 without, every summary reached the site |
| `just docs-probe` | both cases failed the job, as they must |

The record check reports two records that nothing cites. Neither is new, and the
check reports them without failing.

The footnote check reports 68 documents whose footnotes are out of body order,
and 101 baselined. Neither number changed with this work.

## References

[^1]: Backlog item 0450, read a unit type and the table from Python. `docs/backlog/complete/0450-read-a-unit-type-and-the-table-from-python.md`
[^2]: Backlog item 0451, let Python read the four unread step logs. `docs/backlog/complete/0451-let-python-read-the-four-unread-step-logs.md`
[^3]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
