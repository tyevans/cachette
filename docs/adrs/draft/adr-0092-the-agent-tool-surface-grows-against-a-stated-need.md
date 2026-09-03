# ADR-0092: The agent tool surface grows one tool at a time, against a stated need

## Context

This repository holds a protocol server that lets an agent run the engine. An
agent builds a world, steps it, and reads what happened, through tool calls
rather than by writing a throwaway test.[^1]

The server is a control plane over the engine, and the boundary rule binds it:
Python builds a selector or names an address, and the engine does the work. No
tool loops over entities.[^2]

**The server has a growth policy, and until now nothing recorded it.** The
policy was written once, in the closing section of the backlog item that built
the first slice, in a file that then moved to the completed directory. No
product record and no decision record held it. A worker who did not build the
server could not find it. Two workers then reached the same gap from different
directions. Neither learned that a rule already governed the answer. The
findings register holds that instance.[^3]

The policy matters because the pressure on this surface is one-directional.
Every reader who meets a gap wants a tool. A surface that answers every wish
grows past what anything calls. This project already names a capability nobody
invokes as a recurring defect shape.[^4] A surface that answers nothing sends
the reader back to the throwaway test. That is the state the server exists to
end.

Two other options were live.

**Mirror the engine.** Give one tool for every public reader of the bindings,
and generate them where possible. This removes the judgement, and it removes
the record of who needed what. It also fixes the tool set to the shape of the
bindings. A reader whose question crosses two readers still gets no answer.

**Wait for a product record for each tool.** This is the heavier process, and
it prices a five-line reader at the cost of a shaped requirement. The reader
who needs it would write the throwaway test instead.

## Decision

### D1 A tool is added because a reader needed it, and never ahead of one

Add a tool when somebody states the question they could not answer. Name the
need in the commit that adds the tool. Do not add a tool because the engine
happens to hold the value. Do not add a tool that nothing has asked for.

A tool that no reader asked for is a capability nobody invokes. It passes its
own test, it costs an agent context on every listing, and nothing fails when it
goes stale.[^4]

This is a rule about the surface, not about the engine. A reader may exist in
the engine for a long time before any tool reports it.

### D2 A gap the server cannot close is recorded, not worked around

A quantity that no engine call answers needs an engine verb. Until that verb
exists, the gap is a finding. It is never a
computation in the server.

The server computes nothing. It calls the engine, renames the fields for a
reader, and truncates a long answer. The server must not total, average, search
or filter. Work of that kind is the control plane acting as a data plane.[^2]
It also puts a second copy of an engine rule where nothing fails when the
copies disagree.[^4]

A tool may name a window, a set or an address, and hand it to the engine. That
is a selector, and it is what the boundary is for. An identity a tool passes
back is the engine's own, and the engine resolves it.[^5]

## Consequences

The tool set is a record of what agents have actually needed. A listing of it
answers "what has anyone been unable to do", which no other document holds.

**The surface is incomplete on purpose, and a reader will meet a gap.** That is
the cost. The compensation is D2: the gap goes into a register, so the next
reader who meets it finds the answer rather than the wall.

An agent cannot read a quantity the engine does not compute. Where the engine
holds only the parts, the tool reports the parts and says so, rather than
combining them.

**The server can go stale against the engine, and nothing fails when it does.**
This record does not fix that. A reader whose question the engine gained an
answer for last month still meets the wall until somebody states the need.
Reviewing the surface against the engine is work this record does not schedule.

A tool that reports what the engine computed is not the same as a tool a reader
can check. Both are allowed. A tool that restates an engine number must not be
described as though a reader could verify it independently.

## References

[^1]: Backlog item 0152. `docs/backlog/complete/0152-let-an-agent-drive-the-engine-through-a-protocol-server.md`
[^2]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^3]: Findings register, FND-202. `docs/FINDINGS.md`
[^4]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
[^5]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
