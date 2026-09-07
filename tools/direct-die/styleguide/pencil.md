# Style guide: pencil

This guide states the rules for one style: pencil and paper. The asset
is a terrain tile or an upgrade tile on a strategy game world map. A
tile is one SVG document. The map draws it at 64 pixels. A person looks
at it at 384 pixels only when that person judges it.

The style is a graphite sketch in a field notebook. A surveyor drew it
quickly on paper. The tone comes from hatch lines, not from filled
colour.

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

Use these fills and strokes, and no others.

| Name | Value | Use |
|------|-------|-----|
| paper | `#efe8da` | the base of every tile |
| graphite | `#3a3a38` | every line |
| soft graphite | `#6f6f6a` | a lighter hatch |
| pale graphite | `#a5a49c` | the lightest hatch |
| wash | `#b9c6c2` | one muted wash in one tile, and no more |

The base fill of the hexagon is always paper. The paper stays visible
between the hatch lines. A shape is a hatched area, not a solid area.

Use one wash colour in one tile, or use none.

## 3. Line

- Every line is a stroke. The stroke is 1.2 units wide.
- Use `stroke-linecap="round"`.
- No line is exactly straight. Give each line a small bend.
- A contour does not close. Leave a gap of 2 units or more where two
  contour lines would meet.
- Hatch lines run in one direction inside one shape. Space them 2.5
  units apart.

## 4. Shape and tone

- Build tone from hatch lines. Three hatch densities exist: pale, soft,
  and graphite.
- Fill no area with a solid colour, except the paper base and one wash.
- Draw five contour shapes or fewer inside the hexagon.
- Make every contour shape 12 units wide or more.

## 5. What survives at 64 pixels

At 64 pixels a hatch line disappears. A hatch field becomes an even
grey. The drawing therefore reads as a light grey tile, a dark grey
tile, or a shape of one tone against another.

Choose the hatch density for the value it makes, not for the texture. A
pale hatch reads as light. A graphite hatch reads as dark. Two subjects
must not share a density, or two tiles look the same on the map.

Keep the contour lines long. A contour shorter than 8 units vanishes.

## 6. What varies by subject

The style is fixed. The subject changes only the marks below, and the
hatch density.

| Subject | The marks and the density |
|---------|--------------------------|
| water | pale horizontal hatch, and three broken ripple lines |
| plain | pale open paper, and a few short upright grass ticks |
| forest | graphite hatch. Three sharp triangular conifer contours |
| hill | soft hatch that follows two dome contours |
| mountain | graphite hatch on one flank, and one hard peak contour |
| road | two long parallel contours, with pale hatch between them |
| terrace | three long horizontal step lines, with soft hatch below each |
| wonder | one tall contour with a dome, and dense graphite hatch |
| store | two round contours, with soft cross hatch inside |
| wall | one crenellated contour band, with graphite hatch on one face |
| lodging | one small house contour, with soft hatch on the roof face |

An upgrade sits on open paper, so a viewer sees the upgrade and not the
ground.

**Draw the subject, not the exemplar.** A conifer keeps its sharp
triangle. Do not round it because a hill exemplar uses a dome.

## 7. What this style forbids

- A saturated colour of any kind.
- A solid filled area larger than 6 units square, except the paper base.
- A perfectly straight line.
- A closed contour with no gap in it.
- An outline of 2 units or wider.
- More than one wash colour in one tile.
- A soft edge, a blur, or a shadow.
