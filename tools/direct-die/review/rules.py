"""The one write into a style rules file.

The analysis proposes a rule that the guide does not state. A person reads it,
edits it, and accepts it. This module appends it.

It appends under one heading that it creates once. It does not try to find the
section that the model named. A section name from a model is a guess, and a
wrong guess puts a rule where nobody reads it. A person moves the line later
if they want it somewhere else.

The style guide version is a digest of the rules and the exemplars, so an
accepted rule changes the version that the next session records.[^1]

## References

[^1]: The tool guide, the style guide. `tools/direct-die/README.md`
"""

from __future__ import annotations

import os
import tempfile
from pathlib import Path

# The heading that holds every rule a person accepted from an analysis.
ADDED_HEADING = "## Rules the art director added"


class RuleError(RuntimeError):
    """The rule cannot be written."""


def rules_file(styleguide_root: Path, style: str) -> Path:
    """Give the rules file of one style.

    Raise `RuleError` when the name is not one path segment, and when the
    file does not exist.
    """
    if not style or "/" in style or "\\" in style or style in (".", ".."):
        raise RuleError(f"not a style name: {style!r}")
    path = Path(styleguide_root) / f"{style}.md"
    if not path.is_file():
        raise RuleError(f"no rules file for {style!r}")
    return path


def append_rule(styleguide_root: Path, style: str, rule: str) -> Path:
    """Append one rule to a style rules file, and give the path.

    The write is atomic. It writes a temporary file in the same directory and
    then renames it, so a reader never sees half a file.

    Raise `RuleError` when the rule is empty and when it holds a newline. A
    rule is one line, because the heading holds a list.
    """
    path = rules_file(styleguide_root, style)
    text = rule.strip()
    if not text:
        raise RuleError("the rule is empty")
    if "\n" in text or "\r" in text:
        raise RuleError("a rule is one line")

    body = path.read_text(encoding="utf-8").rstrip("\n")
    if ADDED_HEADING not in body:
        body += f"\n\n{ADDED_HEADING}\n"
    body += f"\n- {text}\n"

    directory = path.parent
    handle, temporary = tempfile.mkstemp(dir=directory, suffix=".tmp")
    try:
        with os.fdopen(handle, "w", encoding="utf-8") as file:
            file.write(body)
        os.replace(temporary, path)
    except BaseException:
        Path(temporary).unlink(missing_ok=True)
        raise
    return path
