# Broken toolchain files (fixture)

This directory proves that the toolchain pin check can fail. Every
configuration below is invalid on purpose. Do not repair them.

A check with no proven failure mode is decoration, so continuous integration
runs the check against these fixtures and fails if the check passes.[^1]

Each file holds one failure shape.

| File | Shape |
|---|---|
| `bare-nightly.toml` | A floating nightly channel with no date |
| `stable.toml` | A floating stable channel |
| `beta.toml` | A floating beta channel |
| `undated.toml` | A nightly channel with an incomplete date |
| `missing-channel.toml` | A toolchain section with no channel |

## References

[^1]: Testing Rules. `.claude/rules/testing.md`
