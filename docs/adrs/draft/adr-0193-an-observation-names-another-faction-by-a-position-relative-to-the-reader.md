# ADR-0193: A faction's observation names another faction by a position relative to the reader

## Context

The engine gives one faction one flat array of signed integers on every
decision, and one schema that says where each field of the array starts.[^1] A
learner trains a function of that layout. Some fields of the array hold one
position for each faction of the world: the relation the reader holds toward
each faction, and the trade board that each faction posts.

An accepted record fixes how a caller finds a field. The engine builds the
schema and the array from one field list, and a caller decodes by arithmetic
over the schema.[^1] That record says nothing about the order inside a field
that holds one position for each faction. **The order was therefore never
decided, and the writer chose the seat number because it was the index it
already had.**

**A seat number makes one array mean two things.** The training loop seats one
candidate in more than one seat of one world, and it turns the seat of a
candidate at each seed index of a generation.[^2] One set of weights therefore
reads the array from one seat and then from another. A weight that learned
"this position is my rival" in one seat reads a different faction under the
same weight in the next seat. The array holds no wrong number. It is addressed
inconsistently between readers, and a policy cannot learn a relation that moves
under it.

The problem is not confined to another faction count. It is live at the count
the trainer runs, because the seat turns inside one generation.

**A faction count changes the length of the array, and that is a separate
matter.** A stored policy already refuses a world whose faction count differs
from the one it was trained against.[^3] The defect this record answers appears
inside one faction count.

**One field of the array names the reader itself, and it stays.** The array
reports which faction reads it. That is the reader's own identity, not the name
of a rival, and a caller that decodes a relative position needs it.

## Decision

**D1. A field that holds one position for each faction is addressed by the
distance from the faction that reads it, and never by a seat number.**

Position zero of such a field names the reader. Position `k` names the faction
`k` seats after the reader, counting round the seats of the world. The rule is
one addition and one remainder over the faction count.

The reader's own quantities therefore sit at one place, whatever seat the reader
holds. Its rivals follow in one order that every reader of one world agrees on.

**D2. The order of the rivals is the rotation by seat distance, and never a sort
by a game quantity.**

A sort by held ground or by strength would give a policy a meaningful order. It
would also make the meaning of the array depend on the state of the game. A
rival would move between positions from one decision to the next, whenever the
quantity that sorts it crosses another faction's. A policy would then learn a
function of an address that moves under it, which is the defect this record
answers rather than a cure for it.

The rotation reads no state. It is a function of the reader and of the faction
count, so two readers of one world agree on it, and one reader agrees with
itself across a game.

**D3. One function inside the engine states the mapping from a relative
position to a faction, and every writer of a faction-indexed field calls it.**

A field that addressed its own positions would be a second declaration of the
rule, and nothing would fail when the two disagreed.[^4] The function is not
public, because no caller outside the engine reads it. A caller that decodes the
array holds the seat of the reader at the field that reports it, and it applies
the rule.

**D4. The layout version rises when the addressing changes.**

The array carries a version beside the schema, and a stored policy states the
version it was trained against.[^1] The addressing is part of what a position
means, so a change to it moves the version, in the same way a field added or
removed moves it. A stored policy under the earlier version is refused and never
read under the new one.

## The alternatives this rejects

**Leave the seat number and let the trainer hold one seat.** A policy that only
ever plays one seat reads one meaning. This costs the sample efficiency the
seated league buys, and it costs the cancellation of the seat advantage that the
seat rotation exists to give.[^2] It also leaves the array wrong for any later
caller that seats a policy elsewhere.

**Leave the seat number and let the control plane rotate the array.** A decoder
outside the engine could permute the faction-indexed fields into a relative
order. That puts a statement of the layout outside the engine, which the schema
record refuses.[^1] It also gives the rule two declaration sites.

**Sort the rivals by a game quantity.** Rejected under D2.

**Give each rival a fixed identity that the world assigns at construction.** A
world could name a "rival one" and a "rival two" for each seat, stored rather
than derived. The mapping would then be state, it would need a hash, and it
would answer the same question the rotation answers with arithmetic.

## Consequences

**Every stored policy of the earlier layout is retired.** The version rises, and
the fit check refuses a file that states the earlier one.[^3] This record writes
no migration path. A permutation of the weights of the faction-indexed fields
would recover a policy trained in one seat, and it would recover nothing for a
policy trained across seats, because such a policy learned a function of two
meanings at once. Retrain rather than convert.

**A policy cannot address a named faction.** A weight cannot say "the faction in
seat two". It can only say "the faction one seat after me". A study that wanted
per-seat behaviour would have to read the field that names the reader and learn
an interaction, which a linear policy cannot do.

**The rotation carries no game meaning.** The first rival of a reader is not its
strongest rival, nor its nearest. A policy that needs "the strongest rival" must
read the quantities of every rival and combine them, and the array gives it the
quantities to do that.

**A change to the seat count changes which faction each position names.** The
array length already follows the faction count, and a stored policy already
refuses a world of another count, so this adds no new failure.[^3]

**A test that reads one seat cannot hold this record.** The seat that the writer
already agreed with is seat zero, so a one-seat test passes over the defect. The
findings register holds that, with the evidence.[^5]

## References

[^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decisions D1 and D2 and the consequences. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^2]: The seated league of the control plane, the seat-matched plan. `python/cachette/learn/league.py`
[^3]: The policy fit of the control plane. `python/cachette/learn/policy.py`
[^4]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
[^5]: Findings register, FND-647. `docs/FINDINGS.md`
