"""Memorable names for the things a watcher sees.

A watcher forms no attachment to faction 0. A run becomes a story when a
nation, a town and a person carry a name, so this turns an index or an
identity into words.

**A name is flavour and not simulation.** It changes no outcome, it enters no
state hash and a learner never needs one. The engine therefore holds indices
and identities alone, and the control plane holds the words. A game developer
who builds something else on this engine replaces the word tables and touches
no Rust.[^1]

**The same seed gives the same names.** A name comes from the world seed and
from the identity of the thing, and from nothing else. It does not come from a
counter, from the order that things were named in, or from the random module
of the standard library. Two watchers of one seed therefore see one story, and
a watcher who repeats a seed hears the same names again. This is the rule the
engine follows for a random draw, applied to a word.[^2]

**Nothing runs out.** A name is computed and not taken from a list, so a run
that founds a thousand towns names a thousand towns. The generator covers
about three million place names, so two towns of one run share a name about
one run in seventy at three hundred towns. A shared name is a repeat and not a
defect: the two towns stay separate everywhere the engine looks at them.

**Two things of one kind may share a name, and factions may not.** A faction
is the loudest name in the demonstration, and two factions called one thing
would make the toasts unreadable. A faction that draws a name another faction
already holds therefore draws again. The retry reads only the factions below
it by index, so the answer does not depend on the order a caller asks in.

References
----------
The Python boundary survey, the finding that names are flavour and not
simulation. ``docs/research/reports/``

ADR-0001, one binary gives one answer at any thread count.
``docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md``
"""

from __future__ import annotations

from types import ModuleType

from cachette.names import syllables

__all__ = ["Names", "syllables"]

# The width of the arithmetic. Every step stays inside 64 bits, so the answer
# is the same on any machine and in any interpreter.
_MASK = 0xFFFF_FFFF_FFFF_FFFF

# The three constants of the mixing function. They are the published ones of
# the split mix generator.
_STEP = 0x9E37_79B9_7F4A_7C15
_FIRST = 0xBF58_476D_1CE4_E5B9
_SECOND = 0x94D0_49BB_1331_11EB

# The kind of thing a name is for. The kind enters the key, so a faction and a
# person of one identity get different names.
_KIND_FACTION = 1
_KIND_PLACE = 2
_KIND_PERSON = 3

# How many times a faction draws again when another faction holds its name.
# A table of two hundred thousand nation names makes a second collision
# vanishingly rare, and the bound stops a very small table from looping.
_RETRIES = 8


def _mix(value: int) -> int:
    """Give back a well spread 64-bit number for a 64-bit number.

    The same input gives the same output on every machine, because every step
    is integer arithmetic inside a fixed width.
    """
    state = (value + _STEP) & _MASK
    state = ((state ^ (state >> 30)) * _FIRST) & _MASK
    state = ((state ^ (state >> 27)) * _SECOND) & _MASK
    return state ^ (state >> 31)


def _stem(seed: int, kind: int, identity: int) -> int:
    """Give back the key of one thing, from the seed, the kind and the identity.

    A caller mixes a draw number into this to get one word part. The stem is
    computed once for each thing, so a name of five parts costs five mixes and
    not twenty.
    """
    value = _mix(seed & _MASK)
    value = _mix(value ^ (kind & _MASK))
    return _mix(value ^ (identity & _MASK))


def _pick(table: tuple[str, ...], stem: int, draw: int) -> str:
    """Give back one entry of a table, for one draw of one thing."""
    return table[_mix(stem ^ (draw & _MASK)) % len(table)]


def _repeats(name: str, ending: str, vowels: str) -> bool:
    """Say whether an ending repeats the sound the name already ends with.

    A name that ends in a hissing sound cannot take an ending whose own first
    consonant hisses. "Toriashath" and "ish" give "Toriashathish", and a
    reader stops. "Toriashathian" and "Toriashathic" both read.

    The vowel at the front of the ending is skipped, because the consonant
    after it is the sound that clashes.
    """
    hisses = ("s", "sh", "ch", "th", "x", "z")
    consonant = ending.lstrip(vowels)
    return name.endswith(hisses) and consonant.startswith(hisses)


def _address(q: int, r: int) -> int:
    """Give back one number for one place on the map.

    The two coordinates are whole numbers that may be negative, so each takes
    the low 32 bits of its own half. Two different places give two different
    numbers inside a world of four thousand million tiles on a side.
    """
    return ((q & 0xFFFF_FFFF) << 32) | (r & 0xFFFF_FFFF)


class Names:
    """The names of one world.

    A namer holds the seed of its world and the tables it builds words from.
    It reads no world and calls no engine method, so a caller may build one
    before the world runs and keep it for the whole run.
    """

    __slots__ = ("_factions", "_seed", "_tables")

    def __init__(self, seed: int, tables: ModuleType = syllables) -> None:
        """Build the namer of the world this seed made.

        The tables module holds the word parts. Replace it to change the
        flavour of every name without changing any logic.
        """
        self._seed = seed & _MASK
        self._tables = tables
        # The names of the factions, by index, once each has been asked for.
        # A faction avoids a name a lower faction holds, so the answer for one
        # faction needs the answers for the ones below it.
        self._factions: dict[int, str] = {}

    def faction(self, faction: int) -> str:
        """Give back the name of the nation this faction is.

        A faction whose name another faction already holds draws again, so no
        two factions of one world share a name. The retry reads the factions
        below this one by index and nothing else, so the answer is the same
        whichever faction a caller asks for first.
        """
        known = self._factions.get(faction)
        if known is not None:
            return known
        taken = {self.faction(lower) for lower in range(faction)}
        name = ""
        for attempt in range(_RETRIES):
            name = self._nation(faction, attempt)
            if name not in taken:
                break
        self._factions[faction] = name
        return name

    def faction_adjective(self, faction: int) -> str:
        """Give back the word for a thing that belongs to this nation.

        A toast says that a nation declares war, and another says that its
        army marches. The second needs an adjective.

        The ending follows the last letters of the name, so the two words
        sound like one language. An ending that repeats the sound the name
        already ends with is dropped first: "Toriashath" with "ish" gives
        "Toriashathish", which no reader says twice.
        """
        name = self.faction(faction)
        vowels = self._tables.VOWEL_LETTERS
        if name[-1] in vowels:
            return name + "n" if name.endswith("a") else name + "an"
        endings = [
            ending
            for ending in self._tables.CONSONANT_ADJECTIVES
            if not _repeats(name, ending, vowels)
        ]
        if not endings:
            endings = list(self._tables.CONSONANT_ADJECTIVES)
        stem = _stem(self._seed, _KIND_FACTION, faction)
        return name + endings[_mix(stem ^ 0xAD) % len(endings)]

    def place(self, q: int, r: int) -> str:
        """Give back the name of the ground at this address.

        **The ground carries the name, not the settlement that stands on it.**
        The engine publishes a settlement identity nowhere a watcher reads a
        place: a campaign names the tile it marches on, and a founding names
        the tile it seated. A settlement never moves, so the name of its
        ground is the name of the settlement for the whole of its life, and it
        holds for a battlefield that carries no settlement at all.

        Every address has a name, so a run that founds towns for hours never
        runs out.
        """
        stem = _stem(self._seed, _KIND_PLACE, _address(q, r))
        word = _pick(self._tables.PLACE_WORDS, stem, 0)
        body = (
            _pick(self._tables.ONSETS, stem, 1)
            + _pick(self._tables.VOWELS, stem, 2)
            + _pick(self._tables.CODAS, stem, 3)
            + _pick(self._tables.PLACE_TAILS, stem, 4)
        )
        return f"{word} {body}" if word else body

    def person(self, character: int) -> str:
        """Give back the name of the person this identity is.

        The identity holds a slot and a generation, so a person born into the
        slot of a person who died gets a different name.
        """
        stem = _stem(self._seed, _KIND_PERSON, character)
        given = (
            _pick(self._tables.ONSETS, stem, 0)
            + _pick(self._tables.VOWELS, stem, 1)
            + _pick(self._tables.LINKS, stem, 2)
            + _pick(self._tables.GIVEN_TAILS, stem, 3)
        )
        family = (
            _pick(self._tables.ONSETS, stem, 4)
            + _pick(self._tables.VOWELS, stem, 5)
            + _pick(self._tables.CODAS, stem, 6)
            + _pick(self._tables.FAMILY_TAILS, stem, 7)
        )
        return f"{given} {family}"

    def _nation(self, faction: int, attempt: int) -> str:
        """Build one candidate name for one faction.

        The attempt number enters every draw, so a second attempt gives a
        whole new name rather than one letter of a different one.
        """
        stem = _mix(_stem(self._seed, _KIND_FACTION, faction) ^ (attempt & _MASK))
        return (
            _pick(self._tables.ONSETS, stem, 0)
            + _pick(self._tables.VOWELS, stem, 1)
            + _pick(self._tables.LINKS, stem, 2)
            + _pick(self._tables.NATION_TAILS, stem, 3)
        )
