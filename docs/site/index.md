# The Cachette documentation (index)

This document is an **index**. It says what this site holds today, and what it
does not hold yet.

Cachette is a world simulation engine. The core is Rust. The control plane is
Python. A program that drives the engine is a Python program.

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
