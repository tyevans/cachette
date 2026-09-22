//! How the controller picks its targets: the rival that nears a win, and the
//! tile a crossing or a settling aims at.
//!
//! A crossing and a settling each survey a plane of candidate tiles and take
//! one. The two surveys share the shape of the answer and the constants that
//! bound it, so they sit together rather than inside the controller pass that
//! calls them. The win threat reads the reading of every faction on every win
//! path, and it names the city a march on a wonder aims at.

use super::victory::WonderSite;
use super::World;
use crate::bridge::BLOCK_BITS_DEFAULT;
use crate::controller::{self, WinPath, WIN_PATH_COUNT};
use crate::founding::{self};
use crate::hex::Axial;
use crate::sim_math;
use crate::types::{Accum, Entity, FactionId, Fix32, TileIdx};

/// One rival that nears a win, as one faction sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WinThreat {
    /// The rival.
    pub(super) faction: FactionId,
    /// The tile of the city whose ground holds the wonder of the rival, or
    /// `None` when the rival does not near the wonder path, or when the
    /// wonder lookup names no live city for it.
    pub(super) wonder_city: Option<TileIdx>,
}

/// The group that the crossing order surveys for.
///
/// The survey scores a place against a group it must feed, and it refuses a
/// place that cannot.[^1] The crossing order founds nothing and seats nobody.
/// It names a place to walk to, so it asks for the smallest group the survey
/// admits and takes the best place that group could live at.
///
/// # References
///
/// [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D1. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
const CROSSING_SURVEY_GROUP: u32 = 1;

/// The group the settling survey asks a place to hold.
///
/// The survey scores a place against the group that would live there, and the
/// settle verb seats the group the settle column of the type names. The
/// target choice asks for the smallest group the survey admits, because a
/// place the smallest group cannot take is a place no settler can found on.
const SETTLING_SURVEY_GROUP: u32 = 1;

/// The ticks a settler lives away from a site that feeds it.
///
/// A settler that walks carries no home, so nothing feeds it and its deficit
/// grows by a fixed step each tick until it reaches the bound. The value is
/// measured and not derived: a probe sends one settler at distant ground and
/// reads the arena after every tick, and the starved log names the unit on
/// the same tick on every world the probe ran.[^1]
///
/// The figure is a property of the need rule and of nothing else. Standing on
/// water does not change it. The probe measured the same lifetime for a
/// settler that spent a third of it at sea.
///
/// # References
///
/// [^1]: The settler range probe. `crates/cachette-core/examples/settler_range_probe.rs`
const SETTLER_LIFETIME: u32 = 89;

/// How far a settler may be sent from the sites of its faction.
///
/// **A settler walks to its target and it eats on the way.** A place on the
/// far side of the world is a place the settler starves before it reaches, so
/// the target choice takes the best place inside this reach rather than the
/// best place in the world.
///
/// The value is half the measured lifetime, because a settler walks one tile
/// in one tick and it does not walk in a straight line. Ground that refuses a
/// step sends it round, and each detour costs a tick it cannot get back. Half
/// the lifetime leaves it as many ticks in hand as it spent walking.
///
/// **The last cell is no longer part of that margin.** The approach field
/// resolves the block that holds the target at the pitch of one tile, so a
/// settler that reaches that block walks at the target rather than wandering
/// inside it.[^2]
///
/// The two assertions below are floors and not knobs. A reach at or under the
/// founding distance would leave no place eligible, because the survey
/// refuses every place nearer than that distance. A reach under the edge of a
/// level 1 cell would name a target in the cell the settler already stands
/// in, and the coarse field steers nobody inside one cell.
///
/// [^2]: Findings register, FND-315. `docs/FINDINGS.md`
const SETTLER_REACH: u32 = SETTLER_LIFETIME / 2;

const _: () = assert!(
    SETTLER_REACH > founding::MINIMUM_FOUNDING_DISTANCE,
    "a reach inside the founding distance leaves no place eligible"
);

const _: () = assert!(
    SETTLER_REACH > 1 << BLOCK_BITS_DEFAULT,
    "a reach inside one level 1 cell names a target that steers nobody"
);

impl World {
    /// Returns the reading of every faction on every win path, by faction
    /// number and then by the number of the path.
    ///
    /// A reading is a bounded share. Its numerator is what the faction has
    /// done toward the requirement of the path, and its denominator is the
    /// requirement.[^1] **Each numerator and each requirement comes from a
    /// reader the engine already holds**, and this states no rule of its own
    /// for how far a faction has come:
    ///
    /// - Domination reads the seats of the factions still in the game that
    ///   the faction holds, over those seats.
    /// - Territory reads the held ground of the faction, over the passable
    ///   ground of the world.
    /// - Wonder reads the wonder lookup. The reading is the highest progress
    ///   share among the wonder sites whose holder is the faction.
    /// - Renown reads the best renown of a live character of the faction,
    ///   over the renown target.
    ///
    /// **The observation publishes the same progress to each faction about
    /// itself.** For domination and territory it calls the same readers. For
    /// renown and the wonder it folds the same columns in a pass of its own,
    /// so a test compares the two and fails when they part.[^2]
    ///
    /// **This reads the whole world, and not what one faction observes.** The
    /// controller reads the whole world for its rival and its prey as well.
    /// Whether the standing of a rival toward a win is public is an open
    /// question, and a blocker holds it.[^3]
    ///
    /// The walk is over the seats, the characters and the wonder sites, so
    /// it is not a walk over the units or the tiles.[^4]
    ///
    /// # References
    ///
    /// [^1]: ADR-0204, every win path holds a bar of its own, decision D4. `docs/adrs/draft/adr-0204-every-win-path-holds-a-bar-of-its-own.md`
    /// [^2]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    /// [^3]: Blockers register, BLK-160. `docs/BLOCKERS.md`
    /// [^4]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D1. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    #[must_use]
    pub fn win_readings(&self) -> Vec<[Fix32; WIN_PATH_COUNT]> {
        self.win_readings_over(&self.wonder_sites())
    }

    /// Returns the readings, over a wonder lookup the caller already holds.
    ///
    /// The controller stage reads the lookup once for the readings and once
    /// for the city of a wonder, so it passes one copy to both.
    pub(super) fn win_readings_over(&self, wonders: &[WonderSite]) -> Vec<[Fix32; WIN_PATH_COUNT]> {
        let seats = usize::from(self.config.faction_count.max(1));
        let seat_holding = self.seat_holding(seats);
        let renown = self.best_renown();
        let passable = self.pyramid().total().open_tiles();
        let target = i64::from(self.balance.renown_target());
        (0..seats)
            .map(|at| {
                let faction = FactionId(at as u16);
                WinPath::ALL.map(|path| match path {
                    WinPath::Domination => sim_math::bounded_share(
                        seat_holding.live_held.get(at).copied().unwrap_or(0),
                        seat_holding.live_seats,
                    ),
                    WinPath::Territory => {
                        sim_math::bounded_share(self.holding.holding_of(faction), passable)
                    }
                    WinPath::Wonder => wonders
                        .iter()
                        .filter(|site| site.holder == Some(faction))
                        .map(|site| site.progress_share())
                        .max()
                        .unwrap_or(Fix32::ZERO),
                    WinPath::Renown => {
                        sim_math::bounded_share(renown.get(at).copied().unwrap_or(0), target)
                    }
                })
            })
            .collect()
    }

    /// Returns the win threat of every faction, by faction number.
    ///
    /// The threat of a faction is the rival that nears a win, as the choice
    /// in the controller module names it from the share of that faction.[^1]
    /// Only a faction still in the game is a candidate, because a faction
    /// that has left the game wins nothing.[^2]
    ///
    /// **A threat that nears the wonder path names the city whose ground
    /// holds its wonder.** The city is the one the wonder lookup names, so
    /// this states no rule of its own for which city holds a wonder.[^3] A
    /// rival with more than one wonder site names the site with the highest
    /// share, and a tie takes the lower tile, because the lookup is in tile
    /// order and the scan keeps the first site at the highest share.
    ///
    /// # References
    ///
    /// [^1]: Balance register, the win threat share. `docs/reference/balance.md`
    /// [^2]: ADR-0181, a faction that holds no site and no unit leaves the game, decision D5. `docs/adrs/draft/adr-0181-a-faction-that-holds-no-site-and-no-unit-leaves-the-game.md`
    /// [^3]: ADR-0174, a wonder is a win path and a stock total is not, decision D1. `docs/adrs/draft/adr-0174-a-wonder-is-a-win-path-and-a-stock-total-is-not.md`
    pub(super) fn win_threats(
        &self,
        readings: &[[Fix32; WIN_PATH_COUNT]],
        wonders: &[WonderSite],
    ) -> Vec<Option<WinThreat>> {
        let highest: Vec<(FactionId, Fix32)> = readings
            .iter()
            .enumerate()
            .map(|(at, paths)| {
                (
                    FactionId(at as u16),
                    paths.iter().copied().max().unwrap_or(Fix32::ZERO),
                )
            })
            .filter(|(faction, _)| !self.is_eliminated(*faction))
            .collect();
        (0..readings.len())
            .map(|at| {
                let faction = FactionId(at as u16);
                let share = self.controller.win_threat_share(faction)?;
                let threat = controller::win_threat_of(
                    faction,
                    share,
                    highest
                        .iter()
                        .copied()
                        .filter(|(other, _)| self.has_met(faction, *other)),
                )?;
                let nears_the_wonder = readings
                    .get(usize::from(threat.0))
                    .is_some_and(|paths| paths[WinPath::Wonder.index()].0 >= share.0);
                let wonder_city = if nears_the_wonder {
                    self.city_of_the_wonder_of(threat, wonders)
                } else {
                    None
                };
                Some(WinThreat {
                    faction: threat,
                    wonder_city,
                })
            })
            .collect()
    }

    /// Returns the tile of the city whose ground holds the furthest wonder of
    /// one faction, or `None` when the lookup names no live city for it.
    fn city_of_the_wonder_of(&self, faction: FactionId, wonders: &[WonderSite]) -> Option<TileIdx> {
        let mut furthest: Option<WonderSite> = None;
        for site in wonders.iter().filter(|site| site.holder == Some(faction)) {
            match furthest {
                Some(best) if site.progress_share().0 <= best.progress_share().0 => {}
                _ => furthest = Some(*site),
            }
        }
        furthest
            .and_then(|site| site.settlement)
            .and_then(|city| self.settlements.tile(city))
    }

    /// Sends the idle units of one faction to the projects its plan zones.
    ///
    /// **Each unit takes the project nearest to it by hex distance, and a tie
    /// takes the lower tile index.** A unit that is not idle is not moved,
    /// and the order goes through the send verb and the build verb that a
    /// Python caller also calls.[^1] [^2]
    ///
    /// The faction climbs one destination plane, and the plane of a faction
    /// is its number. A faction that holds a live campaign or a carrier is
    /// already climbing that plane, so it takes no project order and the
    /// command is refused. That rule is the one the campaign already applies
    /// to a faction that holds a carrier.
    ///
    /// The cost follows the idle units multiplied by the plan bound. The
    /// bound is fixed, so the cost follows the population and not the
    /// world.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0152, a faction plans its roads and zones with one solver, decision D5. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
    /// [^2]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    /// Returns the destination plane that one faction crosses water on.
    ///
    /// **The crossing takes a plane of its own, and it never shares one.** A
    /// campaign, a carrier and a project order all climb the plane whose
    /// number is the faction number, so one of those three must yield to
    /// another. A crossing cannot yield: a faction on an island that waits
    /// for its war to end waits for a war it cannot reach, and the capability
    /// then ships inert.
    ///
    /// The crossing plane of a faction is its number raised by the faction
    /// count, so no faction takes the plane of another and no crossing takes
    /// the plane of a march. A world with too few planes for that answers
    /// nothing, and the faction takes no crossing order until a caller raises
    /// the plane count.
    ///
    /// Returns `None` when the world holds no such plane.
    fn crossing_plane_of(&self, faction: FactionId) -> Option<u16> {
        let plane = faction.0.checked_add(self.config.faction_count.max(1))?;
        (plane < self.destinations.plane_count()).then_some(plane)
    }

    /// Returns the water-crossing units of one faction that nobody has sent
    /// anywhere, in ascending identity order.
    ///
    /// The crossing is a column of the shared type table, and zero means
    /// cannot.[^1] The scan follows the population of the faction and reads
    /// no tile, so its cost does not grow with the world.
    ///
    /// # References
    ///
    /// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    fn idle_crossers_of(&self, faction: FactionId) -> Vec<Entity> {
        let mut units: Vec<Entity> = self
            .soldiers
            .iter_faction(faction)
            .filter(|unit| self.soldiers.sent(*unit) == Some(None))
            .filter(|unit| {
                self.soldiers
                    .unit_type(*unit)
                    .is_some_and(|unit_type| self.unit_types.row(unit_type).water_crossing > 0)
            })
            .collect();
        units.sort_unstable_by_key(|unit| unit.to_bits());
        units
    }

    /// Returns the tile that one faction would send its water-crossing units
    /// at, or nothing.
    ///
    /// **The choice reads the same bounded sample the founding choice
    /// reads.** It draws a fixed number of candidate places and reads a fixed
    /// number of tiles around each, so its cost does not grow with the
    /// world.[^1] The draw is keyed on the faction and on nothing that
    /// changes, so the target of a faction is the same tile on every tick and
    /// the plane does not thrash.[^2]
    ///
    /// The places the faction already holds are the places taken, so the
    /// sample offers ground at least the founding distance away from every
    /// site the faction owns.[^3] That is ground the faction has not
    /// settled, on its own landmass or on another.
    ///
    /// The answer is nothing when the faction holds no idle unit that crosses
    /// water, and when a campaign or a carrier already climbs its plane. One
    /// plane serves one purpose at a time, which is the rule the campaign and
    /// the carriers already keep.
    ///
    /// # References
    ///
    /// [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D1. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
    /// [^2]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
    /// [^3]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D1. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
    pub(super) fn controller_crossing_target(&self, faction: FactionId) -> Option<TileIdx> {
        if self.idle_crossers_of(faction).is_empty() {
            return None;
        }
        self.crossing_plane_of(faction)?;
        // The walk is over the settlement slots in ascending order, so the
        // list of taken places is a property of the arena and not of a visit
        // order.
        let taken: Vec<Axial> = self
            .settlements
            .iter()
            .filter(|site| self.settlements.faction(*site) == Some(faction))
            .filter_map(|site| self.settlements.address(site))
            .collect();
        let survey = self
            .survey_founding_apart(CROSSING_SURVEY_GROUP, faction, &taken)
            .ok()?;
        // **The order takes the best eligible candidate, and not the best
        // one.** The founding takes the best one and refuses the whole sample
        // when that place is too near a place already taken, because a
        // founding that walked down the list would seat a group somewhere
        // nobody chose. A crossing order names a place to walk to, so it
        // takes the next place down instead of refusing.
        //
        // The candidates are ordered on a total key, so the first eligible
        // one is a property of the sample and not of the order it was drawn
        // in.[^4]
        //
        // [^4]: ADR-0004, iteration order is explicit, decision D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
        survey
            .candidates()
            .iter()
            .find(|candidate| candidate.is_eligible())
            .map(|candidate| candidate.tile())
    }

    /// Sends the idle water-crossing units of one faction at one tile.
    ///
    /// **The order goes through the send verb a Python caller calls.**[^1]
    /// The set holds only units that cross water, so the plane it climbs
    /// conducts across water and the field steers the set over a strait
    /// rather than at the near shore of one.[^2]
    ///
    /// The faction climbs the destination plane whose number is its own, in
    /// the way a campaign and a project order do.
    ///
    /// # References
    ///
    /// [^1]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    /// [^2]: ADR-0125, the control plane names the seed set of a destination field, decision D1. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
    pub(super) fn controller_cross(&mut self, faction: FactionId, tile: TileIdx) -> bool {
        let Some(plane) = self.crossing_plane_of(faction) else {
            return false;
        };
        let Some(address) = self.grid.address_of(tile) else {
            return false;
        };
        let set = self.idle_crossers_of(faction);
        if set.is_empty() {
            return false;
        }
        self.send_units_to(&set, &[address], plane).is_ok()
    }

    /// Returns the destination plane that one faction settles on.
    ///
    /// **The settling takes a plane of its own, and it never shares one.** A
    /// campaign, a carrier and a project order all climb the plane whose
    /// number is the faction number, and a crossing climbs the plane above
    /// those. A settling cannot yield to a campaign: a faction fights for
    /// most of a run, so a settler that waits for the war to end never founds
    /// anything, and the capability then ships inert.
    ///
    /// The settling plane of a faction is its number raised by twice the
    /// faction count, so no faction takes the plane of another, no settling
    /// takes the plane of a march, and no settling takes the plane of a
    /// crossing. A world with too few planes for that answers nothing, and
    /// the faction founds only where a settler already stands.
    ///
    /// Returns `None` when the world holds no such plane.
    pub(super) fn settling_plane_of(&self, faction: FactionId) -> Option<u16> {
        let above = self.config.faction_count.max(1).checked_mul(2)?;
        let plane = faction.0.checked_add(above)?;
        (plane < self.destinations.plane_count()).then_some(plane)
    }

    /// Returns the settlers of one faction, in a fixed order.
    ///
    /// A settler is a unit whose type row holds a settle column above zero.
    /// The gate reads that column and never a type index, so no rule here
    /// names a type.[^1]
    ///
    /// The walk is over the units of the faction, and the answer is sorted on
    /// the identity bits, so the order is the same at every thread count.[^2]
    ///
    /// # References
    ///
    /// [^1]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    pub(super) fn settlers_of(&self, faction: FactionId) -> Vec<Entity> {
        let mut units: Vec<Entity> = self
            .soldiers
            .iter_faction(faction)
            .filter(|unit| {
                self.soldiers
                    .unit_type(*unit)
                    .is_some_and(|unit_type| self.unit_types.row(unit_type).settle_group > 0)
            })
            .collect();
        units.sort_unstable_by_key(|unit| unit.to_bits());
        units
    }

    /// Returns the tile that one faction would send its settlers at, or
    /// nothing.
    ///
    /// **The choice reads the same bounded sample the founding choice
    /// reads.** It draws a fixed number of candidate places and reads a fixed
    /// number of tiles around each, so its cost does not grow with the
    /// world.[^1]
    ///
    /// **Every settlement that stands is a place taken, whatever faction owns
    /// it.** The sample therefore offers ground at least the founding distance
    /// away from every city in the world, which is the list the settle verb
    /// compares against when the settler arrives. A place that became too near
    /// while the settler walked is still refused.[^2] [^5]
    ///
    /// **The order takes the best eligible candidate inside the reach of a
    /// settler.** The rule has three parts, and each part is a filter or an
    /// order over the sample.
    ///
    /// **Beyond a distance.** A candidate must keep the founding distance
    /// from every settlement that stands. The survey applies that rule, with
    /// every standing place as the places taken, and the settle verb applies
    /// the same rule over the same list when the settler arrives.[^2] The
    /// distance is the founding distance and not a second one. A second
    /// constant would be one fact in two places, and the verb would then
    /// admit a place the target choice refused.[^4]
    ///
    /// **The reach is measured from the seats of this faction alone.** The
    /// distance a settler must walk runs from the city it leaves, and a
    /// faction leaves from its own city. The separation list and the reach
    /// list are therefore two lists, and each answers the question it is
    /// asked.
    ///
    /// **Eligible.** A candidate must be what the survey already calls
    /// eligible: ground that admits a settlement, ground that keeps the
    /// distance, and ground with room for the group.
    ///
    /// **Best.** Among what passes those two, the order takes the highest
    /// score. A tie takes the lower tile index, so the answer is a property
    /// of the sample and not of the draw order.[^3]
    ///
    /// **The reach is what keeps the settler alive.** A settler carries no
    /// home, so nothing feeds it and it starves after a measured number of
    /// ticks. The best place in a world-wide sample is usually a place the
    /// settler dies before reaching, which is why the order takes the best
    /// place inside the reach rather than the best place in the world.
    ///
    /// **A faction with nothing in reach takes the nearest eligible place
    /// instead.** The sample is drawn over the whole world, so a faction
    /// hemmed in by held ground may draw no eligible place inside the reach.
    /// The order sends the settler at the nearest such place rather than
    /// keeping it at home, because a settler that never walks founds nothing
    /// and the walk may still end at a place the settle verb admits.
    ///
    /// # References
    ///
    /// [^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D1. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
    /// [^2]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D1. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
    /// [^3]: ADR-0004, iteration order is explicit, decision D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^4]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    /// [^5]: Findings register, FND-693. `docs/FINDINGS.md`
    #[must_use]
    pub fn settling_target_of(&self, faction: FactionId) -> Option<Axial> {
        self.settling_target(faction)
    }

    /// How far a settler may be sent from the sites of its faction.
    ///
    /// The reader states the one value the target choice applies. A caller
    /// that checks the rule reads it here rather than holding a second copy
    /// of the number.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
    #[must_use]
    pub const fn settler_reach() -> u32 {
        SETTLER_REACH
    }

    pub(super) fn settling_target(&self, faction: FactionId) -> Option<Axial> {
        // **The separation rule reads the reader the settle verb reads.** The
        // verb compares a place against every settlement that stands, of every
        // faction, so a target chosen against the sites of one faction alone
        // is a place the verb then refuses. The two lists disagreed, and the
        // walk sent settlers at ground beside a rival city for the whole
        // run.[^5]
        //
        // [^5]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
        let taken = self.standing_places();
        // The walk is over the settlement slots in ascending order, so the
        // list of the seats of this faction is a property of the arena and
        // not of a visit order.
        let mine: Vec<Axial> = self
            .settlements
            .iter()
            .filter(|site| self.settlements.faction(*site) == Some(faction))
            .filter_map(|site| self.settlements.address(site))
            .collect();
        let survey = self
            .survey_founding_apart(SETTLING_SURVEY_GROUP, faction, &taken)
            .ok()?;
        // The eligibility and the founding distance both come from the
        // survey, so this walk states no rule of its own. It measures the
        // distance from the site of the faction nearest to the candidate,
        // because that is the site the settler leaves from.
        let in_reach: Vec<(u32, Accum, TileIdx, Axial)> = survey
            .candidates()
            .iter()
            .filter(|candidate| candidate.is_eligible())
            .filter_map(|candidate| {
                let tile = candidate.tile();
                let address = self.grid.address_of(tile)?;
                if self
                    .holding
                    .holder(address)
                    .is_some_and(|holder| !holder.is_nobody())
                {
                    return None;
                }
                let near = mine
                    .iter()
                    .map(|seat| seat.distance(address))
                    .min()
                    .unwrap_or(0);
                Some((near, candidate.score(), tile, address))
            })
            .collect();
        // **The best place inside the reach.** The score is the key and the
        // tile index breaks a tie, so no two keys are equal and the answer
        // does not depend on the order the candidates were drawn in.
        let best = in_reach
            .iter()
            .filter(|(near, _, _, _)| *near <= SETTLER_REACH)
            .max_by_key(|(_, score, tile, _)| (score.0, core::cmp::Reverse(tile.0)));
        if let Some((_, _, _, address)) = best {
            return Some(*address);
        }
        // Nothing eligible lies inside the reach. The nearest eligible place
        // is the fallback, and the tile index breaks a tie for the same
        // reason.
        in_reach
            .iter()
            .min_by_key(|(near, _, tile, _)| (*near, tile.0))
            .map(|(_, _, _, address)| *address)
    }

    /// Returns the best eligible unheld place within settler reach, or `None`.
    pub(super) fn settling_target_within_reach(&self, faction: FactionId) -> Option<Axial> {
        let mine: Vec<Axial> = self
            .settlements
            .iter()
            .filter(|site| self.settlements.faction(*site) == Some(faction))
            .filter_map(|site| self.settlements.address(site))
            .collect();
        let taken: Vec<Axial> = self
            .settlements
            .iter()
            .filter_map(|site| self.settlements.address(site))
            .collect();
        let survey = self
            .survey_founding_apart(SETTLING_SURVEY_GROUP, faction, &taken)
            .ok()?;
        let in_reach: Vec<(u32, Accum, TileIdx, Axial)> = survey
            .candidates()
            .iter()
            .filter(|candidate| candidate.is_eligible())
            .filter_map(|candidate| {
                let tile = candidate.tile();
                let address = self.grid.address_of(tile)?;
                if self
                    .holding
                    .holder(address)
                    .is_some_and(|holder| !holder.is_nobody())
                {
                    return None;
                }
                let near = mine
                    .iter()
                    .map(|seat| seat.distance(address))
                    .min()
                    .unwrap_or(0);
                (near <= SETTLER_REACH).then_some((near, candidate.score(), tile, address))
            })
            .collect();
        let best = in_reach
            .iter()
            .max_by_key(|(_, score, tile, _)| (score.0, core::cmp::Reverse(tile.0)));
        best.map(|(_, _, _, address)| *address)
    }

    /// Founds a city from every settler of one faction that stands on ground
    /// a city may take, and sends the rest at ground worth founding on.
    ///
    /// **A settler that never walks is inert.** A site builds a settler
    /// inside the ground its own faction holds, and the settle verb refuses
    /// held ground, so a settler that stays where it was built never founds
    /// anything.[^1] The order therefore does two things, and which one a
    /// settler gets depends on where it stands.
    ///
    /// The founding is tried first, over every settler of the faction, so a
    /// settler that has arrived founds on the tick it arrives. The order
    /// sends the settlers only when no founding was made, so a settler that
    /// stands on a place a city may take is never walked away from it.
    ///
    /// **The send goes through the send verb a Python caller calls**, and it
    /// climbs the destination plane whose number is the faction number, in
    /// the way a campaign and a project order do.[^2] The send is refused
    /// when a campaign or a carrier already climbs that plane, which is the
    /// rule the project order keeps.
    ///
    /// # References
    ///
    /// [^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D5. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
    /// [^2]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
    pub(super) fn controller_settle(&mut self, faction: FactionId) -> bool {
        let settlers = self.settlers_of(faction);
        if settlers.is_empty() {
            return false;
        }
        // The founding is tried over every settler, sent or free, because a
        // settler that has walked to its target is still climbing the plane
        // on the tick it arrives.
        if self
            .settle_set(&settlers)
            .iter()
            .any(crate::founding::SettleOutcome::founded)
        {
            return true;
        }
        // Nothing founded, so every settler stands on ground a city may not
        // take. The order walks them at ground that a city may.
        let Some(plane) = self.settling_plane_of(faction) else {
            return false;
        };
        let Some(address) = self.settling_target(faction) else {
            return false;
        };
        self.send_units_to(&settlers, &[address], plane).is_ok()
    }
}
