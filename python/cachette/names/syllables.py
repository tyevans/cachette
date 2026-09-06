"""The word parts a namer builds a name from.

**This module holds data. It holds no logic.** A game developer who wants
another flavour replaces the tables and changes nothing else. The namer reads
each table by name and assumes no length.

**Every word part here is original to this project.** Nobody copied a list
from another game, from a gazetteer or from a census, so the project carries
no third-party attribution for a name and a fork inherits no notice. The parts
are invented syllables that sound like a language. A name therefore resembles
no real nation, no real town and no real person.

A table holds at least one entry. A longer table gives more names and fewer
repeats. The namer multiplies the tables together, so four more entries in one
table add thousands of names.

References
----------
The project licence. ``LICENSE-MIT`` and ``LICENSE-APACHE``
"""

from __future__ import annotations

# The sound a word starts with.
ONSETS: tuple[str, ...] = (
    "B",
    "Br",
    "C",
    "Cal",
    "Ch",
    "D",
    "Dr",
    "F",
    "Fen",
    "G",
    "Gr",
    "H",
    "J",
    "K",
    "Kor",
    "L",
    "M",
    "Mar",
    "N",
    "P",
    "Pr",
    "R",
    "S",
    "Sk",
    "Sh",
    "St",
    "T",
    "Th",
    "Tor",
    "V",
    "Ver",
    "W",
    "Y",
    "Z",
)

# The vowel that follows the first sound.
#
# A simple vowel is repeated and a double vowel is not, so a name takes a
# double vowel about one time in four. A table that gave each the same chance
# put a double vowel next to a heavy joining sound too often, and the result
# was a name a reader could not say. The repeats hold the weight, so no second
# table of weights exists to disagree with this one.
VOWELS: tuple[str, ...] = (
    "a",
    "a",
    "a",
    "e",
    "e",
    "e",
    "i",
    "i",
    "i",
    "o",
    "o",
    "o",
    "u",
    "u",
    "y",
    "ae",
    "ei",
    "ia",
    "au",
    "oa",
)

# The sound that joins a word to an ending that starts with a vowel.
#
# The vowel of the ending pulls the whole sound onto the next syllable, so a
# cluster that only works at the start of a word belongs here. Read the parts
# of "Poagrede" as "Po-gre-de".
LINKS: tuple[str, ...] = (
    "lm",
    "nd",
    "rk",
    "sk",
    "st",
    "th",
    "rn",
    "ld",
    "mb",
    "nt",
    "rv",
    "sh",
    "gr",
    "dr",
    "tr",
    "ss",
    "ll",
    "rr",
    "n",
    "r",
    "l",
    "m",
    "v",
    "k",
)

# The sound that joins a word to an ending that starts with a consonant.
#
# **These are not the sounds above.** A place name and a family name join two
# whole words, so the joining sound has to end a syllable on its own. A start
# of word cluster cannot: "Skae" and "fell" joined by "dr" gives "Skaedrfell",
# which no reader can say.
CODAS: tuple[str, ...] = (
    "n",
    "r",
    "l",
    "m",
    "d",
    "th",
    "sh",
    "ss",
    "st",
    "ld",
    "nd",
    "rn",
    "rd",
    "ll",
    "lm",
    "ng",
    "rk",
    "lk",
    "rth",
    "nt",
    "rt",
    "ck",
    "rl",
)

# The sound a nation name ends with.
NATION_TAILS: tuple[str, ...] = (
    "ia",
    "esh",
    "arn",
    "oth",
    "une",
    "ath",
    "iel",
    "ov",
    "ara",
    "eim",
    "ur",
    "yn",
    "and",
    "or",
    "ess",
    "ai",
    "ium",
    "ede",
    "ost",
    "arth",
    "ige",
    "ola",
)

# The sound a place name ends with.
PLACE_TAILS: tuple[str, ...] = (
    "ford",
    "holm",
    "wick",
    "gate",
    "mere",
    "cliff",
    "bury",
    "stead",
    "hollow",
    "reach",
    "march",
    "fell",
    "combe",
    "dale",
    "haven",
    "crest",
    "moor",
    "row",
    "stone",
    "brook",
    "ridge",
    "hearth",
    "watch",
    "barrow",
    "fen",
    "gap",
    "hold",
    "vale",
)

# The word that stands before a place name, and the empty text for a place
# that takes none.
#
# The empty entry is repeated, so most places take no word and the few that
# do stand out. A table of weights beside a table of words would be a second
# declaration site, and this needs none.
PLACE_WORDS: tuple[str, ...] = (
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "New",
    "Old",
    "Upper",
    "Lower",
    "East",
    "West",
    "North",
    "South",
    "Little",
    "Great",
)

# The sound a given name ends with. Each one starts with a vowel, because the
# joining sound before it may be a start of word cluster.
GIVEN_TAILS: tuple[str, ...] = (
    "in",
    "us",
    "el",
    "ar",
    "ic",
    "and",
    "ien",
    "os",
    "ea",
    "ith",
    "or",
    "an",
    "ard",
    "ell",
    "is",
    "ay",
    "un",
    "eth",
    "ia",
    "on",
    "ek",
    "ael",
    "ana",
    "ine",
)

# The sound a family name ends with.
FAMILY_TAILS: tuple[str, ...] = (
    "son",
    "wold",
    "mark",
    "by",
    "strand",
    "hall",
    "vane",
    "crest",
    "ford",
    "hart",
    "bane",
    "well",
    "thorn",
    "gard",
    "rike",
    "stow",
    "wain",
    "drake",
    "moor",
    "fast",
)

# The ending an adjective takes when the nation name ends in a consonant.
CONSONANT_ADJECTIVES: tuple[str, ...] = ("ian", "ish", "ene", "ic")

# The letters the namer reads as a vowel when it builds an adjective.
VOWEL_LETTERS = "aeiouy"
