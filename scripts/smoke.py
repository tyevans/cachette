#!/usr/bin/env python3
"""Exercise the installed package through its public interface.

This is the check that an installed artefact runs at all. It imports the
package by name, builds a world, steps it, and reads the results back. It
never imports from the source tree, so it fails when a wheel or a source
distribution is built wrongly, and it passes only when the thing a user
installs works.

It lives in a file rather than inline in the workflow. A snippet inside a
workflow is a second declaration site for the public interface, and nobody
greps a YAML file when they rename a constructor argument. That is how this
script came to exist: a constructor changed, the whole tree was searched, and
the workflow was missed because the search ran over the source tree only.

Run it against the current environment with `just smoke`. Continuous
integration runs it against a wheel and against a source distribution.

Exit 0 when the package works, 1 otherwise.
"""

from __future__ import annotations

import sys


def main() -> int:
    import cachette

    world = cachette.World(width=8, height=8, seed=1, faction_count=2)

    if world.tile_count != 64:
        print(f"tile count is {world.tile_count}, expected 64", file=sys.stderr)
        return 1
    if (world.width, world.height) != (8, 8):
        print(f"extent is {world.width}x{world.height}, expected 8x8", file=sys.stderr)
        return 1
    if world.tick != 0:
        print(f"a new world is at tick {world.tick}, expected 0", file=sys.stderr)
        return 1
    if not world.check_invariants():
        print("a new world does not hold its invariants", file=sys.stderr)
        return 1

    before = world.state_hash()
    world.step(threads=2)

    if world.tick != 1:
        print(f"one step gave tick {world.tick}, expected 1", file=sys.stderr)
        return 1
    if world.state_hash() == before:
        print("one step did not change the state hash", file=sys.stderr)
        return 1
    if not world.check_invariants():
        print("the world does not hold its invariants after a step", file=sys.stderr)
        return 1
    if len(world.tile_values()) != world.tile_count:
        print("the tile column length does not match the tile count", file=sys.stderr)
        return 1

    print(f"cachette {cachette.__version__} ok: {world!r}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
