"""The asset set that a style must cover.

This module is the only declaration of the subject list. A style guide
names the same subjects in its own subject table, and the driver runs
the list from here. A style does not carry its own copy of the list.

The terrain names come from the engine terrain list. The upgrade names
come from the engine upgrade category list. The engine also has an
`OPEN` category, which means no upgrade. `OPEN` needs no art, so it is
not here.
"""

from __future__ import annotations

# The five terrain kinds. The key is the subject name. The value is the
# phrase that the tool sends to the model.
TERRAIN: dict[str, str] = {
    "water": "water: open sea or a lake tile",
    "plain": "plain: flat open grassland",
    "forest": "forest: dense conifer woodland",
    "hill": "hill: rolling raised ground",
    "mountain": "mountain: a high rocky peak",
}

# The six upgrade categories that need art.
UPGRADE: dict[str, str] = {
    "road": "road: a built route that crosses the tile",
    "terrace": "terrace: cut steps that farm a slope",
    "wonder": "wonder: one great monument",
    "store": "store: a place that holds goods",
    "wall": "wall: a defensive rampart",
    "lodging": "lodging: housing for the people",
}

# Every subject, terrain first.
SUBJECTS: dict[str, str] = {**TERRAIN, **UPGRADE}

# The order in which the driver runs the subjects.
SUBJECT_ORDER: list[str] = list(SUBJECTS)


def phrase(name: str) -> str:
    """Give the subject phrase for one subject name."""
    try:
        return SUBJECTS[name]
    except KeyError:
        known = ", ".join(SUBJECT_ORDER)
        raise KeyError(f"no subject {name!r}; the set holds: {known}") from None


def resolve(names: list[str] | None) -> list[str]:
    """Give the subject names to run.

    An empty request gives the whole set. The function raises KeyError
    for a name that the set does not hold.
    """
    if not names:
        return list(SUBJECT_ORDER)
    for name in names:
        phrase(name)
    return list(names)
