//! A zoned project, and the plan that reaches it.
//!
//! A project names work on a tile. A plan is the path a unit walks to reach
//! it. The register, the path readers and the solver sit together, because a
//! plan means nothing without the project it serves.

use super::upgrades::resolve_build_row;
use super::World;
use crate::hex::Axial;
use crate::holding::Holder;
use crate::plan::{self, Needs, PlanRefusal, PlanRegister, PlanRules, Project};
use crate::types::{Entity, FactionId, TileIdx};
use crate::upgrade::UpgradeCategory;

impl World {
    /// Writes one project into the plan of one faction.
    ///
    /// **The solver calls this verb and a Python caller calls it.** No path
    /// exists for the solver alone, so a god that zones a project by hand
    /// puts it in the same list the solver writes to, and a unit cannot tell
    /// the two apart.[^1] [^2]
    ///
    /// The verb refuses a tile past the plan bound, a category that no row of
    /// the table fits, and a tile the faction does not hold when the row asks
    /// for held ground. A row that asks for no held ground is permitted
    /// anywhere, because that row is how a faction reaches ground it does not
    /// yet hold.[^3]
    ///
    /// # Errors
    ///
    /// Returns a refusal when the number names no faction, when the address
    /// lies outside the world, when no row fits, when the row asks for held
    /// ground that the faction does not hold, and when the plan is full.
    ///
    /// # References
    ///
    /// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D4. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
    /// [^2]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    /// [^3]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D4. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    pub fn zone_project(
        &mut self,
        faction: FactionId,
        address: Axial,
        category: UpgradeCategory,
    ) -> Result<(), PlanRefusal> {
        if usize::from(faction.0) >= self.plan.faction_count() {
            self.plan.count_refusal();
            return Err(PlanRefusal::NoSuchFaction(faction));
        }
        let (Some(tile), Some(ground)) = (self.grid.index_of(address), self.terrain.kind(address))
        else {
            self.plan.count_refusal();
            return Err(PlanRefusal::AddressOutsideWorld(address));
        };
        let row = resolve_build_row(
            &self.upgrade_table,
            ground,
            self.upgrades.at(tile),
            category,
        )
        .map_err(|_| {
            self.plan.count_refusal();
            PlanRefusal::NoRowFits { category }
        })?;
        let holder = self
            .holding
            .holders()
            .get(tile.0 as usize)
            .copied()
            .unwrap_or(Holder::NOBODY);
        if row.own_ground_required != 0 && holder.faction().map(|held| held.0) != Some(faction.0) {
            self.plan.count_refusal();
            return Err(PlanRefusal::GroundNotHeld { category });
        }
        self.plan.write(faction, Project::new(tile, category))
    }

    /// Removes the project one faction zoned on one tile.
    ///
    /// Reports whether it removed one. A caller and the solver both reach
    /// this verb.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D4. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
    pub fn clear_project(&mut self, faction: FactionId, address: Axial) -> bool {
        let Some(tile) = self.grid.index_of(address) else {
            return false;
        };
        self.plan.clear(faction, tile)
    }

    /// Returns the projects one faction has zoned, in ascending tile order.
    #[must_use]
    pub fn plan_of(&self, faction: FactionId) -> &[Project] {
        self.plan.projects_of(faction)
    }

    /// Returns the category one faction zoned on one tile.
    #[must_use]
    pub fn project_at(&self, faction: FactionId, address: Axial) -> Option<UpgradeCategory> {
        let tile = self.grid.index_of(address)?;
        self.plan.zones(faction, tile)
    }

    /// Returns the project one unit takes: the nearest by hex distance.
    ///
    /// **This is the assignment rule, and the controller calls this one
    /// function.** When two projects tie on distance, the lower tile index
    /// wins. A unit of another faction, a dead unit and a faction with an
    /// empty plan each give nothing.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D5. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
    #[must_use]
    pub fn project_for(&self, faction: FactionId, unit: Entity) -> Option<Project> {
        if self.soldiers.faction(unit) != Some(faction) {
            return None;
        }
        let here = self.grid.address_of(self.soldiers.tile(unit)?)?;
        // The comparison is the tie rule, and it is written out rather than
        // left to the order of the list. The plan is in tile order, so a
        // comparison that took the last of several equals would take the
        // highest tile index.
        let mut best: Option<(u32, u32, Project)> = None;
        for project in self.plan.projects_of(faction) {
            let Some(there) = self.grid.address_of(project.tile) else {
                continue;
            };
            let key = (here.distance(there), project.tile.0, *project);
            if best.is_none_or(|held| key < held) {
                best = Some(key);
            }
        }
        best.map(|(_, _, project)| project)
    }

    /// Returns the way the solver would lay between two places.
    ///
    /// **This is the path search of the plan, and the solver walks the same
    /// window.** The search relaxes the tiles inside the radius a fixed
    /// number of times and then walks back from the far end, taking the
    /// neighbour with the lowest pair of cost and tile index at every step.
    /// It never runs until the frontier settles.[^1] [^2]
    ///
    /// Returns an empty list when either address lies outside the world, when
    /// the far end lies past the search radius, and when no path of the pass
    /// budget reaches it.
    ///
    /// # References
    ///
    /// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D3. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
    /// [^2]: ADR-0005, a solver runs a fixed iteration count, decision D1. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`
    #[must_use]
    pub fn planned_path(&self, from: Axial, to: Axial) -> Vec<Axial> {
        let ground = plan::Ground {
            grid: self.grid,
            terrain: self.terrain,
            upgrades: &self.upgrades,
            table: &self.upgrade_table,
        };
        let Some(start) = self.grid.index_of(from) else {
            return Vec::new();
        };
        let Some(window) = plan::PathWindow::build(&ground, start, self.plan.rules()) else {
            return Vec::new();
        };
        window
            .path_to(self.grid, to)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|tile| self.grid.address_of(tile))
            .collect()
    }

    /// Returns the cost the path search charged to reach one place from
    /// another.
    ///
    /// The cost is the whole number the ground charges along the way. It is
    /// the value the tie rule of the path compares first, so a reader can see
    /// which two ways tie.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D3. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
    #[must_use]
    pub fn planned_path_cost(&self, from: Axial, to: Axial) -> Option<i64> {
        let ground = plan::Ground {
            grid: self.grid,
            terrain: self.terrain,
            upgrades: &self.upgrades,
            table: &self.upgrade_table,
        };
        let start = self.grid.index_of(from)?;
        let window = plan::PathWindow::build(&ground, start, self.plan.rules())?;
        window.cost_of(to)
    }

    /// Returns the values the plan and its solver read.
    #[must_use]
    pub const fn plan_rules(&self) -> PlanRules {
        self.plan.rules()
    }

    /// Sets the values the plan and its solver read.
    ///
    /// **The call clears every plan.** The bound decides the size of the
    /// register, so a register built with another bound holds its rows
    /// elsewhere, and carrying them over would put a project of one faction
    /// into the plan of another. A caller that changes the rules zones again.
    ///
    /// Every value is a balance row, and a blocker governs each of them.[^1]
    /// [^2]
    ///
    /// # References
    ///
    /// [^1]: Balance register, the plan. `docs/reference/balance.md`
    /// [^2]: Blockers register, BLK-050. `docs/BLOCKERS.md`
    pub fn set_plan_rules(&mut self, rules: PlanRules) {
        self.plan = PlanRegister::new(self.config.faction_count, rules);
    }

    /// Assigns the carriers of one faction, and releases the ones it no
    /// longer needs.
    ///
    /// **A carrier is an idle unit whose type carries.** A unit type column
    /// of zero means that the unit cannot carry, so such a unit is never
    /// assigned.[^1] The lowest identities are taken, so two runs over one
    /// arena take one set.
    ///
    /// The unit takes the site of its own faction as its home and is sent to
    /// the site of the other party, through the two verbs a Python caller
    /// calls.[^2] The delivery pass then moves the quantity when the unit
    /// stands there with a load.[^3]
    ///
    /// **A carrier arrives only while its load stays below the carry mark.**
    /// A unit that holds a home and a load at the mark is laden, and a laden
    /// unit walks home rather than where it was sent.[^4] The mark is a world
    /// parameter that a caller sets, and a world whose mark is below what a
    /// carrier picks up on the way sends its carriers home instead.
    ///
    /// Returns whether the stage assigned or released anything.
    ///
    /// # References
    ///
    /// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D1. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    /// [^2]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    /// [^3]: ADR-0147, a contract consideration is a tagged kind, decision D2. `docs/adrs/accepted/adr-0147-a-contract-consideration-is-a-tagged-kind.md`
    /// [^4]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction, decision D1. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
    /// Writes the plan of one faction, in a fixed pass count.
    ///
    /// **The solver reads three things of one faction**: the tiles of its
    /// settlements, the ground it holds, and whether its stores fall short of
    /// the mark above which a site offers. It reads no unit, and it reads no
    /// tile outside the window it builds around the seat.[^1]
    ///
    /// The pass count and the window are balance values, and a blocker
    /// governs each of them.[^2] [^3]
    ///
    /// # References
    ///
    /// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D2. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
    /// [^2]: Balance register, the plan. `docs/reference/balance.md`
    /// [^3]: Blockers register, BLK-050. `docs/BLOCKERS.md`
    pub(super) fn solve_plan(&mut self, faction: FactionId) -> u32 {
        let Some(seat) = self.seat(faction) else {
            return 0;
        };
        // The settlements of one faction, in ascending tile order. The scan
        // follows the settlements and never the tile count.
        let mut sites: Vec<TileIdx> = self
            .settlements
            .iter()
            .filter(|site| self.settlements.faction(*site) == Some(faction))
            .filter_map(|site| self.settlements.tile(site))
            .collect();
        sites.sort_unstable();
        sites.dedup();
        let stores = self.faction_stores(faction);
        let mark = i64::from(self.controller.surplus_mark());
        let short_of_stores = stores.iter().any(|held| *held < mark);
        // Whether every site of the faction is full. A faction with no free
        // place anywhere grows nobody, so the plan puts a lodging first
        // until one site has room again.[^1]
        //
        // The scan follows the settlements of the faction and never the
        // population.[^2]
        //
        // [^1]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D2. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
        // [^2]: ADR-0096, cost follows the lattice, not the population, decision D1. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
        let mut holds_a_site = false;
        let mut every_site_full = true;
        for site in self.settlements.iter() {
            if self.settlements.faction(site) != Some(faction) {
                continue;
            }
            holds_a_site = true;
            if self
                .settlements
                .slot_of(site)
                .is_some_and(|slot| self.free_places_of(slot) > 0)
            {
                every_site_full = false;
            }
        }
        let short_of_places = holds_a_site && every_site_full;
        let ground = plan::Ground {
            grid: self.grid,
            terrain: self.terrain,
            upgrades: &self.upgrades,
            table: &self.upgrade_table,
        };
        let needs = Needs {
            seat,
            sites: &sites,
            holders: self.holding.holders(),
            short_of_stores,
            short_of_places,
        };
        plan::solve(&ground, faction, &needs, &mut self.plan)
    }
}
