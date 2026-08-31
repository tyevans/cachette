# A deliberately broken backlog

This directory is not a backlog. It exists so that the backlog check can be
shown to fail. Continuous integration runs the check against it and fails
when the check passes.

| File | The defect |
|---|---|
| `proposed/0001-take-a-number.md` | The number is taken twice, once here |
| `refined/0001-take-the-same-number.md` | The number is taken twice, once here |
| `proposed/not-a-backlog-item.md` | The name is not `NNNN-short-slug.md` |
