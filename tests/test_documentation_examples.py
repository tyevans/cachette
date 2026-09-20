"""Execute every Python example from the documentation files.

PRD-0021 requires that a check executes every documentation example that a
reader can copy.[^1] ADR-0107 establishes that the Python control plane
documentation reflects the compiled module.[^2] Testing Rules require a
test to prove that it can fail.[^3]

This module extracts Python code blocks from markdown source files. It executes
each block in a clean global scope. It fails when any snippet raises an
unhandled exception.

References
----------
[^1]: PRD-0021, a developer can use the control plane without reading its
source.
``docs/product/accepted/prd-0021-a-developer-can-use-the-control-plane-without-reading-its-source.md``

[^2]: ADR-0107, the Python reference is generated from the compiled module.
``docs/adrs/draft/adr-0107-the-python-reference-is-generated-from-the-compiled-module.md``

[^3]: Testing Rules, section 2a.
``.agents/rules/testing.md``
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any

import pytest

ROOT = Path(__file__).resolve().parents[1]


@dataclass(frozen=True)
class CodeBlock:
    """A single code block extracted from a markdown document."""

    source_path: Path
    block_index: int
    start_line: int
    code: str


class DocumentationExampleError(Exception):
    """Raised when a documentation example fails to execute."""

    def __init__(
        self,
        path: Path,
        block_index: int,
        original_error: BaseException,
        line_number: int | None = None,
    ) -> None:
        """Initialize the error with document location and original exception."""
        self.path = path
        self.block_index = block_index
        self.original_error = original_error
        self.line_number = line_number
        loc = f"{path}:{line_number}" if line_number is not None else str(path)
        super().__init__(
            f"Documentation example block {block_index} in {loc} failed: "
            f"{type(original_error).__name__}: {original_error}"
        )


def extract_python_blocks(path: Path) -> list[CodeBlock]:
    """Extract all fenced Python code blocks from a markdown file."""
    text = path.read_text(encoding="utf-8")
    lines = text.splitlines(keepends=True)
    blocks: list[CodeBlock] = []

    in_fence = False
    fence_char = ""
    fence_len = 0
    block_lines: list[str] = []
    start_line = 0
    block_index = 0

    for line_idx, line in enumerate(lines, start=1):
        lstripped = line.lstrip()
        if not in_fence:
            if lstripped.startswith(("```", "~~~")):
                char = lstripped[0]
                run_len = len(lstripped) - len(lstripped.lstrip(char))
                rest = lstripped[run_len:].strip()
                tokens = rest.split()
                lang = tokens[0].lower() if tokens else ""
                if lang in ("python", "py"):
                    in_fence = True
                    fence_char = char
                    fence_len = run_len
                    start_line = line_idx
                    block_lines = []
        else:
            if lstripped.startswith(fence_char):
                run_len = len(lstripped) - len(lstripped.lstrip(fence_char))
                if run_len >= fence_len and not lstripped[run_len:].strip():
                    in_fence = False
                    block_index += 1
                    code = "".join(block_lines)
                    blocks.append(
                        CodeBlock(
                            source_path=path,
                            block_index=block_index,
                            start_line=start_line,
                            code=code,
                        )
                    )
                    block_lines = []
                    continue
            block_lines.append(line)

    return blocks


def execute_code_block(block: CodeBlock) -> None:
    """Execute a single extracted code block with a clean global namespace."""
    globals_dict: dict[str, Any] = {"__name__": "__main__"}
    try:
        compiled = compile(block.code, str(block.source_path), "exec")
        exec(compiled, globals_dict)
    except BaseException as exc:
        raise DocumentationExampleError(
            path=block.source_path,
            block_index=block.block_index,
            original_error=exc,
            line_number=block.start_line,
        ) from exc


def execute_markdown_file(path: Path) -> list[CodeBlock]:
    """Extract and execute all Python code blocks from a markdown document."""
    blocks = extract_python_blocks(path)
    for block in blocks:
        execute_code_block(block)
    return blocks


def test_readme_example() -> None:
    """The worked example in README.md executes successfully."""
    readme_path = ROOT / "README.md"
    blocks = extract_python_blocks(readme_path)
    assert len(blocks) >= 1, "README.md must contain at least one Python example"
    for block in blocks:
        execute_code_block(block)


def test_docs_site_examples() -> None:
    """All Python examples under docs/site/*.md execute successfully."""
    site_dir = ROOT / "docs" / "site"
    md_files = sorted(site_dir.glob("*.md"))
    assert md_files, f"No markdown files found under {site_dir}"
    for md_path in md_files:
        execute_markdown_file(md_path)


def test_broken_example_fixture_fails() -> None:
    """The harness fails when an example snippet contains an intentional defect."""
    fixture_path = ROOT / "tests" / "fixtures" / "docs-broken-example" / "broken.md"
    with pytest.raises(DocumentationExampleError) as exc_info:
        execute_markdown_file(fixture_path)

    err = exc_info.value
    assert err.path == fixture_path
    assert "broken.md" in str(err)
    assert err.block_index == 1
    assert "block 1" in str(err)
    assert isinstance(err.original_error, AttributeError)
