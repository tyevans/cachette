# The Cachette documentation (index)

This document is an **index**. It says what this site holds today, and what it
does not hold yet.

Cachette is a world simulation engine. The core is Rust. The control plane is
Python. A program that drives the engine is a Python program.

**This site is for a programmer who drives a simulation from Python.** One
example is a game server that must replay a run exactly from a seed.

## How to install the engine

**No public package index carries this engine.** A checkout of the repository
builds it. The index name `cachette` belongs to a different project, so do not
install that name.[^1]

```text
git clone https://github.com/tyevans/cachette
cd cachette
uv sync
uv run python -c "import cachette; print(cachette.version())"
```

A blocker holds the question of which name this project publishes under.[^2]

## What this site holds

**The reference.** It holds every public name of the compiled extension module
of the Python control plane: what a call takes, what it returns, and which
error it raises. A build generates that page from the module itself, so it
changes when the module changes.

## What this site does not hold yet

**A tutorial, a set of how-to guides, and a set of explanation pages.** Three
separate pieces of work write them. Until they land, this site answers the
question "what does this call do" and it does not answer "how do I start".

A reader who wants the reasoning behind a constraint reads the decision records
in the repository. This site does not repeat them.

## References

[^1]: Findings register, FND-341. `docs/FINDINGS.md`
[^2]: Blockers register, BLK-040. `docs/BLOCKERS.md`
