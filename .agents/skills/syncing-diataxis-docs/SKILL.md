---
name: syncing-diataxis-docs
description: Use at the end of any workstream, coding session, or after code changes have been made to a repository — before ending the conversation, handing off work, or merging a branch. Diffs local state against the last documented commit and updates how-to, tutorial, reference, and explanation docs plus their tests to match what changed. Triggers on "wrap up", "finishing up", "before I go", "done with this feature", "update the docs", "docs are stale".
---

# Syncing Diataxis Docs

## Overview

Maintain a persistent, LLM-owned knowledge layer that compiles once on ingest
and stays current with code changes. Ground truth is the code and its git
history. The documentation tree is an incrementally updated knowledge base,
diffed against the last processed commit rather than resynthesized from scratch.

This skill targets the four Diataxis categories explicitly:

- **Tutorial**: learning-oriented, guides a beginner to an initial success.
- **How-to**: task-oriented, solves a specific practical problem.
- **Reference**: information-oriented, provides technical descriptions and APIs.
- **Explanation**: understanding-oriented, clarifies architectural rationale.

## When to Use

- At the end of a workstream: feature completed, bug resolved, or before handoff.
- Any time code changes span more than one undocumented commit.
- Do not run for a single trivial typo fix with no behavioral change.

## State File

The state file stores the git commit SHA that was last processed:
Check `.agents/last-docs-run`.

1. Read the baseline SHA from the state file. If missing, treat as first run
   and scan all tracked files (`git ls-files`).
2. Run `git rev-parse HEAD` to obtain the target SHA.
3. Compute the file diff: `git diff --name-status <baseline>..<target>`.
4. After completing the run, update the state file with the target SHA.
5. Ensure the state file is ignored in `.gitignore`. Never commit the state file.

## Isolation

When dispatching workers, give each worker its own git worktree so that
scratch files do not pollute the working directory.

## Wave Orchestration

Do not run this workflow as a single agent reading a diff serially. Orchestrate
work in distinct waves using subagents:

1. **Wave 1: Scope (Single Agent)**
   - Resolve baseline SHA from `.agents/last-docs-run`.
   - Diff `<baseline>..HEAD`.
   - Partition changed files into bounded contexts (top-level modules or packages).
   - Return changed files and a summary for each bounded context.

2. **Wave 2: Research (Parallel Subagents)**
   - Launch one research subagent per bounded context.
   - Each agent reads the changed source files and existing documentation.
   - Identify necessary documentation updates across the four Diataxis categories.
   - Flag test coverage gaps against the changed behavior.

3. **Wave 3: Outline (Parallel Subagents)**
   - Launch one outline subagent per proposed document change.
   - Produce a structured outline (ordered section headers, no prose).
   - Ground every section in the changed code.

4. **Wave 4: Consolidation (Single Agent Barrier)**
   - Merge outlines from all bounded contexts.
   - Resolve overlapping file paths and deduplicate sections.
   - Map necessary cross-links between documents.

5. **Wave 5: Populate (Parallel across files; Serial within each file)**
   - Write sections into files.
   - Run files concurrently with each other.
   - Run sections within a file sequentially to maintain continuity.
   - Ground all statements in the actual source code and tests.

6. **Wave 6: Tests (Parallel Subagents per Bounded Context)**
   - Add or extend tests to close the test coverage gaps identified in Wave 2.
   - Follow repository testing conventions and property test requirements.

7. **Wave 7: Finalize (Single Agent)**
   - Write the processed HEAD SHA to `.agents/last-docs-run`.
   - Verify `.gitignore` entry exists.
   - Report a summary of modified documentation files and added tests.

## Common Mistakes

- Resynthesizing the entire repository instead of diffing against baseline SHA.
- Assigning one agent to write all four Diataxis categories for a module.
- Writing sections of a single file concurrently instead of sequentially.
- Omitting the consolidation barrier between research and writing.
- Committing the state file instead of keeping it git-ignored.
