---
project_name: 'Morva'
user_name: 'alex'
date: '2026-08-10'
sections_completed: ['technology_stack', 'critical_implementation_rules']
existing_patterns_found: 14
status: 'complete'
---

# Project Context for AI Agents

_This file contains critical rules and patterns that AI agents must follow when implementing code in this project. It intentionally focuses on unobvious constraints; the full requirements live in `docs/requirements.md`._

---

## Technology Stack & Versions

- Rust edition 2024 workspace, package version 0.1.0.
- Documentation-baseline verification used rustc 1.95.0 and cargo 1.95.0; this is an environment snapshot, not a declared MSRV.
- Cargo workspace resolver 2.
- `morva-core`: dependency-free library and sole language-semantics layer.
- `morva-cli`: binary depending only on the local `morva-core` crate.
- Tests use Rust's built-in test harness; no external test framework.
- No database, network service, UI framework, auth, deployment runtime, or CI configuration.

## Critical Implementation Rules

- Treat the human-reviewed `.morva` model as the Source of Truth. Preserve `Morva` / `morva` naming; `.morva` is the project convention, not a CLI-enforced suffix.
- Keep all language semantics in `morva-core`. The CLI may read files and render results, but must not duplicate parser, checker, or simulator rules.
- Model `system`, `entity`, `enum`, `action`, and `scenario` as typed semantics. `module`, `service`, `event`, `flow`, `lifecycle`, and `policy` remain compatibility containers; soft action items are accepted but not executed or validated as behavior.
- Resolve type, action, and scenario short names globally. Do not invent module scope or qualified names.
- Do not claim complete static type safety: the checker normalizes builtin aliases and validates the existing predicate, comparison, and effect forms, but it does not implement general inference, conversions, data-flow analysis, or theorem proving.
- Preserve the expression type boundary: `Bool`/`Boolean`, `Int`/`Integer`, and `ID`/`Id` are canonical families; ordered comparison accepts Integer or Decimal, and a non-negative Integer literal is an exact Decimal constant only in an explicit Decimal operand or target context. Entity is path-only, not a comparable or assignable whole value.
- Keep simulation to one action per scenario, direct entity fields, isolated in-memory enum/Boolean/Integer state, and the fixed seven phases. Never execute `implementation_hint`, user code, IO, or external state.
- Preserve UTF-8 byte spans, stable diagnostic codes, CLI exit codes, deterministic ordering, and the meaning of `examples/order.morva`.
- Before changing syntax, CLI contracts, dependencies, simulation boundaries, names, or the meaning of an approved example, obtain human approval and create or update an implementation specification. Never edit an approved specification's frozen block to fit code.
- Preserve existing worktree changes. Every behavior change requires focused tests and documentation, followed by `cargo fmt --check`, strict Clippy, workspace tests, and the relevant example commands.

See [requirements.md](requirements.md), [architecture.md](architecture.md), [testing-strategy.md](testing-strategy.md), and [ai-handoff.md](ai-handoff.md) for the full contracts and handoff procedure.
