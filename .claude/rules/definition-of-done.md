# Definition of Done

This rule applies to every unit of work in this repository: code, decision
records, research, and documentation.

Work is done when every statement below is true. If one is not true, the work
is not done, whatever else has been achieved.

## 1. Before you start: review the decision impact

Answer these before writing anything. Write the answers into the plan.

- **Which decision records govern this work?** Read them. The registry lists
  them and gives their dependencies.
- **Does this work contradict a record?** If it does, stop. Either change the
  work, or write a record that supersedes the old one. Do not proceed against
  a record silently.
- **Does this work create a decision that no record holds?** If it does, that
  record is a deliverable of this work, not a byproduct of it.
- **Is this blocked?** Check the blockers register. If a blocker governs a
  value you need, express the work parametrically and cite the blocker rather
  than inventing the value.
- **Has this been settled before?** Check the findings register. It records
  what the project believed and had to correct. A conflict may already have an
  answer.

## 2. Plan the record work, not only the code

A plan that changes an architectural decision must say so.

State in the plan:

- Which records the work implements, by number and decision.
- Which records the work will change, and how they will be superseded.
- Which records the work will create. **Add the registry row before you start
  writing**, so the number is allocated and no other work can take it.
- Which register entries will open or close.

A decision discovered halfway through is normal. Recording it is not
optional. An undocumented decision becomes an assumption, and an assumption
becomes a defect.

## 3. Before you claim done: review the implementation against the records

Go through each governing record decision by decision. For each one, state
whether the implementation honours it. Do not summarise; check them
individually.

Give particular attention to the determinism record, because its rules are
invisible at review time and expensive later:

- No floating point in simulated or aggregated state.
- Simulation arithmetic goes through the arithmetic module.
- Random draws come from a counter keyed on system, frame, entity and draw.
- Iteration order is explicit and stable. No thread completion order. No hash
  iteration order.
- Solvers run a fixed iteration count. No convergence test. No time budget.
- Event types are plain data with declared padding. The apply function is
  pure.

**A record the code contradicts is worse than no record, because it lies.**
When implementation and record disagree, one of them changes before the work
is done.

## 4. Update the registers

- **Findings.** If the work corrected something the project believed, record
  it. State what was believed, what is true, the evidence, and what follows.
  This is precedent, and it is how future conflicts get settled cheaply.
- **Blockers.** If the work resolved one, close the row and record the
  outcome. If it found a new one, open a row.

  **When you close a blocker, search the tree for its number and repair every
  record that calls it open.** Put the search command in the commit body. A
  record written parametrically under a blocker is correct when it is
  written, and it states a false thing the moment the blocker closes. Nothing
  fails, because a record is prose. This has happened twice.[^2]
- **Decisions.** If the work closed an open choice, record the outcome and the
  reasoning. If it opened one, add it with options and a recommendation.
- **Registry.** Set the status of any record you wrote. An author may set
  `Draft`. Only a reviewer may set `Accepted`. The registry says who holds
  review rights and what a delegated review must do that a second reader
  would do for free.[^1]

  **Edit a draft freely.** A draft exists to be edited, and its status is not
  a reason to hesitate. An accepted record with no dependents may be repaired
  in place inside the retcon window; say so in the commit.[^1] Do not revert
  an accepted record to `Draft` in order to amend it, because that records a
  history that did not happen.

## 5. Pass the gates

- Formatting, lint, and type checks pass.
- Tests pass, including the property-based tests.
- The two determinism tests pass: thread-count equivalence, and the golden
  state hash.
- New behaviour has a test that goes through the public interface.
- The whole check command runs green. Do not hand over a red pipeline. If a
  gate cannot pass, remove it and say why, rather than leaving it broken and
  training everyone to ignore it.

## 6. Report the work honestly

- Say what was done, and what was left undone.
- If a test fails, say so and give the output.
- If a step was skipped, say so.
- Do not claim a measurement that was not taken. Every cost figure in this
  project is derived, not measured, and the difference matters.
- Cite a game for observed behaviour only, never for implementation. That
  finding is in the register with its evidence.

## By kind of work

**Code.** All six sections apply.

**A decision record.** Sections 1, 2 and 4 apply. Section 5 applies to the
prose rules rather than to tests. The record must follow the documentation
rule, must not hold material that changes, and must cite its evidence in
footnotes.

**Research.** Sections 1 and 4 apply. Deliver findings, not a survey. Mark a
claim you could not verify as unverified rather than asserting it. Say plainly
where the research contradicts a record.

**Documentation.** Sections 1, 4 and 6 apply. Documents stand alone, and every
external reference sits in a footnote.

## The rule behind the rule

The purpose is not process. It is that this project has one property it cannot
recover if it is lost — determinism — and one asset that decays silently if it
is not maintained: the record of why things are the way they are.

Code that violates a record costs a defect. A record that no longer describes
the code costs every future decision made from it.

## References

[^1]: ADR Registry, who reviews. `docs/adrs/REGISTRY.md`
[^2]: Findings register, FND-042. `docs/FINDINGS.md`
