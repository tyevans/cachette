"""The engine asset names that a pack holds, and the subject text for each.

A pack names a drawing by the engine's own name for the thing. A later
consumer therefore needs no mapping table. The names are the terrain names
and the upgrade names of the simulation.[^1]

A style is an asset type in the tool, so one drawing is one style and slug
pair. This module declares the slug list once. The pack export, the matrix
view, and the start form all read it from here.

The subject text is a default. A person edits it in the start form before a
run begins. The tool records the text it used in the round metadata, so the
default here never becomes a second copy of what a session did.

## References

[^1]: The tool guide. `tools/direct-die/README.md`
"""

from __future__ import annotations

# The five terrain names, in the order that the matrix shows them.
TERRAIN_SLUGS = ("water", "plain", "forest", "hill", "mountain")

# The six upgrade names, in the order that the matrix shows them.
UPGRADE_SLUGS = ("road", "terrace", "wonder", "store", "wall", "lodging")

# Every slug, terrain first. The matrix rows follow this order.
SLUGS = TERRAIN_SLUGS + UPGRADE_SLUGS

# The kind of each slug. The matrix groups the rows by this.
KIND_OF = {slug: "terrain" for slug in TERRAIN_SLUGS} | {
    slug: "upgrade" for slug in UPGRADE_SLUGS
}

# The default subject text of each slug. A person edits it before a run.
SUBJECT_OF = {
    "water": "open water, deep and still",
    "plain": "flat open grassland",
    "forest": "a dense stand of trees",
    "hill": "a low rolling hill",
    "mountain": "a high rocky peak",
    "road": "a paved road across the tile",
    "terrace": "a terraced field cut into the ground",
    "wonder": "a great monument that one faction built",
    "store": "a granary that holds a harvest",
    "wall": "a defensive stone wall",
    "lodging": "a cluster of dwellings for people",
}


def is_slug(name: str) -> bool:
    """Report whether the name is one of the engine asset names."""
    return name in SLUGS


def subject_for(slug: str) -> str:
    """Give the default subject text of one slug."""
    return SUBJECT_OF.get(slug, slug)
