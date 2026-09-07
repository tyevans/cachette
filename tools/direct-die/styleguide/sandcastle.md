# Style guide: sandcastle

This guide states the rules for one style: silly sand castles and beach.
The asset is a terrain tile or an upgrade tile on a strategy game world
map. A tile is one SVG document. The map draws it at 64 pixels. A person
looks at it at 384 pixels only when that person judges it.

Every subject is built from wet sand on a beach. A child moulded it with
a bucket. Nothing stands up straight, and one plastic toy sits in every
tile.

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
| dry sand | `#e8d29a` | the base of every tile |
| wet sand | `#c9a460` | a moulded form |
| shade sand | `#a37f42` | the shaded face of a moulded form |
| sea | `#3fa9c9` | water, and a wet patch |
| foam | `#eef6f4` | the edge of the sea |
| toy | `#ff5a4d` | one plastic toy in one tile |

Use five fills or fewer in one tile.

Every land subject uses the sand fills. The subject changes the form,
not the material. A sand mountain is sand. A sand forest is sand.

## 3. Edge and line

- A moulded edge is stepped, not smooth. Build a slope from steps of 3
  to 5 units, like a bucket turned out.
- Use no outline. Separate a form from the base with the shade fill.
- Give each form one shade face. Put it on the right side of the form,
  in every tile.

## 4. Shape and lean

- Nothing stands upright. Lean each tall form 5 to 12 degrees off the
  vertical.
- Lean two forms in one tile in two different directions.
- Draw five shapes or fewer inside the hexagon, and count the toy as
  one of them.
- Make every shape 10 units wide or more, except the toy. The toy is 6
  units wide or more.
- Crumble one corner of one form in each tile. A crumble is a small
  notch cut out of the silhouette.

## 5. What survives at 64 pixels

At 64 pixels a step of 3 units becomes one pixel, so the stepped edge
reads as a soft ragged edge. That raggedness is the style, and it
survives. The lean survives too, because a lean changes the silhouette.

The toy survives only through its colour. It is the one saturated mark
on a sand tile, so it reads as a bright dot. Keep it 6 units wide or
more, and keep it clear of the shade fill.

Check the drawing this way. Shrink it to 64 pixels. The tile must look
like sand, it must look tilted, and one bright dot must show.

## 6. What varies by subject

The style is fixed. The subject changes only the form below. Every land
form is moulded sand.

| Subject | The form and the toy |
|---------|---------------------|
| water | a sea pool with a foam edge, and one floating toy ring |
| plain | flat raked sand with two shallow furrows, and one toy spade |
| forest | three leaning sand cones, each a stack of narrowing steps, and one toy flag |
| hill | one wide leaning sand mound with a stepped side, and one toy bucket |
| mountain | one tall leaning stepped sand cone with a crumbled top, and one toy flag |
| road | a band of packed damp sand with two footprint dents, and one toy spade |
| terrace | three stepped sand ledges, each leaning, and one toy bucket |
| wonder | one tall leaning sand tower with a stepped crown, and one toy flag |
| store | two leaning upturned bucket forms, and one toy ring |
| wall | a low crenellated sand wall that sags in the middle, and one toy flag |
| lodging | one leaning sand dome with an arch dug into it, and one toy spade |

An upgrade sits on a dry sand base, so a viewer sees the upgrade and not
the ground.

**Draw the subject, not the exemplar.** A forest keeps three cone forms.
Do not make it a mound because a hill exemplar uses a mound.

## 7. What this style forbids

- A material that is not sand, sea, foam, or plastic toy. No grass green,
  no wood brown, no grey stone.
- A smooth diagonal edge on a moulded form. The edge is stepped.
- A form that stands upright.
- An outline or a stroke.
- A tile with no toy, or a tile with two toys.
- A hatch line, a gradient, or a drop shadow.
- More than five shapes inside the hexagon.
