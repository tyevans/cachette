# Broken fixture: a stale baseline entry

This directory holds one correct document and a baseline that names a failure
the document does not hold. The check must reject the pair. The probe recipe
runs it and fails when the check passes.[^1]

That is the proof that the baseline is falsifiable. A baseline entry that
matches nothing fails, so the real baseline can only shrink and can never go
stale.

## References

[^1]: The check targets. `justfile`
