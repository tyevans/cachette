//! The record of descent.
//!
//! A character records the two characters it came from. The record of
//! descent holds those edges, and it holds them for every character the
//! world has ever created. It is append-only. A character gains its parents
//! when it is born and never changes them.[^1]
//!
//! **The record is not keyed on a slot.** The character arena reuses a slot
//! after the character in it is gone, so a slot names a different character
//! later.[^2] A descent identity is allocated once and is never reissued, so
//! it names one character for the life of the world. That is what lets the
//! record of descent survive the death of the character it names.[^3]
//!
//! The identity that the arena minted stays in the record beside each row. A
//! caller reads it back and resolves it against the arena. A live character
//! resolves. A character who is gone resolves to nothing, and the character
//! created next in that slot never answers to it, because the generation in
//! the identity no longer matches the generation in the slot.[^4]
//!
//! **The columns are struct-of-arrays: one array for each field.** The pass
//! that decides that is descent and succession, and every kernel of it reads
//! one or two columns for each row it visits.[^7] The layout follows the
//! column count of that pass and not the name of the tier the shape sits
//! in.[^8]
//!
//! A character also belongs to a house, which is the group that shares a
//! patrilineal founder. A birth copies the house of the father. A cadet split
//! moves a whole patrilineal line into a house of its own, so the house is
//! not derivable from the parent edges and the record stores it.[^9]
//!
//! Two walks read the record. One walks to the ancestors of a character. One
//! walks to its descendants. Both return the set in ascending descent
//! identity order, which is the birth order of the world. The order is
//! explicit, so no walk can carry a visit order into a result.[^5]
//!
//! The relation between two characters is the coefficient of relationship.
//! The module computes it by a bounded recursion over the parent edges. It
//! never walks two lines up to a common ancestor. Every step of the
//! recursion halves a value, so every value is an integer over a power of
//! two, and the Q16.16 fixed-point form holds it exactly.[^6]
//!
//! # References
//!
//! [^1]: The character graph and inheritance, section 3.2. `docs/research/reports/14-character-graph-and-inheritance.md`
//! [^2]: ADR-0014, entity identity is an index plus a generation, decision D4. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
//! [^3]: ADR-0078, descent is a bounded record, and a relation is a bounded recursion, decision D1. `docs/adrs/draft/adr-0078-descent-is-a-bounded-record-and-a-relation-is-a-bounded-recursion.md`
//! [^4]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
//! [^5]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
//! [^6]: The character graph and inheritance, section 3.6. `docs/research/reports/14-character-graph-and-inheritance.md`
//! [^7]: The character graph and inheritance, section 3.2. `docs/research/reports/14-character-graph-and-inheritance.md`
//! [^8]: ADR-0021, a layout claim names one structure and one pass, and never a tier, decisions D2 and D3. `docs/adrs/draft/adr-0021-layout-follows-the-access-pattern.md`
//! [^9]: The character graph and inheritance, section 3.5. `docs/research/reports/14-character-graph-and-inheritance.md`

use std::collections::BTreeMap;

use crate::hash::StateHash;
use crate::sim_math;
use crate::types::{Entity, Fix32, Tick, FIX_FRACTIONAL_BITS};

/// The largest number of rows that the record of descent holds.
///
/// The record holds every character the world has ever created, so the
/// living ceiling alone does not bound it. The scale constants table gives
/// the derivation from the living ceiling and the dead-to-living ratio.[^1]
/// Every cost figure in this project is derived and not measured.[^2]
///
/// # References
///
/// [^1]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
/// [^2]: Blockers register, BLK-007. `docs/BLOCKERS.md`
pub const DESCENT_CEILING: u32 = 4_194_304;

/// The number of generations that the relation recursion expands.
///
/// The recursion halves a value at each step, so the smallest term it can
/// reach is two to the power of the negative of this depth. The Q16.16 form
/// holds that term exactly while the depth stays below the number of
/// fractional bits. The scale constants table gives the value and the
/// derivation.[^1]
///
/// # References
///
/// [^1]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
pub const RELATION_DEPTH: u32 = 12;

/// The exactness of the relation rests on this bound, so the compiler holds
/// it. A halved sum is exact while the raw value stays even, and the raw
/// value stays even while the depth leaves a fractional bit spare.
const _: () = assert!(RELATION_DEPTH < FIX_FRACTIONAL_BITS);

/// The value that means no row.
const NONE: u32 = u32::MAX;

/// The index of the mother in a parent pair.
const MOTHER: usize = 0;

/// The index of the father in a parent pair.
const FATHER: usize = 1;

/// The number of parents that a character has.
const PARENT_COUNT: usize = 2;

/// The identity of a character in the record of descent.
///
/// The value is allocated once, when the world creates the character, and
/// it is never reissued. It therefore names one character after that
/// character is gone.[^1]
///
/// A descent identity is not an entity identity. It resolves in the record
/// of descent and it says nothing about whether the character is alive. A
/// caller that wants to know reads the entity identity back and resolves it
/// against the arena.[^2]
///
/// # References
///
/// [^1]: ADR-0078, descent is a bounded record, and a relation is a bounded recursion, decision D1. `docs/adrs/draft/adr-0078-descent-is-a-bounded-record-and-a-relation-is-a-bounded-recursion.md`
/// [^2]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DescentId(u32);

impl DescentId {
    /// Returns the row that this identity names.
    ///
    /// The value is the birth order of the character in the world, counted
    /// from zero. Two identities compare in that order.
    #[must_use]
    pub const fn birth_order(self) -> u32 {
        self.0
    }
}

impl core::fmt::Display for DescentId {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "descent {}", self.0)
    }
}

/// The two parents of a character.
///
/// A parent is absent when the character founds a line. That is a real
/// state and not an invented one. A character raised from the ranks
/// receives no ancestry, so both parents are absent and the character is
/// not a special case in the recursion.[^1]
///
/// # References
///
/// [^1]: Blockers register, BLK-011. `docs/BLOCKERS.md`
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Parents {
    /// The mother, or `None` when the character founds a line.
    pub mother: Option<DescentId>,
    /// The father, or `None` when the character founds a line.
    pub father: Option<DescentId>,
}

impl Parents {
    /// The pair that a character who founds a line holds.
    pub const NONE: Self = Self {
        mother: None,
        father: None,
    };

    /// Reports whether the character founds a line.
    #[must_use]
    pub const fn is_founder(self) -> bool {
        self.mother.is_none() && self.father.is_none()
    }
}

/// The house that a character belongs to.
///
/// A house is a group of characters that share a patrilineal founder. The
/// identifier is the descent identity of that founder, so a house names the
/// character who founded it and needs no separate allocator.[^1] A character
/// with no father founds a house of its own. A character with a father takes
/// the house of its father at birth.
///
/// **A house is not a second copy of the parent edges.** A cadet branch
/// leaves the house of its father, and after that split the house of a
/// character is no longer derivable from the edges alone. That is why the
/// house is a stored column and why the record hashes it.[^2]
///
/// # References
///
/// [^1]: The character graph and inheritance, section 3.5. `docs/research/reports/14-character-graph-and-inheritance.md`
/// [^2]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HouseId(u32);

impl HouseId {
    /// Returns the character that founded the house.
    #[must_use]
    pub const fn founder(self) -> DescentId {
        DescentId(self.0)
    }
}

impl core::fmt::Display for HouseId {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "house of descent {}", self.0)
    }
}

/// The reason that the record refused a caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DescentError {
    /// The record holds its largest number of rows.
    RecordFull,
    /// The named parent is not an older row of this record.
    ///
    /// The record allocates a row after both parents, so a parent always
    /// carries the smaller identity. An identity that the record has not
    /// issued names no parent, and that is what refuses a character as its
    /// own ancestor.
    ParentIsUnknown(DescentId),
    /// The two parents are one character.
    ParentsAreOneCharacter(DescentId),
}

impl core::fmt::Display for DescentError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::RecordFull => write!(
                formatter,
                "the record of descent holds its {DESCENT_CEILING} rows"
            ),
            Self::ParentIsUnknown(parent) => {
                write!(formatter, "the record holds no older row for {parent}")
            }
            Self::ParentsAreOneCharacter(parent) => {
                write!(formatter, "the two parents are both {parent}")
            }
        }
    }
}

impl std::error::Error for DescentError {}

/// The append-only record of descent.
///
/// The record holds one row for each character the world has created. It
/// never removes a row, so a descent identity names one character for the
/// life of the world.[^1]
///
/// The columns are dense arrays indexed by the descent identity. The record
/// holds no hash map, because a hash map carries an iteration order that no
/// key fixes.[^2]
///
/// # References
///
/// [^1]: ADR-0078, descent is a bounded record, and a relation is a bounded recursion, decision D1. `docs/adrs/draft/adr-0078-descent-is-a-bounded-record-and-a-relation-is-a-bounded-recursion.md`
/// [^2]: ADR-0014, entity identity is an index plus a generation, decision D7. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
#[derive(Clone, Debug, Default)]
pub struct Descent {
    /// The identity that the arena minted for each row.
    born_as: Vec<u64>,
    /// The two parents of each row. `NONE` means the parent is absent.
    parents: Vec<[u32; PARENT_COUNT]>,
    /// The first child of each row, in each parent role.
    first_child: Vec<[u32; PARENT_COUNT]>,
    /// The last child of each row, in each parent role.
    last_child: Vec<[u32; PARENT_COUNT]>,
    /// The next child of the same parent, in each parent role.
    next_sibling: Vec<[u32; PARENT_COUNT]>,
    /// The tick that each row last bore a child on.
    last_birth: Vec<Tick>,
    /// The number of children that each row bore on that tick.
    ///
    /// The birth draw keys on this count, because the child has no identity
    /// when the draw happens.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
    births_on_that_tick: Vec<u32>,
    /// The house of each row.
    ///
    /// A birth copies the house of the father. A character with no father
    /// founds a house of its own, and the identifier of that house is its own
    /// row. A cadet split rewrites this column over a subtree, so the value
    /// is not derivable from the parent edges and the record hashes it.[^1]
    ///
    /// # References
    ///
    /// [^1]: The character graph and inheritance, section 3.5. `docs/research/reports/14-character-graph-and-inheritance.md`
    house: Vec<u32>,
    /// The preorder position of each row in the father forest.
    ///
    /// The label is meaningful only for a row below the relabel watermark.
    /// The record leaves it at `NONE` above the watermark, so a stale label
    /// is an absent label and never a wrong answer.[^1]
    ///
    /// # References
    ///
    /// [^1]: The character graph and inheritance, section 3.2. `docs/research/reports/14-character-graph-and-inheritance.md`
    father_tin: Vec<u32>,
    /// The last preorder position inside the subtree of each row, inclusive.
    ///
    /// A row is a patrilineal descendant of another row when its preorder
    /// position lies between the two labels of that row. The test is two
    /// integer comparisons and it walks nothing.[^1]
    ///
    /// # References
    ///
    /// [^1]: The character graph and inheritance, section 3.1. `docs/research/reports/14-character-graph-and-inheritance.md`
    father_tout: Vec<u32>,
    /// The row that sits at each preorder position.
    ///
    /// The record stores its rows in birth order, so a subtree is contiguous
    /// in this array and never in the columns. A pass over a subtree is
    /// therefore a contiguous scan of this array and a gather into the
    /// columns.
    euler_order: Vec<u32>,
}

impl Descent {
    /// Builds an empty record.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of rows that the record holds.
    #[must_use]
    pub fn len(&self) -> u32 {
        self.born_as.len() as u32
    }

    /// Reports whether the record holds no row.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.born_as.is_empty()
    }

    /// Adds a row for a character and returns its descent identity.
    ///
    /// The caller passes the identity that the arena minted and the two
    /// parents. A parent of `None` means the character founds a line.
    ///
    /// # Errors
    ///
    /// Returns an error when the record is full, when a named parent is not
    /// a row of this record, when a named parent is not older than the
    /// child, or when the two parents are one character.
    pub(crate) fn record(
        &mut self,
        born_as: Entity,
        parents: Parents,
    ) -> Result<DescentId, DescentError> {
        let child = self.len();
        if child >= DESCENT_CEILING {
            return Err(DescentError::RecordFull);
        }
        // A parent must already hold a row, and a row is allocated in birth
        // order, so a parent always carries the smaller identity. The check
        // is what refuses a character as its own ancestor: a cycle needs an
        // edge to an identity that is not older, and there is no other way
        // to make one.
        let mut raw = [NONE; PARENT_COUNT];
        for (role, parent) in [(MOTHER, parents.mother), (FATHER, parents.father)] {
            let Some(parent) = parent else {
                continue;
            };
            if parent.0 >= child {
                return Err(DescentError::ParentIsUnknown(parent));
            }
            raw[role] = parent.0;
        }
        if raw[MOTHER] != NONE && raw[MOTHER] == raw[FATHER] {
            return Err(DescentError::ParentsAreOneCharacter(DescentId(raw[MOTHER])));
        }
        // A birth copies the house of the father. A character with no father
        // founds a house of its own, and the identifier of that house is its
        // own row, so the record needs no separate house allocator.
        let house = if raw[FATHER] == NONE {
            child
        } else {
            self.house[raw[FATHER] as usize]
        };
        self.born_as.push(born_as.to_bits());
        self.parents.push(raw);
        self.first_child.push([NONE; PARENT_COUNT]);
        self.last_child.push([NONE; PARENT_COUNT]);
        self.next_sibling.push([NONE; PARENT_COUNT]);
        self.last_birth.push(Tick(0));
        self.births_on_that_tick.push(0);
        self.house.push(house);
        // The new row sits above the relabel watermark, so it carries no
        // label. An absent label answers nothing. A stale label would answer
        // wrongly, and nothing would fail.
        self.father_tin.push(NONE);
        self.father_tout.push(NONE);
        for role in [MOTHER, FATHER] {
            let parent = raw[role];
            if parent == NONE {
                continue;
            }
            let parent = parent as usize;
            // The child list appends at the tail, and a row is allocated in
            // birth order, so the list is always in ascending order. A walk
            // over it therefore needs no sort.
            if self.first_child[parent][role] == NONE {
                self.first_child[parent][role] = child;
            } else {
                let tail = self.last_child[parent][role] as usize;
                self.next_sibling[tail][role] = child;
            }
            self.last_child[parent][role] = child;
        }
        Ok(DescentId(child))
    }

    /// Counts one birth to a mother and returns the sequence of that birth.
    ///
    /// The sequence counts the births to one mother within one tick. The
    /// birth draw keys on the mother and on this sequence, because the
    /// child has no identity when the draw happens.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
    pub(crate) fn take_birth_sequence(&mut self, mother: DescentId, tick: Tick) -> u32 {
        let row = mother.0 as usize;
        if self.last_birth[row] != tick {
            self.last_birth[row] = tick;
            self.births_on_that_tick[row] = 0;
        }
        let sequence = self.births_on_that_tick[row];
        self.births_on_that_tick[row] = sequence.saturating_add(1);
        sequence
    }

    /// Returns the identity of a row, or `None` when the record holds no
    /// such row.
    #[must_use]
    pub fn id_at(&self, row: u32) -> Option<DescentId> {
        if (row as usize) < self.born_as.len() {
            Some(DescentId(row))
        } else {
            None
        }
    }

    /// Returns the identity that the arena minted for a row.
    ///
    /// The identity carries the generation that the slot held at the birth.
    /// A caller resolves it against the arena to learn whether the
    /// character is alive. A character who is gone resolves to nothing, and
    /// the character created next in that slot never answers to it.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
    #[must_use]
    pub fn born_as(&self, id: DescentId) -> Option<Entity> {
        Entity::from_bits(*self.born_as.get(id.0 as usize)?)
    }

    /// Returns the two parents of a character.
    ///
    /// Returns `None` when the record holds no such row. Returns a pair of
    /// absent parents when the character founds a line.[^1]
    ///
    /// # References
    ///
    /// [^1]: Blockers register, BLK-011. `docs/BLOCKERS.md`
    #[must_use]
    pub fn parents(&self, id: DescentId) -> Option<Parents> {
        let raw = self.parents.get(id.0 as usize)?;
        Some(Parents {
            mother: Self::option(raw[MOTHER]),
            father: Self::option(raw[FATHER]),
        })
    }

    /// Returns the children of a character, in ascending order.
    ///
    /// The order is the birth order, and it is the same on every run.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn children(&self, id: DescentId) -> Vec<DescentId> {
        let Some(first) = self.first_child.get(id.0 as usize) else {
            return Vec::new();
        };
        // Each of the two lists is already ascending, so one merge gives an
        // ascending result. The two lists share no entry, because the record
        // refuses a child whose two parents are one character.
        let mut walk = [first[MOTHER], first[FATHER]];
        let mut out = Vec::new();
        loop {
            let next = match (walk[MOTHER], walk[FATHER]) {
                (NONE, NONE) => break,
                (_, NONE) => MOTHER,
                (NONE, _) => FATHER,
                (left, right) => {
                    if left <= right {
                        MOTHER
                    } else {
                        FATHER
                    }
                }
            };
            let row = walk[next];
            out.push(DescentId(row));
            walk[next] = self.next_sibling[row as usize][next];
        }
        out
    }

    /// Returns every ancestor of a character, in ascending order.
    ///
    /// The walk sorts each frontier before it expands, and it visits each
    /// row once. The result is the set of ancestors and never holds the
    /// character itself, because a parent is always older than its
    /// child.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn ancestors(&self, id: DescentId) -> Vec<DescentId> {
        self.walk(id, true)
    }

    /// Returns every descendant of a character, in ascending order.
    ///
    /// The result never holds the character itself.
    #[must_use]
    pub fn descendants(&self, id: DescentId) -> Vec<DescentId> {
        self.walk(id, false)
    }

    /// Walks the record upward or downward from one row.
    ///
    /// The frontier is a sorted vector and never a hash set, so the visit
    /// order is explicit at every step.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn walk(&self, id: DescentId, upward: bool) -> Vec<DescentId> {
        let mut out = Vec::new();
        if id.0 as usize >= self.parents.len() {
            return out;
        }
        let mut seen = vec![0u8; self.parents.len()];
        let mut frontier = vec![id];
        while !frontier.is_empty() {
            let mut next = Vec::new();
            for row in &frontier {
                if upward {
                    let raw = self.parents[row.0 as usize];
                    for role in [MOTHER, FATHER] {
                        Self::push_once(&mut next, &mut seen, raw[role]);
                    }
                } else {
                    for child in self.children(*row) {
                        Self::push_once(&mut next, &mut seen, child.0);
                    }
                }
            }
            // The frontier is sorted before it expands, so no visit order
            // reaches the result.
            next.sort_unstable();
            out.extend_from_slice(&next);
            frontier = next;
        }
        out.sort_unstable();
        out
    }

    /// Adds a row to a frontier once.
    fn push_once(next: &mut Vec<DescentId>, seen: &mut [u8], row: u32) {
        if row == NONE || seen[row as usize] == 1 {
            return;
        }
        seen[row as usize] = 1;
        next.push(DescentId(row));
    }

    /// Returns the coefficient of relationship between two characters.
    ///
    /// The value is Wright's coefficient, which is twice the kinship
    /// coefficient. A parent and a child give one half. A character
    /// against itself gives one plus its inbreeding coefficient. Two
    /// characters with no ancestor in common give zero.[^1]
    ///
    /// The recursion expands the younger of the two characters, and it
    /// stops at a fixed depth. It never walks two lines up to a common
    /// ancestor.[^2]
    ///
    /// Every step halves a value, so every value is an integer over a power
    /// of two and the Q16.16 form holds it exactly. No step rounds.[^1]
    ///
    /// Returns zero when the record holds no such row.
    ///
    /// # References
    ///
    /// [^1]: The character graph and inheritance, section 3.6. `docs/research/reports/14-character-graph-and-inheritance.md`
    /// [^2]: ADR-0078, descent is a bounded record, and a relation is a bounded recursion, decision D2. `docs/adrs/draft/adr-0078-descent-is-a-bounded-record-and-a-relation-is-a-bounded-recursion.md`
    #[must_use]
    pub fn relation(&self, left: DescentId, right: DescentId) -> Fix32 {
        let rows = self.len();
        if left.0 >= rows || right.0 >= rows {
            return Fix32::ZERO;
        }
        let mut memo = BTreeMap::new();
        let kinship = self.kinship(left.0, right.0, RELATION_DEPTH, &mut memo);
        // Wright's coefficient is twice the kinship coefficient. A doubling
        // is exact, so the result carries no rounding either.
        sim_math::add(kinship, kinship)
    }

    /// Returns the kinship coefficient of two rows.
    ///
    /// The memo is an ordered map. The recursion reads it by key and never
    /// iterates it, so it carries no iteration order into the result.[^1]
    ///
    /// # References
    ///
    /// [^1]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    fn kinship(
        &self,
        left: u32,
        right: u32,
        depth: u32,
        memo: &mut BTreeMap<(u32, u32, u32), Fix32>,
    ) -> Fix32 {
        if left == NONE || right == NONE || depth == 0 {
            return Fix32::ZERO;
        }
        let key = (left.min(right), left.max(right), depth);
        if let Some(value) = memo.get(&key) {
            return *value;
        }
        let value = if left == right {
            // The kinship of a character with itself is one half of one plus
            // its inbreeding coefficient, and the inbreeding coefficient is
            // the kinship of its two parents.
            let raw = self.parents[left as usize];
            let inbreeding = self.kinship(raw[MOTHER], raw[FATHER], depth - 1, memo);
            Self::half(sim_math::add(Fix32::ONE, inbreeding))
        } else {
            // Expand the younger character. A row is allocated in birth
            // order, so the younger character carries the larger identity,
            // and the recursion therefore always ends.
            let younger = left.max(right);
            let older = left.min(right);
            let raw = self.parents[younger as usize];
            let mother = self.kinship(raw[MOTHER], older, depth - 1, memo);
            let father = self.kinship(raw[FATHER], older, depth - 1, memo);
            Self::half(sim_math::add(mother, father))
        };
        memo.insert(key, value);
        value
    }

    /// Halves a fixed-point value.
    ///
    /// The recursion reaches a smallest term of two to the power of the
    /// negative of the depth, and the depth stays below the number of
    /// fractional bits, so the raw value is always even and the halving is
    /// exact.
    fn half(value: Fix32) -> Fix32 {
        sim_math::div(value, Fix32::from_int(2)).expect("two is not zero")
    }

    /// Absorbs the record into the state hash.
    ///
    /// The hash covers the parent edges and the birth counters, because both
    /// Rebuilds the Euler interval labels over the father forest.
    ///
    /// The pass is a depth-first walk of the father forest in preorder. It
    /// writes three things: the preorder position of each row, the last
    /// position inside the subtree of each row, and the row that sits at each
    /// position. After it runs, "is this character a patrilineal descendant
    /// of that one" is two integer comparisons, and "every patrilineal
    /// descendant of this character" is one contiguous span.[^1]
    ///
    /// **The order is explicit at every step.** The roots enter in ascending
    /// descent identity, which is birth order, and the children of a row
    /// enter in the same order, because the child list is built by appending
    /// at the tail of an ascending record.[^2] The labels therefore do not
    /// depend on how a caller reached this pass.
    ///
    /// The walk carries its own stack. A father line of four million rows
    /// would overflow the call stack, and the record admits that many
    /// rows.[^3]
    ///
    /// **The labels are ungapped, and a birth after this pass leaves the new
    /// row unlabelled.** The research names the gapped variant as the
    /// optimisation to take if a measurement ever demands it, and it says not
    /// to build it in the first version.[^1] No measurement exists on the
    /// target platform, so nothing demands it.[^4]
    ///
    /// # References
    ///
    /// [^1]: The character graph and inheritance, sections 3.2 and 3.4. `docs/research/reports/14-character-graph-and-inheritance.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    /// [^3]: ADR-0078, descent is a bounded record, and a relation is a bounded recursion, decision D1. `docs/adrs/draft/adr-0078-descent-is-a-bounded-record-and-a-relation-is-a-bounded-recursion.md`
    /// [^4]: Blockers register, BLK-007. `docs/BLOCKERS.md`
    pub fn relabel(&mut self) {
        let rows = self.born_as.len();
        self.euler_order.clear();
        self.euler_order.reserve(rows);
        for row in 0..rows {
            self.father_tin[row] = NONE;
            self.father_tout[row] = NONE;
        }
        // A frame holds the row being visited and the next child of it that
        // the walk has not entered. The stack replaces recursion, because a
        // father line may be as long as the record.
        let mut stack: Vec<(u32, u32)> = Vec::new();
        let mut position: u32 = 0;
        for root in 0..rows as u32 {
            if self.parents[root as usize][FATHER] != NONE {
                continue;
            }
            self.father_tin[root as usize] = position;
            self.euler_order.push(root);
            position += 1;
            stack.push((root, self.first_child[root as usize][FATHER]));
            while let Some((node, next)) = stack.pop() {
                if next == NONE {
                    // Every child of this row has been entered, so the
                    // subtree ends at the last position the walk wrote.
                    self.father_tout[node as usize] = position - 1;
                    continue;
                }
                stack.push((node, self.next_sibling[next as usize][FATHER]));
                self.father_tin[next as usize] = position;
                self.euler_order.push(next);
                position += 1;
                stack.push((next, self.first_child[next as usize][FATHER]));
            }
        }
    }

    /// Returns the number of rows that the current labels cover.
    ///
    /// The Euler order holds one position for each labelled row, so its
    /// length is the count. **The record does not store this number.** A
    /// stored watermark would be a second declaration of which rows carry a
    /// label, and the absent-label sentinel in the two label columns is
    /// already the first. Nothing would fail when the two disagreed.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[must_use]
    pub fn labelled_rows(&self) -> u32 {
        self.euler_order.len() as u32
    }

    /// Returns the house of a character.
    ///
    /// Returns `None` when the record holds no such row. Every row that the
    /// record holds carries a house, so a live row always answers.
    #[must_use]
    pub fn house_of(&self, id: DescentId) -> Option<HouseId> {
        self.house.get(id.0 as usize).map(|house| HouseId(*house))
    }

    /// Reports whether one character is a patrilineal ancestor of another.
    ///
    /// The answer is two integer comparisons against the Euler labels. It
    /// walks nothing, whatever the depth of the line.[^1]
    ///
    /// A character is not its own ancestor.
    ///
    /// Returns `None` when either row carries no label, which is the case
    /// when the record holds no such row or when the row was recorded after
    /// the last relabel. An absent label answers nothing rather than
    /// answering from a stale one.
    ///
    /// # References
    ///
    /// [^1]: The character graph and inheritance, section 3.1. `docs/research/reports/14-character-graph-and-inheritance.md`
    #[must_use]
    pub fn is_patrilineal_ancestor(&self, ancestor: DescentId, of: DescentId) -> Option<bool> {
        let (high, low) = (self.label(ancestor)?, self.label(of)?);
        Some(ancestor != of && high.0 <= low.0 && low.0 <= high.1)
    }

    /// Returns every patrilineal descendant of a character, in birth order.
    ///
    /// The set is one contiguous span of the Euler order, so the pass reads
    /// a range and never walks the tree.[^1] The result never holds the
    /// character itself.
    ///
    /// The span is contiguous in the Euler order and not in the columns,
    /// because the record stores its rows in birth order. The result is
    /// sorted before it is returned, so the order of the answer is birth
    /// order and does not depend on the shape of the tree.[^2]
    ///
    /// Returns `None` when the row carries no label.
    ///
    /// # References
    ///
    /// [^1]: The character graph and inheritance, section 3.3. `docs/research/reports/14-character-graph-and-inheritance.md`
    /// [^2]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
    #[must_use]
    pub fn patrilineal_descendants(&self, id: DescentId) -> Option<Vec<DescentId>> {
        let (tin, tout) = self.label(id)?;
        let mut out: Vec<DescentId> = self.euler_order[tin as usize + 1..=tout as usize]
            .iter()
            .map(|row| DescentId(*row))
            .collect();
        out.sort_unstable();
        Some(out)
    }

    /// Makes a character the founder of a house, and moves its patrilineal
    /// line into it.
    ///
    /// This is the cadet split. The character and every patrilineal
    /// descendant of it leave the house they were in and join a house whose
    /// founder is the character. The identifier of the new house is the
    /// descent identity of the character, so a split allocates nothing and
    /// two splits at one character give one house.
    ///
    /// The pass reads a contiguous span of the Euler order and writes the
    /// house column at the rows that span names. It is a map over a span and
    /// never a walk of the tree.[^1]
    ///
    /// Returns the number of rows that moved, or `None` when the character
    /// carries no label. A character that already founds its house moves its
    /// whole line and reports it, because the count is what the pass wrote
    /// and not what it changed.
    ///
    /// # References
    ///
    /// [^1]: The character graph and inheritance, section 3.3. `docs/research/reports/14-character-graph-and-inheritance.md`
    pub fn found_house_at(&mut self, id: DescentId) -> Option<u32> {
        let (tin, tout) = self.label(id)?;
        for position in tin..=tout {
            let row = self.euler_order[position as usize] as usize;
            self.house[row] = id.0;
        }
        Some(tout - tin + 1)
    }

    /// Returns every member of a house, in birth order.
    ///
    /// The scan reads the house column once. A house is not a contiguous
    /// span of anything after a cadet split has divided a line, so this is a
    /// filter over the column and not a range read.
    #[must_use]
    pub fn house_members(&self, house: HouseId) -> Vec<DescentId> {
        self.house
            .iter()
            .enumerate()
            .filter(|(_, held)| **held == house.0)
            .map(|(row, _)| DescentId(row as u32))
            .collect()
    }

    /// Returns the two Euler labels of a row, or `None` when it has none.
    fn label(&self, id: DescentId) -> Option<(u32, u32)> {
        let tin = *self.father_tin.get(id.0 as usize)?;
        let tout = *self.father_tout.get(id.0 as usize)?;
        if tin == NONE || tout == NONE {
            return None;
        }
        Some((tin, tout))
    }

    /// decide a later frame. The child lists are derived from the parent
    /// edges, so the hash does not repeat them.[^1]
    ///
    /// The house column enters the hash, because a cadet split writes it and
    /// nothing else records that the split happened. The two Euler labels do
    /// not, because a relabel derives them from the parent edges in one
    /// order and the parent edges are already here.
    ///
    /// # References
    ///
    /// [^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
    #[must_use]
    pub fn hash_into(&self, hash: StateHash) -> StateHash {
        let mut hash = hash.write_u64(u64::from(self.len()));
        for row in 0..self.born_as.len() {
            hash = hash
                .write(&self.born_as[row].to_le_bytes())
                .write(&self.parents[row][MOTHER].to_le_bytes())
                .write(&self.parents[row][FATHER].to_le_bytes())
                .write(&self.last_birth[row].0.to_le_bytes())
                .write(&self.births_on_that_tick[row].to_le_bytes())
                .write(&self.house[row].to_le_bytes());
        }
        hash
    }

    /// Reports whether the record holds its invariants.
    ///
    /// The child lists hold the same facts as the parent edges a second
    /// time. This check fails when the two copies disagree.[^1]
    ///
    /// # References
    ///
    /// [^1]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
    #[must_use]
    pub fn check_invariants(&self) -> bool {
        let rows = self.born_as.len();
        if self.parents.len() != rows
            || self.first_child.len() != rows
            || self.last_child.len() != rows
            || self.next_sibling.len() != rows
            || self.last_birth.len() != rows
            || self.births_on_that_tick.len() != rows
            || self.house.len() != rows
            || self.father_tin.len() != rows
            || self.father_tout.len() != rows
        {
            return false;
        }
        if self.euler_order.len() > rows {
            return false;
        }
        // Each position of the Euler order holds one row, and that row agrees
        // that it sits there. This is what fails when the order and the two
        // label columns disagree about which rows carry a label.
        let mut labelled = 0;
        let mut seen = vec![false; rows];
        for (position, row) in self.euler_order.iter().enumerate() {
            let row = *row as usize;
            if row >= rows || seen[row] {
                return false;
            }
            seen[row] = true;
            if self.father_tin[row] as usize != position {
                return false;
            }
        }
        for row in 0..rows {
            if (self.father_tin[row] == NONE) != (self.father_tout[row] == NONE) {
                return false;
            }
            if self.father_tin[row] != NONE {
                labelled += 1;
            }
        }
        if labelled != self.euler_order.len() {
            return false;
        }
        // A house names a row that the record holds, and it names the row
        // itself or a patrilineal ancestor of it. A birth copies the house of
        // the father, and a cadet split writes a whole subtree, so no other
        // shape is reachable. This check fails when a split writes a row that
        // the span does not cover.
        for row in 0..rows {
            let house = self.house[row];
            if house as usize >= rows {
                return false;
            }
            let id = DescentId(row as u32);
            let founder = DescentId(house);
            if founder == id {
                continue;
            }
            match self.is_patrilineal_ancestor(founder, id) {
                Some(true) => {}
                // A row above the relabel watermark carries no label, so
                // nothing can be said about it here.
                None => {}
                Some(false) => return false,
            }
        }
        if rows > DESCENT_CEILING as usize {
            return false;
        }
        // A parent is always older than its child, so no character is its own
        // ancestor and the graph holds no cycle.
        for row in 0..rows {
            for role in [MOTHER, FATHER] {
                let parent = self.parents[row][role];
                if parent != NONE && parent as usize >= row {
                    return false;
                }
            }
            if self.parents[row][MOTHER] != NONE
                && self.parents[row][MOTHER] == self.parents[row][FATHER]
            {
                return false;
            }
            if self.born_as[row] == 0 {
                return false;
            }
        }
        // Every child list holds exactly the children that the parent edges
        // name, and it holds them in ascending order.
        let mut counted = vec![0u32; rows];
        for row in 0..rows {
            for role in [MOTHER, FATHER] {
                let parent = self.parents[row][role];
                if parent != NONE {
                    counted[parent as usize] += 1;
                }
            }
        }
        let mut walked = vec![0u32; rows];
        for (row, count) in walked.iter_mut().enumerate() {
            for role in [MOTHER, FATHER] {
                let mut child = self.first_child[row][role];
                let mut previous = NONE;
                while child != NONE {
                    if child as usize >= rows || self.parents[child as usize][role] != row as u32 {
                        return false;
                    }
                    if previous != NONE && previous >= child {
                        return false;
                    }
                    *count += 1;
                    previous = child;
                    child = self.next_sibling[child as usize][role];
                }
                if previous != self.last_child[row][role] {
                    return false;
                }
            }
        }
        counted == walked
    }

    /// Turns a raw row into an optional identity.
    const fn option(row: u32) -> Option<DescentId> {
        if row == NONE {
            None
        } else {
            Some(DescentId(row))
        }
    }
}

#[cfg(test)]
mod tests {
    //! Unit tests for the cases that the public interface cannot reach.
    //!
    //! The record allocates a row after both parents, so a parent always
    //! carries the smaller identity. A caller outside this module therefore
    //! cannot name a parent that is not older than its child, and cannot
    //! make a character its own ancestor. The guard still exists, because it
    //! is the only thing that refuses a cycle, and a later caller inside
    //! this crate could reach it.

    use super::*;

    /// Builds an identity without an arena.
    fn entity(index: u32) -> Entity {
        Entity::new(index, 1).expect("a generation of one makes the identity non-zero")
    }

    #[test]
    fn a_parent_that_is_the_child_is_refused() {
        let mut record = Descent::new();
        let first = record
            .record(entity(0), Parents::NONE)
            .expect("the first row must record");
        // The row that the record would allocate next is the row after
        // `first`, so naming `first` as its own parent needs the identity
        // that the record has not issued yet.
        let itself = DescentId(record.len());
        assert_eq!(
            record.record(
                entity(1),
                Parents {
                    mother: Some(itself),
                    father: None
                }
            ),
            Err(DescentError::ParentIsUnknown(itself))
        );
        assert_eq!(record.len(), 1);
        assert_eq!(record.parents(first), Some(Parents::NONE));
        assert!(record.check_invariants());
    }

    #[test]
    fn a_parent_the_record_has_not_issued_is_refused() {
        let mut record = Descent::new();
        record
            .record(entity(0), Parents::NONE)
            .expect("the first row must record");
        let unissued = DescentId(record.len() + 1);
        assert_eq!(
            record.record(
                entity(1),
                Parents {
                    mother: Some(unissued),
                    father: None
                }
            ),
            Err(DescentError::ParentIsUnknown(unissued))
        );
        assert_eq!(record.len(), 1);
        assert!(record.check_invariants());
    }

    #[test]
    fn two_parents_that_are_one_character_are_refused() {
        let mut record = Descent::new();
        let one = record
            .record(entity(0), Parents::NONE)
            .expect("the first row must record");
        assert_eq!(
            record.record(
                entity(1),
                Parents {
                    mother: Some(one),
                    father: Some(one)
                }
            ),
            Err(DescentError::ParentsAreOneCharacter(one))
        );
        assert_eq!(record.len(), 1);
        assert!(record.check_invariants());
    }

    #[test]
    fn a_parent_edge_that_names_a_younger_row_fails_the_check() {
        // The record cannot reach this state, because it allocates a row
        // after both parents. The check exists because a cycle is the one
        // failure that every walk in this module would run on forever.
        let mut record = Descent::new();
        record
            .record(entity(0), Parents::NONE)
            .expect("the first row must record");
        record
            .record(
                entity(1),
                Parents {
                    mother: Some(DescentId(0)),
                    father: None,
                },
            )
            .expect("the second row must record");
        assert!(record.check_invariants());
        record.parents[0][MOTHER] = 1;
        assert!(!record.check_invariants());
    }

    #[test]
    fn a_child_list_that_disagrees_with_the_parent_edges_fails_the_check() {
        let mut record = Descent::new();
        record
            .record(entity(0), Parents::NONE)
            .expect("the first row must record");
        record
            .record(
                entity(1),
                Parents {
                    mother: Some(DescentId(0)),
                    father: None,
                },
            )
            .expect("the second row must record");
        assert!(record.check_invariants());
        record.first_child[0][MOTHER] = NONE;
        assert!(!record.check_invariants());
    }

    #[test]
    fn the_relation_depth_keeps_every_value_exact() {
        // Every step of the recursion halves a value, so the smallest term
        // is two to the power of the negative depth. The raw value stays
        // even while the depth leaves a fractional bit spare, and an even
        // raw value halves without a remainder.
        let smallest = Fix32(1 << (FIX_FRACTIONAL_BITS - RELATION_DEPTH));
        assert_eq!(Descent::half(smallest).0 * 2, smallest.0);
        assert!(smallest.0 > 1);
    }
}
