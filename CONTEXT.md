# Morva Language Context

Morva describes reviewable semantic models whose structured source is the shared truth for people, tools, and AI.

## Language

**State transition**:
An action's ordered change from a state satisfying its preconditions to a resulting state checked against its postconditions.
_Avoid_: Unordered update set, guaranteed-success function

**Preconditions**:
The `requires` predicates and action invariants that must hold before effects execute.
_Avoid_: Input validation

**Postconditions**:
The action invariants and `ensures` predicates that must hold after effects execute.
_Avoid_: Output validation

**Obvious contradiction**:
A set of predicates whose unsatisfiability follows directly from resolved literal facts in one state phase, without symbolic execution or general theorem proving.
_Avoid_: Formal proof, complete contradiction detection
