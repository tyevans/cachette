"""The fragment shaders that composite the sketchbook page.

**This module holds the same arithmetic the array renderer holds, written
once per pixel.** The array renderer builds one whole array for each stage.
The shader runs the whole chain for one pixel and keeps every stage in a
register, so the page costs one read of its inputs and one write of its
result rather than a dozen passes over memory.

The constants are not repeated here. The renderer that builds these strings
puts every number in as a definition, and it takes each one from the module
that already declares it, so the two renderers cannot drift apart on a value.

What the shader reads
---------------------

The build gives the page the height of the ground under each point, the
depth of the water, which points are drawn, which are a cliff and which are
water, and which tile of the window each point shows. Those are a function of
the camera alone, so they cross to the device when the camera moves.

The cloud, the wind and the faction that holds each tile change with the
world. They cross to the device on every frame, at the size of the window of
tiles and not at the size of the page, because a tile of the world covers many
points of the page.

Why the fit is in the shader
----------------------------

The page is wider and shorter than the frame, so the array renderer builds the
page and then samples it into the frame. The shader runs at the size of the
frame and reads the point of the page that each pixel shows, so the page never
exists as a picture at all. That removes the largest single array of the pass.
"""

from __future__ import annotations

# The common head of every fragment shader here: the version, and the helper
# that says how much of a pixel one set of ruled lines covers.
#
# **This is the array renderer's line function, one pixel at a time.** The
# remainder of a negative number is not negative in either language, so the
# phase behaves the same on both sides.
HEAD = """#version 330 core
out vec4 result;

float lines(float phase, float spacing, float weight) {
    float across = abs(mod(phase / spacing, 1.0) - 0.5) * 2.0;
    return clamp((weight - across) / max(weight, 1e-3), 0.0, 1.0);
}
"""

# What the page holds at each point, and how the shader reads it.
#
# The flags are one byte with one bit for each of the three questions, because
# three separate textures would be three fetches for three bits.
PAGE = """
uniform sampler2D page_held;
uniform sampler2D page_depth;
uniform usampler2D page_flags;
uniform isampler2D page_take;
uniform sampler2D page_grain;
uniform ivec2 page_size;

const uint DRAWN = 1u;
const uint CLIFF = 2u;
const uint WATER = 4u;

ivec2 wrapped(ivec2 at) {
    return ivec2(
        ((at.x % page_size.x) + page_size.x) % page_size.x,
        ((at.y % page_size.y) + page_size.y) % page_size.y
    );
}

ivec2 held_in(ivec2 at) {
    return clamp(at, ivec2(0), page_size - 1);
}

uint flags_at(ivec2 at) {
    return texelFetch(page_flags, at, 0).r;
}

bool drawn_at(ivec2 at) {
    return (flags_at(at) & DRAWN) != 0u;
}

float height_at(ivec2 at) {
    return texelFetch(page_held, held_in(at), 0).r;
}
"""

# How the shader reads a field the engine publishes for each tile.
#
# The page names a tile by its place in the window the build covered, so one
# number gives the row and the column of the tile. A point of the page that
# the ground does not reach reads nothing, and the array renderer gives it
# zero, so this gives it zero too.
TILES = """
uniform ivec2 window_size;

ivec2 tile_of(int take) {
    return ivec2(take % window_size.x, take / window_size.x);
}
"""

# The tone of the light on the ground.
#
# The slope of the height gives a normal and the light gives the tone. The
# slope is the central difference of the array renderer, which is one sided at
# the two edges of the page.
TONE = """
uniform float rise;
uniform vec3 light;

float slope(ivec2 at, ivec2 step_by) {
    ivec2 back = at - step_by;
    ivec2 on = at + step_by;
    bool at_start = (back.x < 0) || (back.y < 0);
    bool at_end = (on.x >= page_size.x) || (on.y >= page_size.y);
    if (at_start) { return height_at(on) - height_at(at); }
    if (at_end) { return height_at(at) - height_at(back); }
    return (height_at(on) - height_at(back)) * 0.5;
}

float tone_at(ivec2 at) {
    float scale = max(rise, 1.0);
    float slope_x = slope(at, ivec2(1, 0)) * scale;
    float slope_y = slope(at, ivec2(0, 1)) * scale;
    vec3 normal = vec3(-slope_x, -slope_y, 1.0);
    normal /= max(length(normal), 1e-4);
    float lit = clamp(dot(normal, light), 0.0, 1.0);
    return clamp(1.0 - lit * 1.3, 0.0, 1.0);
}
"""

# Every set of marks the ground carries, and the ink line on its silhouette.
#
# Each set carries one quantity. The sketch module names which.
HATCH = """
float coverage_at(ivec2 at) {
    uint mark = flags_at(at);
    bool is_drawn = (mark & DRAWN) != 0u;
    bool is_cliff = (mark & CLIFF) != 0u;
    bool is_water = (mark & WATER) != 0u;
    bool is_land = is_drawn && !is_water;
    float held = height_at(at);
    float depth = texelFetch(page_depth, at, 0).r;
    float tone = tone_at(at);
    float page_x = float(at.x);
    float page_y = float(at.y);

    float cover = 0.0;
    float contour = lines(held, CONTOUR_STEP, CONTOUR_WEIGHT);
    cover = max(cover, is_land ? contour * 0.60 : 0.0);
    float shade = clamp((tone - SHADOW_START) / (1.0 - SHADOW_START), 0.0, 1.0);
    float first = lines(page_x * 0.5 + page_y * 0.866, HATCH_SPACING, shade * 0.42);
    cover = max(cover, is_land ? first * 0.82 : 0.0);
    float deep = clamp((tone - CROSS_START) / (1.0 - CROSS_START), 0.0, 1.0);
    float second = lines(page_x * 0.5 - page_y * 0.866, HATCH_SPACING, deep * 0.45);
    cover = max(cover, is_land ? second * 0.86 : 0.0);
    float down = lines(page_x, CLIFF_SPACING, 0.38);
    cover = max(cover, is_cliff ? down * 0.75 : 0.0);
    float across = lines(page_y, WATER_SPACING, 0.16 + depth * 0.5);
    cover = max(cover, is_water ? across * 0.52 : 0.0);
    return cover;
}

float outline_at(ivec2 at) {
    bool here = drawn_at(at);
    float edge = 0.0;
    edge = max(edge, here != drawn_at(wrapped(at - ivec2(0, 1))) ? 1.0 : 0.0);
    edge = max(edge, here != drawn_at(wrapped(at + ivec2(0, 1))) ? 1.0 : 0.0);
    edge = max(edge, here != drawn_at(wrapped(at - ivec2(1, 0))) ? 1.0 : 0.0);
    edge = max(edge, here != drawn_at(wrapped(at + ivec2(1, 0))) ? 1.0 : 0.0);
    bool cliff_here = (flags_at(at) & CLIFF) != 0u;
    bool cliff_above = (flags_at(wrapped(at - ivec2(0, 1))) & CLIFF) != 0u;
    float top = (cliff_here && !cliff_above) ? 1.0 : 0.0;
    return max(edge, top * 0.85) * 0.85;
}
"""

# The pigment a named overlay laid down, settled into the grain of the paper.
#
# The strength of the wash reaches the device as a field for each tile, and
# the grain belongs to the page, so the two meet here. The edge of a wash
# dries darker than its middle, and the two blurred copies find that edge.
WASH_SETTLED = """
uniform sampler2D tile_flow;

void main() {
    ivec2 at = ivec2(gl_FragCoord.xy);
    int take = texelFetch(page_take, at, 0).r;
    float flow = (take >= 0) ? texelFetch(tile_flow, tile_of(take), 0).r : 0.0;
    float grain = texelFetch(page_grain, at, 0).r;
    result = vec4(flow * (1.0 - GRANULATION + GRANULATION * 2.0 * grain), 0, 0, 1);
}
"""

# One axis of the two blurs the rim of a wash needs.
#
# The array renderer takes the mean over a window of twice the reach, running
# from one short of the reach behind a point to the reach ahead of it, and it
# holds the value at the edge of the page steady past that edge. This is that
# window, and the two reaches run together so that one pass serves both.
WASH_BLUR = """
uniform sampler2D source;
uniform ivec2 along;
uniform int paired;

void main() {
    ivec2 at = ivec2(gl_FragCoord.xy);
    float near_total = 0.0;
    float far_total = 0.0;
    for (int step_by = -FAR_REACH + 1; step_by <= FAR_REACH; ++step_by) {
        ivec2 from = clamp(at + along * step_by, ivec2(0), page_size - 1);
        vec2 held = texelFetch(source, from, 0).rg;
        // The first pass reads one field and blurs it at both reaches. The
        // second pass reads the two results of the first and blurs each at
        // its own reach, so the two never mix.
        float near_value = (paired != 0) ? held.r : held.r;
        float far_value = (paired != 0) ? held.g : held.r;
        far_total += far_value;
        if (step_by > -NEAR_REACH && step_by <= NEAR_REACH) {
            near_total += near_value;
        }
    }
    result = vec4(
        near_total / float(2 * NEAR_REACH),
        far_total / float(2 * FAR_REACH),
        0.0,
        1.0
    );
}
"""

# The whole page, composited for one pixel of the frame.
#
# **The order is the order a pencil study is made in.** The paper carries its
# grain, the wash of the faction goes on it, the cloud crosses over the ground,
# the wash of an overlay glazes what is under it, and the ink goes on last.
COMPOSITE = """
uniform sampler2D tile_cloud;
uniform sampler2D tile_across_x;
uniform sampler2D tile_across_y;
uniform isampler2D tile_holder;
uniform sampler2D tile_hue;
uniform sampler2D wash_blurred;
uniform sampler2D wash_settled;
uniform sampler2D palette;
uniform int palette_len;
uniform int faction_count;
uniform int draws_sky;
uniform int draws_wash;
uniform int cloud_step;
uniform int cloud_lift;
uniform ivec2 box_origin;
uniform ivec2 fit_size;
uniform ivec2 fit_at;
uniform ivec2 page_span;
uniform float fit_scale;

float share_at(ivec2 at) {
    int take = texelFetch(page_take, at, 0).r;
    return (take >= 0) ? texelFetch(tile_cloud, tile_of(take), 0).r : 0.0;
}

vec2 across_at(ivec2 at) {
    int take = texelFetch(page_take, at, 0).r;
    if (take < 0) { return vec2(0.0); }
    ivec2 tile = tile_of(take);
    return vec2(
        texelFetch(tile_across_x, tile, 0).r,
        texelFetch(tile_across_y, tile, 0).r
    );
}

vec3 sky_over(vec3 page, ivec2 at) {
    // The cloud casts its shadow a step ahead of itself across the page.
    ivec2 shadow_from = wrapped(at - ivec2(cloud_step, 0));
    float under = clamp(share_at(shadow_from) - CLOUD_FLOOR, 0.0, 1.0);
    page = page * (1.0 - under * CLOUD_SHADOW_DEPTH);

    // The cloud layer stands above the tallest ground, so a mass crosses the
    // paper over a mountain rather than behind it.
    ivec2 above_at = wrapped(at + ivec2(0, cloud_lift));
    float above = share_at(above_at);
    vec2 turn = across_at(above_at);
    float thick = clamp((above - CLOUD_FLOOR) / (1.0 - CLOUD_FLOOR), 0.0, 1.0);
    float phase = float(at.x) * turn.x + float(at.y) * turn.y;
    float along = lines(phase, CLOUD_SPACING, thick * 0.42);
    float marks = along * 0.50;
    return page * (1.0 - marks) + SKY_INK * marks;
}

vec3 shade(ivec2 at) {
    uint mark = flags_at(at);
    bool is_drawn = (mark & DRAWN) != 0u;
    int take = texelFetch(page_take, at, 0).r;

    // The paper, with its grain.
    float grain = texelFetch(page_grain, at, 0).r;
    vec3 page = PAPER * (0.955 + grain * 0.070);

    // The wash of the faction that holds the ground.
    int held = (take >= 0) ? texelFetch(tile_holder, tile_of(take), 0).r : 0;
    bool owned = is_drawn && (held < faction_count);
    // The array renderer holds the number inside the palette and then takes
    // the remainder, so the remainder never changes it. This does the same.
    int slot = clamp(held, 0, palette_len - 1) % palette_len;
    vec3 colour = texelFetch(palette, ivec2(slot, 0), 0).rgb;
    vec3 tint = clamp(colour / 255.0, 0.0, 1.0);
    float weight = owned ? HOLDER_WASH : 0.0;
    page = page * (1.0 - weight) + page * tint * weight * 2.0;

    if (draws_sky != 0) { page = sky_over(page, at); }

    // The wash of a named overlay, as a glaze over what is under it.
    if (draws_wash != 0) {
        // The near blur and the far blur of the settled pigment. Their
        // difference is the rim, which is where the wash dried darker.
        vec2 blurred = texelFetch(wash_blurred, at, 0).rg;
        float rim = clamp(blurred.r - blurred.g, 0.0, 1.0);
        float settled = texelFetch(wash_settled, at, 0).r;
        vec3 hue = (take >= 0) ? texelFetch(tile_hue, tile_of(take), 0).rgb : vec3(0.0);
        float wash_weight = clamp(settled * WASH_GAIN + rim * RIM_GAIN, 0.0, 1.0)
            * (is_drawn ? 1.0 : 0.0);
        vec3 wash_tint = clamp(hue / 255.0 * GLAZE_LIFT, 0.0, 1.0);
        page = page * (1.0 - wash_weight) + page * wash_tint * wash_weight;
    }

    // The ink goes on last, over every wash.
    float cover = coverage_at(at);
    page = page * (1.0 - cover) + INK * cover;
    float edge = outline_at(at);
    return clamp(page * (1.0 - edge) + INK * edge, 0.0, 255.0);
}

void main() {
    ivec2 pixel = ivec2(gl_FragCoord.xy);
    vec3 out_colour = PAPER * 0.98;
    ivec2 inside = pixel - fit_at;
    if (inside.x >= 0 && inside.x < fit_size.x
        && inside.y >= 0 && inside.y < fit_size.y) {
        ivec2 source = ivec2(vec2(inside) / fit_scale);
        source = clamp(source, ivec2(0), page_span - 1) + box_origin;
        out_colour = shade(source);
    }
    // The array renderer packs the colour by cutting the fraction away, so
    // the shader writes the whole number it would have kept. A target of one
    // byte for each band rounds, and a whole number over 255 rounds to
    // itself, so the two renderers pack the same byte.
    result = vec4(floor(out_colour) / 255.0, 1.0);
}
"""
