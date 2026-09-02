# Broken fixture: footnotes

This directory is deliberately broken. The footnote check must reject it. The
probe recipe scans it and fails when the check passes.[^1]

The repository scan passes over the whole fixture tree, so a real run of the
check never reads it. An explicit scan reads it and applies no baseline.

One document breaks one test.

| File | Test it breaks |
|---|---|
| `undefined-marker.md` | The body cites a marker the document does not define |
| `duplicate-label.md` | The document defines one label twice |
| `repeated-source.md` | Two labels hold the same definition text |
| `uncited-definition.md` | The document defines a label that nothing cites |
| `code-is-not-a-footnote.md` | Nothing. It holds the shapes the check must leave alone |

## References

[^1]: The check targets. `justfile`
