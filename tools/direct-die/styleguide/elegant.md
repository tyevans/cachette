# Style guide: elegant

This guide states the rules for one style: elegant and classy. The asset
is a terrain tile or an upgrade tile on a strategy game world map. A
tile is one SVG document. The map draws it at 64 pixels. A person looks
at it at 384 pixels only when that person judges it.

The style is heraldic. It reduces a subject to a symbol on a shield. The
drawing is calm, symmetric, and mostly empty.

## 1. Geometry and mechanics

- The tile is one pointy-top regular hexagon.
- The document uses the attribute `viewBox="0 0 64 64"`.
- The hexagon has these six points, and no other:
  `32,2 58,17 58,47 32,62 6,47 6,17`.
- Nothing goes outside the hexagon. The four square corners stay empty.
- The hexagon edge itself carries no stroke. A stroke makes a seam
  where two tiles meet.
- The view is flat and from above.
- The document holds no script, no animation, no filter, no gradient,
  no `<image>` element, no font, and no text.
- The document is valid SVG 1.1.

## 2. Palette

Use these fills, and no others.

| Name | Value | Use |
|------|-------|-----|
| ink blue | `#1b2a3a` | water, and a dark base |
| slate | `#33414d` | mountain, wall, stone |
| moss | `#2f4436` | plain, forest, and a green base |
| bone | `#e6ddcd` | a light figure on a dark base |
| gold | `#c2a25a` | one accent figure in one tile |

Use three fills in one tile. Use four only when the subject needs a
gold accent.

The colours are dark and low in chroma. The tile is dark. The figure on
it is bone or gold.

## 3. Line

- Use no outline. A shape reads by its fill against the base.
- Where a line is needed as a figure, draw it as a filled bar of 3
  units or wider. Do not use a stroke.

## 4. Shape and symmetry

- Every tile is symmetric about its vertical centre line at `x=32`.
- Every shape is a clean geometric form: a triangle, a circle, a
  trapezium, an arc, or a straight-sided polygon.
- Draw three shapes or fewer inside the hexagon.
- Make every shape 14 units wide or more.
- Leave the base empty around the figure. Keep 6 units or more of empty
  base between the figure and the hexagon edge.

## 5. What survives at 64 pixels

At 64 pixels a viewer sees one symmetric mark on a dark field. That
mark is the whole asset. The style therefore spends everything on one
strong figure and on the empty space around it.

A second detail buys nothing at 64 pixels and costs clarity. Cut it.

Check the drawing this way. Shrink it to 64 pixels. One mark must
remain, centred, and readable.

## 6. What varies by subject

The style is fixed. The subject changes only the figure below. Each
figure is symmetric about the vertical centre line.

| Subject | The figure and the base |
|---------|------------------------|
| water | ink blue base. Three bone horizontal bars, the middle one longest |
| plain | moss base. One wide bone chevron, pointing up |
| forest | moss base. One tall bone triangle above a narrow bone trunk |
| hill | moss base. One bone arc, like a low half circle |
| mountain | slate base. One bone triangle with a notched peak |
| road | slate base. One bone vertical bar from edge to edge |
| terrace | moss base. Three bone bars, each shorter than the one below |
| wonder | ink blue base. One gold circle above one bone trapezium |
| store | slate base. One bone trapezium with a gold band across it |
| wall | slate base. One bone bar with three square teeth on its top |
| lodging | moss base. One bone triangle roof above one bone square |

**Draw the subject, not the exemplar.** A forest keeps its triangle. Do
not make it an arc because a hill exemplar uses an arc.

## 7. What this style forbids

- An outline or a stroke of any kind.
- A figure that is not symmetric about `x=32`.
- A bright or saturated colour.
- A rounded bulge, a wobble, or a hand-drawn line.
- A hatch line, a stipple, or a texture.
- More than three shapes inside the hexagon.
- A shape narrower than 14 units.
- A gold fill used for more than one figure in one tile.
