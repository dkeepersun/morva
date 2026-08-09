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

The current semantic core models `system`, `entity`, `enum`, and `action`, including enum members, fields, parameters, and `requires`, `effects`, `ensures`, and `invariant` clauses. Broader declaration kinds remain compatibility containers. Within an action, only the documented soft items `atomic`, `idempotent`, `timeout`, `retry`, and `implementation_hint` may be ignored by the current semantic model; unknown items are errors.

## Sketch grammar

This is an orientation draft, not a frozen grammar:

```ebnf
document    = { declaration } ;
system      = "system", identifier, "{", { declaration }, "}" ;
enum        = "enum", identifier, "{", { identifier, line-end }, "}" ;
entity      = "entity", identifier, "{", { field | invariant }, "}" ;
field       = identifier, ":", identifier, line-end ;
action      = "action", identifier, [ parameters ], "{", { clause | soft-item }, "}" ;
parameters  = "(", [ parameter, { ",", parameter } ], ")" ;
parameter   = identifier, ":", identifier ;
clause      = ("requires" | "effects" | "ensures" | "invariant"),
              (expression | "{", { expression, line-end }, "}") ;
expression  = path | integer | boolean | expression, comparison, expression ;
path        = identifier, { ".", identifier } ;
```

An `effects` expression is an assignment (`=`, `+=`, or `-=`). Its target must be a field path rooted at an action parameter.

A bare identifier is not a general symbolic value. It is valid only when a comparison or assignment supplies a specific expected enum type and that enum declares the member. For example, `order.status == Pending` resolves `Pending` through the `OrderStatus` type of `order.status`. A misspelled or wrong-enum member is an error.

## Validation boundary

The current analyzer performs semantic static validation, not theorem proving. It checks:

- lexical and delimiter errors;
- declaration, enum member, field, parameter, clause, path, comparison, and assignment shape;
- duplicate declaration, enum member, field, and parameter names;
- presence of exactly one top-level `system` and rejection of nested systems;
- known and globally unambiguous short type names;
- parameter/field paths, contextual enum members, and writable `effects` targets.

Morva does not yet implement module lookup rules. If compatibility containers introduce multiple types with the same short name, use of that short name is rejected as ambiguous regardless of declaration order.

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
