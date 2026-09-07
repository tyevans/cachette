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
from dataclasses import dataclass, field
from pathlib import Path

from . import guide as guide_module
from . import render, session
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
    parent: str | None
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


def build_creation_prompt(
    the_guide: guide_module.Guide,
    subject: str,
    lens: str,
    sizes: render.SizeSet,
) -> str:
    """Build the prompt that makes the first drawing of an asset."""
    return (
        f"Draw one {the_guide.asset} for a strategy game world map.\n"
        f"The subject is: {subject}\n\n"
        "The style guide follows. Obey every rule in it.\n"
        "-----\n"
        f"{the_guide.rules}\n"
        "-----\n\n"
        f"Your direction for this drawing: {lens}\n\n"
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
) -> str:
    """Build the prompt that revises a drawing.

    Human feedback goes above the model critique, and the prompt says
    that the human outranks the model.
    """
    direction = []
    if human_text:
        direction.append(
            "DIRECTION FROM THE HUMAN ART DIRECTOR. This outranks every "
            "other note below it. Do what it says first.\n"
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
        "-----\n\n"
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


def choose_parent(
    store: session.Session, index: int
) -> tuple[str | None, list[str], str | None]:
    """Choose the parent of the next round.

    The function gives the parent reference, the fault list of that
    parent, and the human feedback text.

    A human choice wins, because the human outranks the model. When no
    human chose, the parent is the highest scoring variant of every
    round so far, and not only of the last round. A critique names a
    fault even in a good drawing, and a revision that acts on that fault
    can make the drawing worse. The loop must not walk away from its
    best work when that happens. A later round wins a tie, so the loop
    still moves.
    """
    if index <= 0:
        return None, [], None
    previous = index - 1
    feedback = store.feedback(previous)
    human_text = None
    chosen: tuple[int, str] | None = None
    if feedback:
        text = feedback.get("text")
        if isinstance(text, str) and text.strip():
            human_text = text.strip()
        choice = feedback.get("choice")
        if isinstance(choice, str) and choice in session.VARIANT_LETTERS:
            chosen = (previous, choice)

    if chosen is None:
        best_score = -1
        for round_index in store.existing_rounds():
            if round_index > previous:
                continue
            for candidate in session.VARIANT_LETTERS:
                critique = store.critique(round_index, candidate)
                if not critique:
                    continue
                score = critique.get("score")
                if isinstance(score, int) and score >= best_score:
                    best_score = score
                    chosen = (round_index, candidate)

    if chosen is None:
        return None, [], human_text

    round_index, letter = chosen
    critique = store.critique(round_index, letter) or {}
    faults = [item for item in critique.get("faults", []) if isinstance(item, str)]
    return (
        f"{session.round_name(round_index)}/variant-{letter}",
        faults,
        human_text,
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
    parent, faults, human_text = choose_parent(store, index)
    parent_svg = None
    if parent:
        previous_index = int(parent.split("/")[0][len("round-") :])
        letter = parent.rsplit("-", 1)[1]
        parent_svg = store.svg(previous_index, letter)

    result = RoundResult(index=index, parent=parent)
    letters = session.VARIANT_LETTERS[:variants]
    round_path = store.round_path(index)

    for letter in letters:
        lens, temperature = VARIANT_LENSES[letter]
        variant = VariantResult(letter=letter)
        result.variants.append(variant)
        if parent_svg:
            prompt = build_revision_prompt(
                the_guide, subject, lens, sizes, parent_svg, faults, human_text
            )
        else:
            prompt = build_creation_prompt(the_guide, subject, lens, sizes)

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
    summary = "first drawing" if not parent else "revision"
    if human_text:
        summary += "; the human gave direction"
    session.write_json(
        round_path / "meta.json",
        {
            "round": index,
            "prompt_summary": f"{subject}; {summary}",
            "parent": parent,
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
