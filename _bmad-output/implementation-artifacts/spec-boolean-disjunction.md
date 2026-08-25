---
status: approved
story: 3.2
date: 2026-08-26
---

# Boolean Disjunction

## Frozen Intent

### Always

- `left || right` builds `ExprKind::Or` with the full covering span. Precedence: comparison > `!` > `||`; consecutive `||` is left-associative; parentheses override the default shape using the Story 3.1 grammar.
- The lexer tokenizes `||` as a single operator; a single `|` remains a plain symbol (no operator meaning). A no-newline comment between the two bars is a `MORVA1025` token split, via the shared operator-start split guard.
- Both sides of `||` must each type-check as Boolean (`MORVA2013` on the offending side). The right side is fully statically checked even when the left is constant `true`; unknown references keep their existing codes.
- Missing right operand fails with existing `MORVA1013`; a stray `|` after a complete expression fails with the existing clause-boundary error.
- Simulation evaluates `||` deterministically left-to-right with short-circuit: a `true` left never reads right-side runtime state; a `false` left evaluates the right, and an uninitialized right read fails with the right path's span.
- Assignment values treat `Or` like `Binary`/`Not` (Boolean predicate value for `=`, `MORVA2017` for compound). Scenario given values reject it via `MORVA3012`. `Or` predicates contribute no contradiction facts yet (Story 3.3).
- The capability inventory declares `Boolean disjunction (left-associative, short-circuit)` and replaces the unsupported 'or' entry with the explicit `&&` gap (conjunction stays multiple clauses).

### Never

- No `&&` operator, no implicit conjunction changes, no arithmetic, no new value types, no change to `!=`/comparison behavior, diagnostics, exit codes, or phases.

### Ask First

- Right-associative or non-short-circuit semantics, `|` as an operator, or extending disjunction facts before Story 3.3.

## Verification

- `language.rs`: left-associativity, precedence vs `!` and comparison, parenthesized override, error codes 1013/1018/2013/2009/1025.
- `simulation.rs`: short-circuit both directions with uninitialized-read span, cross-phase deterministic evaluation.
- `cli.rs`: parse rendering and the updated capabilities expression-forms line.
