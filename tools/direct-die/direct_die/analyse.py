"""The analysis of a liked set against a refused set.

The analysis runs between two rounds, and not inside one. A person ranks the
drawings of a round, reads this analysis, tunes the prompt, and then asks for
the next round.

It answers three questions. What do the liked drawings share that the refused
ones lack? Which liked drawing is best? Which rule of the guide would have
told the artist this without a person in the room?

## Why an order and not a score

A large change to a drawing moves the absolute score by a few points.[^1] An
order between two drawings does not have that defect, because it compares the
two drawings and not each drawing against a table. The order does not replace
the score. The score still decides which drawing wins when no person chose.

## References

[^1]: The tool guide, the known limits. `tools/direct-die/README.md`
"""

from __future__ import annotations

import json
import re
from collections.abc import Sequence
from pathlib import Path

from . import guide as guide_module
from . import render, session, steer
from .client import ClientError, Image, ask_with_images

ANALYST_SYSTEM = (
    "You are the assistant of an art director for a strategy game. The art "
    "director accepted some drawings and refused others. You say what the "
    "two groups differ by. You answer with one JSON object and nothing else."
)

ANALYSIS_SHAPE = (
    "Answer with one JSON object of this exact shape:\n"
    '{"preference": "<two sentences or fewer>", '
    '"order": ["<letter>", "..."], '
    '"reasons": {"<letter>": "<one sentence>"}, '
    '"guide_edit": {"section": "<a section name>", "rule": "<one sentence>"}}\n'
    "Rules for the answer:\n"
    "- Write the preference in the vocabulary of the style guide above.\n"
    "- Name a difference only when you can see it in a picture.\n"
    "- The field 'order' holds every accepted letter once, best first, and "
    "no other letter.\n"
    "- The field 'reasons' gives one sentence for each accepted letter. Say "
    "why it beats the letter below it.\n"
    "- The field 'guide_edit' proposes one rule that the guide does not "
    "state, which would have stopped the refused drawings. Write it as an "
    "instruction to an artist. Use null when the guide already states it.\n"
    "- Write no text outside the JSON object."
)


class AnalysisError(RuntimeError):
    """The analysis could not run, or the answer stayed malformed."""


_JSON_BLOCK = re.compile(r"\{.*\}", re.DOTALL)


def build_analysis_prompt(
    the_guide: guide_module.Guide, subject: str, liked: Sequence[str]
) -> str:
    """Build the prompt that compares the accepted set to the refused set."""
    letters = ", ".join(f'"{letter}"' for letter in liked)
    return (
        f"The drawings above are candidates for one {the_guide.asset}. Their "
        f"subject is: {subject}\n\n"
        "Each label says whether the art director accepted the drawing or "
        "refused it.\n\n"
        "The style guide follows.\n"
        "-----\n"
        f"{the_guide.rules}\n"
        "-----\n\n"
        f"The accepted letters are: {letters}\n\n"
        "Say what the accepted drawings share that the refused drawings "
        "lack. Judge at the display size, because that is where a player "
        "sees the asset.\n\n" + ANALYSIS_SHAPE
    )


def parse_analysis(text: str, liked: Sequence[str]) -> dict:
    """Take the analysis object out of an answer, and check its shape.

    Raise ValueError when the answer holds no valid object.
    """
    match = _JSON_BLOCK.search(text)
    if not match:
        raise ValueError("the answer holds no JSON object")
    try:
        value = json.loads(match.group(0))
    except json.JSONDecodeError as error:
        raise ValueError(f"the JSON does not parse: {error}") from error
    if not isinstance(value, dict):
        raise ValueError("the JSON is not an object")

    preference = value.get("preference")
    if not isinstance(preference, str) or not preference.strip():
        raise ValueError("the field 'preference' is missing or empty")

    order = value.get("order")
    if not isinstance(order, list) or not all(isinstance(item, str) for item in order):
        raise ValueError("the field 'order' is not a list of strings")
    if sorted(order) != sorted(liked):
        raise ValueError(
            "the field 'order' must hold every accepted letter once: "
            f"{sorted(liked)}"
        )

    reasons_in = value.get("reasons")
    reasons = {
        letter: (
            reasons_in.get(letter, "").strip()
            if isinstance(reasons_in, dict) and isinstance(reasons_in.get(letter), str)
            else ""
        )
        for letter in order
    }

    edit = value.get("guide_edit")
    if edit is None:
        parsed_edit = None
    elif isinstance(edit, dict):
        rule = edit.get("rule")
        if not isinstance(rule, str) or not rule.strip():
            raise ValueError("the guide edit holds no rule")
        section = edit.get("section")
        parsed_edit = {
            "section": section.strip() if isinstance(section, str) else "",
            "rule": rule.strip(),
        }
    else:
        raise ValueError("the field 'guide_edit' is not an object and not null")

    return {
        "preference": preference.strip(),
        "order": list(order),
        "reasons": reasons,
        "guide_edit": parsed_edit,
    }


def _rasters(
    store: session.Session, index: int, letters: Sequence[str], verdict: str, sizes
) -> tuple[list[Image], list[str]]:
    """Rasterise each named variant at the display size, with a label.

    Give the images, and the letters that a picture actually went out for.
    A letter drops out of the second list when its SVG is missing, or when
    it will not rasterise. The caller must not ask the model about a letter
    that dropped out.
    """
    images: list[Image] = []
    sent: list[str] = []
    for letter in letters:
        source = store.svg(index, letter)
        if not source:
            continue
        try:
            data = render.render(source, sizes.display)
        except render.RenderError:
            continue
        images.append(
            Image(
                label=(
                    f"{verdict} BY THE ART DIRECTOR: variant {letter}, at "
                    f"{sizes.display} pixels, the size a player sees."
                ),
                data=data,
            )
        )
        sent.append(letter)
    return images, sent


def run(
    asset: str,
    session_id: str,
    index: int,
    root: Path = session.SESSION_ROOT,
    ask=ask_with_images,
    log=print,
) -> dict:
    """Analyse one round, write `analysis.json`, and give what it wrote.

    Raise `AnalysisError` when the round holds no like, when the round is not
    on disk, and when the answer stays malformed after a second ask.
    """
    store = session.Session(asset, session_id, root=root)
    directory = store.round_path(index)
    if not directory.is_dir():
        raise AnalysisError(f"no such round: {asset}/{session_id}/round-{index:02d}")

    found = steer.read_feedback(store.feedback(index))
    if not found.likes:
        raise AnalysisError(
            "the analysis needs at least one liked drawing. Like one, then "
            "ask again."
        )

    the_guide = guide_module.load(asset, limit=0)
    sizes = render.sizes_for(asset)
    images, liked_sent = _rasters(store, index, found.order, "ACCEPTED", sizes)
    missing = [letter for letter in found.order if letter not in liked_sent]
    if missing:
        raise AnalysisError(
            f"the picture for variant {missing[0]} is missing, and it was "
            "liked. Draw it again, or refuse it, before you ask for an "
            "analysis."
        )
    if not images:
        raise AnalysisError("no liked drawing has a picture on disk")
    refused_images, _ = _rasters(store, index, found.denies, "REFUSED", sizes)
    images.extend(refused_images)

    meta = session.read_json(directory / "meta.json") or {}
    summary = meta.get("prompt_summary")
    subject = summary.split(";")[0].strip() if isinstance(summary, str) else asset

    prompt = build_analysis_prompt(the_guide, subject, liked_sent)
    log(f"analysing {asset}/{session_id}/round-{index:02d}: {len(images)} pictures")

    last_error = "no answer"
    for attempt in range(2):
        text = prompt if attempt == 0 else prompt + (
            "\n\nYour last answer was not valid JSON of the shape above. "
            "Answer with the JSON object only, and write nothing else."
        )
        try:
            reply = ask(
                text,
                images,
                system=ANALYST_SYSTEM,
                temperature=0.3 if attempt == 0 else 0.0,
            )
        except ClientError as error:
            raise AnalysisError(f"the endpoint failed: {error}") from error
        try:
            parsed = parse_analysis(reply.text, liked_sent)
        except ValueError as error:
            last_error = str(error)
            continue
        payload = {"round": index, **parsed, "at": session.now_iso()}
        session.write_json(directory / "analysis.json", payload)
        log(f"  the analysis ranks {', '.join(parsed['order'])}")
        return payload

    raise AnalysisError(f"the analysis stayed malformed: {last_error}")
