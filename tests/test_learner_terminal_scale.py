"""Pin what the terminal weights do to the ranking of one generation.

The register holds the measurement, and these tests read it.

An evolution strategy ranks the candidates of one generation and moves the
centre along the ranking. The score of a candidate is the mean over its
seeds, and every play style of the table except one pays 100 for a win and
minus 100 for a loss. **One seed that changes from a loss to a win therefore
moves a candidate mean by 200 over the seed count.** The engine names a
winner at the tick limit, so an episode that runs out of ticks still ends won
or lost and no episode is drawn.

An audit argued from the weights alone that this quantum is of the same order
as the whole quantity the search compares. It marked the comparison
unverified, because it did not measure how far the shaped part of the score
separates the candidates of one generation.

A measurement answered it, and it confirmed the audit. The terminal part of
the score separates the candidates of one generation further than the shaped
part does, and one seed changing its outcome moves a candidate by more than
three quarters of the whole shaped separation. The register holds every
figure, and the commit holds the command that took them.[^1]

**The correction is not a figure a measurement answers.** Lowering the
terminal weights states what winning is worth against the shaped objectives,
and one blocker holds the rules of the downstream game.[^2] Raising the seed
count lowers the quantum in proportion, and it costs episodes.

**These tests therefore pin the measurement rather than a target.** A change
to the win weight, to the loss weight or to the seed count of the measured
generation fails them, and the failure asks for a new measurement rather than
letting the next reader inherit this one. A second copy of a measured figure
is the defect shape this project names first.[^3]

# References

[^1]: Reinforcement learning parameters, the spread of a generation.
`docs/reference/rl-costs.md`

[^2]: Blockers register, BLK-050. `docs/BLOCKERS.md`

[^3]: Recurring defect shapes, shape 1, redundant declaration sites.
`.agents/rules/recurring-defects.md`
"""

from __future__ import annotations

import re
from pathlib import Path

import pytest

from cachette.learn import load_library

ROOT = Path(__file__).resolve().parents[1]
REGISTER = ROOT / "docs" / "reference" / "rl-costs.md"

# The heading of the register section that holds the measurement.
SECTION = "## The spread of a generation, against the terminal quantum"

# The style the measurement played. Every style of the table except one pays
# the same win and the same loss, and one test below holds that.
MEASURED_STYLE = "aggressive"

# How close a figure derived here must come to the figure the register holds.
# The register states two decimal places, so this is the rounding and nothing
# more.
ROUNDING = 0.01


def register_figures() -> dict[str, float]:
    """Return the measured figures of the register section, by name."""
    text = REGISTER.read_text(encoding="utf-8")
    assert SECTION in text, f"the register holds no section named {SECTION!r}"
    section = text.split(SECTION, 1)[1].split("\n## ", 1)[0]
    figures: dict[str, float] = {}
    for line in section.splitlines():
        if not line.startswith("| `"):
            continue
        cells = [cell.strip() for cell in line.split("|")[1:-1]]
        name = re.fullmatch(r"`([a-z_]+)`", cells[0])
        if name is None:
            continue
        figures[name.group(1)] = float(cells[1])
    return figures


def test_the_register_holds_every_figure_these_tests_read() -> None:
    """A register row that goes missing must fail here and not read as zero."""
    figures = register_figures()

    assert set(figures) >= {
        "population",
        "seeds",
        "episodes",
        "won_episodes",
        "shaped_spread_range",
        "shaped_spread_deviation",
        "terminal_spread_deviation",
        "total_spread_deviation",
        "terminal_quantum",
        "shaped_range_over_quantum",
    }, f"the register section holds {sorted(figures)}"
    assert figures["episodes"] == figures["population"] * figures["seeds"]
    assert 0.0 < figures["won_episodes"] < figures["episodes"]


def test_the_terminal_quantum_is_the_one_the_style_table_states() -> None:
    """A change to the win or the loss weight fails, and asks for a measurement.

    **This is the guard the audit asked for.** The quantum comes from the
    style table and the seed count. The register holds the quantum that the
    measured spread was read against, so a weight that moves and a register
    that does not would leave the spread answering for a reward nobody plays.
    """
    figures = register_figures()
    style = load_library().style(MEASURED_STYLE)
    quantum = abs(style.won - style.lost) / figures["seeds"]

    assert quantum == pytest.approx(figures["terminal_quantum"], abs=ROUNDING), (
        f"the style table pays {style.won} for a win and {style.lost} for a "
        f"loss, which moves a candidate mean over {figures['seeds']:.0f} seeds "
        f"by {quantum:.2f}. The register records a quantum of "
        f"{figures['terminal_quantum']:.2f}, measured against a shaped spread "
        f"of {figures['shaped_spread_range']:.2f}. Take the measurement again "
        "under the new weights, and write the new rows into the register."
    )


def test_the_register_ratio_agrees_with_its_own_rows() -> None:
    """The register must not hold a ratio its own figures contradict."""
    figures = register_figures()
    ratio = figures["shaped_spread_range"] / figures["terminal_quantum"]

    assert ratio == pytest.approx(figures["shaped_range_over_quantum"], abs=ROUNDING)


def test_the_terminal_part_carried_more_spread_than_the_shaped_part() -> None:
    """The measurement confirmed the audit, and this states what it found.

    A reader who took the audit for refuted would leave the quantum out of
    every later decision about the seed count and the population. The
    inequality below is what the measurement says, and it fails when a later
    measurement overturns it, which is the moment somebody must read this
    file again.
    """
    figures = register_figures()

    assert figures["terminal_spread_deviation"] > figures["shaped_spread_deviation"]
    assert figures["terminal_quantum"] > figures["shaped_spread_deviation"]
    assert figures["shaped_range_over_quantum"] < 2.0


def test_the_style_that_pays_nothing_for_an_outcome_has_no_quantum() -> None:
    """One style of the table wants no win, and it quantises nothing.

    **A fixture of one style would measure the typical case.** This is the
    extreme of the table: a style whose terminal weights are all zero cannot
    quantise a generation at all, whatever the seed count.
    """
    hoarding = load_library().style("wealth_hoarding")

    assert hoarding.won == pytest.approx(0.0)
    assert hoarding.lost == pytest.approx(0.0)
    assert hoarding.drawn == pytest.approx(0.0)
    assert abs(hoarding.won - hoarding.lost) == pytest.approx(0.0)


def test_every_other_style_pays_the_same_win_and_loss() -> None:
    """One measurement answers for the table, because the table agrees.

    A style that paid another win would need its own row in the register, so
    this fails when one arrives rather than letting the measurement answer
    for a weight it never read.
    """
    library = load_library()
    paying = [
        library.style(name)
        for name in library.names
        if library.style(name).won != 0.0 or library.style(name).lost != 0.0
    ]
    assert paying, "no style of the table pays for an outcome"

    read = library.style(MEASURED_STYLE)
    for style in paying:
        assert style.won == pytest.approx(read.won), (
            f"the style {style.name} pays {style.won} for a win against "
            f"{read.won} for {MEASURED_STYLE}, so the register needs a row "
            "for it"
        )
        assert style.lost == pytest.approx(read.lost), (
            f"the style {style.name} pays {style.lost} for a loss against "
            f"{read.lost} for {MEASURED_STYLE}, so the register needs a row "
            "for it"
        )


def test_no_style_pays_for_a_draw_because_no_episode_draws() -> None:
    """The engine names a winner at the tick limit, so a draw never pays.

    The audit read one seed moving from a draw to a loss as a second, smaller
    quantum. There is no such move in the measured generation: every one of
    its episodes ended won or lost.
    """
    library = load_library()
    figures = register_figures()

    for name in library.names:
        assert library.style(name).drawn == pytest.approx(0.0)
    assert figures["won_episodes"] < figures["episodes"]
