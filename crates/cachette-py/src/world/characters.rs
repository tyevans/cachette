//! The characters, their lineage, their deeds and their influence.
//!
//! This module holds the character readers, the lineage and relation reports,
//! the deeds of a unit, the verbs that create, bear and remove characters, and
//! the rules that govern renown, influence, a home site and the schedule the
//! character pass runs on.
//!
//! The grouping is by the subject. A character is an entity of its own arena,
//! with its own identity resolution, so its helpers live beside its
//! methods.

use super::PyWorld;
use crate::errors::{VerbError, ViewError};
use crate::world::identity::{resolve, resolve_site};
use cachette_core::character::CharacterArena;
use cachette_core::descent::{DescentId, DESCENT_CEILING};
use cachette_core::{Axial, Entity, FactionId, Fix32, Influence, World as CoreWorld};
use numpy::{PyArray1, ToPyArray};
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pymethods]
impl PyWorld {
    /// Sets how often the engine applies production and upkeep.
    ///
    /// Returns `None`.
    ///
    /// The period is a count of ticks, and it is at least one and at most
    /// 32767. A period of one applies the economy on every step. The phase is
    /// the offset inside the period, so a period of four and a phase of one
    /// apply on the ticks one, five and nine. A phase at or above the period
    /// wraps into it.
    ///
    /// **Raising the period does not raise what a site earns over a span of
    /// ticks.** A rate is what one tick earns, and the engine multiplies it by
    /// the period. The period decides how often a store moves. It does not
    /// decide how much the store moves over time.[^1]
    ///
    /// **The schedule may be set at any time.** It is world-wide, so it is one
    /// write for the world and never one write for each site. It enters the
    /// state hash, because two worlds that apply the economy on different
    /// ticks must diverge.[^2]
    /// Returns every living character, as a `dict` of columns.
    ///
    /// The faction is the number of one faction, or `None` for the whole
    /// world. The default is `None`. A faction number that names no faction
    /// of this world gives empty columns. That is an answer and not an
    /// error, because no character holds that number.
    ///
    /// **The call answers about a set and crosses once.** A caller reads the
    /// whole population in one call and then works on the arrays. It never
    /// asks about one person at a time, because that is the loop the control
    /// plane rule forbids.[^1]
    ///
    /// Seven entries are one-dimensional NumPy arrays with one entry for
    /// each living character.
    ///
    /// - `character`, `numpy.uint64`. The identity of the character. Keep it
    ///   and pass it to any other call that takes a character. Take one entry
    ///   as a Python integer.
    /// - `birth_order`, `numpy.uint32`. The position of the character in the
    ///   record of descent. It counts from zero over every character the
    ///   world has ever made. **It is data and not an identity.** No call in
    ///   this module takes it.
    /// - `faction`, `numpy.uint16`. The number of the faction the character
    ///   belongs to.
    /// - `birth`, `numpy.uint64`. The tick the character was made on.
    /// - `renown`, `numpy.int32`. How much the character is thought of.
    ///   **This is a Q16.16 value as its raw integer. Divide by 65536.**
    ///   Nothing in the engine writes it, so it is zero until a caller writes
    ///   it with `set_character_renown`.
    /// - `sex`, `numpy.uint8`. Zero is female and one is male. The engine
    ///   draws it and no caller sets it.
    /// - `house`, `numpy.uint32`. The birth order of the character who
    ///   founded the house. A character with no father founds a house, and
    ///   its own birth order names it.
    ///
    /// **Every entry names a live character, so no entry stands for
    /// nothing.** The engine builds the set at the moment of the call and
    /// takes no identity from the caller.
    ///
    /// **A character identity and a unit identity share one range of
    /// numbers.** Each arena numbers its own slots, so the first character of
    /// a world and the first unit of a world carry the same number. A call
    /// that takes a character reads the character arena. A call that takes a
    /// unit reads the soldier arena. Neither refuses the number of the other,
    /// and no check reports the mistake. Keep the two kinds of identity
    /// apart. A finding records the measurement.[^3]
    ///
    /// The order is the slot order of the arena. It is the same on every run
    /// and at every thread count. It is never a thread completion order.[^2]
    /// It is not the birth order. A slot returns to the arena when a
    /// character is removed, and the next character takes it. Sort on
    /// `birth_order` for the birth order.
    ///
    /// # References
    ///
    /// [^1]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^3]: Findings register, FND-472. `docs/FINDINGS.md`
    #[pyo3(signature = (faction = None))]
    fn characters<'py>(
        &self,
        python: Python<'py>,
        faction: Option<u16>,
    ) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let arena = world.characters();
        let mut character: Vec<u64> = Vec::new();
        let mut birth_order: Vec<u32> = Vec::new();
        let mut faction_of: Vec<u16> = Vec::new();
        let mut birth: Vec<u64> = Vec::new();
        let mut renown: Vec<i32> = Vec::new();
        let mut sex: Vec<u8> = Vec::new();
        let mut house: Vec<u32> = Vec::new();
        for entity in arena.iter() {
            let held = arena
                .faction(entity)
                .expect("a live identity from the walk names a faction");
            if faction.is_some_and(|wanted| held.0 != wanted) {
                continue;
            }
            character.push(entity.to_bits());
            birth_order.push(
                arena
                    .descent_id(entity)
                    .expect("a live identity from the walk names a descent row")
                    .birth_order(),
            );
            faction_of.push(held.0);
            birth.push(
                arena
                    .birth(entity)
                    .expect("a live identity from the walk names a birth")
                    .0,
            );
            renown.push(
                arena
                    .renown(entity)
                    .expect("a live identity from the walk names a renown")
                    .0,
            );
            sex.push(
                arena
                    .sex(entity)
                    .expect("a live identity from the walk names a sex")
                    .to_column(),
            );
            house.push(
                arena
                    .house(entity)
                    .expect("a live identity from the walk names a house")
                    .founder()
                    .birth_order(),
            );
        }
        let columns = PyDict::new(python);
        columns.set_item("character", character.to_pyarray(python))?;
        columns.set_item("birth_order", birth_order.to_pyarray(python))?;
        columns.set_item("faction", faction_of.to_pyarray(python))?;
        columns.set_item("birth", birth.to_pyarray(python))?;
        columns.set_item("renown", renown.to_pyarray(python))?;
        columns.set_item("sex", sex.to_pyarray(python))?;
        columns.set_item("house", house.to_pyarray(python))?;
        Ok(columns)
    }

    /// Returns the whole lineage of one character, as a `dict`.
    ///
    /// The character is one identity that a call of this module gave.
    ///
    /// **One call answers the whole question.** The parents, every ancestor
    /// and every descendant come back together. A caller never walks the
    /// record one step at a time. A walk across the boundary is the loop the
    /// control plane rule forbids.[^1] The engine walks the record in Rust
    /// and answers once.
    ///
    /// Three entries are plain values that describe the character asked
    /// about.
    ///
    /// - `character`, an `int`. The identity that was asked about.
    /// - `birth_order`, an `int`. The position of the character in the record
    ///   of descent, counted from zero.
    /// - `house`, an `int`. The birth order of the character who founded the
    ///   house. Two characters of one house share that number.
    ///
    /// **This call does not say whether a line has ended.** The engine
    /// reports that, and it reports it for a character who is gone. This call
    /// takes a living character. A line with a living member has not ended,
    /// so the answer would be the same word every time. A finding records
    /// the gap.[^5]
    ///
    /// Three groups of entries describe other people. Each group holds four
    /// parallel NumPy arrays of the same length. The four arrays of one group
    /// describe the same people in the same order.
    ///
    /// - `parent`, `parent_birth_order`, `parent_alive` and `parent_role`.
    ///   The mother and the father. The group holds no row at all when the
    ///   character founds a line. That is a real answer: the world invents no
    ///   parent.[^2]
    /// - `ancestor`, `ancestor_birth_order`, `ancestor_alive` and
    ///   `ancestor_role`. Every ancestor, at every depth. The character is
    ///   never in this group.
    /// - `descendant`, `descendant_birth_order`, `descendant_alive` and
    ///   `descendant_role`. Every descendant, at every depth. The character
    ///   is never in this group.
    ///
    /// The four arrays of a group carry these types.
    ///
    /// - The first array, `numpy.uint64`. The identity the engine minted for
    ///   that person at their birth. **The engine never issues one identity
    ///   twice**, so this value names one person for ever.[^3]
    /// - `_birth_order`, `numpy.uint32`. The position in the record of
    ///   descent. It is data and no call takes it.
    /// - `_alive`, `numpy.uint8`. One when the person is alive now, zero when
    ///   they are gone. **Read this before you pass an identity to another
    ///   call.** An identity with a zero here is refused everywhere, because
    ///   the record of descent outlives the person it names.
    /// - `_role`, `numpy.uint8`. Zero for a mother and one for a father, in
    ///   the parent group. It is zero in the other two groups. A person there
    ///   is reached through many steps and holds no one role.
    ///
    /// **The ancestor and descendant groups are in ascending birth order.**
    /// The order is explicit, and it is the same on every run and at every
    /// thread count.[^4] The parent group holds the mother before the father
    /// when both exist. A role is a fixed position and not a birth order.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the identity names no living character. The
    /// read changes nothing, so a refusal leaves the world as it was.
    ///
    /// # References
    ///
    /// [^1]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
    /// [^2]: Blockers register, BLK-011. `docs/BLOCKERS.md`
    /// [^3]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    /// [^4]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^5]: Findings register, FND-471. `docs/FINDINGS.md`
    fn character_lineage<'py>(
        &self,
        python: Python<'py>,
        character: u64,
    ) -> PyResult<Bound<'py, PyDict>> {
        let world = self.lock();
        let entity = resolve_character(&world, character)?;
        let arena = world.characters();
        let id = arena
            .descent_id(entity)
            .expect("a resolved identity names a descent row");
        let parents = arena
            .parents(entity)
            .expect("a resolved identity names a pair of parents");
        let mut parent_rows: Vec<(DescentId, u8)> = Vec::new();
        if let Some(mother) = parents.mother {
            parent_rows.push((mother, MOTHER_ROLE));
        }
        if let Some(father) = parents.father {
            parent_rows.push((father, FATHER_ROLE));
        }
        let ancestors = with_no_role(&world.character_ancestors(entity));
        let descendants = with_no_role(&world.character_descendants(entity));

        let columns = PyDict::new(python);
        columns.set_item("character", entity.to_bits())?;
        columns.set_item("birth_order", id.birth_order())?;
        columns.set_item(
            "house",
            arena
                .house(entity)
                .expect("a resolved identity names a house")
                .founder()
                .birth_order(),
        )?;
        write_kin(python, &columns, "parent", arena, &parent_rows)?;
        write_kin(python, &columns, "ancestor", arena, &ancestors)?;
        write_kin(python, &columns, "descendant", arena, &descendants)?;
        Ok(columns)
    }

    /// Returns how closely one character is related to each of a set of
    /// others.
    ///
    /// The subject is one identity. The others are a sequence of identities,
    /// or the `character` array that `characters` returned. Returns a
    /// one-dimensional NumPy array of `numpy.int32`, one value for each
    /// entry of `others`, in the order of `others`.
    ///
    /// **This call takes a set and answers once.** A caller that reads one
    /// person against the whole population crosses the boundary once per
    /// pair. The number of pairs is the thing that grows.[^1]
    ///
    /// The value is the coefficient of relationship. A parent and a child
    /// give one half. Two children of one pair of parents give one half as
    /// well. A character against itself gives one, plus its inbreeding
    /// coefficient. Two characters with no ancestor in common give zero. A
    /// character who founds a line has no ancestor. It stands at zero to
    /// every character who is not descended from it.
    ///
    /// **The value is a Q16.16 fixed-point number as its raw integer. Divide
    /// by 65536.** One half is 32768. The value is exact. Every step of the
    /// recursion halves a value, so no step rounds. No floating point number
    /// is involved.[^2]
    ///
    /// **The engine answers only for two living characters.** The record of
    /// descent outlives a character. The relation reads the row that the
    /// arena slot points at. Only a living character has one. A caller
    /// therefore cannot ask how a living person is related to a dead
    /// ancestor. `character_lineage` answers who the ancestors are.
    ///
    /// **The set is all or nothing.** Every identity resolves before any
    /// value is computed, so one dead identity answers nothing and raises.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the subject or any member of `others` names no
    /// living character. The read changes nothing.
    ///
    /// # References
    ///
    /// [^1]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
    /// [^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    fn character_relations<'py>(
        &self,
        python: Python<'py>,
        subject: u64,
        others: Vec<u64>,
    ) -> PyResult<Bound<'py, PyArray1<i32>>> {
        let world = self.lock();
        let left = resolve_character(&world, subject)?;
        let mut resolved = Vec::with_capacity(others.len());
        for other in &others {
            resolved.push(resolve_character(&world, *other)?);
        }
        let values: Vec<i32> = resolved
            .into_iter()
            .map(|right| world.character_relation(left, right).0)
            .collect();
        Ok(values.to_pyarray(python))
    }

    /// Returns what each unit of a set has ever gathered.
    ///
    /// The units are a sequence of identities, or the NumPy array of
    /// `numpy.uint64` that `spawn_soldiers` returned. Returns a
    /// one-dimensional NumPy array of `numpy.uint64`, one entry for each
    /// unit, in the order of the units.
    ///
    /// The value is a whole quantity of resource, summed over every kind and
    /// over the whole life of the unit. It is not a rate and it carries no
    /// fixed-point scale. It never falls while the unit lives.
    ///
    /// A unit becomes eligible to be raised into a character when this value
    /// reaches the level that `deed_threshold` reports.
    ///
    /// **The set is all or nothing.** Every identity resolves before any
    /// value is read, so one dead identity answers nothing and raises.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live soldier. The read
    /// changes nothing.
    fn unit_deeds<'py>(
        &self,
        python: Python<'py>,
        units: Vec<u64>,
    ) -> PyResult<Bound<'py, PyArray1<u64>>> {
        let world = self.lock();
        let mut resolved = Vec::with_capacity(units.len());
        for unit in &units {
            resolved.push(resolve(&world, *unit)?);
        }
        let values: Vec<u64> = resolved
            .into_iter()
            .map(|entity| {
                world
                    .unit_deeds(entity)
                    .expect("a resolved identity names a live soldier")
            })
            .collect();
        Ok(values.to_pyarray(python))
    }

    /// Returns the character that each unit of a set was raised into.
    ///
    /// The units are a sequence of identities, or the NumPy array of
    /// `numpy.uint64` that `spawn_soldiers` returned. Returns a
    /// one-dimensional NumPy array of `numpy.uint64`, one entry for each
    /// unit, in the order of the units.
    ///
    /// **A zero means the unit carries no living character.** The engine
    /// never issues zero as an identity, so zero cannot be confused with a
    /// person.[^1] A unit that was never raised reads zero, and so does a
    /// unit whose character has been removed.
    ///
    /// **A raised unit is not turned into a character.** The engine creates a
    /// character beside the unit and links the two. The unit stays a unit. It
    /// keeps its tile and keeps moving. An entity declares its tier when it
    /// is created and never changes tier.[^2]
    ///
    /// **The set is all or nothing.** Every identity resolves before any
    /// value is read, so one dead identity answers nothing and raises.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live soldier. The read
    /// changes nothing.
    ///
    /// # References
    ///
    /// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D1. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
    /// [^2]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D4. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
    fn unit_characters<'py>(
        &self,
        python: Python<'py>,
        units: Vec<u64>,
    ) -> PyResult<Bound<'py, PyArray1<u64>>> {
        let world = self.lock();
        let mut resolved = Vec::with_capacity(units.len());
        for unit in &units {
            resolved.push(resolve(&world, *unit)?);
        }
        let values: Vec<u64> = resolved
            .into_iter()
            .map(|entity| {
                world
                    .unit_character(entity)
                    .expect("a resolved identity names a live soldier")
                    .map_or(0, |character| character.to_bits())
            })
            .collect();
        Ok(values.to_pyarray(python))
    }

    /// Returns the deeds at which the engine raises a unit into a character.
    ///
    /// The value is a whole quantity of resource, summed over every kind. It
    /// carries no fixed-point scale. A unit whose deeds reach this level
    /// becomes eligible. The engine then chooses among the eligible by a rule
    /// of its own.
    fn deed_threshold(&self) -> u64 {
        self.lock().deed_threshold()
    }

    /// Sets the deeds at which the engine raises a unit into a character.
    ///
    /// The threshold is a whole quantity of resource, summed over every
    /// kind. It carries no fixed-point scale. Returns `None`.
    ///
    /// The threshold is a content parameter and not a budget. Raise it to
    /// make a named person rare. Lower it to make one common. A threshold of
    /// zero makes every unit eligible.
    ///
    /// **A caller sets the level. It does not choose who is raised.** The
    /// engine collects the eligible units. It ranks them by a key vector of
    /// its own. It cuts the list at a budget.[^1] Nothing in this module
    /// names a unit to raise.
    ///
    /// # References
    ///
    /// [^1]: ADR-0104, a soldier is promoted from a level that never falls, decision D4. `docs/adrs/draft/adr-0104-a-soldier-is-promoted-from-a-level-that-never-falls.md`
    fn set_deed_threshold(&self, threshold: u64) {
        self.lock().set_deed_threshold(threshold);
    }

    /// Gives a set of units the settlement that they draw from.
    ///
    /// Returns `None`.
    ///
    /// The units are a sequence of unit identities, or the NumPy array of
    /// `numpy.uint64` that `spawn_soldiers` returned. The site is one
    /// settlement identity, or `None`.
    ///
    /// A unit draws its rations from the store of its home, and it lives
    /// there. A unit whose home is `None` draws from nothing. That is a state
    /// the world represents, not an error.
    ///
    /// **The home may be set at any time.** It is simulated state, and it
    /// enters the state hash. Two worlds that feed a unit from different
    /// stores must diverge.[^1]
    ///
    /// **The set is all or nothing.** The site resolves first, then every unit
    /// identity, and nothing is written until all of them resolve.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no live unit, and when the
    /// site names no live settlement.
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[pyo3(signature = (units, site = None))]
    fn set_home_site(&self, units: Vec<u64>, site: Option<u64>) -> PyResult<()> {
        let mut world = self.lock();
        let home = match site {
            Some(site) => Some(resolve_site(&world, site)?),
            None => None,
        };
        let mut resolved = Vec::with_capacity(units.len());
        for unit in &units {
            resolved.push(resolve(&world, *unit)?);
        }
        for unit in resolved {
            world.set_home_site(unit, home);
        }
        Ok(())
    }

    /// Returns what one faction reaches at the block of ground that covers a
    /// tile.
    ///
    /// **The value is unsigned fixed point against a fixed reference, and
    /// 65535 means one reference unit.** It is not the Q16.16 scale that the
    /// rates use, and no cell holds more than 65535.[^1]
    ///
    /// **The field answers for a block of ground and not for a tile.** Two
    /// addresses in one block give one answer.
    ///
    /// The field is derived at the end of a step, from the sources that
    /// `set_influence_source` wrote and from the ground. A world that has not
    /// stepped since a source was written reports what the last step
    /// left.[^2]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when the address lies outside the world, and when
    /// the world holds no such faction.
    ///
    /// # References
    ///
    /// [^1]: ADR-0060, an influence map is stored as a shared basis, decision D2. `docs/adrs/draft/adr-0060-an-influence-map-is-stored-as-a-shared-basis.md`
    /// [^2]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
    fn influence(&self, faction: u16, q: i32, r: i32) -> PyResult<u16> {
        let world = self.lock();
        world
            .influence(FactionId(faction), Axial { q, r })
            .map(|value| value.0)
            .ok_or_else(|| {
                ViewError::new_err(format!(
                    "the faction {faction} or the address ({q}, {r}) is outside this world"
                ))
            })
    }

    /// Sets what one faction injects at a set of places.
    ///
    /// Returns `None`.
    ///
    /// The addresses are a sequence of `(q, r)` pairs. Each names a tile, and
    /// the engine writes the source at the block of ground that covers that
    /// tile. Two addresses in one block are two writes of one cell, and the
    /// last one stands.
    ///
    /// **The source is unsigned fixed point against a fixed reference, and
    /// 65535 means one reference unit.** It is not the Q16.16 scale that the
    /// rates use. A source of 0 is the ordinary value and it is not an
    /// absence.[^1]
    ///
    /// **The engine holds no rule that decides this value.** A rule that
    /// writes a source term lives above the engine, which is why the control
    /// plane holds the write.[^1]
    ///
    /// **A source may be set at any time, and the next step spreads it.** The
    /// solve runs last in a step, over the whole plane, for a fixed number of
    /// passes.[^2] A source enters the state hash, because the next solve
    /// starts from it.[^3]
    ///
    /// **The set is all or nothing.** Every address and the faction are
    /// checked before anything is written.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an address lies outside the world, and when the
    /// world holds no such faction.
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-041. `docs/DECISIONS.md`
    /// [^2]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
    /// [^3]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    fn set_influence_source(
        &self,
        faction: u16,
        addresses: Vec<(i32, i32)>,
        source: u16,
    ) -> PyResult<()> {
        let mut world = self.lock();
        let who = FactionId(faction);
        for (q, r) in &addresses {
            if world.influence(who, Axial { q: *q, r: *r }).is_none() {
                return Err(ViewError::new_err(format!(
                    "the faction {faction} or the address ({q}, {r}) is outside this world"
                )));
            }
        }
        for (q, r) in addresses {
            world.set_influence_source(who, Axial { q, r }, Influence(source));
        }
        Ok(())
    }

    /// Sets how often the engine looks for a unit to raise.
    ///
    /// Returns `None`.
    ///
    /// The period is a count of ticks, and it must be at least one. A period
    /// of one looks on every step. The phase is the offset inside the period.
    /// A period of four and a phase of one look on the ticks one, five and
    /// nine. A phase at or above the period wraps into it.
    ///
    /// The schedule decides which frames run the promotion pass.[^1] The
    /// schedule enters the state hash, because the next step reads it.[^2]
    ///
    /// The world starts with a schedule already set. This call replaces it.
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the period is zero, or above the range that the
    /// scaling multiply takes. The message names the limit it applied.
    ///
    /// # References
    ///
    /// [^1]: ADR-0104, a soldier is promoted from a level that never falls, decision D5. `docs/adrs/draft/adr-0104-a-soldier-is-promoted-from-a-level-that-never-falls.md`
    /// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    fn set_character_schedule(&self, period: u32, phase: u32) -> PyResult<()> {
        self.lock()
            .set_character_schedule(period, phase)
            .map_err(|error| VerbError::new_err(error.to_string()))
    }

    /// Makes a number of characters in one faction and returns their
    /// identities.
    ///
    /// The faction names the faction that owns the new characters, by its
    /// number. The count is how many to make.
    ///
    /// Returns a one-dimensional NumPy array of `numpy.uint64`, one identity
    /// for each character. Keep the array and pass it to any other call that
    /// takes a character.
    ///
    /// **Every character this call makes founds a line.** It has no parents,
    /// and its relation to every other character is zero. The world invents
    /// no ancestry, which is the same rule that governs a unit the engine
    /// raises from the ranks.[^1] Call `bear_children` to give somebody
    /// parents.
    ///
    /// A new character holds a renown of zero. Zero is a real state and not
    /// an absent one.[^2] The engine draws the sex, and no caller sets it.
    /// The character is born on the current tick of the world.
    ///
    /// **The set is all or nothing.** The call checks the room in both stores
    /// before it makes anybody. It removes every character it made when a
    /// later one refuses. A refusal therefore makes nobody.
    ///
    /// **The arena refuses the faction, and this call does not check it a
    /// second time.** A check here would be a second declaration site of one
    /// rule. Nothing would fail when the two copies disagreed.[^3] The first
    /// creation of the set refuses, so nothing is made.
    ///
    /// **A character identity and a unit identity share one range of
    /// numbers.** Each arena numbers its own slots. The first character of a
    /// world and the first unit of a world carry the same number. Neither
    /// kind of call refuses the number of the other, and no check reports the
    /// mistake.[^4]
    ///
    /// # Errors
    ///
    /// Raises `VerbError` when the world has no such faction. It raises the
    /// same error when the count is above the room either store has left.
    /// The message names the value that refused.
    ///
    /// # References
    ///
    /// [^1]: Blockers register, BLK-011. `docs/BLOCKERS.md`
    /// [^2]: Findings register, FND-043. `docs/FINDINGS.md`
    /// [^3]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
    /// [^4]: Findings register, FND-472. `docs/FINDINGS.md`
    fn create_characters<'py>(
        &self,
        python: Python<'py>,
        faction: u16,
        count: u32,
    ) -> PyResult<Bound<'py, PyArray1<u64>>> {
        let mut world = self.lock();
        refuse_no_room(&world, count)?;
        let mut made: Vec<u64> = Vec::with_capacity(count as usize);
        for _ in 0..count {
            match world.create_character(FactionId(faction)) {
                Ok(entity) => made.push(entity.to_bits()),
                Err(error) => return Err(undo_births(&mut world, &made, &error.to_string())),
            }
        }
        Ok(made.to_pyarray(python))
    }

    /// Bears one child for each pair of parents and returns the children.
    ///
    /// The births are a sequence of `(mother, father)` pairs of identities.
    /// Returns a one-dimensional NumPy array of `numpy.uint64`, one identity
    /// for each child, in the order of the pairs.
    ///
    /// A child takes the faction of its mother and it records both parents.
    /// The record of descent keeps those two edges after either parent is
    /// gone. A caller reads a dead parent through a living child.[^1] The
    /// child is born on the current tick of the world, and it holds a renown
    /// of zero.
    ///
    /// **The two names are roles and not a test of sex.** The engine puts the
    /// first identity in the mother role and the second in the father role.
    /// It reads the sex column of neither. A game that wants a rule about sex
    /// reads the `sex` column that `characters` returns and applies the rule
    /// itself.
    ///
    /// **Both parents must be alive.** The record of descent outlives a
    /// character, so a caller reads a dead parent. It cannot name one as a
    /// parent of a new child.
    ///
    /// **A caller states a birth. The engine states none.** No pass in the
    /// engine bears a child on its own. Every child in a run comes from this
    /// call.
    ///
    /// **The set is all or nothing.** Every identity resolves, and every pair
    /// is checked. The call checks the room in both stores before any child
    /// is born.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no living character. Raises
    /// `VerbError` when the two parents of a pair are one character. It also
    /// raises `VerbError` when the number of births is above the room either
    /// store has left.
    ///
    /// # References
    ///
    /// [^1]: Decisions register, DEC-003. `docs/DECISIONS.md`
    fn bear_children<'py>(
        &self,
        python: Python<'py>,
        births: Vec<(u64, u64)>,
    ) -> PyResult<Bound<'py, PyArray1<u64>>> {
        let mut world = self.lock();
        let mut resolved = Vec::with_capacity(births.len());
        for (mother, father) in &births {
            let mother = resolve_character(&world, *mother)?;
            let father = resolve_character(&world, *father)?;
            if mother == father {
                return Err(VerbError::new_err(format!(
                    "the identity {} names both parents, and a child has two parents",
                    mother.to_bits()
                )));
            }
            resolved.push((mother, father));
        }
        refuse_no_room(&world, resolved.len() as u32)?;
        let mut made: Vec<u64> = Vec::with_capacity(resolved.len());
        for (mother, father) in resolved {
            match world.bear_character(mother, father) {
                Ok(child) => made.push(child.to_bits()),
                Err(error) => return Err(undo_births(&mut world, &made, &error.to_string())),
            }
        }
        Ok(made.to_pyarray(python))
    }

    /// Removes every character the identities name.
    ///
    /// The characters are a sequence of identities, or the NumPy array of
    /// `numpy.uint64` that `characters` or `create_characters` returned.
    /// Returns `None`.
    ///
    /// **The set is all or nothing.** Every identity resolves before anything
    /// is removed, so one dead identity removes nobody and raises.
    ///
    /// A removed character leaves its slot to the next character. Its
    /// identity is then stale, and every call that takes a character refuses
    /// it.[^1]
    ///
    /// **The record of descent keeps the person.** A removal releases the
    /// slot columns and nothing else. The parent edges stay. A living child
    /// still reads a removed parent. `character_lineage` still names the
    /// removed person with a zero in its `_alive` column.[^2]
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no living character.
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    /// [^2]: ADR-0078, descent is a bounded record, and a relation is a bounded recursion, decision D1. `docs/adrs/draft/adr-0078-descent-is-a-bounded-record-and-a-relation-is-a-bounded-recursion.md`
    fn remove_characters(&self, characters: Vec<u64>) -> PyResult<()> {
        let mut world = self.lock();
        let mut resolved = Vec::with_capacity(characters.len());
        for character in &characters {
            resolved.push(resolve_character(&world, *character)?);
        }
        for entity in resolved {
            assert!(
                world.remove_character(entity),
                "a resolved identity must name a character the arena can remove"
            );
        }
        Ok(())
    }

    /// Writes how much each character of a set is thought of.
    ///
    /// The characters are a sequence of identities, or the NumPy array of
    /// `numpy.uint64` that `characters` returned. Returns `None`.
    ///
    /// **The renown is a Q16.16 fixed-point value as its raw integer.
    /// Multiply by 65536.** One whole unit of renown is 65536. The value may
    /// be negative. No floating point number crosses this boundary, because
    /// a float sum is not associative and the engine hashes this column.[^1]
    ///
    /// **One call writes one value to the whole set.** A caller that wants
    /// two values makes two calls, one for each value. That is a loop over
    /// values and not a loop over people. The number of values a game uses
    /// does not grow with the population.
    ///
    /// A renown of zero is a real state and not an absent one. A write of
    /// zero is a write.[^2]
    ///
    /// **Nothing in the engine reads this column.** It is a value for the
    /// control plane, and no simulation pass consumes it. The engine writes
    /// it once, at a creation, with a zero. After that, it stays at whatever
    /// a caller last wrote.
    ///
    /// **The set is all or nothing.** Every identity resolves before anything
    /// is written, so one dead identity writes nothing and raises.
    ///
    /// # Errors
    ///
    /// Raises `ViewError` when an identity names no living character.
    ///
    /// # References
    ///
    /// [^1]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
    /// [^2]: Findings register, FND-043. `docs/FINDINGS.md`
    fn set_character_renown(&self, characters: Vec<u64>, renown: i32) -> PyResult<()> {
        let mut world = self.lock();
        let mut resolved = Vec::with_capacity(characters.len());
        for character in &characters {
            resolved.push(resolve_character(&world, *character)?);
        }
        for entity in resolved {
            assert!(
                world.set_character_renown(entity, Fix32(renown)),
                "a resolved identity must name a character the arena can write"
            );
        }
        Ok(())
    }
}

/// The role value that names the mother of a character.
///
/// The lineage read answers with a role column, and the two values live here
/// once. A second copy elsewhere would be one value in two places with
/// nothing that fails when the copies disagree.[^1]
///
/// # References
///
/// [^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
const MOTHER_ROLE: u8 = 0;

/// The role value that names the father of a character.
const FATHER_ROLE: u8 = 1;

/// Resolves a character identity that Python handed back, or raises.
///
/// The engine compares the generation, so a character who is gone never
/// answers for the character made next in their slot.[^1]
///
/// # References
///
/// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
fn resolve_character(world: &CoreWorld, character: u64) -> PyResult<Entity> {
    world
        .resolve_character(character)
        .map_err(|error| ViewError::new_err(error.to_string()))
}

/// Pairs each row of a walk with the role value that means no one role.
///
/// An ancestor and a descendant are reached through many steps, so neither
/// holds the mother role or the father role. The column exists so that the
/// three groups of a lineage answer have one shape.
fn with_no_role(rows: &[DescentId]) -> Vec<(DescentId, u8)> {
    rows.iter().map(|row| (*row, MOTHER_ROLE)).collect()
}

/// Writes one group of four parallel columns into a lineage answer.
///
/// The four keys are the name, and the name with each of the three suffixes.
/// The identity is the one the arena minted at the birth of that person, and
/// it never names anybody else.[^1] The live flag says whether the arena
/// still holds it, because the record of descent outlives the person.[^2]
///
/// # References
///
/// [^1]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
/// [^2]: ADR-0078, descent is a bounded record, and a relation is a bounded recursion, decision D1. `docs/adrs/draft/adr-0078-descent-is-a-bounded-record-and-a-relation-is-a-bounded-recursion.md`
fn write_kin(
    python: Python<'_>,
    columns: &Bound<'_, PyDict>,
    name: &str,
    arena: &CharacterArena,
    rows: &[(DescentId, u8)],
) -> PyResult<()> {
    let descent = arena.descent();
    let mut identity: Vec<u64> = Vec::with_capacity(rows.len());
    let mut birth_order: Vec<u32> = Vec::with_capacity(rows.len());
    let mut alive: Vec<u8> = Vec::with_capacity(rows.len());
    let mut role: Vec<u8> = Vec::with_capacity(rows.len());
    for (row, held) in rows {
        let entity = descent
            .born_as(*row)
            .expect("a row of the record of descent names the identity it was born as");
        identity.push(entity.to_bits());
        birth_order.push(row.birth_order());
        alive.push(u8::from(arena.contains(entity)));
        role.push(*held);
    }
    columns.set_item(name, identity.to_pyarray(python))?;
    columns.set_item(
        format!("{name}_birth_order"),
        birth_order.to_pyarray(python),
    )?;
    columns.set_item(format!("{name}_alive"), alive.to_pyarray(python))?;
    columns.set_item(format!("{name}_role"), role.to_pyarray(python))?;
    Ok(())
}

/// Refuses a set of new characters that the storage has no room for.
///
/// The check runs before anything is made, so a refusal makes nobody. The
/// arena refuses a creation beyond its own capacity whatever this says, so
/// this is a cut and never a second enforcement of the ceiling.[^1]
///
/// Two stores bound a creation. The arena holds the living characters, and a
/// slot returns to it when a character is removed. The record of descent
/// holds every character the world has ever made, and it never releases a
/// row.[^2]
///
/// # References
///
/// [^1]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decisions D2 and D3. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
/// [^2]: ADR-0078, descent is a bounded record, and a relation is a bounded recursion, decision D1. `docs/adrs/draft/adr-0078-descent-is-a-bounded-record-and-a-relation-is-a-bounded-recursion.md`
fn refuse_no_room(world: &CoreWorld, count: u32) -> PyResult<()> {
    let arena = world.characters();
    let living = arena
        .capacity()
        .saturating_sub(arena.len())
        .saturating_sub(arena.retired_count());
    if count > living {
        return Err(VerbError::new_err(format!(
            "{count} new characters do not fit, and the arena has room for {living}"
        )));
    }
    let recorded = DESCENT_CEILING.saturating_sub(arena.descent().len());
    if count > recorded {
        return Err(VerbError::new_err(format!(
            "{count} new characters do not fit, and the record of descent has room for {recorded}"
        )));
    }
    Ok(())
}

/// Removes every character a refused set made, and returns the refusal.
///
/// A set-valued creation leaves nothing half made. The room checks run before
/// the first creation, so this path is unreachable today. It stays because a
/// verb that writes must state what it does on a refusal, and the statement
/// belongs in code rather than in prose.[^1]
///
/// **The record of descent keeps the rows.** It never releases one, so the
/// characters this removes end in the same state as characters who lived and
/// were then removed.[^2]
///
/// # References
///
/// [^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
/// [^2]: ADR-0078, descent is a bounded record, and a relation is a bounded recursion, decision D1. `docs/adrs/draft/adr-0078-descent-is-a-bounded-record-and-a-relation-is-a-bounded-recursion.md`
fn undo_births(world: &mut CoreWorld, made: &[u64], said: &str) -> PyErr {
    for identity in made {
        if let Ok(entity) = world.resolve_character(*identity) {
            world.remove_character(entity);
        }
    }
    VerbError::new_err(format!("the character arena refused: {said}"))
}
