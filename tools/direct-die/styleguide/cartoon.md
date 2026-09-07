# Style guide: cartoon

This guide states the rules for one style: cartoon. The asset is a
terrain tile or an upgrade tile on a strategy game world map. A tile is
one SVG document. The map draws it at 64 pixels. A person looks at it at
384 pixels only when that person judges it.

The style is bold and friendly. The drawing looks moulded from thick
coloured plastic.

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
| ink | `#2b1d2e` | every outline |
| deep water | `#1f6fb2` | water |
| bright water | `#4aa8e8` | a water highlight |
| grass | `#57b83f` | plain, and the base under an upgrade |
| deep grass | `#2f8a2b` | the shaded part of grass |
| leaf | `#1f7a4a` | forest |
| earth | `#a9713f` | hill, trunk, road |
| deep earth | `#7a4a26` | the shaded part of earth |
| stone | `#9aa4ad` | mountain, wall |
| cream | `#f4e2b8` | wonder, store, lodging |
| hot | `#e8503f` | one accent in one tile |

Use five fills or fewer in one tile, and the ink beside them.

## 3. Line

- Draw an ink outline around every shape that sits on the base.
- The outline is 2 units wide. Use `stroke-linejoin="round"`.
- The outline keeps one width inside one tile.

## 4. Shape

- Every shape is fat and rounded. Each corner carries a radius.
- Give each shape a bulge. A trunk swells at its foot. A rock swells at
  its middle.
- Draw four shapes or fewer inside the hexagon.
- Make every shape 12 units wide or more.
- Give each large shape one highlight in a lighter fill of the same
  hue. Put the highlight at the top left.

## 5. What survives at 64 pixels

At 64 pixels a viewer sees the silhouette and the ink outline, and
little else. The outline is therefore the style. Three fat shapes with a
2 unit outline read as three shapes. Eight thin shapes read as a grey
smear.

Test the drawing this way. Hide the colour and look at the ink alone.
The subject must stay clear.

## 6. What varies by subject

The style is fixed. The subject changes only the shapes below.

| Subject | The shapes |
|---------|-----------|
| water | two or three fat wave humps. Each hump is a rounded lozenge |
| plain | two or three rounded grass tufts, wide and low |
| forest | three fat conifers. A conifer is a rounded triangle, not a spike |
| hill | two overlapping rounded domes |
| mountain | one fat peak with a rounded top and one snow cap |
| road | one thick band across the tile, with a rounded end at each edge |
| terrace | three wide steps. Each step has a rounded front lip |
| wonder | one tall rounded tower with a dome, and one hot accent flag |
| store | two fat barrels or sacks, each wider at the middle |
| wall | three thick crenellated blocks in a row |
| lodging | one fat hut with a rounded roof and one round door |

An upgrade sits on a grass base, so a viewer sees the upgrade and not
the ground.

**Draw the subject, not the exemplar.** A conifer keeps its triangle. Do
not make it a hump because a water exemplar uses humps.

## 7. What this style forbids

- A shape with no ink outline.
- An outline thinner than 2 units.
- A sharp corner on a shape that the table above calls rounded.
- A muted colour or a grey outside the stone fill.
- A hatch line, a stipple, a texture, or a paper grain.
- More than four shapes inside the hexagon.
- A shape narrower than 12 units.
