//! The influence field.
//!
//! An influence field carries the reach of a faction across the world. A
//! source term raises the field at one cell, and a solve spreads what the
//! field holds to the cells around it. A consumer reads the cell it already
//! reads and follows no link to a faction and no link to a ruler.[^1]
//!
//! **The field is a plane over the level 1 cell lattice, and it is not a
//! summary.** A cell of this plane is not the combination of the level 0
//! tiles it covers: it is the result of a relaxation that reads the
//! neighbours of the cell. The plane also carries what the last solve left
//! it, so it holds a value that appears nowhere at level 0. The record states
//! the boundary that this draws, and an open choice asks whether the record
//! that owns level 0 needs a clarifying amendment.[^2] [^3]
//!
//! **The solve runs a fixed number of passes.** It holds no convergence test
//! and no time budget. A convergence test makes the pass count depend on the
//! arithmetic, and the parallel reduction it invites makes the result depend
//! on the thread count.[^2] [^4]
//!
//! **Every value here is an exact unsigned integer.** A cell holds one byte
//! against a fixed reference, and the kernel divides by a truncating shift,
//! which is exact and gives the same answer on every target.[^5] Every
//! arithmetic step goes through the arithmetic module.[^6]
//!
//! **A pass writes disjoint output.** It reads the whole of one plane and
//! writes a contiguous run of the other, so no two threads write one cell and
//! no atomic operation appears. A cell is named by its index rather than by
//! the thread that filled it.[^7] [^8]
//!
//! # References
//!
//! [^1]: Decisions register, DEC-040. `docs/DECISIONS.md`
//! [^2]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
//! [^3]: Decisions register, DEC-067. `docs/DECISIONS.md`
//! [^4]: Influence maps, section 6.5. `docs/research/reports/09-influence-maps.md`
//! [^5]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^6]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
//! [^7]: ADR-0009, parallel stages write disjoint outputs, because the memory model is weak. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
//! [^8]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`

use crate::hash::StateHash;
use crate::hex::{Axial, Grid};
use crate::pyramid::CellSummary;
use crate::sim_math;
use crate::types::{FactionId, Fix32, TileIdx, FACTION_CEILING, FIX_FRACTIONAL_BITS};
use bytemuck::{Pod, Zeroable};

/// The reason that a field refused a call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InfluenceError {
    /// The caller asked for a solve at no threads.
    ZeroThreads,
    /// The faction count is above the ceiling the project supports.
    FactionCountAboveCeiling(u16),
    /// The conductance plane does not cover the cell lattice.
    ConductanceLengthMismatch,
}

/// One cell of an influence field.
///
/// The value is unsigned fixed point against a fixed reference: the ceiling
/// means one reference unit of influence. It is not the project-wide scale,
/// and the record states why a narrow type is correct here.[^1]
///
/// The combine is saturating addition at the ceiling. That operation is
/// exactly associative and commutative, and its identity is zero, so a fold
/// over a set of contributions gives one answer whatever the order.[^2]
///
/// # References
///
/// [^1]: ADR-0060, an influence map is stored as a shared basis, decision D2. `docs/adrs/draft/adr-0060-an-influence-map-is-stored-as-a-shared-basis.md`
/// [^2]: ADR-0023, an aggregate combines exactly, in any order, decisions D1 and D2. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct Influence(pub u16);

impl Influence {
    /// The value zero. It is the identity of the combine.
    pub const ZERO: Self = Self(0);
    /// One reference unit of influence. No cell holds more.
    pub const UNIT: Self = Self(u16::MAX);

    /// Combines two contributions at one cell.
    ///
    /// The operation saturates at the ceiling. It is exactly associative and
    /// commutative, so a fold over a set gives one answer whatever the
    /// grouping and whatever the order.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0023, an aggregate combines exactly, in any order, decisions D1 and D2. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
    #[must_use]
    pub const fn combine(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    /// Widens the value into the project-wide fixed-point scale.
    ///
    /// One reference unit maps just below the whole number that the byte
    /// ceiling used to map to, so the kernel arithmetic keeps the range it
    /// had when the cell was one byte wide. The widening is exact: it is a
    /// shift, and the result is far inside the range of the scale.
    #[must_use]
    const fn to_fix(self) -> Fix32 {
        Fix32((self.0 as i32) << (FIX_FRACTIONAL_BITS - u8::BITS))
    }

    /// Narrows a fixed-point value back into a cell.
    ///
    /// The narrowing truncates towards negative infinity, which is a shift
    /// and not a divide, and then clamps at the ceiling. Truncation is exact
    /// and reproducible on every target.[^1]
    ///
    /// # References
    ///
    /// [^1]: Influence maps, section 6.2, condition 1. `docs/research/reports/09-influence-maps.md`
    #[must_use]
    const fn from_fix(value: Fix32) -> Self {
        let narrowed = value.0 >> (FIX_FRACTIONAL_BITS - u8::BITS);
        if narrowed <= 0 {
            Self::ZERO
        } else if narrowed >= u16::MAX as i32 {
            Self::UNIT
        } else {
            Self(narrowed as u16)
        }
    }
}

/// How freely influence crosses one cell.
///
/// The ceiling means the ground puts up no resistance. Zero means influence
/// does not cross the cell at all, in either direction.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct Conductance(pub u8);

impl Conductance {
    /// Ground that stops influence.
    pub const BLOCKED: Self = Self(0);
    /// Ground that puts up no resistance.
    pub const FREE: Self = Self(u8::MAX);

    /// Returns the coupling of the edge between two cells.
    ///
    /// The coupling is the lower of the two conductances, so it is the same
    /// value read from either end. An edge with one blocked end carries
    /// nothing, and a solve over it neither fills the blocked cell nor drains
    /// through it.
    #[must_use]
    const fn coupling(self, other: Self) -> Self {
        if self.0 <= other.0 {
            self
        } else {
            other
        }
    }

    /// Returns the coupling as a fraction in the project-wide scale.
    ///
    /// The denominator is the range of the byte rather than its ceiling, so
    /// the conversion is a shift and not a divide. Free ground therefore
    /// carries a fraction just below one, which is exact and is what makes an
    /// unforced field fall rather than stand.
    #[must_use]
    const fn to_fraction(self) -> Fix32 {
        Fix32((self.0 as i32) << (FIX_FRACTIONAL_BITS - u8::BITS))
    }
}

/// The number of relaxation passes that one solve runs.
///
/// The count is fixed. A solve runs it whatever the field holds, whatever the
/// sources hold, and whatever the thread count. It is not a budget and no
/// measurement chose it: it is the reach that one solve adds to a field, in
/// cells, and the record states why a constant is the deterministic form.[^1]
///
/// # References
///
/// [^1]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
pub const PASSES_FOR_EACH_SOLVE: u32 = 8;

/// The weight that a pass leaves on the cell itself.
///
/// The unit of the weight is the range of a byte, so this weight and the
/// neighbour weight below are both read against that range. The self weight
/// and the six neighbour weights sum to less than the range, and the
/// difference is what a cell loses on every pass. A field with no source therefore falls to nothing, and it
/// falls at the edge first, because the edge is the part that the interior
/// was holding up.
const SELF_WEIGHT: i16 = 80;

/// The weight that a pass takes from each neighbour, before the coupling.
///
/// Six of these and one self weight sum to less than the range of the byte.
const NEIGHBOUR_WEIGHT: i16 = 28;

/// Returns a weight as a fraction in the project-wide scale.
const fn weight(raw: i16) -> Fix32 {
    Fix32((raw as i32) << (FIX_FRACTIONAL_BITS - u8::BITS))
}

/// An influence field over the level 1 cell lattice.
///
/// The field holds one plane for each faction and one conductance plane that
/// every faction shares. Nothing that does not depend on the faction is
/// stored more than once.[^1]
///
/// # References
///
/// [^1]: ADR-0060, an influence map is stored as a shared basis, decision D1. `docs/adrs/draft/adr-0060-an-influence-map-is-stored-as-a-shared-basis.md`
#[derive(Clone, Debug)]
pub struct InfluenceField {
    /// The cell lattice. It is a hex grid at the pitch of one level 1 cell.
    cells: Grid,
    faction_count: u16,
    /// How freely influence crosses each cell. One plane, shared by every
    /// faction, because the ground does not depend on who crosses it.
    conductance: Vec<Conductance>,
    /// What each faction holds at each cell. The plane of a faction is one
    /// contiguous run, and the faction is the major index.
    planes: Vec<Influence>,
    /// What is injected at each cell on every pass. Same layout as the
    /// planes.
    sources: Vec<Influence>,
    /// The write half of the ping-pong. It belongs to the field rather than
    /// to a faction, so the plane count does not multiply it.
    scratch: Vec<Influence>,
    /// The relaxation passes that have run since the field was built.
    ///
    /// The count is what a caller checks the fixed pass count against. A
    /// solve adds the constant to it whatever the input.
    passes: u64,
}

impl InfluenceField {
    /// Builds a field over a cell lattice, with no resistance anywhere.
    ///
    /// # Errors
    ///
    /// Returns an error when the faction count is above the ceiling the
    /// project supports.
    pub fn new(cells: Grid, faction_count: u16) -> Result<Self, InfluenceError> {
        if faction_count > FACTION_CEILING {
            return Err(InfluenceError::FactionCountAboveCeiling(faction_count));
        }
        let cell_count = cells.tile_count() as usize;
        let plane_len = cell_count * faction_count as usize;
        Ok(Self {
            cells,
            faction_count,
            conductance: vec![Conductance::FREE; cell_count],
            planes: vec![Influence::ZERO; plane_len],
            sources: vec![Influence::ZERO; plane_len],
            scratch: vec![Influence::ZERO; plane_len],
            passes: 0,
        })
    }

    /// Returns the cell lattice the field covers.
    #[must_use]
    pub const fn cells(&self) -> Grid {
        self.cells
    }

    /// Returns the number of factions the field holds a plane for.
    #[must_use]
    pub const fn faction_count(&self) -> u16 {
        self.faction_count
    }

    /// Returns the relaxation passes that have run since the field was built.
    #[must_use]
    pub const fn passes(&self) -> u64 {
        self.passes
    }

    /// Returns the position of one cell of one faction in the planes.
    fn slot(&self, faction: FactionId, cell: Axial) -> Option<usize> {
        if faction.0 >= self.faction_count {
            return None;
        }
        let index = self.cells.index_of(cell)?;
        Some(faction.0 as usize * self.cells.tile_count() as usize + index.0 as usize)
    }

    /// Returns how freely influence crosses one cell.
    #[must_use]
    pub fn conductance(&self, cell: Axial) -> Option<Conductance> {
        let index = self.cells.index_of(cell)?;
        self.conductance.get(index.0 as usize).copied()
    }

    /// Replaces the conductance of the whole lattice.
    ///
    /// The plane is shared by every faction, because how freely influence
    /// crosses the ground does not depend on who is spreading it.[^1]
    ///
    /// # Errors
    ///
    /// Returns an error when the plane does not cover the lattice.
    ///
    /// # References
    ///
    /// [^1]: ADR-0060, an influence map is stored as a shared basis, decision D1. `docs/adrs/draft/adr-0060-an-influence-map-is-stored-as-a-shared-basis.md`
    pub fn set_conductance(&mut self, plane: Vec<Conductance>) -> Result<(), InfluenceError> {
        if plane.len() != self.cells.tile_count() as usize {
            return Err(InfluenceError::ConductanceLengthMismatch);
        }
        self.conductance = plane;
        Ok(())
    }

    /// Reads the conductance plane from the ground of each level 1 cell.
    ///
    /// The conductance of a cell is the share of its ground that admits a
    /// unit. Ground that holds nobody carries nobody's writ, so a cell of
    /// open water conducts nothing and a cell of open land conducts freely.
    ///
    /// **This is the whole of the ground rule today, and it says so.** The
    /// project already chose that the influence plane carries terrain
    /// conductance, so that influence flows around ground which resists it
    /// rather than through it.[^1] The ground that resists a unit is the only
    /// ground the terrain table distinguishes today, because the multiplier
    /// that separates a mountain from a plain has no recorded value yet.[^2]
    /// A second conductance table beside the capacity table would be one fact
    /// in two places, and nothing would fail when the two disagreed.[^3] The
    /// solve reads a conductance of any value, so the rule that produces it
    /// may be replaced without touching the solve.
    ///
    /// A cell that covers no ground conducts nothing.
    ///
    /// # Errors
    ///
    /// Returns an error when the summaries do not cover the lattice.
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-005. `docs/DECISIONS.md`
    /// [^2]: Decisions register, DEC-017. `docs/DECISIONS.md`
    /// [^3]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    pub fn read_the_ground(&mut self, summaries: &[CellSummary]) -> Result<(), InfluenceError> {
        if summaries.len() != self.cells.tile_count() as usize {
            return Err(InfluenceError::ConductanceLengthMismatch);
        }
        for (cell, summary) in self.conductance.iter_mut().zip(summaries) {
            *cell = match summary.open_share() {
                Some(share) => Conductance(narrow_to_conductance(share)),
                None => Conductance::BLOCKED,
            };
        }
        Ok(())
    }

    /// Returns what one faction holds at one cell.
    ///
    /// Returns `None` when the faction is outside the set the field holds, or
    /// when the cell is outside the lattice.
    #[must_use]
    pub fn at(&self, faction: FactionId, cell: Axial) -> Option<Influence> {
        self.planes.get(self.slot(faction, cell)?).copied()
    }

    /// Returns what one faction injects at one cell on every pass.
    #[must_use]
    pub fn source(&self, faction: FactionId, cell: Axial) -> Option<Influence> {
        self.sources.get(self.slot(faction, cell)?).copied()
    }

    /// Sets what one faction injects at one cell on every pass.
    ///
    /// This is the whole of the write side. What decides the value is a rule
    /// that lives above this module, and the field holds no such rule: a
    /// source of zero is not a case, it is the ordinary value.[^1]
    ///
    /// Returns `false` when the faction or the cell is outside the field.
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-041. `docs/DECISIONS.md`
    pub fn set_source(&mut self, faction: FactionId, cell: Axial, source: Influence) -> bool {
        let Some(slot) = self.slot(faction, cell) else {
            return false;
        };
        self.sources[slot] = source;
        true
    }

    /// Runs one solve.
    ///
    /// A solve runs the fixed number of passes. It reads no clock, it tests
    /// no residual, and it takes no branch on whether a source exists, so it
    /// costs the same whatever the field holds.[^1]
    ///
    /// Each pass reads the whole of one plane and writes a contiguous run of
    /// the other. The runs are disjoint, so no two threads write one cell and
    /// the pass needs no atomic operation.[^2]
    ///
    /// # Errors
    ///
    /// Returns an error when the caller asks for a solve at no threads.
    ///
    /// # References
    ///
    /// [^1]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
    /// [^2]: ADR-0009, parallel stages write disjoint outputs, because the memory model is weak. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
    pub fn solve(&mut self, threads: usize) -> Result<(), InfluenceError> {
        if threads == 0 {
            return Err(InfluenceError::ZeroThreads);
        }
        for _ in 0..PASSES_FOR_EACH_SOLVE {
            self.relax(threads);
            core::mem::swap(&mut self.planes, &mut self.scratch);
            self.passes = self.passes.saturating_add(1);
            // The perturbed build reads a residual here and stops the solve
            // when the last pass changed nothing. The ordinary build has no
            // residual to read, which is the point: a convergence test cannot
            // be added by accident to a loop that computes nothing to test.
            if the_solve_stops_early(&self.scratch, &self.planes) {
                break;
            }
        }
        Ok(())
    }

    /// Runs one relaxation pass into the scratch plane.
    ///
    /// The cells are visited in ascending index and the neighbours of a cell
    /// in direction order. Both orders are fixed, and a cell is named by its
    /// index rather than by the thread that filled it.[^1]
    ///
    /// **A field with fewer slots than the caller has threads is filled on
    /// one thread.** A slot is one cell of one plane. Starting a thread for
    /// one slot costs more than the slot does. The rule reads the two numbers
    /// the caller already supplied and holds no constant of its own.
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn relax(&mut self, threads: usize) {
        let pass = Pass {
            cells: self.cells,
            cell_count: self.cells.tile_count() as usize,
            conductance: &self.conductance,
            planes: &self.planes,
            sources: &self.sources,
        };
        let total = self.scratch.len();

        if total <= threads {
            pass.fill(0, total, &mut self.scratch);
            return;
        }

        let chunk_len = total.div_ceil(threads).max(1);
        std::thread::scope(|scope| {
            let mut first = 0usize;
            for chunk in self.scratch.chunks_mut(chunk_len) {
                let start = first;
                let end = first + chunk.len();
                first = end;
                scope.spawn(move || pass.fill(start, end, chunk));
            }
        });
    }

    /// Folds the field into a state hash.
    ///
    /// The order is the conductance plane, then the sources, then the field,
    /// and each of them in ascending slot order. The order is fixed and the
    /// hash is order-sensitive, so a reader does not have to prove that the
    /// order does not matter.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        hash.write_u64(u64::from(self.faction_count))
            .write_u64(self.passes)
            .write(bytemuck::cast_slice(&self.conductance))
            .write(bytemuck::cast_slice(&self.sources))
            .write(bytemuck::cast_slice(&self.planes))
    }
}

/// Returns a share of the ground as a conductance.
///
/// The share is a fixed-point fraction of one. The product is exact and the
/// narrowing truncates, so the whole conversion is integer arithmetic.
fn narrow_to_conductance(share: Fix32) -> u8 {
    let scaled = sim_math::mul(share, Fix32::from_int(u8::MAX as i16)).to_int_floor();
    if scaled <= 0 {
        0
    } else if scaled >= u8::MAX as i32 {
        u8::MAX
    } else {
        scaled as u8
    }
}

/// What one relaxation pass reads.
///
/// The view holds no mutable state, so every thread of a pass takes a copy of
/// it and the copies cannot disagree.
#[derive(Clone, Copy)]
struct Pass<'a> {
    cells: Grid,
    cell_count: usize,
    conductance: &'a [Conductance],
    planes: &'a [Influence],
    sources: &'a [Influence],
}

impl Pass<'_> {
    /// Fills one run of the scratch plane.
    ///
    /// The run is named by its position in the planes, and the position
    /// decides both the faction and the cell. Nothing in the body reads which
    /// thread called it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0009, parallel stages write disjoint outputs, because the memory model is weak. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
    fn fill(&self, start: usize, end: usize, out: &mut [Influence]) {
        for (offset, cell) in out.iter_mut().enumerate() {
            let slot = start + offset;
            let index = slot % self.cell_count;
            let base = slot - index;
            let Some(address) = self.cells.address_of(TileIdx(index as u32)) else {
                continue;
            };
            let here = self.conductance[index];

            // The self term. A pass leaves less on a cell than it found, and
            // the difference is what makes an unforced field fall.
            let mut total = sim_math::mul(weight(SELF_WEIGHT), self.planes[slot].to_fix());

            // The neighbour terms. A neighbour outside the lattice
            // contributes nothing, and so does a neighbour that the perturbed
            // build cannot see.
            let neighbours = self.cells.neighbours(address);
            for neighbour in neighbours.into_iter().flatten() {
                let Some(at) = self.cells.index_of(neighbour) else {
                    continue;
                };
                let reached = base + at.0 as usize;
                if !the_neighbour_is_visible(reached, start, end) {
                    continue;
                }
                let coupling = here.coupling(self.conductance[at.0 as usize]).to_fraction();
                let share = sim_math::mul(weight(NEIGHBOUR_WEIGHT), coupling);
                total = sim_math::add(total, sim_math::mul(share, self.planes[reached].to_fix()));
            }

            // The source is injected on every pass, and no branch asks
            // whether it is there. The combine saturates at the ceiling.[^1]
            //
            // [^1]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
            *cell = Influence::from_fix(total).combine(self.sources[slot]);
        }
    }
}

/// Reports whether the solve stops before its fixed pass count.
///
/// It never does. The parameters are the field before the pass and the field
/// after it, which is the residual a convergence test would read, and the
/// ordinary build reads neither.[^1]
///
/// # References
///
/// [^1]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
#[cfg(not(feature = "probe-nondeterminism"))]
const fn the_solve_stops_early(_before: &[Influence], _after: &[Influence]) -> bool {
    false
}

/// The perturbed stop. It ends the solve when a pass changed nothing.
///
/// This is the convergence test that the record forbids. It is deterministic
/// across thread counts, because the comparison it reads is exact and its
/// result does not depend on how the work was split. The thread-count test
/// therefore passes over it, and only a test of the pass count sees it.[^1]
///
/// # References
///
/// [^1]: Testing rules, section 2. `.claude/rules/testing.md`
#[cfg(feature = "probe-nondeterminism")]
fn the_solve_stops_early(before: &[Influence], after: &[Influence]) -> bool {
    before == after
}

/// Reports whether a pass may read a neighbour.
///
/// It always may. The bounds are the run the caller is filling, which a
/// stencil that forgot its halo would clamp its reads to.[^1]
///
/// # References
///
/// [^1]: ADR-0009, parallel stages write disjoint outputs, because the memory model is weak. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
#[cfg(not(feature = "probe-nondeterminism"))]
const fn the_neighbour_is_visible(_reached: usize, _start: usize, _end: usize) -> bool {
    true
}

/// The perturbed read. A neighbour outside the run reads as nothing.
///
/// This is the missing halo. The run boundary follows the thread count, so
/// the field it produces follows the thread count too, and it does so while
/// every cell still holds a plausible value.[^1]
///
/// # References
///
/// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D5. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
#[cfg(feature = "probe-nondeterminism")]
const fn the_neighbour_is_visible(reached: usize, start: usize, end: usize) -> bool {
    reached >= start && reached < end
}
