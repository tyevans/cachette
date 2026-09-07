"""direct-die: a proof of concept that refines an SVG game asset.

The tool draws an asset as SVG, rasterises it at two sizes, shows both to
a vision model, and revises the drawing from the critique that comes
back. A living style guide holds the rules and the accepted exemplars.

This package is a side project. It touches no part of the engine.
"""

__all__ = ["client", "render", "guide", "session", "loop", "cli"]
