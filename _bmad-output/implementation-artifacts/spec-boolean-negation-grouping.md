---
status: approved
story: 3.1
date: 2026-08-26
---

# Boolean Negation and Grouping

## Frozen Intent

### Always

- Predicates gain a recursive Boolean layer limited to `!` and balanced parentheses; comparison operands stay non-recursive (literals and paths only).
- Precedence: comparison binds tighter than `!` — `!a == b` parses as `!(a == b)`. Consecutive `!` and nested balanced parentheses are allowed at any depth; redundant parentheses change only the outer span, never the AST shape.
- `ExprKind::Not(Box<Expr>)` carries the full original byte span from `!` through its operand. A parenthesized group returns the inner expression's kind with the span extended over both parentheses.
- Unclosed group: `MORVA1026: expected ')' to close the grouped predicate`. Empty group: existing `MORVA1013` at the closing token. `!` with a non-Boolean operand: existing `MORVA2013` anchored on the operand expression.
- All predicate positions (entity/action invariant, requires, ensures, scenario expect) share the same core type rules and the same seven-phase deterministic in-memory evaluation. Negation over an uninitialized read keeps the existing failure contract with the responsible path span.
- Lexer treatment of `!=` is byte-for-byte unchanged; `!` acts only as the predicate negation operator. The `MORVA1025` comment-token-split contract still covers `!` + `=`.
- Assignment values treat `Not` exactly like `Binary`: a Boolean-typed predicate value for `=`, and an `MORVA2017` type error for `+=`/`-=`. Scenario given values reject negation via existing `MORVA3012`.
- `Not` predicates do not participate in contradiction fact derivation yet (Story 3.3); `literal_fact`/`plain_literal` return no fact for them.
- The capability inventory declares `Boolean negation` and `grouped predicate` expression forms and narrows the unsupported entry to `logical 'or'`.

### Never

- No `||`, arithmetic, string/decimal literals, parenthesized comparison operands, or partial-AST recovery.
- No change to existing diagnostic codes, exit codes, simulation phases, or the newline/comment contracts.

### Ask First

- Different precedence, a dedicated Group AST node, or extending negation facts before Story 3.3.

## Verification

- `language.rs`: precedence equivalence, span coverage for `!...` and `((...))`, double negation, error codes 1026/1013/2013, all-position acceptance.
- `simulation.rs`: negation across phases, requires failure, uninitialized-read span through `!`.
- `cli.rs`: parse rendering (`!path`, `!(comparison)`) and updated capabilities line.
