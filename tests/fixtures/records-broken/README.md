# Broken records (fixture)

This directory exists to prove that the record check can fail. The record in
it is wrong on purpose. Do not repair it.

A check with no proven failure mode is decoration, so continuous integration
runs the check against this directory and fails if the check passes.[^1]

The record check reads a directory that holds a registry and an `accepted/`
directory. This directory has that shape and binds nothing.

Each file holds one failure shape.

| File | Shape |
|---|---|
| `accepted/adr-0001-the-record-states-its-own-status.md` | A record that states a status of its own |

The registry holds the status of a record, and it is the only document that
does.[^2] A record file that states one holds a second copy, and nothing fails
when the two copies disagree. This check fails instead.

## References

[^1]: Testing Rules. `.claude/rules/testing.md`
[^2]: ADR Registry. `docs/adrs/REGISTRY.md`
