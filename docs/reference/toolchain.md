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

The project owner decided this on 31 August 2026.[^4]

**The project tracks a recent stable release.** The pin is the current stable
version at the time it was set, and it moves forward as the project adopts a
later stable release.

**This reason is a policy. It is not a derived property.** The owner chose it
against the recommendation in the decision register, which asked the project to
state the property it needs from the toolchain first, and then to pin the
lowest version that provides that property. The owner took the decision
knowingly.

The consequence follows from that, and a reader must not miss it: **the pinned
version is not evidence of a minimum requirement.** It does not mean the
project needs that version. Nobody has established the lowest version that
works. Do not read the pin as a floor, and do not cite it as one.

## What the project depends on in the toolchain today

One dependency is known, and it is load-bearing for a hard invariant.

The float ban forbids floating point in simulated or aggregated state, and two
mechanisms enforce it. A lint rejects the float types by name. A script rejects
what the lint cannot see: a float literal whose type the compiler infers, and
the reassociating methods.[^5]

**The reassociating methods are in the script because they do not resolve on
the current pin.** A lint entry cannot name a method that does not exist, so
the script covers them by name instead.

## The condition that would change the picture

If a later toolchain makes the reassociating methods resolvable, three things
follow.

1. The lint can name them, so the script no longer has to.
2. The project then has one real property to require from the toolchain: the
   methods resolve, and the lint therefore covers them.
3. The pin can state that property, and the policy above can be replaced by it.

Until that happens, the project has no stated property to pin against, and the
policy is the whole reason.

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
[^2]: The channel read from the manifest on 31 August 2026 was `1.91.1`, with the `rustfmt` and `clippy` components.
[^3]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^4]: Decisions register, DEC-013. `docs/DECISIONS.md`
[^5]: The float ban check. `scripts/check-float-ban.sh`
[^6]: Budgets and costs, the target register. `docs/reference/budgets.md`
[^7]: Development budgets, the local register. `docs/reference/development-budgets.md`
