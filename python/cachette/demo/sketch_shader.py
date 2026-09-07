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
uniform sampler2D page;
uniform sampler2D page_grain;
uniform ivec2 page_size;

const uint DRAWN = 1u;
const uint CLIFF = 2u;
const uint WATER = 4u;

// What the geometry pass wrote at one point of the page.
//
// **One texture holds the whole page.** The mesh pass writes the height of
// the ground, the depth of the water, the flags and the tile in one target,
// because a target for each of them would be four attachments and four
// fetches for numbers that are always read together.
vec4 page_at(ivec2 at) {
    return texelFetch(page, clamp(at, ivec2(0), page_size - 1), 0);
}

// Bring a point back inside the page, the way the array renderer rolls a
// field: a point off one edge comes back on the other.
//
// **The remainder operator of this language is undefined when either side is
// negative.** The array renderer relies on a remainder that is never
// negative, so the two disagree at the first row and the first column alone.
// Nothing else on the page reads a negative coordinate, so a page whose
// ground stops short of the edge hides the fault completely. This form is
// defined for every input. The page is far smaller than the largest whole
// number a real number holds exactly, so the division below is exact.
int wrap_one(int at, int by) {
    return at - by * int(floor(float(at) / float(by)));
}

ivec2 wrapped(ivec2 at) {
    return ivec2(wrap_one(at.x, page_size.x), wrap_one(at.y, page_size.y));
}

uint flags_at(ivec2 at) {
    return uint(page_at(at).b + 0.5);
}

bool drawn_at(ivec2 at) {
    return (flags_at(at) & DRAWN) != 0u;
}

float height_at(ivec2 at) {
    return page_at(at).r;
}

float depth_at(ivec2 at) {
    return page_at(at).g;
}

// The tile the point shows, or a number below nought where the ground does
// not reach. The mesh pass clears the target to that number, so a point no
// triangle covered says so.
int take_at(ivec2 at) {
    return int(floor(page_at(at).a + 0.5));
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
    float depth = depth_at(at);
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
    int take = take_at(at);
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
# The ways that run over the ground, drawn where a road stands.
#
# **A road is a way and not a tile.** It runs from somewhere to somewhere, it
# joins another road at a junction, it bends, and it ends. The ribbon runs from
# the middle of a tile out to the middle of the edge it shares with each
# neighbour that carries a road, so the two halves of a join meet exactly.
#
# **The offset inside a tile comes from the page and not from a second
# texture.** The mesh pass writes the height of the ground and the tile at each
# point. The lift is a whole count of rows and it follows that height, so the
# row of the flat page follows from the row of the lifted page. Turning that
# row and its column back into the plan of the world gives the place inside the
# tile, and the arithmetic is the arithmetic of the array renderer read
# backwards.
#
# A point on the face that the lift exposed is not on the top of the ground, so
# no way runs over it.
WAYS = """
uniform isampler2D tile_joins;
uniform isampler2D tile_level;
uniform vec2 plan_half;
uniform vec2 turn_by;
uniform vec2 way_page_middle;
uniform float way_scale;
uniform float way_lean;
uniform float way_rise;
uniform float lift_margin;
uniform float row_pitch;
uniform int draws_ways;

// Where inside its own tile one point of the page stands, in tiles, on each
// axis of the grid. Both parts run from minus one half to one half.
vec2 within_tile(ivec2 at, float raised) {
    float flat_row = float(at.y) - way_rise - lift_margin + floor(raised * way_rise);
    float across = float(at.x) - way_page_middle.x;
    float down = (flat_row - way_page_middle.y) / way_lean;
    float plan_x = (across * turn_by.x + down * turn_by.y) / way_scale + plan_half.x;
    float plan_y = (down * turn_by.x - across * turn_by.y) / way_scale + plan_half.y;
    float tile_r = plan_y / row_pitch;
    float tile_q = plan_x - tile_r * 0.5;
    return vec2(tile_q - floor(tile_q + 0.5), tile_r - floor(tile_r + 0.5));
}

// How dark the ways at one point of the page are, from none to one.
float way_ink_at(ivec2 at) {
    if (draws_ways == 0) { return 0.0; }
    uint mark = flags_at(at);
    if ((mark & DRAWN) == 0u || (mark & CLIFF) != 0u) { return 0.0; }
    int take = take_at(at);
    if (take < 0) { return 0.0; }
    ivec2 tile = tile_of(take);
    int level = texelFetch(tile_level, tile, 0).r;
    if (level <= 0) { return 0.0; }
    int joins = texelFetch(tile_joins, tile, 0).r;
    vec2 inside = within_tile(at, height_at(at));
    vec2 plan = vec2(inside.x + inside.y * 0.5, inside.y * row_pitch);
    // The distance to the ribbon. It opens at the distance to the middle of
    // the tile, which is the cap of a way that ends here and the round of a
    // junction that turns here.
    float near = length(plan);
    for (int direction = 0; direction < 6; direction += 1) {
        if (((joins >> direction) & 1) == 0) { continue; }
        vec2 step_by = WAY_STEPS[direction];
        // The middle of the shared edge is half way between the two tile
        // middles, so the neighbour reaches the same point from its side.
        vec2 reach = vec2(
            (step_by.x + step_by.y * 0.5) * 0.5,
            step_by.y * row_pitch * 0.5
        );
        float span = dot(reach, reach);
        float along = clamp(dot(plan, reach) / span, 0.0, 1.0);
        near = min(near, length(plan - along * reach));
    }
    float line = max(WAY_INK / max(way_scale, 1e-3), 1e-6);
    float half_wide = WAY_WIDTH[clamp(level, 0, WAY_LEVELS - 1)] * 0.5;
    float edge = clamp(1.0 - abs(near - half_wide) / line, 0.0, 1.0);
    float middle = clamp(1.0 - near / line, 0.0, 1.0);
    float surface = clamp((half_wide - near) / line, 0.0, 1.0);
    if (level == 1) { return middle * WAY_PLANNED_INK; }
    float ink = max(edge * WAY_EDGE_INK, surface * WAY_FILL_INK);
    if (level >= CROWNED_LEVEL) { ink = max(ink, middle * WAY_CROWN_INK); }
    return ink;
}
"""

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
uniform ivec2 fit_size;
uniform isampler2D fit_x;
uniform isampler2D fit_y;

float share_at(ivec2 at) {
    int take = take_at(at);
    return (take >= 0) ? texelFetch(tile_cloud, tile_of(take), 0).r : 0.0;
}

vec2 across_at(ivec2 at) {
    int take = take_at(at);
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
    int take = take_at(at);

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

    // The ways go over the hatch. A road is a made thing, and a mark that
    // the hatch of the ground crossed would read as ground. The array
    // renderer lays them in this order.
    float way = way_ink_at(at);
    page = page * (1.0 - way) + INK * way;

    float edge = outline_at(at);
    return clamp(page * (1.0 - edge) + INK * edge, 0.0, 255.0);
}

void main() {
    ivec2 pixel = ivec2(gl_FragCoord.xy);
    vec3 out_colour = PAPER * 0.98;
    if (pixel.x >= 0 && pixel.x < fit_size.x
        && pixel.y >= 0 && pixel.y < fit_size.y) {
        // **The two lists say which point of the page each pixel shows.**
        // The array renderer builds them, and the shader reads them rather
        // than working the mapping out again. A mapping worked out twice is
        // one value declared twice: the two divide a whole number by the same
        // scale at different widths, and near a boundary they land on
        // neighbouring points. On a hatched page those two points are far
        // apart in colour, so the picture differs where nothing is wrong.
        //
        // A value below nought names a pixel the page does not reach, and
        // that pixel carries bare paper.
        ivec2 source = ivec2(
            texelFetch(fit_x, ivec2(pixel.x, 0), 0).r,
            texelFetch(fit_y, ivec2(pixel.y, 0), 0).r
        );
        if (source.x >= 0 && source.y >= 0) {
            out_colour = shade(source);
        }
    }
    // The array renderer packs the colour by cutting the fraction away, so
    // the shader writes the whole number it would have kept. A target of one
    // byte for each band rounds, and a whole number over 255 rounds to
    // itself, so the two renderers pack the same byte.
    result = vec4(floor(out_colour) / 255.0, 1.0);
}
"""


# The vertex shader that draws the terrain as a mesh.
#
# **The tile heights never change, so they cross to the device once.** The
# transform below is what a turn, a lean or a move of the camera changes, and
# it is a handful of uniforms. A page rebuilt for each angle would send the
# whole terrain again for a change that costs nine numbers.
#
# One tile carries twelve vertices. Four of them are the top face, which is
# the square of the tile lifted by the height of the ground on it. Four more
# are the same square, un-lifted, which is where the face below the tile
# reaches down to. The last four repeat the top face for the sides, because a
# vertex carries the flag that says whether it belongs to a face and one
# vertex cannot carry two answers.
#
# **The depth of a vertex is the row of the flat page.** That row grows
# towards the watcher, so the depth test keeps the nearest ground and hides
# what stands behind it.
GEOMETRY_VERTEX = """#version 330 core
in vec2 tile;
in vec2 corner;
in float raised;
in float deep;
in float wet;
in float skirt;

uniform vec2 first_tile;
uniform vec2 plan_half;
uniform vec2 turn_by;
uniform float scale;
uniform float lean;
uniform vec2 page_middle;
uniform vec2 page_span;
uniform float rise;
uniform float lift_margin;
uniform float row_pitch;
uniform vec4 window;
uniform int world_wide;

flat out vec4 held;
flat out float keep;

void main() {
    vec2 place = tile - first_tile + corner;
    float plan_x = place.x + place.y * 0.5;
    float plan_y = place.y * row_pitch;
    float across = (plan_x - plan_half.x) * scale;
    float down = (plan_y - plan_half.y) * scale;
    float column = across * turn_by.x - down * turn_by.y + page_middle.x;
    float flat_row =
        (across * turn_by.y + down * turn_by.x) * lean + page_middle.y;
    // The lift moves the top face up the page by the height of the ground.
    // The face below it stays where the ground stood, so the two differ by
    // the rise alone.
    //
    // **The lift is a whole count of rows.** The array renderer moves a point
    // by a whole number of rows, so a tile stands on one plateau rather than
    // on a slope. A lift of a fraction of a row would put every mark on the
    // page half a row from where the other renderer draws it.
    float lift = floor(raised * rise) * (1.0 - skirt);
    float row = flat_row + rise + lift_margin - lift;

    // The flat value comes from the last vertex of a triangle, and every
    // triangle below the top face ends on the lower square. The answer to
    // whether this vertex stands where the ground stood is therefore also
    // the answer to whether the triangle is a face.
    float flags = 1.0 + skirt * 2.0 + wet * 4.0;
    float take = tile.y * float(world_wide) + tile.x;
    held = vec4(raised, deep, flags, take);
    keep = (tile.x >= window.x && tile.x < window.z
            && tile.y >= window.y && tile.y < window.w) ? 1.0 : 0.0;

    // The page numbers its rows from the bottom, and the fragment stage reads
    // the row it wrote at the same number, so the two already agree.
    float clip_x = (column + 0.5) / page_span.x * 2.0 - 1.0;
    float clip_y = (row + 0.5) / page_span.y * 2.0 - 1.0;
    // The nearest ground carries the smallest depth. The row of the flat page
    // grows towards the watcher, so the depth runs the other way. The span is
    // widened so that a corner outside the page keeps its order.
    float clip_z = 1.0 - 2.0 * (flat_row + page_span.y) / (3.0 * page_span.y);
    gl_Position = vec4(clip_x, clip_y, clip_z, 1.0);
}
"""

# The fragment shader that writes one point of the page.
#
# **The hardware resolves the occlusion.** A fragment that reaches here won
# the depth test, so it is the nearest ground at this point of the page. The
# page therefore holds the height, the depth of the water, the flags and the
# tile of that ground, and nothing had to sort anything.
GEOMETRY_FRAGMENT = """#version 330 core
flat in vec4 held;
flat in float keep;
out vec4 result;

void main() {
    if (keep < 0.5) { discard; }
    result = held;
}
"""
