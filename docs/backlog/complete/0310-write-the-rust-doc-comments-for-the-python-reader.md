---
id: 0310
title: Write the Rust doc comments for the Python reader
status: complete
created: 2026-09-03
implements: [ADR-0107 D2, ADR-0107 D3]
changes: []
creates: []
serves: [PRD-0021]
blocked-by: []
---

## Why

**The published reference carries only the prose that the Rust doc comments
hold, and a record says plainly that it makes nothing write that prose.** The
record fixes where the prose lives. A member with no Rust doc comment publishes
with no prose, and nothing fails.[^1]

**The audience of a doc comment is now two audiences.** The record names them:
the Rust reader who changes the core, and the Python developer who never opens
it.[^1] [^2] A comment written for the first alone under-serves the second, and
the record states that no check can see that.

**The product record states three answers the reader must get without opening a
Rust file.** What a returned column holds. In what unit a value is expressed.
Which error a call raises, and what the error means.[^2] Every error the engine
raises is a declared type, and the reader must be able to name it.[^3]

## Impact review

**Governed by.** ADR-0107 D2 and D3.[^1]

**D2, the prose of a member the compiled module provides lives in the Rust doc
comment and nowhere else.** Every word this item writes goes into the bindings
crate. It writes nothing into a documentation page and nothing into a member of
the type stub that the module provides.

**D3, the stub declares types and carries prose only where the module provides
none.** The stub carried eleven docstrings that copy the Rust source, and a
finding holds the evidence.[^7] This item removes all eleven, because a doc
comment written under D2 and a copy of it in the stub are the two declaration
sites the record forbids. The typed dictionaries keep their prose, because the
module does not provide them.

**D1 and D4 are not implemented here.** D1 is the build, and D4 is the job that
runs it. Item 0309 owns both.[^6]

**Changes.** No record.

**Creates.** No record. The work states no constraint that a contributor could
reasonably choose otherwise on, so the test for whether a decision needs a
record fails at its first statement.[^8]

**Blockers.** None governs the work. Two govern what the prose may say. One
holds every cost figure, so no doc comment states a performance number.[^4] One
asks whether an upgrade changes hands when the ground does, so no doc comment
answers that question. No member of the module reaches that behaviour, so
nothing in this item had to cite it.[^5]

**Precedent.** Three findings shape the work. A statement that nothing executes
costs the reader's trust the first time it is wrong, and the project has
published two false claims in the documents a newcomer reads first.[^9] [^10]
The type stub already claimed a generator and a check that do not exist.[^11]
Every claim this item writes was therefore checked against the code or against
a run of the built module before it was written.

## What the work does

Read every public member of the compiled module. For each one, state what it
does, what each argument means, what it returns in Python terms, in what unit a
numeric value is expressed, and which error it raises. Repair the Rust doc
comment of every member that answers one of those badly or not at all.

Remove the eleven copied docstrings from the type stub, and leave the types.

## What it does not do

It does not document the Rust core. A person who changes the engine is a
different audience, and the product record excludes them.[^2]

It does not build the site, the configuration or the job. Item 0309 owns
those.[^6]

It does not change the behaviour of any binding. Two defects it found are in
the findings register rather than repaired here.[^12] [^13]

## Done when

- Every public class, method, function, property and error class of the
  compiled module carries a doc comment written for a Python reader.
- Every entry that carries the fixed-point scale says so, and says to divide by
  65536.
- Every method that can raise names the class it raises.
- No member of the type stub that the compiled module provides carries a
  docstring, and a search of the whole file proves it.
- The gates run green.

## Outcome

Done. Every public member of the module now carries prose written for a Python
reader. The module docstring holds five conventions that the members then rely
on: the fixed-point scale, the opaque identity, the two ways a tile crosses, the
kind numbering, and the set-valued command.

Two findings came out of the reading. Three exception classes are declared,
exported and documented, and nothing raises any of them.[^12] A faction refusal
names the project ceiling while the engine applied the faction count of the
world.[^13] Neither is repaired here, because both are engine work.

Two claims that were already in the tree were false and are now corrected in
place. The spawn doc comment implied that a tile refuses a unit above its
capacity, and a record states the opposite.[^14] The type stub declared one
report entry as a tuple, and the module returns a list.

The review holds the whole account.[^15]

## References

[^1]: ADR-0107, the Python reference is generated from the compiled module, decisions D2 and D3. `docs/adrs/draft/adr-0107-the-python-reference-is-generated-from-the-compiled-module.md`
[^2]: Product requirement record 0021, a developer can use the control plane without reading its source. `docs/product/accepted/prd-0021-a-developer-can-use-the-control-plane-without-reading-its-source.md`
[^3]: ADR-0046, every error is typed, decision D3. `docs/adrs/draft/adr-0046-every-error-is-typed.md`
[^4]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^5]: Blockers register, BLK-034. `docs/BLOCKERS.md`
[^6]: Backlog item 0309, publish the Python reference generated from the compiled module. `docs/backlog/refined/0309-publish-the-python-reference-generated-from-the-compiled-module.md`
[^7]: Findings register, FND-321. `docs/FINDINGS.md`
[^8]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^9]: Findings register, FND-322. `docs/FINDINGS.md`
[^10]: Findings register, FND-323. `docs/FINDINGS.md`
[^11]: Findings register, FND-320. `docs/FINDINGS.md`
[^12]: Findings register, FND-326. `docs/FINDINGS.md`
[^13]: Findings register, FND-327. `docs/FINDINGS.md`
[^14]: ADR-0074, a spawn may over-fill a tile, and only admission enforces the capacity, decision D1. `docs/adrs/accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
[^15]: The review of the Rust doc comments. `docs/reviews/0310-the-rust-doc-comments.md`
