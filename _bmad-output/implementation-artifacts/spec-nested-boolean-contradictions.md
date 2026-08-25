---
status: approved
story: 3.3
date: 2026-08-26
---

# Conservative Nested Boolean Contradictions

## Frozen Intent

### Always

- Predicate formulas built from literals, exact comparisons, `!`, `||`, and parentheses are evaluated with three-valued `True / False / Unknown` logic against the current phase's known exact facts; a contradiction is reported only when the whole predicate is provably `False`.
- Constant-false formulas (e.g. `false || false`, `!(true)`) report `MORVA2018` with message `predicate is always false`; formulas falsified by earlier facts in the same group report the existing `predicate conflicts with an earlier literal constraint on '{path}'` message, where `{path}` is the first fact-consulted path in evaluation order. The span always covers the responsible formula.
- A disjunction with any provably `True` branch never reports; `Unknown` branches keep the formula `Unknown` unless some assignment of branch results makes it definitively `False`. Unreported never means proven satisfiable.
- Only top-level plain exact `==`/`!=` comparisons contribute facts to the group fact set; facts inside `!` or a `||` branch never leak out. Pre-state (`requires` + action invariant) and post-state (action invariant + `ensures`) groups stay separate.
- Post-state formulas are additionally evaluated against final known direct literal `=` effects; a provably `False` formula reports the existing `MORVA2019` message. Non-literal or compound writes demote the path to `Unknown`. The `MORVA2018`-same-span suppression rule is preserved.
- Models without `!`, `||`, or parentheses keep byte-identical `MORVA2018`/`MORVA2019` codes, messages, spans, and ordering.
- Reference or type errors in an action suppress all fact derivation for that action (existing gate), keeping primary diagnostics only.

### Never

- No formula distribution, DNF/CNF conversion, full SAT, ordered-interval solving, entity-invariant instantiation, scenario/action inlining, or cross-phase unwritten-path reasoning.
- No false positives: every reported contradiction must be a real one under the recorded facts.

### Ask First

- Extending fact contribution through negation, evaluating ordered comparisons from equal facts, or bare Boolean paths as facts.

## Verification

- `language.rs`: constant-false nested forms, fact-falsified nested formulas, true-branch immunity, unknown conservation, branch-fact non-leak (both directions), nested `MORVA2019` with final effects, compound-write demotion, primary-diagnostic suppression, plus all pre-existing MORVA2018/2019 regressions unchanged.
