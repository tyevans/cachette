"""Run a whole asset set for one style.

A style is an asset type. The guide loader reads `styleguide/<style>.md`
for the rules, and it reads `styleguide/exemplars/<style>/` for the
accepted exemplars. The render module gives a default size set to any
asset type that no row names, so a style needs only a rules file.

This module runs every subject of the asset set through the refine loop,
one session for each subject, and it reports the best score of each
subject. The subject list has one declaration site, and this module reads
it from there.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from . import guide as guide_module
from . import loop, session, subjects
from .client import ClientError


@dataclass
class SubjectResult:
    """What one subject produced in a set run."""

    name: str
    session_id: str
    best_score: int | None = None
    best_reference: str | None = None
    rounds: int = 0
    seconds: float = 0.0
    tokens: int = 0
    error: str | None = None


def best_of(results: list[loop.RoundResult]) -> tuple[int | None, str | None]:
    """Give the best score of a whole session, and where it came from.

    A later round wins a tie, so a reader sees the newest drawing that
    reached the best score.
    """
    best_score: int | None = None
    reference: str | None = None
    for result in results:
        for variant in result.variants:
            score = variant.score
            if score is None:
                continue
            if best_score is None or score >= best_score:
                best_score = score
                where = session.round_name(result.index)
                reference = f"{where}/variant-{variant.letter}"
    return best_score, reference


def run_set(
    style: str,
    rounds: int,
    variants: int,
    names: list[str] | None = None,
    root: Path = session.SESSION_ROOT,
    exemplar_limit: int = guide_module.DEFAULT_EXEMPLAR_LIMIT,
    log=print,
) -> list[SubjectResult]:
    """Run every subject of the asset set for one style.

    The function raises GuideError when the style has no rules file. It
    catches an endpoint error for one subject, records it, and goes on to
    the next subject, so one bad subject does not lose the whole run.
    """
    guide_module.load(style, limit=exemplar_limit)
    wanted = subjects.resolve(names)
    stamp = session.new_session_id()

    out: list[SubjectResult] = []
    for name in wanted:
        session_id = f"{name}-{stamp}"
        item = SubjectResult(name=name, session_id=session_id)
        out.append(item)
        log(f"[{style}] {name}")
        try:
            results = loop.run_session(
                asset=style,
                subject=subjects.phrase(name),
                rounds=rounds,
                variants=variants,
                session_id=session_id,
                root=root,
                exemplar_limit=exemplar_limit,
                log=log,
            )
        except (ClientError, OSError) as error:
            item.error = str(error)
            log(f"  failed: {error}")
            continue
        item.rounds = len(results)
        item.seconds = sum(result.seconds for result in results)
        item.tokens = sum(
            result.prompt_tokens + result.completion_tokens for result in results
        )
        item.best_score, item.best_reference = best_of(results)
    return out


def format_table(style: str, results: list[SubjectResult]) -> str:
    """Give the summary table of a set run as text."""
    lines = [f"style {style}", "", "subject    best  where              seconds  tokens"]
    for item in results:
        if item.error:
            lines.append(f"{item.name:<10} {'err':<5} {item.error[:18]:<18}")
            continue
        score = "-" if item.best_score is None else str(item.best_score)
        where = item.best_reference or "-"
        lines.append(
            f"{item.name:<10} {score:<5} {where:<18} "
            f"{item.seconds:<8.1f} {item.tokens}"
        )
    scored = [item.best_score for item in results if item.best_score is not None]
    if scored:
        mean = sum(scored) / len(scored)
        lines.append("")
        lines.append(f"{len(scored)} scored, mean {mean:.1f}")
    return "\n".join(lines)
