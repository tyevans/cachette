"""The sketchbook renderer, composited on the graphics device.

This renderer draws the same page as the array renderer. It takes the same
call, from the same place, and it gives back the same reading.[^1] The
difference is where the pixels are made.

**The array renderer builds one whole array for each stage of the page.** The
page is over a million points, it composites in three bands of a real number,
and the array library keeps no stage in a register, so a dozen chained stages
cost hundreds of megabytes of memory traffic for one frame. The device runs
the whole chain for one pixel at a time, many pixels at once, and writes the
result once.

What stays on the processor
---------------------------

**The geometry stays.** The turn of the world and the lift of the ground are a
scatter and a running maximum down each column of the page, and neither is a
function of one pixel. They cost a small part of the build, they run when the
camera moves, and they are not the reason a frame is slow.

**The marks move.** The tone of the light, every set of hatching, the
silhouette, the grain of the paper, the wash of a faction, the cloud, the wash
of an overlay and the fit of the page into the frame are all functions of one
pixel. They are the whole cost, and they are what a fragment shader is for.

What this renderer promises
---------------------------

**It draws what the array renderer draws.** The arithmetic is the same
arithmetic and the constants are the same constants: this module takes every
number from the module that declares it and puts it into the shader, so the
two cannot drift apart on a value. A test draws one frame each way and holds
the difference under a stated bound.

**It writes nothing to the world.** It reads the fields the engine publishes
and it fills the pixels the caller owns. It touches no simulated value, no
state hash and no step.

References
----------
ADR-0094, the caller owns the camera and the pixels, decision D1.
``docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md``

ADR-0067, the viewer reads the world and never writes to it, decision D3.
``docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md``
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

import numpy as np

from cachette.demo import sketch as ink
from cachette.demo import sketch_shader as source
from cachette.demo.glpage import Device, DeviceGap
from cachette.demo.sketch import Ground, Sketch

if TYPE_CHECKING:
    from collections.abc import Sequence

    import numpy.typing as npt

    from cachette import World
    from cachette._core import Camera, FrameReading

__all__ = ["DeviceGap", "GlSketch"]

# How many faction colours the shader holds room for.
#
# **The engine publishes the colours and this only bounds them.** A run with
# more factions than this is refused rather than drawn with the wrong colours,
# because a picture that names the wrong holder is worse than no picture.
PALETTE_ROOM = 64

# The two reaches of the blur that finds the rim of a wash, in page points.
# They are the reaches the array renderer uses, and one test holds them equal.
NEAR_REACH = 4
FAR_REACH = 14

# The bits of the flag byte the page carries for each of its points.
DRAWN_BIT = 1
CLIFF_BIT = 2
WATER_BIT = 4


def _defines() -> str:
    """Give back every constant of the page, as a shader definition.

    **Each number is taken from the module that declares it.** A number typed
    again in the shader would be a second declaration site, and nothing would
    fail when the two disagreed.
    """
    numbers = {
        "CONTOUR_STEP": ink.CONTOUR_STEP,
        "CONTOUR_WEIGHT": ink.CONTOUR_WEIGHT,
        "HATCH_SPACING": ink.HATCH_SPACING,
        "CLIFF_SPACING": ink.CLIFF_SPACING,
        "WATER_SPACING": ink.WATER_SPACING,
        "CLOUD_SPACING": ink.CLOUD_SPACING,
        "SHADOW_START": ink.SHADOW_START,
        "CROSS_START": ink.CROSS_START,
        "CLOUD_FLOOR": ink.CLOUD_FLOOR,
        "CLOUD_SHADOW_DEPTH": ink.CLOUD_SHADOW_DEPTH,
        "HOLDER_WASH": ink.HOLDER_WASH,
        "WASH_GAIN": ink.WASH_GAIN,
        "GRANULATION": ink.GRANULATION,
        "RIM_GAIN": ink.RIM_GAIN,
        "GLAZE_LIFT": ink.GLAZE_LIFT,
    }
    colours = {"PAPER": ink.PAPER, "INK": ink.INK, "SKY_INK": ink.SKY_INK}
    whole = {
        "PALETTE_ROOM": PALETTE_ROOM,
        "NEAR_REACH": NEAR_REACH,
        "FAR_REACH": FAR_REACH,
    }
    lines = [f"#define {name} {value!r}" for name, value in whole.items()]
    lines += [f"const float {name} = {float(value)!r};" for name, value in numbers.items()]
    lines += [
        f"const vec3 {name} = vec3({float(band[0])!r}, {float(band[1])!r}, "
        f"{float(band[2])!r});"
        for name, band in colours.items()
    ]
    return "\n".join(lines) + "\n"


class GlSketch(Sketch):
    """The sketchbook renderer, composited on the graphics device.

    A caller builds one of these exactly as it builds the array renderer, and
    calls it exactly as it calls the drawing method of the world.

    **The device opens on the first frame, not here.** The demonstration builds
    the renderer before it opens its window, and a context opened here would be
    a second context that the window never draws in.

    Raises ``DeviceGap`` on the first frame when the machine gives no graphics
    context. It never falls back to the array renderer on its own: a renderer
    that quietly draws on the processor would report the speed of the processor
    while everyone believed the device was drawing.
    """

    # The build stops at the geometry. This renderer draws every mark itself.
    _marks = False

    __slots__ = ("_device", "_pages", "_palette", "_stamp")

    def __init__(
        self,
        world: World,
        *,
        relief: float = ink.RELIEF,
        lean: float = ink.LEAN,
        sky: bool = True,
    ) -> None:
        """Build the renderer for one world."""
        super().__init__(world, relief=relief, lean=lean, sky=sky)
        self._device: Device | None = None
        # What the page last sent to the device, so that a frame that reuses a
        # page does not send it again.
        self._stamp: tuple[Any, ...] | None = None
        self._pages: dict[str, Any] = {}
        self._palette: np.ndarray | None = None

    # ------------------------------------------------------------------
    # The device

    @property
    def device(self) -> Device:
        """Give back the device, opening it on the first ask."""
        if self._device is None:
            self._device = Device()
        return self._device

    def close(self) -> None:
        """Give the graphics context back."""
        if self._device is not None:
            self._device.close()
            self._device = None
            self._stamp = None

    def _program(self, name: str, body: str) -> Any:
        """Build one program from the shared head and this body."""
        return self.device.program(
            name, source.HEAD + _defines() + source.PAGE + source.TILES + body
        )

    # ------------------------------------------------------------------
    # What crosses to the device

    def _send_page(self, ground: Ground) -> None:
        """Put the page the build made on the device, when it is a new page.

        The turn and the lift are a function of the camera and the size of the
        frame, so this runs when the camera moves and not on every frame.
        """
        stamp = (ground.window, ground.shape, id(ground))
        if self._stamp == stamp:
            return
        device = self.device
        drawn = ground.drawn
        flags = (
            drawn.astype(np.uint8) * DRAWN_BIT
            + (ground.cliff & drawn).astype(np.uint8) * CLIFF_BIT
            + ground.water.astype(np.uint8) * WATER_BIT
        )
        device.upload("page_held", ground.held_height, "r32f")
        device.upload("page_depth", ground.depth, "r32f")
        device.upload("page_flags", flags, "r8ui")
        device.upload("page_take", ground.take.astype(np.int32), "r32i")
        rows, columns = drawn.shape
        device.upload("page_grain", self._paper_grain(rows, columns), "r32f")
        self._stamp = stamp

    def _send_tiles(self, ground: Ground) -> None:
        """Put the fields that change with the world on the device.

        **These cross at the size of the window of tiles, not of the page.** A
        tile covers many points of the page, so the page reads a tile many
        times and the tile crosses once.
        """
        from cachette import faction_colours

        device = self.device
        first_q, first_r, last_q, last_r = ground.window
        rows, columns = self._world.height, self._world.width

        if self._palette is None:
            self._palette = np.array(
                [
                    [(c >> 16) & 0xFF, (c >> 8) & 0xFF, c & 0xFF]
                    for c in faction_colours()
                ],
                dtype=np.float32,
            )
            if not len(self._palette):
                message = "the engine publishes no faction colour"
                raise DeviceGap(message)
            # The colours go across one row, so the shader reads one by its
            # number and this module holds no bound of its own on how many
            # colours the engine publishes.
            device.upload("palette", self._palette[None, :, :], "rgb32f")

        holders = self._world.tile_holders().reshape(rows, columns)
        device.upload(
            "tile_holder",
            holders[first_r:last_r, first_q:last_q].astype(np.int32),
            "r32i",
        )

        if self._sky:
            cloud = self._world.cloud_shares().reshape(rows, columns).astype(
                np.float32
            ) / max(self._whole_sky, 1.0)
            winds = self._world.tile_winds()
            wind_q = winds["q"].reshape(rows, columns).astype(np.float32)
            wind_r = winds["r"].reshape(rows, columns).astype(np.float32)
            # The wind stands on the axes of the hex grid, and the page turns
            # those axes, so the wind turns with them.
            plan_x = wind_q + wind_r * 0.5
            plan_y = wind_r * ink.ROW_PITCH
            page_dx = plan_x - plan_y
            page_dy = (plan_x + plan_y) * self._lean
            length = np.hypot(page_dx, page_dy)
            moving = length > 0.0
            page_dx = np.where(moving, page_dx / np.where(moving, length, 1.0), 1.0)
            page_dy = np.where(moving, page_dy / np.where(moving, length, 1.0), 0.0)
            cut = (slice(first_r, last_r), slice(first_q, last_q))
            device.upload("tile_cloud", cloud[cut], "r32f")
            device.upload("tile_across_x", page_dy[cut].astype(np.float32), "r32f")
            device.upload("tile_across_y", (-page_dx)[cut].astype(np.float32), "r32f")
        else:
            # The shader binds these whether it reads them or not.
            for name in ("tile_cloud", "tile_across_x", "tile_across_y"):
                device.ensure(name, "r32f")

    def _send_wash(
        self,
        ground: Ground,
        overlay: str | None,
        camera: Camera,
        width: int,
        height: int,
    ) -> bool:
        """Put the pigment a named overlay laid down on the device.

        **The colours are the engine's own.** This asks the engine for the
        frame with the overlay and for the frame without it, and the difference
        between the two is the pigment. It then reads that difference at the
        place the engine drew each tile, so the wash follows the ground the
        page shows rather than the flat map the engine drew.

        Gives back whether there is any pigment at all.
        """
        device = self.device
        device.ensure("tile_hue", "rgb32f", bands=3)
        device.ensure("tile_flow", "r32f")
        if not overlay:
            return False
        bare = self._buffer("bare", width, height)
        tinted = self._buffer("tinted", width, height)
        self._world.draw(camera, width, height, bare)
        self._world.draw(camera, width, height, tinted, overlay=overlay)
        plain = ink._channels(bare, width, height)
        painted = ink._channels(tinted, width, height)
        flow = self._sampled(
            ground, np.abs(painted - plain).max(axis=-1) / 255.0, camera, width, height
        )
        if not flow.any():
            return False
        hue = np.stack(
            [
                self._sampled(ground, painted[..., band], camera, width, height)
                for band in range(3)
            ],
            axis=-1,
        )
        device.upload("tile_flow", flow.astype(np.float32), "r32f")
        device.upload("tile_hue", hue.astype(np.float32), "rgb32f")
        return True

    def _sampled(
        self, ground: Ground, frame: np.ndarray, camera: Camera, width: int, height: int
    ) -> np.ndarray:
        """Read a field the engine painted on the flat map, tile by tile.

        This is the reading the array renderer makes, stopped one step earlier.
        The array renderer goes on to spread the reading over the page. The
        shader spreads it instead, so this gives back the window of tiles.
        """
        first_q, first_r, last_q, last_r = ground.window
        across = last_q - first_q
        down = last_r - first_r
        if ground.sample is None:
            columns = np.arange(0, width, ink.SAMPLE_STEP)
            rows = np.arange(0, height, ink.SAMPLE_STEP)
            found_q = np.zeros((down, across), dtype=np.int64)
            found_y = np.zeros((down, across), dtype=np.int64)
            counted = np.zeros((down, across), dtype=np.int64)
            for y in rows:
                for x in columns:
                    tile_q, tile_r = camera.tile_at(float(x), float(y))
                    at_q = tile_q - first_q
                    at_r = tile_r - first_r
                    if 0 <= at_q < across and 0 <= at_r < down:
                        found_q[at_r, at_q] += x
                        found_y[at_r, at_q] += y
                        counted[at_r, at_q] += 1
            safe = np.clip(counted, 1, None)
            ground.sample = (
                np.clip(found_q // safe, 0, width - 1).astype(np.int32),
                np.clip(found_y // safe, 0, height - 1).astype(np.int32),
            )
        at_x, at_y = ground.sample
        return frame[at_y, at_x]

    # ------------------------------------------------------------------
    # The frame

    def __call__(
        self,
        camera: Camera,
        width: int,
        height: int,
        pixels: npt.NDArray[np.uint32],
        reference: bool = False,
        panel: bool = False,
        panels: Sequence[str] | None = None,
        pointer: tuple[int, int] | None = None,
        overlay: str | None = None,
        phase: float = 0.0,
        speed_milli: int = 0,
    ) -> FrameReading:
        """Fill one frame with the sketch, and report what the pass read.

        The arguments are the arguments of the drawing method of the world, so
        a caller swaps one renderer for the other and changes nothing else.
        """
        reading = self._world.draw(
            camera,
            width,
            height,
            pixels,
            reference=reference,
            panel=panel,
            panels=panels,
            pointer=pointer,
            overlay=overlay,
            phase=phase,
            speed_milli=speed_milli,
        )
        ground = self._for(camera, width, height)
        kept = self._panels(pixels, camera, width, height, overlay, phase)
        drawn = self._composite(ground, overlay, camera, width, height)
        frame = pixels.reshape(height, width)
        frame[...] = np.where(kept, frame, drawn)
        return reading

    def _composite(
        self,
        ground: Ground,
        overlay: str | None,
        camera: Camera,
        width: int,
        height: int,
    ) -> npt.NDArray[np.uint32]:
        """Run the page through the device, and give back the packed frame."""
        device = self.device
        device.make_current()
        self._send_page(ground)
        self._send_tiles(ground)
        washes = self._send_wash(ground, overlay, camera, width, height)
        page_rows, page_cols = ground.drawn.shape
        if washes:
            self._blur_wash(ground, page_cols, page_rows)
        device.ensure("wash_settled", "r32f")
        device.ensure("wash_blurred", "rg32f", bands=2)

        program = self._program(
            "composite", source.TONE + source.HATCH + source.COMPOSITE
        )
        program.use()
        units = [
            ("page_held", 0),
            ("page_depth", 1),
            ("page_flags", 2),
            ("page_take", 3),
            ("page_grain", 4),
            ("tile_holder", 5),
            ("tile_cloud", 6),
            ("tile_across_x", 7),
            ("tile_across_y", 8),
            ("tile_hue", 9),
            ("wash_settled", 10),
            ("wash_blurred", 11),
            ("palette", 12),
        ]
        for name, unit in units:
            device.bind(program, name, unit)

        first_row, last_row, first_col, last_col = ground.box
        span_rows = last_row - first_row
        span_cols = last_col - first_col
        scale = min(width / span_cols, height / span_rows)
        fit_w = max(int(span_cols * scale), 1)
        fit_h = max(int(span_rows * scale), 1)
        rise = max(int(page_cols * self._relief), 1)

        program["page_size"] = (page_cols, page_rows)
        program["window_size"] = (
            ground.window[2] - ground.window[0],
            ground.window[3] - ground.window[1],
        )
        program["rise"] = float(rise)
        program["light"] = tuple(float(band) for band in ink.LIGHT)
        program["palette_len"] = len(self._palette)
        program["faction_count"] = int(self._world.faction_count)
        program["draws_sky"] = 1 if self._sky else 0
        program["draws_wash"] = 1 if washes else 0
        program["cloud_step"] = max(int(page_cols * ink.CLOUD_SHADOW_STEP), 1)
        program["cloud_lift"] = int(rise * ink.CLOUD_HEIGHT)
        program["box_origin"] = (first_col, first_row)
        program["page_span"] = (span_cols, span_rows)
        program["fit_size"] = (fit_w, fit_h)
        program["fit_at"] = ((width - fit_w) // 2, (height - fit_h) // 2)
        program["fit_scale"] = float(scale)
        program.stop()

        device.run(program, width, height, "rgba8")
        # **The rows need no turning.** The device numbers the rows of a
        # target from the bottom and reads them back in that order, so the
        # first row it gives back is the row it called zero. The shader calls
        # that row the first row of the frame, so the order already agrees.
        bands = device.read(width, height)
        packed = bands.astype(np.uint32)
        result: npt.NDArray[np.uint32] = (
            (packed[..., 0] << 16) | (packed[..., 1] << 8) | packed[..., 2]
        ).astype(np.uint32)
        return result

    def _blur_wash(self, ground: Ground, page_cols: int, page_rows: int) -> None:
        """Blur the settled pigment at both reaches, down the page and across.

        A box mean is the same in either order, so the two axes run as two
        passes and each pass costs the reach and not the square of it. The two
        reaches run together in one pass, because the near window lies inside
        the far one.
        """
        device = self.device
        window = (
            ground.window[2] - ground.window[0],
            ground.window[3] - ground.window[1],
        )

        settle = self._program("wash_settled", source.WASH_SETTLED)
        settle.use()
        device.bind(settle, "page_take", 0)
        device.bind(settle, "page_grain", 1)
        device.bind(settle, "tile_flow", 2)
        device.put(settle, "page_size", (page_cols, page_rows))
        device.put(settle, "window_size", window)
        settle.stop()
        device.run(settle, page_cols, page_rows, "r32f")
        device.keep("wash_settled", "r32f")

        # The array renderer takes the mean down the page and then across it.
        # The first pass therefore runs down, and it reads one field. The
        # second runs across, and it reads the two the first produced.
        blur = self._program("wash_blur", source.WASH_BLUR)
        steps = (
            ((0, 1), 0, "wash_settled", "wash_down"),
            ((1, 0), 1, "wash_down", "wash_blurred"),
        )
        for along, paired, reads, writes in steps:
            blur.use()
            device.bind(blur, "source", 0, texture=reads)
            device.put(blur, "along", along)
            device.put(blur, "paired", paired)
            device.put(blur, "page_size", (page_cols, page_rows))
            blur.stop()
            device.run(blur, page_cols, page_rows, "rg32f")
            device.keep(writes, "rg32f")
