//! The plan of a faction, and the solver that writes it.
//!
//! A plan is a bounded list of projects. A project is one tile and one
//! category. A unit builds a category that asks for no held ground only
//! inside a project, so a faction reaches new ground along the path it
//! planned and not along every path its units walked.[^1]
//!
//! One solver writes the plan at the controller stage, in a fixed pass count.
//! It reads the sites of one faction, the deposits inside one bounded window
//! and the ground the faction holds. It reads no unit, and it reads no tile
//! outside the window.[^1]
//!
//! **No function here names a category.** The solver asks the upgrade table
//! which category joins two places: the category whose row asks for no held
//! ground is the one a faction builds to reach ground it does not hold. A
//! pass that compared a category index to a constant would be the violation
//! the upgrade record names.[^2] [^3]
//!
//! Every value the solver reads is a balance row, and every one of them is
//! unset until the balance pass measures it.[^4]
//!
//! # References
//!
//! [^1]: ADR-0152, a faction plans its roads and zones with one solver, decisions D1 to D5. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
//! [^2]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
//! [^3]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D4. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
//! [^4]: Balance register, the plan. `docs/reference/balance.md`

use bytemuck::{Pod, Zeroable};

use crate::hash::StateHash;
use crate::hex::{Axial, Grid};
use crate::holding::Holder;
use crate::terrain::Terrain;
use crate::types::{FactionId, TileIdx};
use crate::upgrade::{UpgradeCategory, UpgradeMap, UpgradeTable};

/// The most projects one faction may hold.
///
/// Provisional. The balance register holds the row and the derivation.[^1]
///
/// # References
///
/// [^1]: Balance register, the plan. `docs/reference/balance.md`
pub const PLAN_BOUND_DEFAULT: u32 = 40;

/// The passes the solver makes over the reads of one faction.
///
/// Provisional, as above.[^1]
///
/// # References
///
/// [^1]: Balance register, the plan. `docs/reference/balance.md`
pub const SOLVER_PASSES_DEFAULT: u32 = 2;

/// The most projects one solver pass writes.
///
/// Provisional, as above.[^1]
///
/// # References
///
/// [^1]: Balance register, the plan. `docs/reference/balance.md`
pub const PROJECTS_PER_PASS_DEFAULT: u32 = 17;

/// The passes the path search makes over its window.
///
/// Provisional, as above.[^1]
///
/// # References
///
/// [^1]: Balance register, the plan. `docs/reference/balance.md`
pub const PATH_PASSES_DEFAULT: u32 = 17;

/// The hex steps a path may span.
///
/// Provisional, as above.[^1]
///
/// # References
///
/// [^1]: Balance register, the plan. `docs/reference/balance.md`
pub const ROAD_SEARCH_RADIUS_DEFAULT: u32 = 8;

/// The cost a window cell holds when no path reaches it.
const NO_PATH: i64 = i64::MAX;

/// The tile index a window cell holds when it names no tile.
const NO_TILE: u32 = u32::MAX;

/// One zoned project: one tile and one category.
///
/// The row is plain data with declared padding, so two worlds that hold the
/// same projects hash the same and no uninitialised byte reaches the state
/// hash.[^1]
///
/// # References
///
/// [^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Pod, Zeroable)]
pub struct Project {
    /// The tile the project zones.
    pub tile: TileIdx,
    /// What the project asks a unit to build there.
    pub category: UpgradeCategory,
    /// The declared padding. Always zero.
    pub padding: [u8; 3],
}

impl Project {
    /// Builds a project with zero padding.
    #[must_use]
    pub const fn new(tile: TileIdx, category: UpgradeCategory) -> Self {
        Self {
            tile,
            category,
            padding: [0; 3],
        }
    }
}

/// Why the plan refused a caller.
///
/// The refusal names what refused, so a caller learns which rule it met
/// rather than reading a bare no.[^1]
///
/// # References
///
/// [^1]: ADR-0046, every error is typed. `docs/adrs/draft/adr-0046-every-error-is-typed.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanRefusal {
    /// The number names no faction of this world.
    NoSuchFaction(FactionId),
    /// The address lies outside the world.
    AddressOutsideWorld(Axial),
    /// The plan already holds as many projects as its bound allows.
    PlanFull {
        /// The bound the plan reached.
        bound: u32,
    },
    /// The row of the category asks for held ground, and the faction does not
    /// hold the tile.
    GroundNotHeld {
        /// The category the caller named.
        category: UpgradeCategory,
    },
    /// No row of the table fits the ground under the tile at the level above
    /// what stands there.
    NoRowFits {
        /// The category the caller named.
        category: UpgradeCategory,
    },
}

/// How large a plan grows, how long the solver runs, and how far it looks.
///
/// Every field is a balance row, and every value in that register is unset
/// until the balance pass measures it. The defaults here are the provisional
/// values the register holds, and the register holds the derivation of
/// each.[^1] A blocker governs all of them.[^2]
///
/// # References
///
/// [^1]: Balance register, the plan. `docs/reference/balance.md`
/// [^2]: Blockers register, BLK-050. `docs/BLOCKERS.md`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlanRules {
    bound: u32,
    solver_passes: u32,
    projects_per_pass: u32,
    path_passes: u32,
    radius: u32,
}

impl PlanRules {
    /// The provisional values that the balance register holds.
    pub const DEFAULT: Self = Self {
        bound: PLAN_BOUND_DEFAULT,
        solver_passes: SOLVER_PASSES_DEFAULT,
        projects_per_pass: PROJECTS_PER_PASS_DEFAULT,
        path_passes: PATH_PASSES_DEFAULT,
        radius: ROAD_SEARCH_RADIUS_DEFAULT,
    };

    /// Builds a rule set.
    ///
    /// A bound of zero would refuse every project, and a pass count of zero
    /// would write nothing, so each is a value the caller may choose. Nothing
    /// here is clamped, because a caller that asks for zero asks for a solver
    /// that plans nothing, and that is a legitimate world.
    #[must_use]
    pub const fn new(
        bound: u32,
        solver_passes: u32,
        projects_per_pass: u32,
        path_passes: u32,
        radius: u32,
    ) -> Self {
        Self {
            bound,
            solver_passes,
            projects_per_pass,
            path_passes,
            radius,
        }
    }

    /// Returns the most projects one faction may hold.
    #[must_use]
    pub const fn bound(self) -> u32 {
        self.bound
    }

    /// Returns the rules with another bound.
    #[must_use]
    pub const fn with_bound(self, bound: u32) -> Self {
        Self { bound, ..self }
    }

    /// Returns the passes the solver makes.
    #[must_use]
    pub const fn solver_passes(self) -> u32 {
        self.solver_passes
    }

    /// Returns the most projects one pass writes.
    #[must_use]
    pub const fn projects_per_pass(self) -> u32 {
        self.projects_per_pass
    }

    /// Returns the passes the path search makes.
    #[must_use]
    pub const fn path_passes(self) -> u32 {
        self.path_passes
    }

    /// Returns the hex steps a path may span.
    #[must_use]
    pub const fn radius(self) -> u32 {
        self.radius
    }

    /// Folds the rules into the state hash.
    #[must_use]
    pub fn hash_into(self, hash: StateHash) -> StateHash {
        hash.write_u64(u64::from(self.bound))
            .write_u64(u64::from(self.solver_passes))
            .write_u64(u64::from(self.projects_per_pass))
            .write_u64(u64::from(self.path_passes))
            .write_u64(u64::from(self.radius))
    }
}

impl Default for PlanRules {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// The projects of every faction, in ascending tile order.
///
/// The register holds a fixed number of rows for each faction. A tile appears
/// in one plan of one faction once, because a tile carries one upgrade.[^1]
/// The order of a plan is the tile order and never the order a caller wrote
/// in.[^2]
///
/// # References
///
/// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D1. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
/// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanRegister {
    rules: PlanRules,
    rows: Vec<Project>,
    lengths: Vec<u32>,
    zoned: i64,
    finished: i64,
    dropped: i64,
    refused: i64,
    passes: i64,
}

impl PlanRegister {
    /// Builds a register for a world of a given faction count.
    #[must_use]
    pub fn new(faction_count: u16, rules: PlanRules) -> Self {
        let factions = usize::from(faction_count);
        let bound = rules.bound() as usize;
        Self {
            rules,
            rows: vec![Project::default(); factions * bound],
            lengths: vec![0; factions],
            zoned: 0,
            finished: 0,
            dropped: 0,
            refused: 0,
            passes: 0,
        }
    }

    /// Returns the rules the register was built with.
    #[must_use]
    pub const fn rules(&self) -> PlanRules {
        self.rules
    }

    /// Returns how many factions the register holds a plan for.
    #[must_use]
    pub fn faction_count(&self) -> usize {
        self.lengths.len()
    }

    /// Returns the projects of one faction, in ascending tile order.
    ///
    /// Returns an empty slice for a faction the register does not hold.
    #[must_use]
    pub fn projects_of(&self, faction: FactionId) -> &[Project] {
        let index = usize::from(faction.0);
        let Some(length) = self.lengths.get(index) else {
            return &[];
        };
        let bound = self.rules.bound() as usize;
        let start = index * bound;
        &self.rows[start..start + *length as usize]
    }

    /// Returns the category that one faction's plan zones on one tile.
    ///
    /// The lookup is a binary search of a slice that the tile order sorts, so
    /// it costs the logarithm of the bound and never a scan of the world.
    #[must_use]
    pub fn zones(&self, faction: FactionId, tile: TileIdx) -> Option<UpgradeCategory> {
        let projects = self.projects_of(faction);
        projects
            .binary_search_by_key(&tile, |project| project.tile)
            .ok()
            .map(|position| projects[position].category)
    }

    /// Writes one project into the plan of one faction.
    ///
    /// A tile the plan already names takes the new category. A plan at its
    /// bound drops the project and counts the drop.[^1]
    ///
    /// # Errors
    ///
    /// Returns a refusal when the faction has no plan, and when the plan is
    /// full.
    ///
    /// # References
    ///
    /// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D1. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
    pub fn write(&mut self, faction: FactionId, project: Project) -> Result<(), PlanRefusal> {
        let index = usize::from(faction.0);
        let bound = self.rules.bound() as usize;
        let Some(length) = self.lengths.get(index).copied() else {
            self.refused += 1;
            return Err(PlanRefusal::NoSuchFaction(faction));
        };
        let start = index * bound;
        let slice = &mut self.rows[start..start + length as usize];
        match slice.binary_search_by_key(&project.tile, |held| held.tile) {
            Ok(position) => {
                slice[position] = project;
                self.zoned += 1;
                Ok(())
            }
            Err(position) => {
                if length as usize >= bound {
                    self.dropped += 1;
                    self.refused += 1;
                    return Err(PlanRefusal::PlanFull {
                        bound: self.rules.bound(),
                    });
                }
                let whole = &mut self.rows[start..start + length as usize + 1];
                whole[position..].rotate_right(1);
                whole[position] = project;
                self.lengths[index] = length + 1;
                self.zoned += 1;
                Ok(())
            }
        }
    }

    /// Removes the project one faction zoned on one tile.
    ///
    /// Reports whether it removed one.
    pub fn clear(&mut self, faction: FactionId, tile: TileIdx) -> bool {
        self.remove(faction, tile).is_some()
    }

    /// Removes a project the builders finished, and counts it.
    ///
    /// Reports whether it removed one.
    pub fn finish(&mut self, faction: FactionId, tile: TileIdx) -> bool {
        if self.remove(faction, tile).is_some() {
            self.finished += 1;
            true
        } else {
            false
        }
    }

    /// Counts one refusal that a verb or a pass gave.
    pub fn count_refusal(&mut self) {
        self.refused += 1;
    }

    /// Returns how many projects every plan holds now.
    #[must_use]
    pub fn live(&self) -> i64 {
        self.lengths.iter().map(|length| i64::from(*length)).sum()
    }

    /// Returns how many projects the register has taken.
    #[must_use]
    pub const fn zoned_count(&self) -> i64 {
        self.zoned
    }

    /// Returns how many projects the builders finished.
    #[must_use]
    pub const fn finished_count(&self) -> i64 {
        self.finished
    }

    /// Returns how many projects a full plan dropped.
    #[must_use]
    pub const fn dropped_count(&self) -> i64 {
        self.dropped
    }

    /// Returns how many writes and builds the plan refused.
    #[must_use]
    pub const fn refused_count(&self) -> i64 {
        self.refused
    }

    /// Returns how many passes the solver has made.
    ///
    /// The count rises by the pass count of the rules on every solve, and by
    /// nothing else. A solver that stopped on a condition would raise it by
    /// less, so a reader can see the fixed iteration count from outside.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0005, a solver runs a fixed iteration count, decision D1. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`
    #[must_use]
    pub const fn pass_count(&self) -> i64 {
        self.passes
    }

    /// Counts one pass of the solver.
    pub fn count_pass(&mut self) {
        self.passes += 1;
    }

    /// Folds the register into the state hash.
    ///
    /// The plan is simulated state that the step reads, so two worlds that
    /// differ only in a plan never hash the same.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        let mut hash = self.rules.hash_into(hash);
        hash = hash.write_u64(self.lengths.len() as u64);
        for index in 0..self.lengths.len() {
            let faction = FactionId(index as u16);
            let projects = self.projects_of(faction);
            hash = hash.write_u64(projects.len() as u64);
            hash = hash.write(bytemuck::cast_slice(projects));
        }
        // The counters are census readings. No later frame reads one, so
        // none of them enters the hash.[^2]
        //
        // [^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
        hash
    }

    /// Reports whether every plan is inside its bound, sorted and free of a
    /// repeated tile.
    #[must_use]
    pub fn check_invariants(&self) -> bool {
        for index in 0..self.lengths.len() {
            let faction = FactionId(index as u16);
            let projects = self.projects_of(faction);
            if projects.len() > self.rules.bound() as usize {
                return false;
            }
            if !projects.windows(2).all(|pair| pair[0].tile < pair[1].tile) {
                return false;
            }
            if !projects.iter().all(|project| project.padding == [0; 3]) {
                return false;
            }
        }
        true
    }

    /// Removes one project and returns it.
    fn remove(&mut self, faction: FactionId, tile: TileIdx) -> Option<Project> {
        let index = usize::from(faction.0);
        let bound = self.rules.bound() as usize;
        let length = *self.lengths.get(index)? as usize;
        let start = index * bound;
        let slice = &mut self.rows[start..start + length];
        let position = slice.binary_search_by_key(&tile, |held| held.tile).ok()?;
        let taken = slice[position];
        slice[position..].rotate_left(1);
        self.rows[start + length - 1] = Project::default();
        self.lengths[index] = (length - 1) as u32;
        Some(taken)
    }
}

/// What the path search and the solver read of the world.
///
/// The reads are the grid, the ground, the upgrades that stand and the
/// upgrade table. None of them is a unit, and none of them is a pass over the
/// world.[^1]
///
/// # References
///
/// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D2. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
#[derive(Clone, Copy)]
pub struct Ground<'a> {
    /// The grid of the world.
    pub grid: Grid,
    /// The ground of the world.
    pub terrain: Terrain,
    /// The upgrades that stand and that are under construction.
    pub upgrades: &'a UpgradeMap,
    /// The upgrade table the world was built with.
    pub table: &'a UpgradeTable,
}

impl Ground<'_> {
    /// Returns the category that joins two places on one ground kind.
    ///
    /// **The joining category is the one whose first row asks for no held
    /// ground.** That column is what lets a faction reach ground it does not
    /// hold, so the category that carries it is the category a plan lays
    /// between two places.[^1] No name is compared here, so a table a caller
    /// wrote answers by the same rule as the default table.[^2]
    ///
    /// Returns the lowest such category, so two categories that both carry
    /// the column resolve by the table order and never by a container.
    ///
    /// # References
    ///
    /// [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D4. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    /// [^2]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
    #[must_use]
    pub fn joining_category(&self, tile: TileIdx) -> Option<UpgradeCategory> {
        let kind = self.terrain.kind(self.grid.address_of(tile)?)?;
        UpgradeCategory::ALL.into_iter().find(|category| {
            self.table
                .row(*category, 1)
                .is_some_and(|row| row.exists() && row.fits(kind) && row.own_ground_required == 0)
        })
    }

    /// Reports whether a finished joining upgrade stands on one tile.
    ///
    /// A tile that carries one is already joined, so no project is written
    /// for it.
    #[must_use]
    pub fn joined(&self, tile: TileIdx) -> bool {
        let Some(site) = self.upgrades.at(tile) else {
            return false;
        };
        if !site.is_complete() {
            return false;
        }
        self.table
            .row(site.category, site.level)
            .is_some_and(|row| row.exists() && row.own_ground_required == 0)
    }

    /// Returns the whole-number cost of one step onto a tile.
    ///
    /// The cost is what the ground charges, as a whole number. A tile that
    /// admits nobody has no cost, because no path crosses it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    #[must_use]
    pub fn step_cost(&self, address: Axial) -> Option<i64> {
        let kind = self.terrain.kind(address)?;
        if !kind.is_passable() {
            return None;
        }
        Some(i64::from(kind.step_multiplier().0))
    }
}

/// A square window of the world, centred on one tile.
///
/// The window is dense and it is addressed by the offset from the centre, so
/// every read is an array read at a fixed index. No container decides an
/// order here.[^1]
///
/// # References
///
/// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
pub struct PathWindow {
    centre: Axial,
    radius: i32,
    side: usize,
    tile: Vec<u32>,
    step: Vec<i64>,
    cost: Vec<i64>,
    from: Vec<u32>,
}

impl PathWindow {
    /// Builds the window around one tile and relaxes it a fixed number of
    /// times.
    ///
    /// **The pass count never changes with the input.** The search relaxes
    /// the window exactly as many times as the rules say, and it then reads
    /// the answer. It never runs until the frontier settles, because that is
    /// a convergence test.[^1]
    ///
    /// A pass advances the frontier by at least one step, so a path of up to
    /// the pass count in steps is the cheapest path. A path that needs a
    /// longer detour is deterministic and is not the cheapest, and the
    /// derivation of the pass count says so.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0005, a solver runs a fixed iteration count, decision D1. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`
    /// [^2]: Balance register, the plan. `docs/reference/balance.md`
    #[must_use]
    pub fn build(ground: &Ground<'_>, centre: TileIdx, rules: PlanRules) -> Option<Self> {
        let address = ground.grid.address_of(centre)?;
        let radius = i32::try_from(rules.radius()).ok()?;
        let side = (radius as usize) * 2 + 1;
        let cells = side * side;
        let mut window = Self {
            centre: address,
            radius,
            side,
            tile: vec![NO_TILE; cells],
            step: vec![0; cells],
            cost: vec![NO_PATH; cells],
            from: vec![NO_TILE; cells],
        };
        for dr in -radius..=radius {
            for dq in -radius..=radius {
                let here = Axial::new(address.q + dq, address.r + dr);
                if here.distance(address) > rules.radius() {
                    continue;
                }
                let Some(index) = ground.grid.index_of(here) else {
                    continue;
                };
                let Some(step) = ground.step_cost(here) else {
                    continue;
                };
                let cell = window.cell_of(here)?;
                window.tile[cell] = index.0;
                window.step[cell] = step;
            }
        }
        let start = window.cell_of(address)?;
        if window.tile[start] == NO_TILE {
            return None;
        }
        window.cost[start] = 0;
        for _ in 0..rules.path_passes() {
            window.relax(ground);
        }
        Some(window)
    }

    /// Returns the cost of the cheapest path the search found to one tile.
    #[must_use]
    pub fn cost_of(&self, address: Axial) -> Option<i64> {
        let cell = self.cell_of(address)?;
        let cost = self.cost[cell];
        if cost == NO_PATH {
            None
        } else {
            Some(cost)
        }
    }

    /// Returns the path from the centre to one tile, in walking order.
    ///
    /// The walk takes the neighbour with the lowest pair of cost and tile
    /// index at every step, so a tie takes the lower tile index.[^1] The walk
    /// is bounded by the cells of the window, so a cycle cannot spin it.
    ///
    /// # References
    ///
    /// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D3. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
    #[must_use]
    pub fn path_to(&self, grid: Grid, address: Axial) -> Option<Vec<TileIdx>> {
        let mut cell = self.cell_of(address)?;
        if self.cost[cell] == NO_PATH {
            return None;
        }
        let mut walked = Vec::new();
        for _ in 0..=self.tile.len() {
            walked.push(TileIdx(self.tile[cell]));
            let previous = self.from[cell];
            if previous == NO_TILE {
                walked.reverse();
                return Some(walked);
            }
            let back = grid.address_of(TileIdx(previous))?;
            cell = self.cell_of(back)?;
        }
        None
    }

    /// Relaxes every cell of the window once, in ascending cell order.
    fn relax(&mut self, ground: &Ground<'_>) {
        for cell in 0..self.tile.len() {
            if self.tile[cell] == NO_TILE {
                continue;
            }
            let Some(here) = self.address_of(cell) else {
                continue;
            };
            let mut best = (self.cost[cell], self.from[cell]);
            for neighbour in ground.grid.neighbours(here).into_iter().flatten() {
                let Some(other) = self.cell_of(neighbour) else {
                    continue;
                };
                if self.tile[other] == NO_TILE || self.cost[other] == NO_PATH {
                    continue;
                }
                let candidate = (self.cost[other] + self.step[cell], self.tile[other]);
                if candidate < best {
                    best = candidate;
                }
            }
            self.cost[cell] = best.0;
            self.from[cell] = best.1;
        }
    }

    /// Returns the cell of one address, or `None` when it lies outside the
    /// window.
    fn cell_of(&self, address: Axial) -> Option<usize> {
        let dq = address.q - self.centre.q;
        let dr = address.r - self.centre.r;
        if dq < -self.radius || dq > self.radius || dr < -self.radius || dr > self.radius {
            return None;
        }
        let column = (dq + self.radius) as usize;
        let row = (dr + self.radius) as usize;
        Some(row * self.side + column)
    }

    /// Returns the address of one cell.
    fn address_of(&self, cell: usize) -> Option<Axial> {
        let column = i32::try_from(cell % self.side).ok()?;
        let row = i32::try_from(cell / self.side).ok()?;
        Some(Axial::new(
            self.centre.q + column - self.radius,
            self.centre.r + row - self.radius,
        ))
    }
}

/// What one faction gives the solver to plan from.
///
/// Every field is a bounded read of one faction. The sites follow the
/// settlements of the faction, and the held tiles follow the ground its
/// cities reach. Neither follows the tile count of the world.[^1]
///
/// # References
///
/// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D2. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
pub struct Needs<'a> {
    /// The seat of the faction: the tile of its first founding.
    pub seat: TileIdx,
    /// The tiles of the faction's settlements, in ascending tile order.
    pub sites: &'a [TileIdx],
    /// Who holds each tile of the world.
    pub holders: &'a [Holder],
    /// Whether the faction's stores fall short of the mark it offers above.
    pub short_of_stores: bool,
    /// Whether every site of the faction is short of a free place.
    ///
    /// A faction whose sites are all full grows nobody, however much food it
    /// holds, so the housing is the only thing that moves its
    /// population.[^1] The solver plans a dwelling first while this is true,
    /// and it returns to a way and a yield as soon as one site has room.
    ///
    /// # References
    ///
    /// [^1]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D2. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
    pub short_of_places: bool,
}

/// Writes the plan of one faction, in a fixed pass count.
///
/// The solver runs the pass count the rules give, whatever the world holds.
/// It never stops on a condition and it never reads a clock.[^1]
///
/// Each pass does four things in one order. It clears the projects the
/// builders finished. It builds one window at the seat. It scores the
/// candidates and takes the best. It writes the path of the best candidate
/// into the plan, up to the projects one pass may write.
///
/// Returns how many projects the passes wrote.
///
/// # References
///
/// [^1]: ADR-0005, a solver runs a fixed iteration count, decision D1. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`
pub fn solve(
    ground: &Ground<'_>,
    faction: FactionId,
    needs: &Needs<'_>,
    plan: &mut PlanRegister,
) -> u32 {
    let rules = plan.rules();
    let mut written = 0;
    for _ in 0..rules.solver_passes() {
        plan.count_pass();
        sweep_finished(ground, faction, plan);
        let Some(window) = PathWindow::build(ground, needs.seat, rules) else {
            continue;
        };
        let Some(target) = choose_target(ground, faction, needs, &window, plan) else {
            continue;
        };
        written += write_path(ground, faction, needs, &window, target, plan);
    }
    written
}

/// Removes every project of one faction that a builder finished.
fn sweep_finished(ground: &Ground<'_>, faction: FactionId, plan: &mut PlanRegister) {
    let finished: Vec<TileIdx> = plan
        .projects_of(faction)
        .iter()
        .filter(|project| {
            ground
                .upgrades
                .at(project.tile)
                .is_some_and(|site| site.is_complete() && site.category == project.category)
        })
        .map(|project| project.tile)
        .collect();
    for tile in finished {
        plan.finish(faction, tile);
    }
}

/// Chooses the tile the next path runs to.
///
/// The three candidate classes are the three reads the record names, and the
/// solver takes them in one order: an unconnected site of the faction, then a
/// deposit inside the window, then the ground the faction holds when its
/// stores fall short.[^1]
///
/// **Two of the three have no reader in the engine.** Nothing derives which
/// sites are unconnected, and nothing names the deposits of a faction, so the
/// solver answers both from the window it already built and from one read for
/// each tile of a path. A deposit outside the window is invisible to the
/// plan.[^2]
///
/// # References
///
/// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D2. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
/// [^2]: Findings register, FND-491. `docs/FINDINGS.md`
fn choose_target(
    ground: &Ground<'_>,
    faction: FactionId,
    needs: &Needs<'_>,
    window: &PathWindow,
    plan: &PlanRegister,
) -> Option<Target> {
    // A faction whose every site is full grows nobody, so a lodging comes
    // before a way and before a yield while that holds.
    if let Some(address) = choose_lodging_ground(ground, faction, needs, window, plan) {
        return Some(Target::Lodging(address));
    }
    let mut best: Option<(i64, u32, Axial)> = None;
    for site in needs.sites {
        if *site == needs.seat {
            continue;
        }
        let Some(address) = ground.grid.address_of(*site) else {
            continue;
        };
        let Some(cost) = window.cost_of(address) else {
            continue;
        };
        if joined_all_the_way(ground, faction, needs, window, address, plan) {
            continue;
        }
        let key = (cost, site.0, address);
        if best.is_none_or(|held| key < held) {
            best = Some(key);
        }
    }
    if let Some((_, _, address)) = best {
        return Some(Target::Join(address));
    }
    // No site asks for a way. The deposits of the window come next, and the
    // ground the faction holds comes last.
    if let Some(address) = choose_deposit(ground, faction, needs, window, plan) {
        return Some(Target::Join(address));
    }
    choose_yield_ground(ground, faction, needs, window, plan).map(Target::Raise)
}

/// What the solver plans toward.
///
/// A join asks for a way between the seat and one place, and it writes every
/// tile of the path. A raise asks the ground of one held tile to give more,
/// and it writes one tile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Target {
    /// Join the seat to this place.
    Join(Axial),
    /// Raise the yield of this tile.
    Raise(Axial),
    /// Lodge more people at this tile.
    Lodging(Axial),
}

/// Reports whether every tile of the path to one place already carries a
/// finished joining upgrade or a project of this faction.
fn joined_all_the_way(
    ground: &Ground<'_>,
    faction: FactionId,
    needs: &Needs<'_>,
    window: &PathWindow,
    address: Axial,
    plan: &PlanRegister,
) -> bool {
    let Some(path) = window.path_to(ground.grid, address) else {
        return true;
    };
    path.iter().all(|tile| {
        *tile == needs.seat
            || ground.joined(*tile)
            || plan.zones(faction, *tile).is_some()
            || ground.joining_category(*tile).is_none()
    })
}

/// Chooses the nearest tile of the window that yields a resource and that no
/// finished joining upgrade reaches.
fn choose_deposit(
    ground: &Ground<'_>,
    faction: FactionId,
    needs: &Needs<'_>,
    window: &PathWindow,
    plan: &PlanRegister,
) -> Option<Axial> {
    let mut best: Option<(i64, u32, Axial)> = None;
    for cell in 0..window.tile.len() {
        let index = window.tile[cell];
        if index == NO_TILE {
            continue;
        }
        let tile = TileIdx(index);
        let Some(address) = ground.grid.address_of(tile) else {
            continue;
        };
        if !ground.terrain.kind(address).is_some_and(yields_a_resource) {
            continue;
        }
        let Some(cost) = window.cost_of(address) else {
            continue;
        };
        if joined_all_the_way(ground, faction, needs, window, address, plan) {
            continue;
        }
        let key = (cost, index, address);
        if best.is_none_or(|held| key < held) {
            best = Some(key);
        }
    }
    best.map(|(_, _, address)| address)
}

/// Reports whether a ground kind carries a deposit worth reaching.
///
/// **The engine holds no reader that names the deposits of a faction**, so
/// the solver asks the ground kind whether a gatherer takes anything from
/// it. Water is not walked and the mountain is not gathered, so the three
/// land kinds below are the deposits the plan can see.[^1]
///
/// # References
///
/// [^1]: Findings register, FND-491. `docs/FINDINGS.md`
fn yields_a_resource(kind: crate::terrain::TileKind) -> bool {
    use crate::terrain::TileKind;
    matches!(kind, TileKind::Forest | TileKind::Hill)
}

/// Chooses the held tile whose ground a raise of the yield would improve.
///
/// A faction whose stores fall short plans the category that raises the
/// yield of the ground it holds.[^1] The category is the one whose first row
/// changes the yield, asked of the table and never named.[^2]
///
/// # References
///
/// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D2. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
/// [^2]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
fn choose_yield_ground(
    ground: &Ground<'_>,
    faction: FactionId,
    needs: &Needs<'_>,
    window: &PathWindow,
    plan: &PlanRegister,
) -> Option<Axial> {
    if !needs.short_of_stores {
        return None;
    }
    let mut best: Option<(i64, u32, Axial)> = None;
    for cell in 0..window.tile.len() {
        let index = window.tile[cell];
        if index == NO_TILE {
            continue;
        }
        let tile = TileIdx(index);
        if needs
            .holders
            .get(index as usize)
            .and_then(|holder| holder.faction())
            .map(|held| held.0)
            != Some(faction.0)
        {
            continue;
        }
        if ground.upgrades.at(tile).is_some() || plan.zones(faction, tile).is_some() {
            continue;
        }
        let Some(address) = ground.grid.address_of(tile) else {
            continue;
        };
        if yield_category(ground, address).is_none() {
            continue;
        }
        let Some(cost) = window.cost_of(address) else {
            continue;
        };
        let key = (cost, index, address);
        if best.is_none_or(|held| key < held) {
            best = Some(key);
        }
    }
    best.map(|(_, _, address)| address)
}

/// Returns the category whose first row raises the yield of one ground kind.
fn yield_category(ground: &Ground<'_>, address: Axial) -> Option<UpgradeCategory> {
    let kind = ground.terrain.kind(address)?;
    UpgradeCategory::ALL.into_iter().find(|category| {
        ground
            .table
            .row(*category, 1)
            .is_some_and(|row| row.exists() && row.fits(kind) && row.yield_change > 0)
    })
}

/// Returns the category whose first row raises the housing of one ground
/// kind.
///
/// The category is asked of the table and never named, in the same way the
/// yield category is.[^1]
///
/// # References
///
/// [^1]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
fn lodging_category(ground: &Ground<'_>, address: Axial) -> Option<UpgradeCategory> {
    let kind = ground.terrain.kind(address)?;
    UpgradeCategory::ALL.into_iter().find(|category| {
        ground
            .table
            .row(*category, 1)
            .is_some_and(|row| row.exists() && row.fits(kind) && row.housing_change > 0)
    })
}

/// Chooses the held tile that a lodging would house people at.
///
/// **A lodging only counts where it reaches a site.** The housing of a
/// finished level raises the settlement on its own tile or on one of the six
/// beside it, so the solver zones only a tile that stands on or beside a site
/// of the faction.[^1] A lodging anywhere else would be built and would
/// house nobody.
///
/// The faction must hold the tile, the tile must carry no upgrade, and the
/// table must hold a housing row that fits the ground. An empty tile wins
/// over a tile the plan already names, then the least cost wins, and the
/// lower tile index breaks a tie.
///
/// **A tile the plan already names takes the new category**, which is the
/// rule the plan register already applies.[^2] A plan full of ways would
/// otherwise leave a crowded faction with no way to zone the one thing that
/// moves its population.
///
/// The solver zones one lodging at a time. A plan that already names one
/// returns nothing here, so the solver does its other work until that
/// lodging stands.
///
/// # References
///
/// [^1]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D1. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
/// [^2]: ADR-0152, a faction plans its roads and zones with one solver, decision D1. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
fn choose_lodging_ground(
    ground: &Ground<'_>,
    faction: FactionId,
    needs: &Needs<'_>,
    window: &PathWindow,
    plan: &PlanRegister,
) -> Option<Axial> {
    if !needs.short_of_places {
        return None;
    }
    // The first part of the key says whether the plan already names the
    // tile. An empty tile therefore wins over a tile the plan holds, and the
    // solver takes a held tile only when it has no other choice.
    let mut best: Option<(u32, i64, u32, Axial)> = None;
    for cell in 0..window.tile.len() {
        let index = window.tile[cell];
        if index == NO_TILE {
            continue;
        }
        let tile = TileIdx(index);
        if needs
            .holders
            .get(index as usize)
            .and_then(|holder| holder.faction())
            .map(|held| held.0)
            != Some(faction.0)
        {
            continue;
        }
        if ground.upgrades.at(tile).is_some() {
            continue;
        }
        let Some(address) = ground.grid.address_of(tile) else {
            continue;
        };
        if !reaches_a_site(ground, needs, address) {
            continue;
        }
        let Some(category) = lodging_category(ground, address) else {
            continue;
        };
        let zoned = plan.zones(faction, tile);
        if zoned == Some(category) {
            // The plan already asks for a lodging here. One is enough, so
            // the solver returns to its other work until this one stands.
            return None;
        }
        let Some(cost) = window.cost_of(address) else {
            continue;
        };
        let key = (u32::from(zoned.is_some()), cost, index, address);
        if best.is_none_or(|held| key < held) {
            best = Some(key);
        }
    }
    best.map(|(_, _, _, address)| address)
}

/// Reports whether one address stands on a site of the faction or beside one.
///
/// The sites arrive in ascending tile order, so the test is a search and not
/// a scan.
fn reaches_a_site(ground: &Ground<'_>, needs: &Needs<'_>, address: Axial) -> bool {
    core::iter::once(Some(address))
        .chain(ground.grid.neighbours(address))
        .filter_map(|place| place.and_then(|near| ground.grid.index_of(near)))
        .any(|tile| needs.sites.binary_search(&tile).is_ok())
}

/// Writes the projects of one target into the plan.
///
/// A join writes every tile of the path with the joining category. A raise
/// writes one tile with the yield category, and a lodging writes one tile with
/// the housing category. A tile that already carries a
/// finished joining upgrade is skipped, and so is a tile the plan already
/// names. The write stops at the projects one pass may write, and a plan at
/// its bound drops the rest and counts each drop.[^1]
///
/// # References
///
/// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D1. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
fn write_path(
    ground: &Ground<'_>,
    faction: FactionId,
    needs: &Needs<'_>,
    window: &PathWindow,
    target: Target,
    plan: &mut PlanRegister,
) -> u32 {
    let limit = plan.rules().projects_per_pass();
    if limit == 0 {
        return 0;
    }
    let (tiles, raise) = match target {
        Target::Join(address) => match window.path_to(ground.grid, address) {
            Some(path) => (path, false),
            None => return 0,
        },
        Target::Raise(address) | Target::Lodging(address) => match ground.grid.index_of(address) {
            Some(tile) => (vec![tile], true),
            None => return 0,
        },
    };
    let mut written = 0;
    // The path is walked from the seat outward. The plan holds the tiles in
    // tile order, so the walk decides only which tiles a full plan drops.
    for tile in tiles {
        if written >= limit {
            break;
        }
        if !raise && (tile == needs.seat || ground.joined(tile)) {
            continue;
        }
        // **A lodging takes a tile the plan already names.** The plan
        // register replaces the category on such a tile, and a plan full of
        // ways would otherwise leave a crowded faction unable to zone the
        // one thing that moves its population.[^2]
        //
        // [^2]: ADR-0152, a faction plans its roads and zones with one solver, decision D1. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
        let replaces = matches!(target, Target::Lodging(_));
        if !replaces && plan.zones(faction, tile).is_some() {
            continue;
        }
        let category = match target {
            Target::Join(_) => ground.joining_category(tile),
            Target::Raise(_) => ground
                .grid
                .address_of(tile)
                .and_then(|address| yield_category(ground, address)),
            Target::Lodging(_) => ground
                .grid
                .address_of(tile)
                .and_then(|address| lodging_category(ground, address)),
        };
        let Some(category) = category else {
            continue;
        };
        if plan.write(faction, Project::new(tile, category)).is_ok() {
            written += 1;
        }
    }
    written
}
