"""The agent-facing tools of this repository.

This package holds a Model Context Protocol server over the engine's control
plane. It lets an agent that works on this repository run the engine instead
of reading the source and guessing.

Start the server with ``python -m cachette.agent``.

The package is a contributor tool. The protocol library is a development
dependency, so an installed wheel does not carry it.
"""
