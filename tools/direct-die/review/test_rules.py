"""Tests for the one write into a style rules file.

The analysis proposes a rule. A person accepts it, and this module appends it.
It appends under one heading that it creates once. It does not guess where the
rule belongs, because a wrong guess puts a rule where nobody reads it.
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

import rules  # noqa: E402

BODY = "# cartoon\n\n## 1. Geometry\n\n- The tile is a hexagon.\n"


def _tree(tmp_path: Path) -> Path:
    root = tmp_path / "styleguide"
    root.mkdir()
    (root / "cartoon.md").write_text(BODY, encoding="utf-8")
    return root


def test_the_rule_lands_under_a_new_heading(tmp_path: Path) -> None:
    root = _tree(tmp_path)
    path = rules.append_rule(root, "cartoon", "Use three shapes or fewer.")
    text = path.read_text(encoding="utf-8")
    assert rules.ADDED_HEADING in text
    assert "- Use three shapes or fewer." in text
    assert text.index("## 1. Geometry") < text.index(rules.ADDED_HEADING)


def test_a_second_rule_reuses_the_heading(tmp_path: Path) -> None:
    root = _tree(tmp_path)
    rules.append_rule(root, "cartoon", "Use three shapes or fewer.")
    path = rules.append_rule(root, "cartoon", "Keep the palette to four colours.")
    text = path.read_text(encoding="utf-8")
    assert text.count(rules.ADDED_HEADING) == 1
    assert "- Use three shapes or fewer." in text
    assert "- Keep the palette to four colours." in text


def test_the_original_rules_survive(tmp_path: Path) -> None:
    root = _tree(tmp_path)
    path = rules.append_rule(root, "cartoon", "Use three shapes or fewer.")
    assert "- The tile is a hexagon." in path.read_text(encoding="utf-8")


def test_the_write_leaves_no_temporary_file(tmp_path: Path) -> None:
    root = _tree(tmp_path)
    rules.append_rule(root, "cartoon", "Use three shapes or fewer.")
    assert sorted(item.name for item in root.iterdir()) == ["cartoon.md"]


def test_an_empty_rule_is_refused(tmp_path: Path) -> None:
    root = _tree(tmp_path)
    with pytest.raises(rules.RuleError):
        rules.append_rule(root, "cartoon", "   ")


def test_a_rule_with_a_newline_is_refused(tmp_path: Path) -> None:
    root = _tree(tmp_path)
    with pytest.raises(rules.RuleError):
        rules.append_rule(root, "cartoon", "one line\nand another")


def test_a_style_with_no_rules_file_is_refused(tmp_path: Path) -> None:
    root = _tree(tmp_path)
    with pytest.raises(rules.RuleError):
        rules.append_rule(root, "pencil", "Use three shapes or fewer.")


def test_a_style_name_with_a_separator_is_refused(tmp_path: Path) -> None:
    root = _tree(tmp_path)
    with pytest.raises(rules.RuleError):
        rules.append_rule(root, "../secrets", "Use three shapes or fewer.")
