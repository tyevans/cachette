# Style guide: the hex tile

This guide states the rules for one asset type: a terrain tile on the
world map. A tile is a single SVG document. The map draws it small. A
person looks at it large only when that person judges it.

Keep this guide short. A vague rule gives a vague critique. Add a rule
only when a real fault made the rule necessary. Add an exemplar only when
the drawing is good enough to copy.

## 1. Geometry

- The tile is one pointy-top regular hexagon.
- The document uses the attribute `viewBox="0 0 64 64"`.
- The hexagon has these six points, and no other:
  `32,2 58,17 58,47 32,62 6,47 6,17`.
- Nothing goes outside the hexagon. The four corners of the square stay
  empty, because the map draws the tiles edge to edge.
- The outer edge carries no stroke. A stroke makes a seam when two tiles
  meet.

## 2. Colour

- Use flat fills only. Do not use a gradient, a filter, a blur, or a
  raster image.
- Use five fills or fewer in one tile.
- The base fill covers the whole hexagon. Every other shape sits on top
  of it.
- A shape on top must differ from the base in lightness, not in hue
  alone. A hue-only difference disappears at the display size.

## 3. What reads at 64 pixels

- Draw six shapes or fewer inside the hexagon.
- Make every shape 5 units wide or more in the 64 unit space. A smaller
  shape becomes one grey pixel.
- Keep 3 units or more of clear space between two shapes.
- Give the tile one idea. A player must name the terrain in a quarter of
  a second.

## 4. Style

- The view is flat and from above. Do not draw a horizon, a shadow that
  falls outside the hexagon, or a light source that changes between
  tiles.
- Do not draw an outline around every shape. Use the fill to separate
  the shapes.
- Do not draw text, a number, or a letter.

## 5. Mechanics

- The document holds no script, no animation, and no external reference.
- The document holds no `<image>` element and no font.
- The document is valid SVG 1.1 and it renders in a plain rasteriser.

## 6. What a fault looks like

These are the faults that this guide exists to catch.

- The shape inside is too small, so the tile reads as flat colour at 64
  pixels.
- The tile holds too many shapes, so it reads as noise at 64 pixels.
- The hexagon is the wrong shape, or the art crosses the edge.
- The tile uses a gradient, so it does not match the accepted exemplars.
- Two tiles of different terrain look the same at 64 pixels.
