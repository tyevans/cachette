"""The refine loop.

One round does four things for each variant. It produces or revises the
SVG source. It rasterises that source at the display size and at the
inspection size. It shows both rasters to the model, beside the guide
rules and the accepted exemplars. It stores the critique that comes back.

The critique must come back as JSON with named fields. Free prose cannot
drive a loop, and two prose answers cannot be compared between rounds.
The loop validates the JSON and asks once more when the answer is
malformed.

The generation step sends text only. The critique step is the step that
looks at pictures. This keeps the prompt inside the 16384 token window of
the model.
"""

from __future__ import annotations

import json
import re
from collections.abc import Sequence
from dataclasses import dataclass, field
from pathlib import Path

from . import guide as guide_module
from . import render, session
from . import steer as steer_module
from .client import ClientError, Image, ask, ask_with_images

# The four variant lenses. Each one gives a different direction and a
# different temperature, so the four drawings differ from each other.
VARIANT_LENSES: dict[str, tuple[str, float]] = {
    "a": (
        "Follow the guide exactly. Take the safest reading of every rule.",
        0.30,
    ),
    "b": (
        "Raise the contrast. Make the shapes inside the tile larger and "
        "bolder than you first want to.",
        0.70,
    ),
    "c": (
        "Use fewer shapes than the guide permits. Aim for the strongest "
        "silhouette at the display size.",
        0.85,
    ),
    "d": (
        "Take the largest departure that the rules still permit. Surprise "
        "the art director, but break no rule.",
        1.05,
    ),
}

# How many refused drawings go into one prompt. An SVG document of this tool
# runs about 800 tokens, and the model window is 16384. The newest refusals
# win, because they answer the drawing that the person just saw.
MAX_REFUSALS = 2

ARTIST_SYSTEM = (
    "You are a game artist. You write SVG source and nothing else. "
    "You answer with one SVG document. You do not write a sentence "
    "before it or after it. You do not use a markdown fence."
)

CRITIC_SYSTEM = (
    "You are a strict art director for a strategy game. You judge one "
    "candidate asset against a written style guide and against accepted "
    "exemplars. You answer with one JSON object and nothing else."
)

CRITIQUE_SHAPE = (
    'Answer with one JSON object of this exact shape:\n'
    '{"verdict": "<one sentence>", '
    '"faults": ["<one specific, fixable fault>", "..."], '
    '"score": <integer 0 to 100>}\n'
    "Rules for the answer:\n"
    "- Name a fault only when you can see it in a picture. Quote the "
    "guide rule that the fault breaks.\n"
    "- Write each fault as an instruction that an artist can act on.\n"
    "- List three faults or fewer. List the worst first.\n"
    "- An empty fault list means the asset is ready to accept.\n"
    "- Take the score from the score scale above. Judge it against the "
    "anchors in that table, and not against an earlier round.\n"
    "- Write no text outside the JSON object."
)


@dataclass
class VariantResult:
    """What one variant produced in one round."""

    letter: str
    svg: str | None = None
    critique: dict | None = None
    error: str | None = None
    seconds: float = 0.0
    prompt_tokens: int = 0
    completion_tokens: int = 0

    @property
    def score(self) -> int | None:
        """Give the score of the variant, or None when it has none."""
        if not self.critique:
            return None
        value = self.critique.get("score")
        return value if isinstance(value, int) else None


@dataclass
class RoundResult:
    """What one round produced."""

    index: int
    variants: list[VariantResult] = field(default_factory=list)
    seconds: float = 0.0

    @property
    def prompt_tokens(self) -> int:
        """Give the prompt tokens that the round spent."""
        return sum(variant.prompt_tokens for variant in self.variants)

    @property
    def completion_tokens(self) -> int:
        """Give the completion tokens that the round spent."""
        return sum(variant.completion_tokens for variant in self.variants)

    def best(self) -> VariantResult | None:
        """Give the variant with the highest score."""
        scored = [item for item in self.variants if item.score is not None]
        if not scored:
            return None
        return max(scored, key=lambda item: item.score or 0)


_JSON_BLOCK = re.compile(r"\{.*\}", re.DOTALL)


def parse_critique(text: str) -> dict:
    """Take the critique object out of an answer, and check its shape.

    The function raises ValueError when the answer holds no valid object.
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

    verdict = value.get("verdict")
    faults = value.get("faults")
    score = value.get("score")
    if not isinstance(verdict, str) or not verdict.strip():
        raise ValueError("the field 'verdict' is missing or empty")
    if not isinstance(faults, list) or not all(
        isinstance(item, str) for item in faults
    ):
        raise ValueError("the field 'faults' is not a list of strings")
    if isinstance(score, bool) or not isinstance(score, (int, float)):
        raise ValueError("the field 'score' is not a number")
    return {
        "verdict": verdict.strip(),
        "faults": [item.strip() for item in faults if item.strip()],
        "score": max(0, min(100, int(round(float(score))))),
    }


def _taken_refusals(denied_sources: Sequence[str]) -> list[str]:
    """Give the refusals that a prompt actually holds.

    This is the one place that drops an empty source and caps the count at
    `MAX_REFUSALS`. The prompt builder and the round summary both call this,
    so they cannot disagree on how many refusals went in.
    """
    return [item for item in denied_sources if item and item.strip()][-MAX_REFUSALS:]


def _refusal_block(denied_sources: Sequence[str]) -> str:
    """Give the prompt section that shows the drawings a person refused.

    Give the empty string when nobody refused anything. The section holds SVG
    source, because the generation step sends no picture, and that is what
    keeps the prompt inside the model window.
    """
    taken = _taken_refusals(denied_sources)
    if not taken:
        return ""
    bodies = "\n-----\n".join(item.strip() for item in taken)
    return (
        "\nDRAWINGS THE ART DIRECTOR REFUSED. Do not draw like these. Do not "
        "reuse their shapes or their palette.\n"
        "-----\n"
        f"{bodies}\n"
        "-----\n"
    )


def build_creation_prompt(
    the_guide: guide_module.Guide,
    subject: str,
    lens: str,
    sizes: render.SizeSet,
    denied_sources: Sequence[str] = (),
    standing_note: str | None = None,
) -> str:
    """Build the prompt that makes the first drawing of an asset."""
    standing = ""
    if standing_note:
        standing = (
            "\nDIRECTION FROM THE HUMAN ART DIRECTOR. This outranks every "
            "other note. Do what it says first.\n"
            f"{standing_note.strip()}\n"
        )
    return (
        f"Draw one {the_guide.asset} for a strategy game world map.\n"
        f"The subject is: {subject}\n\n"
        "The style guide follows. Obey every rule in it.\n"
        "-----\n"
        f"{the_guide.rules}\n"
        "-----\n"
        + _refusal_block(denied_sources)
        + standing
        + f"\nYour direction for this drawing: {lens}\n\n"
        f"The map draws this asset at {sizes.display} pixels. A person "
        f"inspects it at {sizes.inspection} pixels. It must read at the "
        "smaller size.\n\n"
        "Answer with the SVG document only."
    )


def build_revision_prompt(
    the_guide: guide_module.Guide,
    subject: str,
    lens: str,
    sizes: render.SizeSet,
    parent_svg: str,
    faults: list[str],
    human_text: str | None,
    standing_note: str | None = None,
    denied_sources: Sequence[str] = (),
) -> str:
    """Build the prompt that revises a drawing.

    The two human notes go above the model critique, and the prompt says that
    the human outranks the model. The standing note goes above the note for
    this round, because it holds for the whole session.
    """
    direction = []
    if standing_note:
        direction.append(
            "STANDING DIRECTION FROM THE HUMAN ART DIRECTOR. It holds for "
            "the whole session, and it outranks every other note below it.\n"
            f"{standing_note.strip()}"
        )
    if human_text:
        direction.append(
            "DIRECTION FROM THE HUMAN ART DIRECTOR FOR THIS ROUND. This "
            "outranks every model note below it. Do what it says first.\n"
            f"{human_text.strip()}"
        )
    if faults:
        listed = "\n".join(f"- {item}" for item in faults)
        direction.append(
            "Notes from the model critique of this same drawing. They "
            "rank below the human direction.\n" + listed
        )
    if not direction:
        direction.append(
            "No critique came back. Improve the weakest part of the "
            "drawing at the display size."
        )

    return (
        f"Revise one {the_guide.asset} for a strategy game world map.\n"
        f"The subject is: {subject}\n\n"
        "The style guide follows. Obey every rule in it.\n"
        "-----\n"
        f"{the_guide.rules}\n"
        "-----\n\n"
        "This is the current SVG source.\n"
        "-----\n"
        f"{parent_svg.strip()}\n"
        "-----\n"
        + _refusal_block(denied_sources)
        + "\n"
        + "\n\n".join(direction)
        + f"\n\nYour direction for this variant: {lens}\n\n"
        f"The map draws this asset at {sizes.display} pixels. It must "
        "read at that size.\n\n"
        "Answer with the revised SVG document only. Keep what already "
        "works. Change what the notes name."
    )


def critique_variant(
    the_guide: guide_module.Guide,
    subject: str,
    sizes: render.SizeSet,
    rasters: dict[str, bytes],
) -> tuple[dict, float, int, int]:
    """Show one candidate to the model and give the parsed critique.

    The function asks once more when the first answer is malformed. It
    raises ValueError when the second answer is malformed too.
    """
    images = list(the_guide.exemplars)
    images.append(
        Image(
            label=(
                f"CANDIDATE AT THE DISPLAY SIZE, {sizes.display} pixels. "
                "This is the size a player sees."
            ),
            data=rasters["display"],
        )
    )
    images.append(
        Image(
            label=(
                f"CANDIDATE AT THE INSPECTION SIZE, {sizes.inspection} "
                "pixels. This is the same drawing, enlarged."
            ),
            data=rasters["inspection"],
        )
    )

    prompt = (
        f"The candidate is a {the_guide.asset}. Its subject is: "
        f"{subject}\n\n"
        "The style guide follows.\n"
        "-----\n"
        f"{the_guide.rules}\n"
        "-----\n\n"
        "The score scale follows.\n"
        "-----\n"
        f"{the_guide.score_scale}\n"
        "-----\n\n"
        "Judge the candidate against the guide and against the accepted "
        "exemplars above. Judge it first at the display size, because "
        "that is where a player sees it.\n\n" + CRITIQUE_SHAPE
    )

    seconds = 0.0
    prompt_tokens = 0
    completion_tokens = 0
    last_error = "no answer"
    for attempt in range(2):
        text = prompt if attempt == 0 else prompt + (
            "\n\nYour last answer was not valid JSON. Answer with the "
            "JSON object only, and write nothing else."
        )
        reply = ask_with_images(
            text,
            images,
            system=CRITIC_SYSTEM,
            temperature=0.2 if attempt == 0 else 0.0,
        )
        seconds += reply.seconds
        prompt_tokens += reply.prompt_tokens
        completion_tokens += reply.completion_tokens
        try:
            return parse_critique(reply.text), seconds, prompt_tokens, completion_tokens
        except ValueError as error:
            last_error = str(error)
    raise ValueError(f"the critique stayed malformed: {last_error}")


def produce_svg(prompt: str, temperature: float) -> tuple[str, float, int, int]:
    """Ask for one SVG document, and give the extracted source.

    The function asks once more when the first answer holds no SVG.
    """
    seconds = 0.0
    prompt_tokens = 0
    completion_tokens = 0
    last_error = "no answer"
    for attempt in range(2):
        text = prompt if attempt == 0 else prompt + (
            "\n\nYour last answer held no SVG document. Start the answer "
            "with '<svg' and end it with '</svg>'."
        )
        reply = ask(text, system=ARTIST_SYSTEM, temperature=temperature)
        seconds += reply.seconds
        prompt_tokens += reply.prompt_tokens
        completion_tokens += reply.completion_tokens
        try:
            source = render.extract_svg(reply.text)
            render.render(source, 32)
            return source, seconds, prompt_tokens, completion_tokens
        except render.RenderError as error:
            last_error = str(error)
    raise render.RenderError(f"no usable SVG came back: {last_error}")


def assign_parents(
    parents: Sequence[str], letters: Sequence[str]
) -> dict[str, str]:
    """Give each variant letter the parent that it revises.

    A person can like more than one drawing. The round cycles the liked
    parents across the variant letters, so each parent gets a spread of
    directions rather than one direction. One parent goes to every letter,
    which is the behaviour of a single choice.
    """
    if not parents:
        return {}
    return {
        letter: parents[position % len(parents)]
        for position, letter in enumerate(letters)
    }


@dataclass(frozen=True)
class RoundPlan:
    """What one round revises, refuses, and reads as direction."""

    parents: dict[str, str] = field(default_factory=dict)
    faults: dict[str, list[str]] = field(default_factory=dict)
    denied_sources: tuple[str, ...] = ()
    note: str | None = None
    text: str | None = None


def plan_round(
    store: session.Session, index: int, letters: Sequence[str]
) -> RoundPlan:
    """Read the session and give the plan of one round.

    The plan names the parent of each variant letter, the faults of each
    parent, and the SVG source of each drawing that the person refused.

    A refusal whose SVG is not on disk drops out. The picture is what the
    prompt shows, and there is nothing to show.
    """
    found = steer_module.collect(store, index)
    sources: list[str] = []
    for reference in found.denied:
        number, letter = steer_module.split_reference(reference)
        text = store.svg(number, letter)
        if text and text.strip():
            sources.append(text)
    return RoundPlan(
        parents=assign_parents(found.parents, letters),
        faults=dict(found.faults),
        denied_sources=tuple(sources),
        note=found.note,
        text=found.text,
    )


def run_round(
    store: session.Session,
    the_guide: guide_module.Guide,
    subject: str,
    index: int,
    variants: int,
    log,
) -> RoundResult:
    """Run one round, and write every file that the round produces."""
    sizes = render.sizes_for(the_guide.asset)
    letters = session.VARIANT_LETTERS[:variants]
    plan = plan_round(store, index, letters)
    result = RoundResult(index=index)
    round_path = store.round_path(index)

    for letter in letters:
        lens, temperature = VARIANT_LENSES[letter]
        variant = VariantResult(letter=letter)
        result.variants.append(variant)
        reference = plan.parents.get(letter)
        parent_svg = None
        if reference:
            number, parent_letter = steer_module.split_reference(reference)
            parent_svg = store.svg(number, parent_letter)
        if parent_svg:
            prompt = build_revision_prompt(
                the_guide,
                subject,
                lens,
                sizes,
                parent_svg,
                plan.faults.get(reference, []),
                plan.text,
                standing_note=plan.note,
                denied_sources=plan.denied_sources,
            )
        else:
            prompt = build_creation_prompt(
                the_guide,
                subject,
                lens,
                sizes,
                denied_sources=plan.denied_sources,
                standing_note=plan.note,
            )

        try:
            source, seconds, prompt_tokens, completion_tokens = produce_svg(
                prompt, temperature
            )
        except (render.RenderError, ClientError) as error:
            variant.error = f"draw: {error}"
            log(f"    variant {letter}: {variant.error}")
            continue
        variant.svg = source
        variant.seconds += seconds
        variant.prompt_tokens += prompt_tokens
        variant.completion_tokens += completion_tokens

        rasters = render.render_both(source, sizes)
        session.write_text(round_path / f"variant-{letter}.svg", source)
        session.write_bytes(round_path / f"variant-{letter}.png", rasters["display"])
        session.write_bytes(
            round_path / f"variant-{letter}.large.png", rasters["inspection"]
        )

        try:
            critique, seconds, prompt_tokens, completion_tokens = critique_variant(
                the_guide, subject, sizes, rasters
            )
        except (ValueError, ClientError) as error:
            variant.error = f"critique: {error}"
            log(f"    variant {letter}: {variant.error}")
            continue
        variant.critique = critique
        variant.seconds += seconds
        variant.prompt_tokens += prompt_tokens
        variant.completion_tokens += completion_tokens
        session.write_json(
            round_path / f"variant-{letter}.critique.json", critique
        )
        log(
            f"    variant {letter}: score {critique['score']}, "
            f"{len(critique['faults'])} faults, "
            f"{variant.prompt_tokens + variant.completion_tokens} tokens"
        )

    result.seconds = sum(variant.seconds for variant in result.variants)
    summary = "first drawing" if not plan.parents else "revision"
    if plan.note or plan.text:
        summary += "; the human gave direction"
    held_refusals = _taken_refusals(plan.denied_sources)
    if held_refusals:
        summary += f"; {len(held_refusals)} refused drawings in the prompt"
    session.write_json(
        round_path / "meta.json",
        {
            "round": index,
            "prompt_summary": f"{subject}; {summary}",
            "parents": dict(plan.parents),
        },
    )
    return result


def run_session(
    asset: str,
    subject: str,
    rounds: int,
    variants: int,
    session_id: str | None = None,
    root: Path = session.SESSION_ROOT,
    exemplar_limit: int = guide_module.DEFAULT_EXEMPLAR_LIMIT,
    log=print,
) -> list[RoundResult]:
    """Run a whole session, and give the result of each round."""
    from .client import MODEL

    the_guide = guide_module.load(asset, limit=exemplar_limit)
    store = session.Session(asset, session_id or session.new_session_id(), root=root)
    started_at = store.existing_rounds()
    first = (max(started_at) + 1) if started_at else 0

    log(
        f"session {store.session_id} on {asset}: guide "
        f"{the_guide.version}, {len(the_guide.exemplars)} exemplars"
    )
    store.write_header(MODEL, the_guide.version)

    results: list[RoundResult] = []
    for offset in range(rounds):
        index = first + offset
        log(f"  {session.round_name(index)}")
        result = run_round(store, the_guide, subject, index, variants, log)
        results.append(result)
        best = result.best()
        if best:
            log(
                f"    best: variant {best.letter} at {best.score}; "
                f"{result.seconds:.1f} s for the round"
            )
    return results
