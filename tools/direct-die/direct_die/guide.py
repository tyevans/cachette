"""The living style guide.

The guide holds two things for each asset type. The first is a markdown
document of stated rules. The second is a directory of accepted
exemplars. The loader assembles the critique prompt from the rules and
attaches the exemplar pictures, so the model judges against pictures and
not only against adjectives.

An exemplar is an SVG file or a PNG file. The loader rasterises an SVG
exemplar at the inspection size of the asset type. It sends a PNG
exemplar as it is.
"""

from __future__ import annotations

import hashlib
from dataclasses import dataclass, field
from pathlib import Path

from . import render
from .client import Image

# The directory that holds the guide. It sits beside the package.
GUIDE_ROOT = Path(__file__).resolve().parent.parent / "styleguide"

# The number of exemplars that the loader attaches by default. Each image
# costs tokens, and the window is 16384 tokens.
DEFAULT_EXEMPLAR_LIMIT = 2


class GuideError(RuntimeError):
    """The guide for the asset type is missing or unreadable."""


# The file that states the score scale. It applies to every asset type,
# and it is the only declaration of the scale.
SCORE_FILE = "score.md"


@dataclass
class Guide:
    """The rules, the score scale, and the exemplars of one asset type."""

    asset: str
    rules: str
    score_scale: str
    version: str
    exemplars: list[Image] = field(default_factory=list)
    exemplar_names: list[str] = field(default_factory=list)


def _version(parts: list[bytes]) -> str:
    """Give a short digest of the guide content."""
    digest = hashlib.sha256()
    for part in parts:
        digest.update(part)
    return digest.hexdigest()[:12]


def asset_types(root: Path = GUIDE_ROOT) -> list[str]:
    """Give the asset types that the guide covers.

    The score scale is not an asset type, so this function drops it.
    """
    return sorted(
        path.stem for path in root.glob("*.md") if path.name != SCORE_FILE
    )


def load(
    asset: str,
    root: Path = GUIDE_ROOT,
    limit: int = DEFAULT_EXEMPLAR_LIMIT,
) -> Guide:
    """Load the rules and the exemplars of one asset type."""
    rules_path = root / f"{asset}.md"
    if not rules_path.is_file():
        known = ", ".join(asset_types(root)) or "none"
        raise GuideError(f"no guide for {asset!r}; the guide covers: {known}")
    rules = rules_path.read_text(encoding="utf-8")

    score_path = root / SCORE_FILE
    if not score_path.is_file():
        raise GuideError(f"the score scale is missing: {score_path.name}")
    score_scale = score_path.read_text(encoding="utf-8")

    sizes = render.sizes_for(asset)
    exemplar_dir = root / "exemplars" / asset
    images: list[Image] = []
    names: list[str] = []
    digest_parts: list[bytes] = [rules.encode("utf-8"), score_scale.encode("utf-8")]

    if exemplar_dir.is_dir():
        paths = sorted(
            path
            for path in exemplar_dir.iterdir()
            if path.suffix.lower() in (".svg", ".png")
        )
        for path in paths:
            raw = path.read_bytes()
            digest_parts.append(path.name.encode("utf-8"))
            digest_parts.append(raw)
            if len(images) >= limit:
                continue
            if path.suffix.lower() == ".svg":
                try:
                    png = render.render(raw.decode("utf-8"), sizes.inspection)
                except render.RenderError as error:
                    raise GuideError(
                        f"the exemplar {path.name} does not rasterise: {error}"
                    ) from error
            else:
                png = raw
            images.append(
                Image(label=f"ACCEPTED EXEMPLAR: {path.stem}", data=png)
            )
            names.append(path.name)

    return Guide(
        asset=asset,
        rules=rules,
        score_scale=score_scale,
        version=_version(digest_parts),
        exemplars=images,
        exemplar_names=names,
    )
