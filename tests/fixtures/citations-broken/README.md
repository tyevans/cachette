# Broken citations (fixture)

This directory exists to prove that the citation check can fail. Every
citation below is wrong on purpose. Do not repair them.

A check with no proven failure mode is decoration, so continuous integration
runs the check against this directory and fails if the check passes.[^1]

Each file holds one failure shape.

| File | Shape |
|---|---|
| `no-such-record.rs` | A record number that no record and no registry row has |
| `no-such-decision.py` | A decision number that the record does not define |
| `dangling-path.md` | A footnote path that does not resolve on disk |

## References

[^1]: Testing Rules. `.claude/rules/testing.md`
