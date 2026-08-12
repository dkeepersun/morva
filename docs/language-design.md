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

The current semantic core models `system`, `entity`, `enum`, `action`, and `scenario`, including enum members, fields, parameters, scenario items, and `requires`, `effects`, `ensures`, and `invariant` clauses. Broader declaration kinds remain compatibility containers. Within an action, only the documented soft items `atomic`, `idempotent`, `timeout`, `retry`, and `implementation_hint` are accepted outside the semantic behavior model; unknown items are errors. Their AST representation retains only kind and keyword span provenance, not payload or executable semantics.

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
scenario    = "scenario", identifier, "{",
              { "given", path, "=", value, line-end },
              "run", identifier, "(", [ identifier, { ",", identifier } ], ")", line-end,
              "expect", expression, line-end,
              { "expect", expression, line-end }, "}" ;
value       = enum-member | boolean | integer ;
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
- canonical builtin aliases, Boolean predicates, compatible comparison operands, same-type set effects, and Integer-only compound effects.

Ordered comparison accepts Integer or Decimal. A non-negative Integer literal in an explicit Decimal comparison or assignment context is treated as an exact Decimal constant; this is a local compatibility rule, not general numeric conversion. Entity values may only be traversed to fields, not compared or assigned as whole objects.

Morva does not yet implement module lookup rules. If compatibility containers introduce multiple types with the same short name, use of that short name is rejected as ambiguous regardless of declaration order.

## Simulation boundary

`simulate` executes the current scenario shape against isolated in-memory state:

```text
given* → exactly one run → expect+
```

Run arguments bind positionally to distinct entity instances for a single action. `given` initializes direct fields with enum members, Boolean, or Integer values. Execution applies givens, initial entity invariants, requires and action invariants, ordered effects, final action/entity invariants, ensures, then expects. All reads must already be initialized, and Integer compound assignments use checked arithmetic.

Simulation does not support scalar action parameters, aliases, multiple actions, flows, lifecycles, cross-scenario state, external I/O, or application code. It is a model-level interpreter, not a general runtime.

## Implementation hints

`implementation_hint` communicates a preference to an AI or generator:

```morva
implementation_hint {
  storage: relational
  consistency: strong
}
```

Explicit analysis and CLI `check` produce one structured non-fatal `MORVA5002` warning per parsed soft item, including `implementation_hint`. The warning records its action, kind, and keyword span. The item still does not enter semantic evaluation or simulation and never creates a semantic error by itself.

## Deferred work

Formal verification, code generation, security/SLA-specific keywords, compensation primitives, package management, macros, and deployment configuration remain deferred until real use cases justify them.
