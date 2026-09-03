# ADR-0097: The toolchain is a dated nightly, pinned to a date and never to a channel

## Context

The Rust compiler ships on three channels. The stable channel gives a release
every six weeks and promises that a program which compiles today compiles on
every later release. The nightly channel gives a build every day and promises
nothing. A nightly build carries the unstable library and language features
that the stable channel withholds.

This project pinned a stable release. That pin decided three things the
project did not decide deliberately, and each of them costs something.

**The portable vector library is not in stable.** The scoring pass of the
choice system reads a fixed set of options, multiplies integers, and takes the
highest. That is the shape a vector unit exists for. A backlog item is limited
to shaping the loop so that the compiler may vectorise it, and the item says
in its own text that the pin is why.[^1] The alternative routes are worse: an
architecture-specific intrinsic is unsafe and needs a second implementation
for every other target, and pinning a wider vector unit decides which machines
the engine runs on.

**Miri runs on nightly and on no other channel.** Miri interprets the program
and reports an aliasing defect, a provenance defect, or a read of an
uninitialised byte. The project orientation states that keeping the core crate
free of the interpreter binding "allows Miri to check the unsafe code", and
the crate split record states that Miri running over the storage code is the
benefit no test gives.[^2] Nothing ran Miri. There was no recipe, no
continuous integration step and no script, and there could not have been,
because the pinned channel does not carry Miri. The claim was unrunnable for
as long as it existed.

This matters more here than the small amount of unsafe code suggests. The
state hash reads whole structures and whole columns as raw bytes, and that
read is sound only while every type it reaches declares its padding.[^3] A
structure that gains an undeclared padding byte compiles, passes every
ordinary test, and puts an uninitialised byte into the hash. The hash then
differs between two runs of one binary. That is the one property this project
cannot recover, and no ordinary test sees it, because a padding byte usually
holds zero. Miri sees it.

**The reassociating float methods behave differently on the two channels.**
The arithmetic boundary is enforced by two mechanisms, because one is not
enough: a lint bans the float types by name, and a script catches what the
lint cannot see. The project believed that the reassociating methods could not
be named in a lint. On the pinned stable release those methods are gated, so
the compiler rejects any call to them and the gate cannot be opened on that
channel. On the nightly this record pins they are ungated, so a call compiles,
and the lint resolves them and rejects them. The move opens a door and hands
the lint the key to it at the same moment. The findings register holds the
evidence.[^4]

**A floating channel is not an option.** The channel named `nightly` resolves
to whatever was built last night. Two machines that check out one commit then
run two compilers, and a contributor's compiler changes without a commit. The
determinism record requires that one binary gives one answer, and it names the
golden state hash as the test that holds the project to it.[^5] A compiler is
an input to that hash: a different optimiser may reorder an integer expression
in a way that is still exact, but a different standard library may not sort a
tie the same way, and neither change announces itself. A date makes the
compiler a versioned input like any other. A channel makes it an unversioned
one.

## Decision

### D1. The toolchain is a nightly build

The project builds on the nightly channel. The toolchain file is the one place
that names it, and every contributor and every machine takes the toolchain
from that file.[^6]

The reason is the portable vector library, and Miri follows from the same
move. This record does not decide that any pass is vectorised. It decides that
the option exists.

### D2. The pin names a date, never a channel

The toolchain file names a dated nightly. It must never name the bare
`nightly` channel, and it must never name a range.

A bare channel makes the compiler an input that changes without a commit. Two
contributors on one commit would then build with two compilers, and a
difference between their golden state hashes would look like a simulation
defect rather than a toolchain one. The date removes that whole class of
question: a hash that differs between two checkouts of one commit is a defect
in the code, because the compiler is the same.

### D3. Moving the date is a change to the simulation until a run proves otherwise

A new date is a new compiler. Treat it as a change that may move the golden
state hash, and run the determinism gate before keeping it.

A date is moved in a commit of its own, so that a bisect can separate a
compiler change from a code change. The commit body states what the gate did.

### D4. A Miri gate runs over what reads the state as bytes

The project runs Miri over the tests that read the simulation state as raw
bytes, and it drives the engine to get there. A test that built a structure
and hashed it would prove that the structure is sound and not that the engine
reaches only sound structures.[^7]

The gate is not the whole test suite. Miri interprets every instruction, and a
world that reserves its unit columns at the target population does not
finish.[^8] The gate runs a fixture that reserves a few thousand slots, which
is enough to hash a populated arena. This record states no test list and no
duration. The recipe holds the list, and a register holds any cost figure.

**A Miri gate that runs nothing is worse than no gate**, because the
orientation would then still claim a check that nothing performs, and a green
run would confirm the claim. The list is a floor, not a ceiling: a subsystem
that adds an unsafe operation adds a test to the gate that reaches it.

## Consequences

**The compiler gives no stability promise.** A feature this project depends on
may change shape or be removed between two dates. The portable vector library
is the one the project moved for, and it is unstable by definition. Nothing
warns before it changes; a date move simply fails to build, or worse, builds
and behaves differently.

**A date move is now project work.** On the stable channel a compiler upgrade
was routine. It is now a commit that runs the determinism gate and records the
result. A project that never moves the date accumulates an old compiler, and
one that moves it often pays the gate each time. Neither is free.

**Every contributor and every machine needs the exact toolchain.** The
toolchain file makes this automatic where the toolchain manager is used, and a
build outside that manager now needs a specific dated build rather than any
recent release. Continuous integration installs it from the same file, which
is one declaration site rather than two.

**The toolchain declaration is no longer the only claim about the compiler.**
The workspace manifest states a minimum supported release, and the lint
configuration states one for the lints. Those three sites now disagree in
kind: two name a stable release and one names a nightly date. Nothing fails
when they disagree, which is the defect shape this project keeps meeting.[^9]
This record does not resolve it. A backlog item does.[^10]

**The gate suite gets slower, and the Miri part is the reason.** Miri
interprets rather than executes. The gain is a class of defect that no other
gate in this project can see. This record states no figure for the cost; a
register holds it.

**A record whose reasoning rests on the old channel is now weaker.** The
overflow record justifies a test rather than a lint by saying that the switch
naming the build is not stable on the pinned toolchain.[^11] That switch is
still unstable on the new pin, but it is now reachable behind a feature
attribute, so the reasoning holds for a smaller reason than it did. This
record does not amend that one. The findings register holds the observation.

**Moving back to stable is possible and is not free.** Nothing here is
one-way while no unstable feature is used. The moment a pass uses the portable
vector library, the return costs a rewrite of that pass, and the Miri gate is
lost with the channel.

## References

[^1]: Backlog item 0270, score the option set with integer vector instructions. `docs/backlog/proposed/0270-score-the-option-set-with-integer-vector-instructions.md`
[^2]: ADR-0041, a crate split enforces the boundary at compile time, decision D1. `docs/adrs/draft/adr-0041-a-crate-split-enforces-the-boundary-at-compile-time.md`
[^3]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
[^4]: Findings register, FND-284. `docs/FINDINGS.md`
[^5]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^6]: The pinned toolchain. `rust-toolchain.toml`
[^7]: Testing rules, drive the real caller. `.claude/rules/testing.md`
[^8]: ADR-0084, the world reserves the unit columns at construction. `docs/adrs/draft/adr-0084-the-world-reserves-the-unit-columns-at-construction.md`
[^9]: Recurring defect shapes, redundant declaration sites. `.claude/rules/recurring-defects.md`
[^10]: Backlog item 0294, reconcile the three declarations of the toolchain. `docs/backlog/proposed/0294-reconcile-the-three-declarations-of-the-toolchain.md`
[^11]: ADR-0083, the gate build checks every integer overflow, decision D2. `docs/adrs/draft/adr-0083-the-gate-build-checks-every-integer-overflow.md`
