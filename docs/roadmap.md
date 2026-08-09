# Roadmap

## v0.1 — semantic core

- [x] Rust workspace and CLI shell
- [x] spanned lexer/parser and strongly typed core AST
- [x] structural, enum-member, and reference semantic checks
- [x] example, public API, and CLI integration tests
- [x] define the minimal expression, type-reference, and diagnostic models
- [x] validate names, types, field paths, contextual enum members, and effect targets
- [x] add single-action scenario simulation with explicit in-memory state

The original two approved v0.1 baseline specifications are complete. Subsequent
approved increments also completed static expression type checks, bounded diagnostic
rendering, the universal newline contract, and conservative exact-literal transition
contradiction checks.

- [x] validate exact-literal state transitions and obvious same-phase clause contradictions

## After the core is stable

- Tree-sitter grammar and syntax highlighting
- LSP diagnostics, navigation, and completion
- AI `grill`, `review`, and `challenge` over a stable semantic representation
- Mermaid/graph export
- Flow and lifecycle simulation, if concrete examples justify it
- Skill and MCP integration

These are directions, not commitments for v0.1. Formal verification and implementation code generation remain outside the initial scope.

## Repository quality gate

The GitHub Actions workflow is configured in local commits but has not yet been pushed
or run by GitHub. `Quality gate` must only become a required branch-protection check
after the workflow is available remotely and has completed at least one hosted run.
