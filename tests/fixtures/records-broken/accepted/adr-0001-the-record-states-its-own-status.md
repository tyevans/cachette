# ADR-0001: The record states its own status

Status: Accepted

## Context

This file is a fixture. It is not a record of this project, and it binds
nothing. It exists so that the record check can prove that it fails.

The file carries a status line, which a contributor copies when they use an
old record as a template. The registry holds the status of a record, and it is
the only document that does. A second site fails silently when the two copies
disagree, so the check must reject this file.

Everything else in this file is correct. The check must therefore report the
status line and nothing else.

## Decision

### D1

The fixture states a status in the record body. The check rejects it.

## Consequences

The probe recipe fails when the check accepts this file.

## References

[^1]: The record check script. `scripts/check_adrs.py`
