# Commit Message Rules

A commit message is the only project document that never decays. It is fixed to
one moment and to one change. Material that decays belongs here, not in a
decision record.

## Format

Write an imperative sentence. Capitalise the first word. Do not put a full stop
at the end.

```
Add the counter-based random number generator
Split the pyramid record into geometry and accumulators
Fix the u32 overflow in the level 1 accumulator
```

Do not use a type prefix. Do not write `feat:` or `fix:`.

The subject line says what changed. Use 72 characters or fewer.

## What belongs in the body

The body says what the change cost and what you learned.

Put these in the body:

- Counts. "Thirteen call sites moved."
- File tables and survivor lists.
- Measured figures and the command that produced them.
- The whole-tree search command you ran, so that a reviewer can run it again.

A decision record must not hold this material, because it decays.[^1] The commit
message is where it stays true.

## Scope

Prefer many small commits to one large commit. A small commit keeps the failure
surface readable when a check fails.

Commit the register update in the same commit as the work that caused it. A
commit that defers work and does not touch a register is incomplete.[^2]

## After a sweep

Treat an incomplete sweep as the normal outcome. When you rename or delete a
name, search the whole tree, not a list of files you thought of. Put the search
command in the commit body.
Test code, fixtures, and example code are call sites too.

## References

[^1]: Decision Record Scope. `.agents/rules/adr-scope.md`
[^2]: Definition of Done. `.agents/rules/definition-of-done.md`
