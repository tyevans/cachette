---
id: 0311
title: Execute every documentation example from the test suite
status: refined
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0021]
blocked-by: []
---

## Why

**The orientation document holds one worked example, and nothing runs it.** The
product record states it: the example runs today, nothing runs it, so nothing
fails on the day it stops running.[^1] The test suite tests the package through
its public interface and does not read the document.[^2]

**The product record admits only documentation that a check can keep true.** It
states the rule in three sentences. A statement that something executes costs
nothing to keep current, because it fails on the day it becomes false. A
statement that nothing executes and nothing derives costs the reader's trust the
first time it is wrong, and it costs that trust for every other statement in the
document at the same time.[^1]

**This project has paid that cost twice, and both cases are recorded.** One
sentence about the state of the project spread through the tree in each writer's
own words, the fact changed in one day, and most copies stayed.[^3] A register
held a list and claimed a check kept it current; the check read a different
list.[^4]

This item builds the thing that makes an example fail.

## Impact review

**Governed by.** PRD-0021 requires that a check executes every documentation
example that a reader can copy.[^1] The check must fail when an example stops
working.[^1] ADR-0107 governs the reference documentation and its compilation
dependency.[^6] Testing Rules require a test to prove that it can fail.[^5]
Section 2a requires a fixture to supply the failing input.[^5]

**Resolved design.** Examples live directly in markdown code fences. The harness
reads source markdown files directly rather than rendered HTML pages. Reading
source files removes a dependency on the documentation site build. It also lets
the test harness run in the standard Python test suite.

**Changes.** None.

**Creates.** None.

**Blockers.** None.

## What the work does

Add a harness that collects the code from every documentation page and from the
orientation document, runs it, and fails when it raises or when its stated
output does not match.

Run the harness in the test recipe, so a contributor who breaks an example sees
it before handing over.

**Put the defect back and watch the harness stay green.** Change one call in
the orientation example to a name the package does not export, run the harness,
and confirm it fails and names the document. A fixture that never receives a
failing input measures the fixture.[^5]

## What it does not do

It does not prove the engine correct. An example that runs proves that the
example runs, and the product record says so.[^1]

It does not write any example. It runs the ones that exist and the ones the
prose items add.

## Done when

- The test harness extracts Python code blocks from source markdown documents.
- The test harness executes each block using a clean global dictionary.
- The worked example in the orientation document executes successfully.
- Any Python code blocks found in documentation site pages execute successfully.
- A negative test fixture supplies a broken documentation example.
- The test harness fails on the broken fixture and names the document and block.
- The documentation test module runs and passes in the test suite.

## References

[^1]: Product requirement record 0021, a developer can use the control plane without reading its source. `docs/product/accepted/prd-0021-a-developer-can-use-the-control-plane-without-reading-its-source.md`
[^2]: The black-box tests of the installed Python package. `tests/test_public_api.py`
[^3]: Findings register, FND-223. `docs/FINDINGS.md`
[^4]: Findings register, FND-242. `docs/FINDINGS.md`
[^5]: Testing Rules, section 1 and section 2a. `.agents/rules/testing.md`
[^6]: ADR-0107, the Python reference is generated from the compiled module. `docs/adrs/draft/adr-0107-the-python-reference-is-generated-from-the-compiled-module.md`
