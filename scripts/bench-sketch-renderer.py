"""Time the sketch renderer at the demonstration size, in the three cases.

The three cases are the ones a watcher meets: a steady view, a drag with the
left button that pans the camera, and a drag with the right button that turns
and leans the view.
"""

import statistics
import sys
import time

from cachette import Camera, World
from cachette.demo.sketch import Sketch
from cachette.demo.surface import Surface

SIDE = 256
WIDTH = 1280
HEIGHT = 900
SEED = 0x0123_4567_89AB_CDEF
RUNS = 8


def build():
    world = World(width=SIDE, height=SIDE, seed=SEED, faction_count=4)
    world.seed_world()
    for _ in range(8):
        world.step(1)
    return world, Camera.fitting(world, WIDTH, HEIGHT)


def main() -> None:
    which = sys.argv[1] if len(sys.argv) > 1 else "cpu"
    world, camera = build()
    surface = Surface(WIDTH, HEIGHT)
    if which == "gpu":
        from cachette.demo.sketch_gl import GlSketch

        renderer = GlSketch(world)
    else:
        renderer = Sketch(world)

    def timed(prepare):
        run = []
        for step in range(RUNS):
            at = prepare(step)
            start = time.perf_counter()
            renderer(at, WIDTH, HEIGHT, surface.pixels)
            run.append((time.perf_counter() - start) * 1000.0)
        return run

    renderer(camera, WIDTH, HEIGHT, surface.pixels)
    cases = {"steady": timed(lambda step: camera)}

    def panned(step):
        moved = Camera.fitting(world, WIDTH, HEIGHT)
        moved.pan(float(9 * (step + 1)), float(7 * (step + 1)))
        return moved

    cases["left drag"] = timed(panned)

    opening = (renderer.view.turn, renderer.view.lean)

    def orbited(step):
        renderer.view.turn = opening[0] + 0.035 * (step + 1)
        renderer.view.lean = opening[1] - 0.012 * (step + 1)
        return camera

    cases["right drag"] = timed(orbited)
    renderer.view.turn, renderer.view.lean = opening

    for name, run in cases.items():
        print(
            f"{which:3} {name:11} median {statistics.median(run):7.1f} ms   "
            f"min {min(run):7.1f}   max {max(run):7.1f}"
        )


if __name__ == "__main__":
    main()
