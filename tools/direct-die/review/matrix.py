"""The state of every asset in every style, and the words for a wait.

A run takes about seventy seconds for one round of four variants, and a set
is eleven assets. A person therefore spends most of the time looking at work
that is part way through. The unfinished state is the interface, and not a
degraded form of it.

This module computes what one cell of the matrix holds now. A cell is one
style and one engine asset name. It reports how far the work went, whether
something draws it at this moment, and what the tool waits for, in words. A
page renders that without waiting for anything.

## The ladder of one cell

    nothing    no session drew this asset
    started    a session exists and holds no drawing yet
    drawn      a drawing exists and no person chose one
    chosen     a person picked a drawing
    promoted   the chosen drawing is an exemplar of the style
    exported   the pack on disk holds this asset

The ladder says how far the work went. It is not the same question as
whether something runs now, so a cell reports that separately. A cell can be
exported and running again in the same moment.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

import exemplars as exemplar_module
import packs as pack_module
import slugs as slug_table
from runs import Item, Job, RunManager
from store import Session, SessionStore

# The ladder of a cell, from the least work to the most.
LADDER = ("nothing", "started", "drawn", "chosen", "promoted", "exported")


def waiting_words(session: Session | None, rounds: int, variants: int) -> str:
    """Say what the tool is waiting for, in words a person can act on.

    A progress bar with no subject does not tell a person whether to wait.
    This names the round and the variant, so they can decide.
    """
    if session is None or not session.rounds:
        return "waiting for the first round"
    entry = session.rounds[-1]
    place = f"round {entry.number + 1}"
    if rounds:
        place = f"round {entry.number + 1} of {rounds}"
    present = entry.present_variants
    if not present:
        return f"{place}, variant a drawing"
    last = present[-1]
    if last.critique is None:
        return f"{place}, the critique of variant {last.letter} is coming"
    if variants and len(present) < variants:
        letters = [item.letter for item in entry.variants]
        following = letters[len(present)] if len(present) < len(letters) else "?"
        return f"{place}, variant {following} drawing"
    return f"{place} is complete, the next round is starting"


@dataclass
class Cell:
    """One style and one engine asset name, as they stand now."""

    style: str
    slug: str
    kind: str
    sessions: list[Session]
    pick: pack_module.Pick | None
    promoted: bool
    exported: dict | None
    job: Job | None
    item: Item | None
    waiting: str

    @property
    def running(self) -> bool:
        """Report whether a job draws this asset at this moment."""
        return self.item is not None

    @property
    def state(self) -> str:
        """Give the rung of the ladder that this cell reached."""
        if self.exported is not None:
            return "exported"
        if self.promoted:
            return "promoted"
        if self.pick is not None and self.pick.chosen_by_person:
            return "chosen"
        if self.pick is not None:
            return "drawn"
        if self.sessions:
            return "started"
        return "nothing"

    @property
    def live_session(self) -> Session | None:
        """Give the session that a running job writes, if there is one."""
        if self.item is None:
            return None
        for session in self.sessions:
            if session.session_id == self.item.session_id:
                return session
        return None

    @property
    def round_link(self) -> str | None:
        """Give the address of the round that a person should look at.

        A running cell points at the round the tool writes now. A finished
        cell points at the round that holds the chosen drawing. A person
        reads the grid to find the next thing to steer, so the link goes to
        the thing they can act on.
        """
        live = self.live_session
        if live is not None and live.rounds:
            return f"/s/{self.style}/{live.session_id}/{live.rounds[-1].name}"
        if self.pick is not None:
            return f"/s/{self.style}/{self.pick.session_id}/{self.pick.round_name}"
        if self.sessions:
            return f"/s/{self.style}/{self.sessions[-1].session_id}"
        return None

    @property
    def thumbnail(self) -> str | None:
        """Give the address of the display render to show in the cell."""
        if self.pick is None or self.pick.png_name is None:
            return None
        return (
            f"/f/{self.style}/{self.pick.session_id}/"
            f"{self.pick.round_name}/{self.pick.png_name}"
        )


def build_cell(
    store: SessionStore,
    style: str,
    slug: str,
    sessions: list[Session],
    runner: RunManager,
    styleguide_root: Path,
    packs_root: Path,
    pack_table: dict | None = None,
) -> Cell:
    """Read the state of one cell from the disk."""
    found = pack_module.sessions_for(store, style, slug, sessions)
    pick = pack_module.pick_for(store, style, slug, sessions)
    assets = (
        pack_module.pack_assets(packs_root, style) if pack_table is None else pack_table
    )
    active = runner.running_item(style, slug)
    job, item = active if active is not None else (None, None)
    cell = Cell(
        style=style,
        slug=slug,
        kind=slug_table.KIND_OF.get(slug, "asset"),
        sessions=found,
        pick=pick,
        promoted=exemplar_module.has_exemplar(styleguide_root, style, slug),
        exported=assets.get(slug) if isinstance(assets.get(slug), dict) else None,
        job=job,
        item=item,
        waiting="",
    )
    if item is not None:
        cell.waiting = waiting_words(
            cell.live_session, job.rounds if job else 0, job.variants if job else 0
        )
    return cell


def build_matrix(
    store: SessionStore,
    styles: list[str],
    runner: RunManager,
    styleguide_root: Path,
    packs_root: Path,
) -> list[tuple[str, list[Cell]]]:
    """Build every cell of the grid, and give the rows with their asset name.

    The call reads the session tree once and passes it down, because the
    grid asks about every style and every asset name.
    """
    sessions = store.list_sessions()
    packs = {style: pack_module.pack_assets(packs_root, style) for style in styles}
    rows = []
    for slug in slug_table.SLUGS:
        cells = [
            build_cell(
                store,
                style,
                slug,
                sessions,
                runner,
                styleguide_root,
                packs_root,
                packs.get(style, {}),
            )
            for style in styles
        ]
        rows.append((slug, cells))
    return rows


def known_styles(styleguide_root: Path, store: SessionStore) -> list[str]:
    """List every style to show, from the guide and from the sessions.

    The guide names the styles that a person can run. The session tree can
    hold an older asset type that the guide no longer names. The grid shows
    both, so no work goes missing from the view.
    """
    from_guide = exemplar_module.styles(styleguide_root)
    from_disk = sorted({session.asset for session in store.list_sessions()})
    extra = [name for name in from_disk if name not in from_guide]
    return from_guide + extra
