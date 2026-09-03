# Toolchain

This document is a **register**. It holds the reason the project pins the Rust
toolchain it pins, and the condition that would change that reason.

**The pin itself lives in one file, and this register is not that file.** The
manifest at the repository root declares the channel, the components and the
targets, and it is the only site that reaches the build.[^1] The version read
from it on the date below is in a footnote, not in this text.[^2] A second
authoritative copy of a value is a defect shape this project has already
recorded, so this register cites the manifest and never overrides it.[^3]

Read the manifest for the current pin. Read this register for why.

## The reason for the pin

The project owner decided the current reason on 2 September 2026, after being
told that the portable vector library is not in the stable channel.[^8]

**The project pins a dated nightly build, and the reason is a stated property
rather than a policy.** Three things the project wants are on the nightly
channel and on no other: the portable vector library, the interpreter that
checks the unsafe code, and the reassociating float methods, which the lint can
name once they compile.[^8]

**The date is part of the reason, not part of the value.** A bare channel makes
the compiler an input that changes without a commit, and this project cannot
have that. A record holds the constraint and the reasoning.[^8]

**The earlier reason was a policy, and it is worth recording what it was.** The
project tracked a recent stable release, chosen on 31 August 2026 against the
recommendation in the decision register, which had asked the project to state
the property it needs and then pin the lowest version that provides it.[^4] The
project now has the stated property that recommendation asked for. It arrived
from a need rather than from the argument.

The consequence of the earlier policy still holds for the current pin, and a
reader must not miss it: **the pinned build is not evidence of a minimum
requirement.** It does not mean the project needs that exact date. Nobody has
established the earliest build that works. Do not read the pin as a floor, and
do not cite it as one.

## What the project depends on in the toolchain today

One dependency is known, and it is load-bearing for a hard invariant.

The float ban forbids floating point in simulated or aggregated state, and two
mechanisms enforce it. A lint rejects the float types by name. A script rejects
what the lint cannot see: a float literal whose type the compiler infers, and
the reassociating methods.[^5]

**The reassociating methods were in the script because the compiler rejected
them, and that has changed.** On the stable release the project used to pin, a
call to one of those methods was a hard error: the library feature was gated,
and a stable channel cannot open a gate. On the current pin the same call
compiles with no attribute, so the method is writable for the first time. The
lint resolves it and rejects it once its list names the path. The list does not
name it yet, so the script's name check is the only thing standing there
today.[^9]

**This register predicted that event and predicted one wrong consequence from
it.** It said that when the methods resolve, the lint can name them and the
script no longer has to. The second half is wrong, and the reason is a
measurement: the lint tool silently ignores a disallowed-method path that
resolves to nothing. It emits no warning and no note. So a lint entry can be
inert and read as live, which is the whole argument for a second mechanism.[^9]

## The condition that would change the picture

The condition this register waited for has arrived, and it changed the pin
rather than the mechanisms.

What the project now requires from the toolchain is three properties, and the
record states them: the portable vector library, the interpreter over the
unsafe code, and a compiler that is fixed to a date.[^8] A build that lacks any
of the three does not serve this project.

What stays open is the balance between the two float mechanisms. A decision row
holds it, a reviewer owns it, and a backlog item holds the work it
recommends.[^10] [^11]

## What belongs here

- The reason for the toolchain pin.
- A property the project requires from the toolchain, once one is established.
- A dependency on toolchain behaviour that a check relies on.

## What does not belong here

- The pinned version, as an authoritative value. The manifest holds it.[^1]
- A cost figure or a budget. Two other registers hold those.[^6] [^7]
- A decision record. This register holds a reason, not a binding constraint.

## References

[^1]: The toolchain manifest, the single declaration site. `rust-toolchain.toml`
[^2]: The channel read from the manifest on 2 September 2026 was `nightly-2026-09-01`, with the `rustfmt`, `clippy`, `miri` and `rust-src` components.
[^3]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^4]: Decisions register, DEC-013. `docs/DECISIONS.md`
[^5]: The float ban check. `scripts/check-float-ban.sh`
[^6]: Budgets and costs, the target register. `docs/reference/budgets.md`
[^7]: Development budgets, the local register. `docs/reference/development-budgets.md`
[^8]: ADR-0097, the toolchain is a dated nightly. `docs/adrs/draft/adr-0097-the-toolchain-is-a-dated-nightly.md`
[^9]: Findings register, FND-263. `docs/FINDINGS.md`
[^10]: Decisions register, DEC-107. `docs/DECISIONS.md`
[^11]: Backlog item 0272, name the reassociating methods in the lint. `docs/backlog/proposed/0272-name-the-reassociating-methods-in-the-lint.md`
