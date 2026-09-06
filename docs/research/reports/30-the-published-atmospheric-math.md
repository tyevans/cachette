# The Published Atmospheric Math

Research report 30. It finds the published curves, constants and models that
drive weather formation, states each one so that an engineer can implement it
without a textbook, and gives an integer form for each with the error stated.
It judges the weather design of report 29 against that literature. Prepared
6 September 2026.

Cachette is a world simulation engine. The core is Rust. The control plane is
Python. Simulated state holds no floating point number, and every aggregate
combines exactly in any order.[^1] The engine holds one property it cannot
recover once lost, which is determinism.[^2] Weather is a field over a lattice
of cells, and a cell covers between one and 1024 tiles.[^3]

**This author ran no build, no test and no benchmark.** The author ran seven
short arithmetic scripts to measure the error of each approximation this report
proposes. Every error figure below comes from one of those scripts. Every
physical constant comes from a cited source. The author marks each claim that
the author could not verify.

---

## 0. The conclusion

**Four findings matter more than the rest.**

**1. The world is 330 km across, and that fact chooses the physics.** The scale
register fixes the tile edge at 80 m and the world at 16.7 million tiles.[^4] A
level 1 cell of 32 tiles is 2.56 km across. A tick is 2.4 minutes of simulated
time.[^4] **A box 330 km across is smaller than one mid-latitude storm and
holds about three degrees of latitude.** Hadley cells, trade winds, the
Coriolis parameter and the Köppen belts all live at 1000 km and above. They do
not exist inside this box. Orographic lift, rain shadow and the land-sea
contrast do live at 2 to 100 km, and they are the published mechanisms that
produce a desert and a rain forest at this size.

**2. The project holds two incompatible readings of its own map, and nobody has
chosen.** The scale register says the map is a 330 km region. The weather model
reads the row axis as a latitude that runs from pole to pole.[^5] Those two
readings disagree by a factor of about 60. **Both are defensible for a game and
they demand opposite weather models.** This report cannot settle it. Section 9
states the choice and what each branch costs.

**3. Report 29 proposes one saturation curve that is wrong by a factor of two
at the cold end.** It states that the capacity of the air doubles at a fixed
temperature step, and that the doubling is therefore exact in integers.[^3]
**The published curve does not double at a fixed step.** The doubling width
runs from 6.7 K at −40 °C to 13.9 K at +50 °C. Even with the best possible
choice of constants, a fixed doubling is wrong by a factor of 1.33 over −40 to
+45 °C. Section 1 gives an integer form that is accurate to 0.09 percent for
about the same cost.

**4. The banded circulation cannot emerge from the current model, and the
literature says why.** The Ferrel cell is thermally indirect and eddy
driven.[^6] An axisymmetric model with no baroclinic eddies does not produce
it.[^7] A single-layer field with a pressure gradient and a deflection term is
weaker than an axisymmetric model. **So the belt of subtropical deserts at 30
degrees will never appear on its own.** If the project wants that belt, it must
impose it. Section 5 says how, for one added term.

**Two smaller findings are worth stating.**

The pass count schedule in the code is physically correct and the pass ceiling
costs nothing. Section 6 gives the arithmetic. The code and report 29 both
treat the ceiling as a compromise, and at an 80 m tile and a 2.4 minute tick it
is not one.

The annual mean insolation against latitude is a quadratic in the sine of the
latitude, to within 2.9 percent. That is exact integer arithmetic with no
table, no divide and no trigonometry. Section 4 gives it.

---

## 1. Saturation vapour pressure against temperature

### 1.1 The published curve

**Use the Alduchov and Eskridge form of the Magnus equation.** It is the
recommended set for meteorology.[^8] The equation gives the saturation vapour
pressure over liquid water.

```
e_s(T) = 6.1094 * exp( 17.625 * T / (243.04 + T) )     [hPa, T in degrees C]
```

The constants hold from −40 °C to +50 °C, and the error against an accurate
reference stays inside about 0.4 percent.[^8] [^9]

The equation is a fitted form of the Clausius-Clapeyron relation. That relation
states how the saturation pressure changes with temperature.[^10]

```
d(e_s)/dT = L_v * e_s / (R_v * T^2)
L_v = 2.5e6 J/kg          latent heat of vaporisation
R_v = 461 J/(K kg)        gas constant for water vapour
```

The well-known consequence is that the capacity of the air rises by about
7 percent for each kelvin near room temperature.[^10]

Alduchov and Eskridge give a second equation for saturation over ice, valid
from −80 °C to 0 °C.[^8] **The author could not read its constants from an
accessible source, and does not state them.**

### 1.2 What the curve does, in numbers

The author computed this table from the equation above.

| T (°C) | e_s (hPa) | Local doubling width (K) |
|---|---|---|
| −40 | 0.190 | 6.67 |
| −30 | 0.511 | 7.34 |
| −20 | 1.258 | 8.05 |
| −10 | 2.868 | 8.79 |
| 0 | 6.109 | 9.56 |
| 10 | 12.260 | 10.36 |
| 20 | 23.334 | 11.20 |
| 30 | 42.366 | 12.06 |
| 40 | 73.747 | 12.96 |
| 50 | 123.606 | 13.90 |

**The doubling width is not constant. It changes by a factor of 2.1 across the
range.**

### 1.3 Where report 29 is wrong

Report 29 states that the capacity doubles at a fixed step, and builds the
integer form on that.[^3] The author tested the claim by search over every
doubling width from 6.0 K to 18.0 K and every anchor temperature from −30 °C to
+30 °C.

- Over −40 °C to +45 °C, the best fixed doubling is 9.5 K anchored at −29 °C.
  **It is still wrong by a factor of 1.33 at the worst point.**
- Over −20 °C to +35 °C, the best fixed doubling is 10.0 K anchored at +26 °C.
  It is wrong by a factor of 1.11 at the worst point.

**So a fixed doubling is acceptable in a temperate band and fails at the poles
by a third.** The cold end is exactly where a player expects tundra and ice, and
it is exactly where the current model already fails.

### 1.4 The integer form to use instead

**The Magnus exponent is a rational function of the temperature.** Take the
base 2 logarithm of the equation.

```
log2( e_s(T) ) = log2(6.1094) + 25.42750 * T / (243.04 + T)
```

The constant 25.42750 is 17.625 divided by the natural logarithm of 2. **So the
exponent needs one fixed-point divide and no table at all.** Only the fraction
of the exponent needs a table.

The procedure is this.

1. Compute `L = 25.42750 * T / (243.04 + T)` in Q16.16. One multiply and one
   divide.
2. Split `L` into the whole part `n` and the fraction `f`. One shift and one
   mask.
3. Read `2^f` from a small table of powers, with a linear interpolation.
4. Shift left by `n`, and scale by the drop count that the model gives 0 °C.

The author measured the error of this scheme against the exact equation, over
−70 °C to +58 °C, at a step of 0.1 K.

| Table entries for the fraction | Worst relative error |
|---|---|
| 8 | 0.094 percent |
| 16 | 0.023 percent |
| 32 | 0.006 percent |

**An 8-entry table gives 0.09 percent.** The error does not grow with the
temperature range, because the range lives in the whole part of the exponent.
The whole part is exact. Report 29 asks for a 256-entry table as the
specification and a fixed doubling as the implementation.[^3] **Make the
arithmetic above the implementation, and keep a table as the test.** The
arithmetic is then correct, and the table still fails the build when the two
disagree.

### 1.5 The temperature scale the engine needs

**The engine has no temperature scale, and every published formula needs one.**
The warmth of a cell is an abstract count from 0 to 256.[^5] A published curve
cannot read that.

**Declare one linear map, in one place.** The author recommends a half kelvin
for each warmth unit, with warmth zero at −70 °C. The range is then −70 °C to
+58 °C, the step is a shift, and the map covers every planetary surface
temperature. The table below gives the capacity at that scale, with 0 °C set to
2048 drops to match the current constant.[^5]

| Warmth | T (°C) | e_s (hPa) | Drops, at 2048 for 0 °C |
|---|---|---|---|
| 0 | −70.0 | 0.0049 | 2 |
| 20 | −60.0 | 0.0189 | 6 |
| 60 | −40.0 | 0.1897 | 64 |
| 100 | −20.0 | 1.2578 | 422 |
| 140 | 0.0 | 6.1094 | 2048 |
| 180 | 20.0 | 23.3344 | 7822 |
| 220 | 40.0 | 73.7472 | 24720 |
| 255 | 57.5 | 178.0179 | 59672 |

The whole range fits a 32-bit plane with a large margin, and it needs 32 bits
because the ratio between the ends is about 36,000 to 1.

**This measures the size of the present defect.** The model holds one
saturation of 2048 drops that no temperature reads.[^5] The published capacity
at 30 °C is 6.9 times the capacity at 0 °C, and the capacity at −30 °C is 0.084
times it. **So one constant stands in for a quantity that varies by a factor of
about 80 between a tropical cell and a polar cell.**

---

## 2. Lapse rate

### 2.1 The three published rates

**The dry adiabatic lapse rate is a derived constant.**

```
DALR = g / c_p = 9.81 / 1004 = 9.8 K/km
```

**The environmental lapse rate is the observed mean.** The standard atmosphere
uses 6.5 K/km. It is an average and not a law.

**The saturated adiabatic lapse rate varies with the temperature, and that is
the part that matters here.** Condensation releases latent heat, which slows
the cooling. Warm air holds more water, so it releases more heat, so it cools
more slowly.[^11]

| Air temperature | Saturated rate |
|---|---|
| +20 °C | about 4 K/km |
| 0 °C | about 6 K/km |
| −40 °C | close to 9 K/km |

The published range is 3.6 to 5.5 K/km in ordinary conditions, and it reaches
the dry rate at very cold temperatures.[^11] The author verified the endpoints
and interpolated the 0 °C row, and marks that row as an interpolation and not a
measurement.

### 2.2 The integer form

**Use a three-piece linear fit in the temperature, and clamp at both ends.**

```
SALR(T) in units of 1/1024 K per km:
   T <= -40 C  ->  9216            (9.0 K/km)
  -40 < T <= 0 ->  9216 - 76*(T+40)
   0 < T <= 20 ->  6144 - 102*T
   T > 20 C    ->  4096            (4.0 K/km)
```

The slopes are integers and the whole calculation is two multiplies. The error
against the three published points is under 0.1 K/km.

### 2.3 What this buys at 80 m tiles, and what it does not

**A lapse rate matters only where the ground has relief.** Over 2.56 km a cell
can hold 500 m of relief in real terrain, which is 3.3 K at the saturated rate
and 4.9 K at the dry rate. That is a visible temperature difference and it
should be in the model.

**The distinction between the dry and the saturated rate buys less.** It changes
the cooling across one ridge by about 2 K. A player sees the rain shadow, and a
player does not see 2 K. **Use one rate, and use the saturated one**, because
the air that matters for rain is the air that is condensing. Section 10 lists
this as fidelity that buys nothing.

---

## 3. Orographic precipitation

### 3.1 The published models

**Two families exist, and the simpler one is enough.**

**The upslope model** assumes that the precipitation is proportional to the
condensation in a column that the terrain lifts.[^12] The lift is the dot
product of the wind with the terrain gradient. The condensed water falls at
once. The form is this.

```
P = C * rho * q_s * ( U . grad h )        when the product is positive
P = 0                                     when the product is negative
```

Here `q_s` is the saturation specific humidity, `U` is the horizontal wind,
`grad h` is the terrain gradient, and `C` is a precipitation efficiency between
0 and 1. The model dates from the 1970s.[^13]

**The linear theory of Smith and Barstad** adds airflow dynamics, advection of
the condensed water, and evaporation on the lee side.[^14] It holds five length
scales: the mountain width, a buoyancy wave scale, the moist layer depth, and
two advection distances for the condensed water.[^14] It uses two time
constants. The published example sets both the conversion time and the fall
time to 1000 s.[^14] The precipitation from a source region spreads downwind
over a distance equal to the wind speed times the cloud time constant.[^14]

The model solves in the Fourier domain in its published form.[^14] **A Fourier
solve is not available here**, because it needs floating point and a global
transform. The useful part is the two time constants.

### 3.2 What to implement

**Implement the upslope lift, and add the two Smith and Barstad time constants
as integer delays.**

The engine already lifts air over a climb and already carries cloud water.[^5]
The published contribution is the delay. A cloud does not fall where it forms.
It forms with a conversion time and it falls with a fall time.

At a 1000 s time constant and a 144 second tick, a cloud converts over about
7 ticks and falls over about 7 more.[^4] [^14] **In the code that is a share of
about one seventh for each tick, on each of two transfers.** That is two
shifts. It is the whole mechanism, and it is what puts the rain on the lee side
of the ridge rather than on the crest.

**Report 29 already proposes a separate cloud plane, and the literature agrees
with it.** Its reason is that a storm must travel.[^3] The published reason is
the same, stated as a time constant.

**What the model needs as input, for each cell, for each tick:**

- The wind, as a vector. The engine holds it.
- The terrain gradient toward each neighbour. The engine holds the mean land
  height for each cell.[^5]
- The saturation capacity at the temperature of the cell. Section 1 gives it.
- A precipitation efficiency. This is a tuning constant with no published
  value that the author could verify. State it in the balance register.

### 3.3 This is the model that fits the world size

**Smith and Barstad works at mountain widths of 10 to 100 km.** The world is
330 km across and a level 1 cell is 2.56 km.[^4] The model resolves a ridge
across 4 to 40 cells. **That is the one published model in this report whose
own scale matches this engine.** Every other model in this report is either
smaller than a cell or larger than the world.

---

## 4. Insolation against latitude and season

### 4.1 The published geometry

**The daily mean insolation at the top of the atmosphere has a closed
form.**[^15]

```
Q = (S0 / pi) * ( H * sin(phi) * sin(delta) + cos(phi) * cos(delta) * sin(H) )
cos(H) = -tan(phi) * tan(delta)
```

Here `phi` is the latitude, `delta` is the solar declination, `H` is the half
day length in radians, and `S0` is the solar constant. `H` is `pi` for polar
day and `0` for polar night.

The day length in hours is `24 * H / pi`.[^15]

**The constants.**

| Constant | Value | Source |
|---|---|---|
| Solar constant | 1361 W/m² | IAU 2015 resolution[^16] |
| Obliquity | 23.44 degrees | standard |
| Eccentricity | 0.0167 | standard |

The declination follows the obliquity through the year. The simple form is
Cooper's equation.[^17]

```
delta = 23.45 degrees * sin( 360 * (284 + n) / 365 )      n = day of year
```

The Spencer Fourier series is accurate to about 0.01 degrees, and Cooper's
equation is less accurate than that.[^17] **The author could not find a stated
worst-case error for Cooper's equation and does not assert one.**

### 4.2 What is worth keeping and what is noise

**Keep the obliquity. Drop everything else.**

- **Obliquity.** It creates the season. Without it there is no season at all.
- **Eccentricity.** The distance changes by 1.67 percent either side of the
  mean, so the irradiance changes by about 3.4 percent either side, or 6.9
  percent between perihelion and aphelion.[^16] **That is one third of one
  season swing and it points the same way all year.** It is noise here.
- **Precession and the Milankovitch terms.** They act over tens of thousands of
  years. The season period in the engine is 2048 ticks, which is 3.4 simulated
  days.[^4] [^5] These terms are meaningless.
- **The equation of time and the hour angle.** A tick is 2.4 minutes but the
  engine has no day and night cycle in the weather. The daily mean is the right
  quantity.

### 4.3 The finding that contradicts the current model

The author computed the daily mean insolation from the equation above, at each
latitude, at three points in the year.

| Latitude | June solstice | Equinox | December solstice | Annual mean |
|---|---|---|---|---|
| 0 | 397.5 | 433.2 | 397.5 | 415.5 |
| 20 | 470.7 | 407.1 | 285.6 | 392.8 |
| 40 | 498.9 | 331.9 | 150.9 | 328.3 |
| 60 | 492.4 | 216.6 | 23.6 | 236.3 |
| 80 | 533.2 | 75.2 | 0.0 | 178.1 |
| 90 | 541.4 | 0.0 | 0.0 | 172.3 |

All values are in watts for each square metre.

**At the June solstice the pole receives more daily energy than the equator.**
It receives 541 against 398, which is 36 percent more. The curve is nearly flat
from 30 degrees to the pole and it rises at the end.

**The current model cannot produce that shape.** It reads how far a cell sits
from where the sun stands, and it applies a fall that is smooth at both
ends.[^5] Report 29 keeps that design.[^3] **Any such model peaks at the sun
latitude and falls away from it. The real curve does not.** The polar day is
the cause, and no distance function holds a polar day.

This is visible in a game. A summer pole with long days is a real and
recognisable thing, and the current model gives a cold pole all year.

### 4.4 The integer forms

**Two forms are worth having, and they answer different questions.**

**Form A, for the annual mean: a quadratic in the sine of the latitude.** The
Budyko and Sellers energy balance models write the mean insolation profile as
a second Legendre polynomial.[^18]

```
S(x) / S_mean = 1 + s2 * P2(x)
P2(x) = (3 * x * x - 1) / 2
x = sin(latitude)
```

The author fitted `s2` against the exact curve by least squares, with the
correct area weighting.

```
s2 = -0.477
worst relative error over the whole globe: 2.9 percent
global mean: 340.2 W/m2, which is the solar constant divided by four
```

**This is a polynomial in the sine of the latitude, with no table, no divide
and no trigonometry.** In Q16.16 it is two multiplies and one add. The author
did not verify the published value of `s2` from a primary source, and states
the fitted value as the author's own derivation.

**Form B, for the season: one table, built once.** The daily insolation is a
pure function of the latitude and the day of the year. **So tabulate it once at
world build and read it for ever after.** A table of 256 latitude bands by 64
season steps, at 16 bits for each entry, is 32 kilobytes. It costs no
arithmetic at all in the pass.

**Build the table from the sine table that the arithmetic module already
holds.** That module holds a quarter turn of a sine as integer data, and it
states that the table is data rather than a polynomial so that every target
gives the same value.[^19] The insolation table must follow that precedent. It
must be a checked-in literal, or it must be built by integer arithmetic from
the existing sine. **It must not be built with floating point at run time**,
because the result then enters simulated state.[^1]

---

## 5. Large-scale circulation

### 5.1 The published structure

**Three cells stand in each hemisphere.**[^20]

| Cell | Latitudes | Surface motion | Surface wind |
|---|---|---|---|
| Hadley | 0 to 30 | rising at the equator, sinking at 30 | easterly trades |
| Ferrel | 30 to 60 | sinking at 30, rising at 60 | westerlies |
| Polar | 60 to 90 | rising at 60, sinking at the pole | polar easterlies |

The pressure belts follow. A low stands at the equator, a high at 30 degrees, a
low at 60 degrees, and a high at each pole.[^21] **The author could not verify
the pressure in hectopascals for each belt from an accessible source, and does
not state numbers.** The global mean is near 1013 hPa.[^21]

The Coriolis effect bends the return flow. It deflects to the right in the
northern hemisphere and to the left in the southern one.[^20]

**The subtropical high at 30 degrees is where the world's deserts are.** The
descending air warms and dries. That is the mechanism that puts the Sahara, the
Arabian, the Kalahari and the Australian deserts at the same latitude.

### 5.2 The Coriolis parameter

```
f = 2 * Omega * sin(phi)
Omega = 7.2921e-5 rad/s
f = 1.4584e-4 * sin(phi)    per second
```

| Latitude | f (per second) | Inertial period |
|---|---|---|
| 10 | 2.53e-5 | 68.9 hours |
| 30 | 7.29e-5 | 23.9 hours |
| 45 | 1.031e-4 | 16.9 hours |
| 60 | 1.263e-4 | 13.8 hours |
| 90 | 1.4584e-4 | 12.0 hours |

**The inertial period is the useful number here, and it derives the deflection
constant.** A parcel with no other force turns one full circle in the inertial
period. So the turn in one tick is this.

```
turn per tick = 360 degrees * tick_seconds / inertial_period
              = 360 * 144 / (2 * pi / f)
```

At 45 degrees that is 0.85 degrees for each tick. The current model deflects by
one third of a sixth of a turn, which is 20 degrees.[^5] **The current
deflection is about 24 times the published rate at 45 degrees latitude.**

That is not automatically a defect, because the model compresses time and space
by an unknown amount. **It is a defect that nobody wrote down the derivation.**
The project rule requires a derivation and not a written-down number.[^22] The
formula above is the derivation, and it takes the latitude as a parameter.

**Report 29 asks for one change to the turn: carry the sign of the
latitude.**[^3] The literature agrees, and the formula above holds it already,
because the sine of a negative latitude is negative.

### 5.3 The judgement: impose the bands, do not wait for them

**The bands cannot emerge from this model, and the reason is published.**

The Ferrel cell is thermally indirect.[^6] It runs backwards against the
temperature gradient. Baroclinic eddies drive it, through the convergence of
their momentum flux.[^6] **An axisymmetric model with no eddies does not
produce it.**[^7] The reference axisymmetric model of Held and Hou gives the
latitudinal extent of the tropical overturning and nothing beyond it.[^7]

The engine's model is weaker than an axisymmetric model. It is a single layer.
It has no vertical structure, so it has no baroclinic instability, so it has no
eddies. **So the Ferrel cell will never form, the mid-latitude westerlies will
never form, and the subtropical high at 30 degrees will never form.**

The subtropical high is the thing the owner wants. It is the mechanism that
makes a desert belt.

**So impose the bands, as a latitude term added to the pressure.** The engine
already derives the pressure from the temperature and already accelerates the
wind down the pressure gradient.[^5] **Add one latitude-banded offset to the
pressure before the gradient reads it.** The offset is a fixed function of the
row, so it is a table of one entry for each row, built once. It costs one load
and one add for each cell.

The result is that the three surface wind belts appear, the descending belt at
30 degrees appears, and the terrain still perturbs all of it. **The circulation
is imposed and the weather is still emergent.** That is what a limited-area
weather model does: it takes the large scale from outside and computes the
small scale itself.[^23]

**This is the one place where this report contradicts report 29 on design and
not on arithmetic.** Report 29 keeps the wind design and says the banded
prevailing winds follow from the sign of the deflection alone.[^3] They do not.
The sign of the deflection gives two hemispheres that turn opposite ways. It
does not give three belts in each hemisphere, because three belts need three
pressure extremes, and one monotone temperature profile has two.

### 5.4 But read section 9 first

**All of section 5 applies only if the map is a planet.** If the map is a 330 km
region, the whole of it sits inside one belt and none of this belongs in the
engine at all.

---

## 6. Advection and diffusion

### 6.1 The published stability condition

**The donor-cell upwind scheme is conditionally stable, and the condition is
the Courant condition.** The first-order upwind method is equivalent to a
semi-Lagrangian scheme with linear interpolation and a Courant number below
one.[^24] Above one, it fails.

**The engine already enforces the condition at build time, and this report
found no defect there.** The transport code holds a compile-time assertion that
the shares a cell sends to its six neighbours add to less than one, at any
wind.[^5] **That assertion is the Courant condition.** A cell never sends more
water than it holds, so the scheme is positive and conservative by
construction, and it cannot go unstable.

The brief asks which schemes are unconditionally stable and cheap. **The honest
answer is that the engine does not need one.**

### 6.2 Semi-Lagrangian, and why to reject it

Semi-Lagrangian advection carries no Courant restriction. Published work runs
it stably at Courant numbers of 2 to 5, and recent conservative forms reach
100.[^24] **Reject it here, for three reasons.**

- It is not conservative by construction.[^24] Conservation is an accepted
  decision of this project.[^25]
- It reads a cell at an arbitrary upstream position, which is a gather. The
  target platform baseline holds no gather instruction.[^3]
- It needs an interpolation, and an integer interpolation over a long trajectory
  loses mass in a way that a donor cell does not.

**Report 29 is right to keep donor cell, and it gives the right reason.**[^3] Its
reason is that a varying donor distance is a gather. The published reason is the
same fact from the other side.

### 6.3 The pass count, and why the ceiling costs nothing

**The pass count is the price of exact conservation, and the price is already
paid in full.** One transport pass carries water at most one cell.[^5] So the
distance in one tick is the pass count. The code doubles the count for each
halving of the cell, and caps it at 32.[^5]

The author combined the pass schedule with the scale register.[^4]

| Cell side | Cell width | Passes | Distance in one tick | Implied ceiling speed |
|---|---|---|---|---|
| 32 tiles | 2560 m | 4 | 10.2 km | 71 m/s |
| 16 tiles | 1280 m | 8 | 10.2 km | 71 m/s |
| 8 tiles | 640 m | 16 | 10.2 km | 71 m/s |
| 4 tiles | 320 m | 32 | 10.2 km | 71 m/s |
| 2 tiles | 160 m | 32 | 5.1 km | 36 m/s |
| 1 tile | 80 m | 32 | 2.6 km | 18 m/s |

**The schedule holds the transport speed constant until the ceiling bites at
four tiles. Below that the ceiling still leaves 18 m/s.** A strong surface wind
is about 10 m/s. **So the pass ceiling costs no physical realism at any pitch
the engine supports.** The code documents the ceiling as a compromise that
carries water more slowly, and report 29 accepts that framing.[^3] [^5] At an
80 m tile and a 2.4 minute tick, it is not a compromise.

### 6.4 Diffusion

**A share-based mixing on a hex lattice is stable when six times the share does
not exceed one.** Each cell gives a share to each of six neighbours and keeps
the rest. If six shares exceed one, the cell gives away more than it holds and
the field oscillates. That is the hex form of the standard explicit diffusion
limit.

**Report 29 states that mixing is not optional, and it gives a measurement.**[^3]
The literature supports the reasoning: pure advection with no diffusion produces
filaments at the grid scale in any first-order scheme.

### 6.5 The boundary

**Report 29 is right that an advecting field needs an outer region, and the
published name is a relaxation zone.** Limited-area weather models relax the
interior toward an externally prescribed flow near the boundary.[^23] The method
comes from Davies.[^26] **A published relaxation zone is 5 to 9 grid points
wide.**[^23]

Report 29 proposes a margin equal to the transport pass count, which is 4 cells
at the level 1 pitch and 32 at the tile pitch.[^3] **The published width agrees
with the coarse case and is far below the fine one.** The author recommends
taking the published range as an upper bound on the useful width, because the
relaxation zone in a real model exists to absorb spurious waves rather than to
supply upwind, and a wider zone does not absorb more.

**Report 29's derivation and the published width disagree, and report 29's
derivation grows without limit.** Set the margin to the smaller of the pass
count and about 9 cells.

---

## 7. Evaporation

### 7.1 The published bulk formula

**Use the bulk aerodynamic formula.** It is the standard form in every
atmospheric model.[^27]

```
E = rho_a * C_E * U * ( q_s(T_surface) - q_air )
```

Here `E` is the evaporation in kilograms for each square metre for each second,
`rho_a` is the air density, `C_E` is the moisture transfer coefficient, `U` is
the wind speed at the reference height, `q_s` is the saturation specific
humidity at the surface temperature, and `q_air` is the specific humidity of
the air.[^27]

The coefficient `C_E` is called the Dalton number.[^27] It is of the order of
1e-3. **The author could not verify a single recommended value for `C_E`, and
found a nearby drag coefficient of 1.5e-3 in one source.**[^27] Treat the value
as a tuning constant of the right order.

The older engineering form is Dalton's equation.[^28]

```
E = f(U) * ( e_s - e_a )
f(U) = a + b * U
```

Published wind functions include `f(U2) = (2.36 + 1.67 * U2) * A^-0.05` in
millimetres for each day for each kilopascal, with `U2` in metres per second at
2 m and `A` the water surface area in square metres.[^28]

### 7.2 What this says about the model

**Both published forms have the same three factors: the wind, the deficit, and
a coefficient.** The deficit is the saturation at the surface temperature less
what the air holds.

**Report 29 proposes exactly that, and the literature confirms it.**[^3] Its
phase function reads the deficit once and moves water in one direction or the
other. The published formula is the positive direction of the same thing.

**Report 29 misses one factor: the wind.** Its evaporation is a share of the
deficit, bounded by what the surface holds.[^3] The published formula multiplies
by the wind speed. **Add the wind.** It costs one multiply, the engine already
holds the wind, and it produces a behaviour a player recognises: a still humid
day evaporates nothing, and a windy day dries the ground.

**The current model is worse than either.** Its lift is proportional to the heat
of the cell and reads no deficit at all.[^5]

### 7.3 The integer form

```
evaporation = (deficit * wind_speed * C_E_numerator) >> C_E_SHIFT
capped at what the surface holds
```

One multiply, one multiply, one shift, one minimum. The deficit comes from
section 1. The surface is unbounded over open water and bounded over ground.

---

## 8. The climate classification

### 8.1 Köppen and Geiger, with numbers

**Köppen is the right target, because it states checkable numbers.** The scheme
divides land climates into five groups. Every group except B is defined by
temperature alone.[^29] Group B is defined by dryness.[^29] The criteria below
come from the standard statement of the scheme.[^29] The modern reference map
is Peel and others.[^30]

**The group tests.**

| Group | Test |
|---|---|
| A tropical | every month averages 18 °C or more |
| B arid | annual precipitation below the threshold below |
| C temperate | coldest month between 0 °C and 18 °C, and at least one month above 10 °C |
| D continental | coldest month below 0 °C, and at least one month above 10 °C |
| E polar | every month below 10 °C |

**The aridity threshold for group B**, in millimetres for each year.

```
threshold = 20 * MAT + K
K = 280   when 70 percent or more of the rain falls in the high sun months
K = 140   when 30 to 70 percent falls in the high sun months
K = 0     when under 30 percent falls in the high sun months

BW desert   when annual precipitation is under 50 percent of the threshold
BS steppe   when it is 50 to 100 percent of the threshold
h hot       when the mean annual temperature is 18 C or more
k cold      otherwise
```

Here `MAT` is the mean annual temperature in degrees Celsius. **This whole test
is integer arithmetic already.**

**The A subtypes.**

| Subtype | Test |
|---|---|
| Af rain forest | every month has at least 60 mm |
| Am monsoon | driest month under 60 mm, but at least `100 - MAP/25` |
| Aw savanna | driest month under 60 mm and under `100 - MAP/25` |

Here `MAP` is the mean annual precipitation in millimetres.

**The C and D second letter.**

| Letter | Test |
|---|---|
| f no dry season | neither test below passes |
| w dry winter | the driest winter month is under one tenth of the wettest summer month |
| s dry summer | the wettest winter month is at least three times the driest summer month, and the driest summer month is under 40 mm |

**The C and D third letter.**

| Letter | Test |
|---|---|
| a hot summer | warmest month 22 °C or more, and four or more months above 10 °C |
| b warm summer | every month under 22 °C, and four or more months above 10 °C |
| c cold summer | one to three months above 10 °C |
| d very cold winter | coldest month under −38 °C, group D only |

**The E subtypes.**

| Letter | Test |
|---|---|
| ET tundra | warmest month between 0 °C and 10 °C |
| EF ice cap | every month under 0 °C |

### 8.2 Why this is worth as much as the generative math

**A classification gives a checkable statement that a test can assert.** The
project needs one, because the determinism tests prove that a run repeats and
say nothing about whether it was right.[^31]

**Two tests become possible.**

- **A distribution test.** Run a world for one simulated year, classify each
  cell, and assert that the share of each class sits inside a stated band. A
  world with 90 percent desert fails. A world with no desert fails.
- **A seeding check.** Generate a map from a projected climate, run the engine
  on it, classify the result, and compare. If the engine drifts far from the
  seed, either the seed or the engine is wrong.

**Köppen needs monthly aggregates, and the engine has no month.** The season
period is 2048 ticks.[^5] Divide the season into 12 equal parts and treat each
as a month. The aggregates are integer sums and minimums over a season, which
the pyramid can hold.

### 8.3 The alternative, and why to reject it

**Whittaker's biome diagram plots the mean annual temperature against the mean
annual precipitation and draws the biomes as regions.**[^32] It is a two-axis
lookup, which suits an integer table exactly.

**Reject it as the primary target.** Its boundaries are hand-drawn curves and
not stated numbers.[^32] The author could not find a numerically stated set of
boundaries from a reliable source. Köppen states its boundaries as arithmetic,
so a check can hold them and a reviewer can find a violation.

**Keep Whittaker for the display.** A two-axis picture of where each cell sits
is a good overlay for a watcher, and it does not need exact boundaries.

---

## 9. The scale question the project must answer

**This is the finding with the largest consequence, and this report cannot
settle it.**

**The scale register says the map is a region.** The tile edge is 80 m and the
world is 330 km across.[^4] At that size the map holds about three degrees of
latitude. The author computed the annual mean insolation across three degrees at
mid-latitude, and it changes by about 4 percent. **A 4 percent change produces no
climate zones at all.**

**The weather model says the map is a planet.** It reads the row axis as a
latitude, it swings a sun across it, and it uses a season swing of 80 heat units
against a heat ceiling of 256.[^5] That is a pole-to-pole model.

**Both are defensible and the project must choose.** The choice decides which
half of this report to implement.

**Branch A, the map is a region.** Delete the latitude term and the banded
circulation. Keep the season as a whole-map swing with no latitude gradient.
**The climate then comes from the terrain and the sea, which is exactly right at
this size.** Orographic lift makes the wet side, the rain shadow makes the dry
side, and the distance from the sea makes the continental interior. Sections 1,
2, 3, 6 and 7 apply in full. Section 5 goes away. Section 8 still works, because
Köppen classifies a region as well as a planet.

**Branch B, the map is a planet.** Keep the latitude, and accept that the tile
edge is a game unit and not 80 m. **Then the scale register is wrong for the
weather, and the weather must state its own pitch.** Section 5 applies, and
section 5.3 says to impose the bands. Section 3 still applies, because the
mountains are still mountains.

**A third option exists and the author recommends it.** **Take the latitude from
a world parameter rather than from the map.** Give the world a centre latitude
and a latitude span. A 330 km map at a span of three degrees is branch A. The
same code at a span of 180 degrees is branch B. **The two branches then differ by
one constant instead of by a model**, and the owner can turn the dial and look at
the result. The bands of section 5 become a table over the row that the span
scales, and at a small span the table is flat and costs nothing.

This is a decision that needs a record, and the record must state the span as a
parameter that a blocker governs rather than invent it.[^22]

---

## 10. Report 29, judged against the literature

| Report 29 proposal | Verdict | Why |
|---|---|---|
| Replace the lift, fall and back-lift with one deficit-driven phase function | **Correct, and it matches the published bulk formula** | The bulk aerodynamic formula is exactly a deficit multiplied by a coefficient[^27] |
| The capacity of the air doubles at a fixed temperature step | **Wrong** | The published doubling width runs from 6.7 K to 13.9 K. The best fixed doubling is wrong by a factor of 1.33 over −40 to +45 °C |
| A 256-entry table as the specification, arithmetic as the implementation | **Right shape, wrong arithmetic** | Use the rational exponent form of section 1.4. It is accurate to 0.09 percent |
| A separate cloud plane that advects and rains a share for each tick | **Correct** | This is the Smith and Barstad conversion time and fall time, which are published as 1000 s each[^14] |
| Keep the wind: pressure gradient, drag, turn, ceiling | **Correct for the local wind** | |
| The turn carries the sign of the latitude | **Correct** | The Coriolis parameter carries the sine of the latitude, which is signed |
| Banded prevailing winds follow from the sign of the turn alone | **Wrong** | Three belts need three pressure extremes. One monotone temperature profile has two. The Ferrel cell is eddy driven and cannot emerge here[^6] [^7] |
| Keep the insolation: distance from where the sun stands, with a smooth fall | **Wrong shape** | At the solstice the pole receives 36 percent more daily energy than the equator. No distance function holds a polar day |
| Keep the land and sea temperature lag | **Correct** | This is the mechanism behind continentality, and it is what makes a coast interesting |
| Keep donor-cell advection to the six neighbours | **Correct** | The Courant condition is already a compile-time assertion in the code |
| Mixing is not optional | **Correct** | First-order advection with no diffusion produces grid-scale filaments |
| The pass count stays a derived function of the pitch | **Correct, and the ceiling costs nothing** | 32 passes at an 80 m tile still transports at 18 m/s, above a strong surface wind |
| An advecting field needs a margin, of the pass count in width | **Right idea, wrong width** | Published relaxation zones are 5 to 9 cells wide.[^23] Cap the margin near that |
| Reject a third dimension | **Correct at this scale, and it has a cost** | With no vertical structure there are no baroclinic eddies, so the Ferrel cell can never emerge. Section 5.3 |
| Reject ocean currents | **Correct** | At a 330 km world there is no ocean gyre to model |
| Reject storms as objects | **Correct** | |
| Reject relative humidity as stored state | **Correct** | A ratio does not conserve. This is standard practice in every atmospheric model |
| Evaporation as a share of the deficit | **Incomplete** | The published formula multiplies by the wind speed. Add it |

---

## 11. What to reject, and where fidelity buys nothing

**A player sees a pattern. A player does not see a value.** Each item below is
published physics that this engine should leave out.

| Rejected | Why it buys nothing here |
|---|---|
| The eccentricity of the orbit | 3.4 percent either side of the mean, and it points the same way all year.[^16] One third of one season swing |
| Precession and the Milankovitch terms | They act over tens of thousands of years. The season period is 2048 ticks |
| The Spencer Fourier declination | It is accurate to 0.01 degrees.[^17] Cooper's equation is enough, and a built table is better than either |
| The distinction between the dry and the saturated lapse rate | It changes the cooling across one ridge by about 2 K. Use the saturated rate alone |
| The Smith and Barstad Fourier solve | It needs floating point and a global transform. Take its two time constants and leave the rest |
| Semi-Lagrangian advection | It is not conservative and it needs a gather. The donor cell is already stable here |
| A vertical layer count above one | It multiplies the whole stage by the layer count, and buys a Ferrel cell that section 5.3 gets for one table |
| Ocean currents | No gyre fits inside 330 km |
| The ice saturation curve | It matters below −40 °C. Clamp the liquid curve there instead, until a player can see the difference |
| An exact value for the moisture transfer coefficient | It is a tuning constant of order 1e-3, and no player can tell 1.2e-3 from 1.5e-3 |

---

## 12. What the author could not verify

The author states each of these plainly rather than asserting a number.

- **The Alduchov and Eskridge constants for saturation over ice.** The paper
  gives a second equation valid from −80 °C to 0 °C.[^8] The author could not
  read its constants from an accessible source.
- **The worst-case error of Cooper's equation for the declination.** The author
  found the accuracy of the Spencer series but not of Cooper's.[^17]
- **The published value of the second Legendre coefficient in the Budyko and
  Sellers insolation profile.** The author computed a value of −0.477 from the
  exact geometry and states it as the author's own derivation.
- **The mean sea level pressure of each belt, in hectopascals.** The author
  verified the positions of the belts but not their amplitudes.[^21]
- **A single recommended value for the moisture transfer coefficient.** The
  author found the form of the bulk formula and a nearby drag coefficient of
  1.5e-3.[^27]
- **A numerically stated set of Whittaker biome boundaries.**[^32]
- **The saturated adiabatic lapse rate at 0 °C.** The author verified the
  endpoints at +20 °C and −40 °C and interpolated the middle row.[^11]

---

## References

[^1]: ADR-0002, simulated and aggregated state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^2]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^3]: Research report 29, the field pattern worked through the weather. `docs/research/reports/29-the-field-pattern-worked-through-the-weather.md`
[^4]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^5]: The weather module. `crates/cachette-core/src/weather.rs`
[^6]: Davis and Birner, eddy influences on the Hadley circulation, Journal of Advances in Modeling Earth Systems, 2019. https://agupubs.onlinelibrary.wiley.com/doi/full/10.1029/2018MS001554
[^7]: A dynamical systems characterization of atmospheric jet regimes, Earth System Dynamics 12, 233 to 251, 2021, citing Held and Hou 1980. https://esd.copernicus.org/articles/12/233/2021/
[^8]: Alduchov and Eskridge, improved Magnus form approximation of saturation vapor pressure, Journal of Applied Meteorology 35(4), 601 to 609, 1996. https://journals.ametsoc.org/view/journals/apme/35/4/1520-0450_1996_035_0601_imfaos_2_0_co_2.xml
[^9]: Tetens equation, Wikipedia. https://en.wikipedia.org/wiki/Tetens_equation
[^10]: Fundamentals of Atmospheric Science, section 3.3, the Clausius-Clapeyron equation, Pennsylvania State University. https://courses.ems.psu.edu/meteo300/node/584
[^11]: Saturated adiabatic lapse rate, Encyclopedia.com. https://www.encyclopedia.com/earth-and-environment/ecology-and-environmentalism/environmental-studies/saturated-adiabatic-lapse-rate
[^12]: Mesoscale Meteorology METR 4433, section 2.3, orographic precipitation, University of Oklahoma. https://twister.caps.ou.edu/MM2015/docs/chapter2/chapter2_e.pdf
[^13]: Minder and Roe, orographic precipitation, encyclopedia article, University of Washington. https://earthweb.ess.washington.edu/roe/GerardWeb/Publications_files/MinderRoe_OrogPrecEncyc.pdf
[^14]: Smith and Barstad, a linear theory of orographic precipitation, Journal of the Atmospheric Sciences 61(12), 1377 to 1391, 2004. https://journals.ametsoc.org/view/journals/atsc/61/12/1520-0469_2004_061_1377_altoop_2.0.co_2.xml
[^15]: Brose, lecture 11, insolation, ATM 623, University at Albany. https://www.atmos.albany.edu/facstaff/brose/classes/ATM623_Spring2015/Notes/Lectures/Lecture11%20--%20Insolation.html
[^16]: Available solar radiation and how it is measured, EME 812, Pennsylvania State University. https://courses.ems.psu.edu/eme812/node/644
[^17]: Declination angle, PVEducation. https://www.pveducation.org/pvcdrom/properties-of-sunlight/declination-angle
[^18]: Budyko transport for energy balance models, climlab documentation. https://climlab.readthedocs.io/en/latest/courseware/Budyko_Transport_EBM.html
[^19]: The arithmetic module, the sine table. `crates/cachette-core/src/sim_math.rs`
[^20]: Global circulation patterns, the Met Office. https://weather.metoffice.gov.uk/learn-about/weather/atmosphere/global-circulation-patterns
[^21]: Practical Meteorology, section 11.2, a simplified description of the global circulation, Geosciences LibreTexts. https://geo.libretexts.org/Bookshelves/Meteorology_and_Climate_Science/Practical_Meteorology_(Stull)/11:_General_Circulation/11.01:_Section_2-
[^22]: Decision Record Scope, section 4.5. `.agents/rules/adr-scope.md`
[^23]: Lateral boundary conditions in regional climate models, Monthly Weather Review 131(3), 461, 2003. https://journals.ametsoc.org/view/journals/mwre/131/3/1520-0493_2003_131_0461_lbcirc_2.0.co_2.pdf
[^24]: Lentine, Aanjaneya and Fedkiw, an unconditionally stable fully conservative semi-Lagrangian method, Stanford University. https://physbam.stanford.edu/papers/stanford2010-01.pdf
[^25]: ADR-0161, water rides the wind, and every transfer is an exact integer move. `docs/adrs/accepted/adr-0161-water-rides-the-wind-and-every-transfer-is-an-exact-integer-move.md`
[^26]: Davies, a lateral boundary formulation for multi-level prediction models, Quarterly Journal of the Royal Meteorological Society, 1976. https://rmets.onlinelibrary.wiley.com/doi/10.1002/qj.49710243210
[^27]: Turbulence and the surface energy balance, chapter 9, University of Utah. https://www.inscc.utah.edu/~krueger/5220/WH-ABL-SEB.pdf
[^28]: Estimation of evaporation and control measures, lecture 12, NPTEL. http://elearn.psgcas.ac.in/nptel/courses/video/105105214/lec12.pdf
[^29]: Köppen climate classification, Wikipedia. https://en.wikipedia.org/wiki/K%C3%B6ppen_climate_classification
[^30]: Peel, Finlayson and McMahon, updated world map of the Köppen-Geiger climate classification, Hydrology and Earth System Sciences 11, 1633 to 1644, 2007. https://hess.copernicus.org/articles/11/1633/2007/
[^31]: Testing Rules, section 2. `.agents/rules/testing.md`
[^32]: Interpreting Whittaker biome diagrams, Wyoming Biodiversity Institute. https://gveg.wyobiodiversity.org/index.php/download_file/view/98/285
