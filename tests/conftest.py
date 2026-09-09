"""Shared fixtures, and the guard that keeps the tests black-box.

The tests import the installed package. They never import a source path.
An installed package is what a user gets, so a test against it is a test
of the product. A test against the source tree can pass while the wheel is
broken.

This file also hands a learner test the shapes it draws its input from. The
lengths come from the engine schema, and the distribution comes from what the
world was measured to hold. A module beside this one builds them.[^1]

References
----------
Testing policy. ``docs/TESTING.md``

[^1]: The learner shapes. ``tests/learner_shapes.py``
"""

from __future__ import annotations

import pathlib

import pytest

from learner_shapes import (
    EngineShapes,
    WorldShapedInputs,
    engine_shapes_of,
    engine_world,
    world_shaped_stack,
)


def pytest_sessionstart(session: pytest.Session) -> None:
    """Fail the session when the tests import the source tree."""
    del session
    import cachette

    module_file = getattr(cachette, "__file__", None)
    if module_file is None:
        message = "the package cachette has no file"
        raise RuntimeError(message)

    location = pathlib.Path(module_file).resolve()
    repository = pathlib.Path(__file__).resolve().parent.parent
    source = repository / "python" / "cachette" / "__init__.py"

    # An editable install points at the source directory on purpose. That
    # is still an install. What must never happen is an import that finds
    # the source because the working directory is on the path.
    if location == source and "" in _import_paths():
        message = (
            "the tests imported cachette from the source tree. "
            "Install the package first: just build"
        )
        raise RuntimeError(message)


def _import_paths() -> list[str]:
    import sys

    return list(sys.path)


@pytest.fixture
def seed() -> int:
    """Return the seed that every test uses unless it needs its own."""
    return 0x0123456789ABCDEF


@pytest.fixture(scope="session")
def engine_shapes() -> EngineShapes:
    """Build one world and return the two lengths it publishes."""
    return engine_shapes_of(engine_world())


@pytest.fixture
def world_shaped_inputs(engine_shapes: EngineShapes) -> WorldShapedInputs:
    """Return world-shaped stacks over the lengths the engine publishes."""
    return world_shaped_stack(
        engine_shapes.action_length, engine_shapes.observation_length
    )
