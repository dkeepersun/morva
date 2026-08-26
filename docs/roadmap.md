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

## Completed after the v0.1 core (2026-08-26)

- [x] honest semantic-coverage feedback: structured container/soft-behavior warnings, the `inspect` unmodeled summary, and the authoritative `capabilities` inventory
- [x] predicate-level Boolean expressiveness: `!`, grouping, short-circuit `||`, and conservative three-valued nested contradiction analysis
- [x] versioned machine output: `--format json` for all commands over the shared `morva-machine` payloads, plus the checked-semantics protocol v1 single-file core slice
- [x] read-only AI integration: the std-only `morva-mcp` stdio server (capability resource + in-memory check/parse/inspect/simulate tools)
- [x] governance: pinned toolchain and verified MSRV 1.85, fact-frozen evolution policy, and a repeatable NFR-04 scale/trend baseline

## After the core is stable

- L2 linear arithmetic with the missing literal kinds (Decimal, negative, String), landing together with the non-linear red line enforcement — needs its own approved spec
- `lifecycle` state-machine semantics
- Tree-sitter grammar and syntax highlighting
- LSP diagnostics, navigation, and completion
- AI `grill`, `review`, and `challenge` over the machine-readable representations
- Mermaid/graph export
- Flow simulation, if concrete examples justify it

These are directions, not commitments. Formal verification and implementation code generation remain outside scope; L3 collections/quantifiers stay frozen pending real-model evidence.

## Repository quality gate

The GitHub Actions workflow runs on GitHub-hosted runners; the most recent verified
successes are the 2026-08-26 pushes through `main` commit `ce0c6c2`, covering the
pinned-toolchain lint job, all test shards (core, machine, CLI, MCP), and the example
loop. The stated precondition for making `Quality gate` a required branch-protection
check remains satisfied, but the check is still not required — enabling it is a remote
repository settings change that must be performed and recorded manually.
