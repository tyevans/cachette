# Broken fixture: merge conflict markers

This directory is deliberately broken. The conflict marker check must reject
it. The probe recipe scans it and fails when the check passes.[^1]

The repository scan passes over this directory by name, so a real run of the
check never reads it.

`register-with-markers.md` holds the three markers of the default conflict
style. `source-with-diff3-markers.rs` holds the fourth, which the `diff3` and
`zdiff3` styles write, and it proves the check reads a file that is not
Markdown.

`not-a-marker.md` holds the shapes the check must leave alone: a rule of eight
or more characters, and a heading underline. It is here so that a run of the
check against this directory reports the two files above and nothing else.

## References

[^1]: The check targets. `justfile`
