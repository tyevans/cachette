# The Observed Zonal Mean Surface Temperature

This report supplies a target profile for a prescribed temperature field. It
gives the mean surface air temperature of each latitude band. It gives the
annual mean, the January mean and the July mean. It gives the land-only profile
first, because a land classification grades against land. It gives the
land-and-ocean profile second, because the land profile has no data over
Antarctica.

I took report number 32. Numbers 00 to 31 were already in use.

## 1. What I found, and what I did not find

I did not find a published table of zonal mean land surface air temperature
against latitude. I searched for the classic climatology tables, and for a
modern reanalysis table. The textbook tables that other documents cite are not
available online. The reanalysis products publish grids and figures, not tables.

I therefore built the tables below from two published gridded climatologies.
Every number in this report is a value I computed from a published grid. No
number in this report is a value I read from a figure. No number in this report
is an estimate.

Both grids give the mean of the years 1961 to 1990. The units are degrees
Celsius throughout.

## 2. Provenance and method

The land-only tables come from a published climatology of global land at a
resolution of 10 minutes of arc.[^1] The grid holds 566262 land cells. Each cell
holds 12 monthly mean temperatures. The grid excludes Antarctica. The grid gives
the temperature at the true elevation of the cell. It does not reduce the
temperature to sea level.

The land-and-ocean table comes from the published absolute temperature
climatology on a grid of 5 degrees.[^2] That grid holds a value in every cell.
It covers Antarctica.

For each band I took the mean of the cells in the band. I weighted each cell by
the cosine of its latitude. This weight makes the mean an area mean. For the
land grid every cell has the same extent in latitude and in longitude, so the
cosine is the whole weight. For the land-and-ocean grid I took the mean along
each row first, then combined the rows with the cosine weight.

I computed the annual mean as the mean of the 12 monthly values. I computed the
coldest month and the warmest month from the 12 zonal mean values of the band,
not from the individual cells.

As a check, the area weighted annual mean of the whole land grid is 13.0 degrees
Celsius. That agrees with the published figure for land without Antarctica.

## 3. Land only, bands of 10 degrees

A negative latitude is south. Each band runs from the first value up to the
second value. The column "Range" is the warmest month minus the coldest month.

| Band | Cells | Jan | Jul | Annual | Coldest month | Warmest month | Range |
|------|-------|-----|-----|--------|---------------|---------------|-------|
| 90S to 60S | 0 | — | — | — | — | — | — |
| 60S to 50S | 1233 | 10.1 | 1.1 | 5.9 | Jul 1.1 | Jan 10.1 | 9.0 |
| 50S to 40S | 4229 | 14.9 | 3.2 | 9.2 | Jul 3.2 | Jan 14.9 | 11.7 |
| 40S to 30S | 14789 | 22.7 | 9.4 | 16.1 | Jul 9.4 | Jan 22.7 | 13.3 |
| 30S to 20S | 30154 | 26.3 | 13.9 | 20.7 | Jul 13.9 | Jan 26.3 | 12.4 |
| 20S to 10S | 28882 | 24.5 | 19.3 | 22.9 | Jul 19.3 | Nov 24.8 | 5.6 |
| 10S to 0 | 31374 | 25.0 | 23.8 | 24.8 | Jul 23.8 | Oct 25.4 | 1.6 |
| 0 to 10N | 29995 | 24.9 | 24.5 | 25.3 | Jul 24.5 | Mar 26.4 | 1.9 |
| 10N to 20N | 34717 | 21.8 | 28.6 | 26.7 | Jan 21.8 | May 30.4 | 8.5 |
| 20N to 30N | 49418 | 13.3 | 29.2 | 22.4 | Jan 13.3 | Jul 29.2 | 15.9 |
| 30N to 40N | 56225 | 0.2 | 23.4 | 12.1 | Jan 0.2 | Jul 23.4 | 23.3 |
| 40N to 50N | 68611 | -9.3 | 20.6 | 6.3 | Jan -9.3 | Jul 20.6 | 29.9 |
| 50N to 60N | 74830 | -17.8 | 16.2 | -0.1 | Jan -17.8 | Jul 16.2 | 33.9 |
| 60N to 70N | 93604 | -26.6 | 11.8 | -8.1 | Jan -26.6 | Jul 11.8 | 38.4 |
| 70N to 80N | 39961 | -32.3 | 2.6 | -17.1 | Feb -32.7 | Jul 2.6 | 35.3 |
| 80N to 90N | 8240 | -34.1 | -0.1 | -20.7 | Feb -35.4 | Jul -0.1 | 35.4 |

The band from 80N to 90N holds Greenland and the northern Canadian islands. All
of its cells lie between 80N and 85N.

## 4. Land only, bands of 5 degrees

This table gives the same quantity at a finer resolution. The last two columns
give the coldest and the warmest of the 12 zonal mean values.

| Band | Cells | Jan | Jul | Annual | Coldest | Warmest |
|------|-------|-----|-----|--------|---------|---------|
| 60S to 55S | 64 | 7.9 | 0.5 | 4.5 | 0.5 | 7.9 |
| 55S to 50S | 1169 | 10.2 | 1.1 | 6.0 | 1.1 | 10.2 |
| 50S to 45S | 1720 | 13.0 | 1.8 | 7.7 | 1.8 | 13.0 |
| 45S to 40S | 2509 | 16.1 | 4.0 | 10.1 | 4.0 | 16.1 |
| 40S to 35S | 4378 | 20.6 | 7.3 | 13.9 | 7.3 | 20.6 |
| 35S to 30S | 10411 | 23.5 | 10.3 | 17.0 | 10.3 | 23.5 |
| 30S to 25S | 14136 | 25.9 | 12.1 | 19.4 | 12.1 | 25.9 |
| 25S to 20S | 16018 | 26.5 | 15.4 | 21.7 | 15.4 | 26.5 |
| 20S to 15S | 15496 | 25.1 | 18.3 | 22.8 | 18.3 | 25.3 |
| 15S to 10S | 13386 | 23.9 | 20.4 | 23.0 | 20.4 | 24.5 |
| 10S to 5S | 15315 | 24.8 | 23.3 | 24.6 | 23.3 | 25.3 |
| 5S to 0 | 16059 | 25.1 | 24.2 | 25.1 | 24.2 | 25.4 |
| 0 to 5N | 14028 | 25.1 | 24.3 | 25.1 | 24.3 | 25.8 |
| 5N to 10N | 15967 | 24.7 | 24.7 | 25.4 | 24.4 | 27.0 |
| 10N to 15N | 15608 | 23.5 | 27.2 | 26.9 | 23.5 | 29.9 |
| 15N to 20N | 19109 | 20.5 | 29.8 | 26.6 | 20.5 | 30.8 |
| 20N to 25N | 23014 | 16.4 | 30.0 | 24.5 | 16.4 | 30.2 |
| 25N to 30N | 26404 | 10.5 | 28.4 | 20.4 | 10.5 | 28.4 |
| 30N to 35N | 27722 | 2.9 | 24.5 | 14.0 | 2.9 | 24.5 |
| 35N to 40N | 28503 | -2.7 | 22.3 | 10.2 | -2.7 | 22.3 |
| 40N to 45N | 31904 | -6.5 | 21.8 | 8.2 | -6.5 | 21.8 |
| 45N to 50N | 36707 | -12.0 | 19.5 | 4.5 | -12.0 | 19.5 |
| 50N to 55N | 38849 | -16.3 | 16.9 | 1.1 | -16.3 | 16.9 |
| 55N to 60N | 35981 | -19.5 | 15.4 | -1.6 | -19.5 | 15.4 |
| 60N to 65N | 46033 | -24.8 | 13.4 | -5.9 | -24.8 | 13.4 |
| 65N to 70N | 47571 | -28.7 | 9.9 | -10.5 | -28.7 | 9.9 |
| 70N to 75N | 23616 | -31.8 | 4.4 | -15.6 | -31.8 | 4.4 |
| 75N to 80N | 16345 | -33.4 | -1.1 | -20.2 | -34.6 | -1.1 |
| 80N to 85N | 8240 | -34.1 | -0.1 | -20.7 | -35.4 | -0.1 |

## 5. Land and ocean, bands of 10 degrees

This table covers the whole globe. Use it only for the two southern polar bands,
where the land table has no data. Over the ocean the grid holds the sea surface
temperature, not the air temperature.

| Band | Jan | Jul | Annual | Coldest | Warmest |
|------|-----|-----|--------|---------|---------|
| 90S to 80S | -21.6 | -51.1 | -40.8 | -51.3 | -21.6 |
| 80S to 70S | -14.1 | -37.4 | -28.9 | -37.8 | -14.1 |
| 70S to 60S | -0.2 | -11.3 | -6.7 | -11.9 | -0.2 |
| 60S to 50S | 4.8 | 0.9 | 2.7 | 0.4 | 5.0 |
| 50S to 40S | 11.6 | 7.8 | 9.8 | 7.8 | 12.1 |
| 40S to 30S | 19.3 | 13.7 | 16.4 | 13.4 | 19.9 |
| 30S to 20S | 24.2 | 18.2 | 21.3 | 18.2 | 24.5 |
| 20S to 10S | 25.5 | 22.5 | 24.3 | 22.5 | 25.9 |
| 10S to 0 | 26.1 | 25.1 | 25.9 | 25.0 | 26.7 |
| 0 to 10N | 25.9 | 25.9 | 26.2 | 25.9 | 26.9 |
| 10N to 20N | 23.9 | 27.0 | 26.0 | 23.9 | 27.3 |
| 20N to 30N | 17.7 | 26.7 | 22.7 | 17.7 | 26.8 |
| 30N to 40N | 7.9 | 22.8 | 15.4 | 7.9 | 23.3 |
| 40N to 50N | -2.1 | 17.9 | 8.2 | -2.1 | 18.2 |
| 50N to 60N | -10.3 | 13.6 | 1.9 | -10.3 | 13.6 |
| 60N to 70N | -22.2 | 10.4 | -6.5 | -22.2 | 10.4 |
| 70N to 80N | -26.3 | 2.6 | -13.4 | -26.6 | 2.6 |
| 80N to 90N | -30.0 | 0.0 | -16.8 | -30.4 | 0.0 |

The Antarctic values are the values of the ice sheet surface. The ice sheet
stands about 2 to 3 kilometres above sea level. The band from 90S to 80S is
therefore much colder than the band from 80N to 90N.

## 6. What the tables say about the two hemispheres

The two hemispheres are not mirror images. The annual mean of northern land is
10.0 degrees Celsius. The annual mean of southern land is 21.5 degrees Celsius.
Southern land sits mostly in the tropics and the subtropics. Northern land
reaches the pole.

The seasonal range shows the same asymmetry. Southern land never exceeds a range
of 13.3 degrees. Northern land reaches a range of 38.4 degrees between 60N and
70N. Do not mirror one hemisphere onto the other.

## 7. What the tables say about the seasonal range

Over land the seasonal range grows with latitude up to 60N to 70N. It then falls
a little toward the pole. It does not stay flat from 50 degrees to the pole. The
values are 29.9 degrees at 40N to 50N, 33.9 degrees at 50N to 60N, 38.4 degrees
at 60N to 70N, and 35.4 degrees at 80N to 90N.

The tropical bands hold almost no seasonal range. The band from 10S to the
equator has a range of 1.6 degrees. The warmest month there is October, not
January. Between 20S and 10S the warmest month is November. Between the equator
and 10N the warmest month is March. A model that puts the extreme of every
tropical band at a solstice will not match this.

## 8. Limitations

Read these before you use the tables.

The land climatology excludes Antarctica. Use the land-and-ocean table for the
bands south of 60S. The two tables measure different things, so do not join them
without saying so.

The land climatology interpolates station observations. Stations sit at low
elevation and in inhabited places more often than the land as a whole. The
authors of the grid state this bias. It makes high plateaus and deserts less
reliable than temperate lowlands.

The land grid gives the temperature at the true elevation. Greenland and Tibet
are therefore cold for their latitude. This matches a land classification, which
also sees the real surface.

The values cover 1961 to 1990. The world has warmed since then. The polar
northern bands have warmed by more than 2 degrees. A target profile built from
this table describes the late twentieth century, not the present.

Both grids come from the same research unit. They are not two independent
sources. I did not find a second source with which to cross-check the numbers.

## 9. How to repeat the computation

Download the two published grids.[^1] [^2] Read every land cell of the
10-minute file. Group the cells by latitude band. Take the mean of each month,
weighted by the cosine of the latitude of the cell. Take the annual mean as the
mean of the 12 monthly means. The grid file format is fixed width. The first two
fields give the latitude and the longitude. The next 12 fields give the monthly
means in degrees Celsius.

## References

[^1]: New, M., Lister, D., Hulme, M., Makin, I. (2002). A high-resolution data set of surface climate over global land areas. Climate Research, volume 21, pages 1 to 25. The grid file. https://crudata.uea.ac.uk/cru/data/hrg/tmc/grid_10min_tmp.dat.gz
[^2]: Jones, P.D., New, M., Parker, D.E., Martin, S., Rigor, I.G. (1999). Surface air temperature and its changes over the past 150 years. Reviews of Geophysics, volume 37, pages 173 to 199. The absolute temperature grid for 1961 to 1990. https://crudata.uea.ac.uk/cru/data/temperature/absolute_v5.nc
