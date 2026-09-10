---
id: 0537
title: Make a map one region of a planet, and take the latitude span from the settings of the world
status: complete
created: 2026-09-09
implements: [ADR-0177 D1]
changes: [ADR-0177]
creates: []
serves: []
blocked-by: []
---

## Why

**Every map showed three horizontal bands of weather at the same latitudes.**
The field imposes a low at the equator, a high at thirty degrees and a low at
sixty, because a single-layer field cannot grow those belts on its own. The
latitude span of a world was configuration already, and the default span ran
from pole to pole, so every world the engine built carried the whole set of
belts at the same rows.

**The project owner reversed a ruling.** He ruled on 6 September 2026 that a
map is a whole planet. He ruled on 9 September 2026 that a map is one region
of a planet, and that the span is configuration.

The findings register holds what the project believed and what is true.[^2]

**The scale register had to deny one of its own rows.** It derives a world
extent of about 330 kilometres from the tile edge at the target tile count.
The planet ruling forced a paragraph under that table saying the figure is not
the extent of the world.

## The architectural impact review

**The records that govern this work.** ADR-0177 D1 governs the latitudes of a
world.[^1] ADR-0177 D2 governs the imposed pressure belts, and it stands
unchanged.

**The records this work changes.** ADR-0177. The record is a draft, so the
work edits it in place rather than superseding it. The Context now records the
reversal, and D1 now states that a world which states no span is one region.

**The records this work creates.** None. The constraint has a record already.

**The blockers that hold it.** None. BLK-130 governs what the wind amplitude
should be worth, and this work changes no amplitude.

## What the work found

**A default that nothing overrides is the value of the parameter.** The span
was configuration before this work, and no caller stated one. So the default
decided what every map looked like.

**The climate spin was a second declaration site.** The spin builds a weather
field of its own, and that field took the default span. A world built at a
stated span would therefore have been spun under a different sky, and nothing
would have failed. The spin now takes the span of the world it spins.

**The settings of a world listed every field at each of its call sites.** The
settings gained two fields, so each site had to state them or take the
default. Each site now takes the default. The commit body holds the count and
the search command.

## What the work delivers

The settings of a world state the latitude of the middle row and the latitude
from the first row to the last, in hundredths of a degree. The default is
three degrees at forty-five degrees north, which is what the extent of a world
of the target tile count is worth. The weather field and the climate spin both
read that one site. The Python constructor takes both values and holds no copy
of either default. A span that does not fit on the globe is a typed refusal
from the constructor of the world.

The planet reading stays reachable. A caller that states a centre at the
equator and a span from pole to pole gets the published belts, and a test
holds that.

**Every golden state hash changed**, because the weather of every world reads
its own span.

## References

[^1]: ADR-0177, the row axis of a world is a latitude that the world states. `docs/adrs/draft/adr-0177-the-row-axis-of-a-world-is-a-latitude-that-the-world-states.md`
[^2]: Findings register, FND-738. `docs/FINDINGS.md`
