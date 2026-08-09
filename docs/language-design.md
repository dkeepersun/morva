# Morva language design draft

## Purpose

Morva describes a system at the semantic design level. It should be more precise than prose and substantially smaller than an implementation language. The model is the source for validation, simulation, visualization, AI challenge/review, and implementation context.

## v0.1 principles

1. Prefer a few composable concepts over a keyword for every concern.
2. Keep executable semantics explicit; natural language may annotate or generate a model but does not replace it.
3. Report semantic mistakes early without claiming formal proof.
4. Treat implementation guidance as soft advice.
5. Add syntax only after concrete examples demonstrate a need.

## Proposed declarations

The initial vocabulary is deliberately limited:

```text
system  module
entity  enum
service action event
flow    lifecycle scenario
policy
```

Behavioral clauses:

```text
requires effects ensures invariant
atomic idempotent timeout retry
implementation_hint
```

The current parser recognizes declarations and balanced blocks. It does not yet assign executable meaning to clauses inside a block.

## Sketch grammar

This is an orientation draft, not a frozen grammar:

```ebnf
document    = { declaration } ;
declaration = kind, identifier, [ parameter-list ], block ;
kind        = "system" | "module" | "entity" | "enum"
            | "service" | "action" | "event" | "flow"
            | "lifecycle" | "scenario" | "policy" ;
block       = "{", { token | declaration }, "}" ;
```

## Validation boundary

The first static analyzer should check syntax, names and references, basic types, state transitions, and obvious contradictory clauses. It is semantic static validation, not theorem proving.

The present skeleton checks:

- lexical and delimiter errors;
- declaration shape;
- duplicate declaration names within the same scope;
- presence of exactly one top-level `system` declaration.

## Simulation boundary

`simulate` will walk a scenario, flow, or lifecycle against an explicit initial model state and show state changes, events, failures, expectations, and invariant results. It will not run application code, access production infrastructure, or become a general runtime.

## Implementation hints

`implementation_hint` communicates a preference to an AI or generator:

```morva
implementation_hint {
  storage: relational
  consistency: strong
}
```

Ignoring a hint may produce information or a warning, never a semantic error by itself.

## Deferred work

Formal verification, code generation, security/SLA-specific keywords, compensation primitives, package management, macros, and deployment configuration remain deferred until real use cases justify them.

