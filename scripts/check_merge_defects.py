#!/usr/bin/env python3
"""Check a change for the defects that a hand-resolved merge produces.

A merge conflict in a register is resolved by choosing between two sides. Each
side is a correct file and the merged result is not, because the merged result
answers a question neither side was asked. Four defects come out of that, and
one dispatcher produced four of these by hand in one day. The fifth was
produced by the merge that brought this check onto its branch, which is the
kind of confirmation nobody has to arrange.

  moved      the change moves or deletes a file and something still names the
             old path
  duplicate  a document defines one footnote label twice, because both sides
             defined it and the resolution kept both
  collision  a register names one number twice, because both sides allocated
             it
  pointer    a register's next-number line disagrees with its own entries,
             because both sides were behind and the resolution took a side
  repeated   a priority index lists one number twice, because a merge kept
             both sides of a conflicted table

**The gate already catches all four.** It catches them in minutes, over the
whole tree, after the commit exists.[^1] [^2] This check asks the same
questions of the staged change in about a second, so that the answer arrives
before the commit rather than after it. The value here is latency, not
coverage.

**Four of the five rules are not restated here.** The duplicate label comes
from the footnote check, the two register rules come from the register check,
and the repeated row comes from the priority check. All are imported and
called.[^1] [^2] [^5] A second copy of a rule is the defect this project keeps
recording: nothing fails when the copies disagree.[^3] Only the moved-path
rule is new, because no existing check ties a move to the citations of the
path it moved from.

**The moved-path rule works during a merge, which is the case that bit.** The
index compares against the first parent, so a file that the *other* branch
moved appears as a rename in the staged change. That is what makes a staged
diff the right input: the merge that introduces the defect is exactly the
moment the check can see it.

A move that nothing cites costs nothing. The check runs no search when the
change moves no file.

**There is no allow-list.** A document that names a moved path on purpose will
fail this check. That has not happened yet, and an escape written before a
real instance would be a capability nobody invokes.[^4] Add one when a real
document needs it, and let that document be the evidence.

Modes:

  (no argument)   check the staged change, against HEAD. This is the hook.
  --since REF     check the change from REF to the working tree.
  --branch        the same, against the merge base with the default branch.
                  This is the gate, which runs with nothing staged.
  --root PATH     read that repository rather than this one. It comes first,
                  and it exists so that a probe can build a history and drive
                  the real command over it.[^6]

**The hook cannot be the whole answer, and not only because it is
bypassable.** Git runs the pre-commit hook when a merge stops and a person
commits it, which is the case that produced all four defects. It does not run
the hook for a merge that applies cleanly and commits itself. The gate mode
covers that, and the gate is the enforcement.

Exit 0 when the change carries none of the four, 1 otherwise. No dependencies
beyond the standard library and git.

# References

[^1]: The footnote check. `scripts/check_footnotes.py`
[^2]: The register check. `scripts/check_registers.py`
[^3]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^4]: Recurring defect shapes, shape 3. `.claude/rules/recurring-defects.md`
[^5]: The priority check. `scripts/check_priority.py`
[^6]: The probe. `scripts/merge-defect-probe.sh`
"""

import re
import subprocess
import sys
from pathlib import Path

# The repository the check reads. `--root` moves it, which is the only way to
# drive this check over a history built for a test: its input is a change, not
# a directory, so a fixture directory cannot express it. The probe uses it.
ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(Path(__file__).resolve().parent))

import check_footnotes  # noqa: E402
import check_priority  # noqa: E402
import check_registers  # noqa: E402

TESTS = {
    "moved": "the change moves a file that something still names",
    "duplicate": "a document defines one footnote label twice",
    "collision": "a register names one number twice",
    "pointer": "a register's next number disagrees with its entries",
    "repeated": "a priority index lists one number twice",
}

INDEXES = (
    Path("docs") / "backlog" / "PRIORITY.md",
    Path("docs") / "adrs" / "PRIORITY.md",
    Path("docs") / "product" / "PRIORITY.md",
)


def git(*args: str) -> str:
    """Run git in the repository and return its output.

    A non-zero exit is not an error here. `git grep` exits 1 when it matches
    nothing, and that is the common case.
    """
    done = subprocess.run(
        ("git", "-C", str(ROOT)) + args,
        capture_output=True,
        text=True,
        check=False,
    )
    return done.stdout


def default_base() -> str | None:
    """Return the merge base with the default branch, or None.

    A worktree may have no remote, and the default branch may be checked out,
    in which case the base is the head and the range is empty. Both are normal
    and neither is a failure: the check then reports no move rather than
    refusing to run.
    """
    for ref in ("origin/main", "origin/master", "main", "master"):
        base = git("merge-base", "HEAD", ref).strip()
        if base:
            return base
    return None


def changed(since: str | None) -> tuple[list[str], list[str]]:
    """Return the paths the change moves away from, and the files it leaves.

    The first list holds every path that will not exist after the change: the
    old side of a rename, and a deletion. The second holds every Markdown file
    the change writes, which is what the footnote rule reads.

    Rename detection is on. Without it a rename reads as a delete and an add,
    and the delete alone would still name the old path, so the rule would
    still fire. Detection is on so that the message can say "moves" rather
    than "deletes", which is what a reader needs in order to fix it.
    """
    if since is None:
        raw = git("diff", "--cached", "--name-status", "--find-renames", "-z")
    else:
        raw = git("diff", since, "--name-status", "--find-renames", "-z")

    fields = raw.split("\0")
    gone: list[str] = []
    written: list[str] = []
    index = 0
    while index < len(fields):
        status = fields[index]
        if not status:
            break
        # A rename and a copy carry two paths. Everything else carries one.
        if status[0] in ("R", "C"):
            old, new = fields[index + 1], fields[index + 2]
            index += 3
            if status[0] == "R":
                gone.append(old)
            written.append(new)
        else:
            path = fields[index + 1]
            index += 2
            if status[0] == "D":
                gone.append(path)
            else:
                written.append(path)
    return gone, [path for path in written if path.endswith(".md")]


# A path is worth searching for only when a citation could hold it. A bare
# number is a substring of ordinary text, so a search for it reports every line
# that happens to contain it. One such name reached this check: a file called
# `0`, whose search returned 14571 lines and no true finding.
#
# The first form of this guard asked for a directory separator or a file
# extension. That rejected the bare number and it also rejected `justfile`,
# which is a distinctive name that documents cite, so a move of it was reported
# as nothing at all. The guard now asks the narrower question the comment above
# always stated: can ordinary prose hold this token? A number can. A word
# cannot.
def citation_shaped(path: str) -> bool:
    """Say whether a moved path is distinctive enough to search for."""
    return not path.rsplit("/", 1)[-1].isdigit()


# A path names a file when it stands on its own. These characters continue a
# path or a word, so a match with one of them on either side is a longer name
# that merely contains the moved one.
CONTINUES = re.compile(r"[A-Za-z0-9_./-]")


def ends_the_sentence(text: str, end: int) -> bool:
    """Say whether the character at `end` is a full stop rather than a path.

    A full stop continues a path in `docs/a.md.bak` and ends a sentence in
    `see docs/a.md.` The two are told apart by what follows the stop: a path
    character continues the name, and anything else closes it. Without this,
    prose that ends a sentence with a bare path is invisible to the rule, and
    prose that names a path outside a code span is the prose most likely to
    go stale.
    """
    if end >= len(text) or text[end] != ".":
        return False
    after = text[end + 1] if end + 1 < len(text) else " "
    return not CONTINUES.match(after)


def names_path(text: str, path: str) -> bool:
    """Say whether the text names the path, rather than containing it."""
    start = text.find(path)
    while start != -1:
        end = start + len(path)
        before = text[start - 1] if start else " "
        after = text[end] if end < len(text) else " "
        closes = not CONTINUES.match(after) or ends_the_sentence(text, end)
        if not CONTINUES.match(before) and closes:
            return True
        start = text.find(path, start + 1)
    return False


def stale_paths(gone: list[str], since: str | None) -> list[tuple[str, str, str]]:
    """Return every place that still names a path the change moved away from.

    One search covers every moved path, because a process for each path is the
    cost that would make this check too slow to keep.
    """
    searched = [path for path in gone if citation_shaped(path)]
    if not searched:
        return []
    patterns: list[str] = []
    for path in searched:
        patterns += ["-e", path]
    # The staged mode reads the index, because the index is what the commit
    # will hold. The range mode reads the working tree.
    where = ["--cached"] if since is None else []
    raw = git("grep", "-F", "-n", "--no-color", *where, *patterns)

    out: list[tuple[str, str, str]] = []
    for line in raw.splitlines():
        parts = line.split(":", 2)
        if len(parts) != 3:
            continue
        where_file, number, text = parts
        named = next((path for path in searched if names_path(text, path)), None)
        if named is None:
            continue
        # A file that names its own path is naming itself, not a lost file.
        if where_file == named:
            continue
        out.append((where_file, "moved", f"line {number} still names {named}"))
    return out


def duplicate_labels(written: list[str]) -> list[tuple[str, str, str]]:
    """Return every footnote label that a written document defines twice.

    The rule is the footnote check's, and this calls it rather than restating
    it.
    """
    out: list[tuple[str, str, str]] = []
    for name in written:
        path = ROOT / name
        if not path.is_file():
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except (UnicodeDecodeError, OSError):
            continue
        document = check_footnotes.Document(path, text)
        for label, line in document.repeats:
            out.append((name, "duplicate", f"line {line} defines [^{label}] again"))
    return out


def register_defects() -> list[tuple[str, str, str]]:
    """Return the numbering defects of the three registers.

    The rule is the register check's, and this calls it. Both the collision
    and the pointer come from that one call. The registers are three files, so
    this runs whether or not the change touches them: a change that moves a
    register entry without touching the register is not expressible, but a
    change that lands beside a broken register still should not be committed.
    """
    out: list[tuple[str, str, str]] = []
    for prefix, path in check_registers.REGISTERS:
        # The register list is the register check's, and its paths are rooted
        # at the real repository. Re-root them rather than restating the list.
        path = ROOT / path.relative_to(check_registers.ROOT)
        if not path.is_file():
            continue
        for failure in check_registers.check(prefix, path):
            test = "pointer" if "next number" in failure else "collision"
            out.append((path.name, test, failure.split(": ", 1)[-1]))
    return out


def repeated_rows() -> list[tuple[str, str, str]]:
    """Return every priority index row that names a number a second time.

    A merge that keeps both sides of a conflicted index produces this, and it
    is the same shape as a register that names one number twice. The rule is
    the priority check's and this calls it.

    The three indexes are three files, so this runs whether or not the change
    touches them.
    """
    out: list[tuple[str, str, str]] = []
    for name in INDEXES:
        path = ROOT / name
        if not path.is_file():
            continue
        for number in check_priority.repeated(check_priority.listed(path)):
            out.append((str(name), "repeated", f"{number} is listed twice"))
    return out


def main() -> int:
    global ROOT
    argv = sys.argv[1:]
    if len(argv) >= 2 and argv[0] == "--root":
        ROOT = Path(argv[1]).resolve()
        argv = argv[2:]
    since: str | None = None
    if argv and argv[0] == "--since":
        if len(argv) < 2:
            print("--since needs a revision", file=sys.stderr)
            return 2
        since = argv[1]
    elif argv and argv[0] == "--branch":
        since = default_base()
        if since is None:
            print(
                "no default branch to compare against, so no move is checked",
                file=sys.stderr,
            )
    elif argv:
        print(f"unknown argument: {argv[0]}", file=sys.stderr)
        return 2

    gone, written = changed(since)
    findings = (
        stale_paths(gone, since)
        + duplicate_labels(written)
        + register_defects()
        + repeated_rows()
    )

    for where, test, detail in findings:
        print(f"FAIL: {where}: {TESTS[test]}. {detail}", file=sys.stderr)

    scope = "the staged change" if since is None else f"the change since {since}"
    skipped = [path for path in gone if not citation_shaped(path)]
    for path in skipped:
        print(
            f"note: {path} moved, and the check did not search for it. "
            "A name with no directory and no extension matches ordinary text",
            file=sys.stderr,
        )
    print(
        f"\nchecked {scope}: {len(gone)} moved, {len(skipped)} not searched, "
        f"{len(written)} documents written, {len(findings)} failures"
    )
    return 1 if findings else 0


if __name__ == "__main__":
    raise SystemExit(main())
