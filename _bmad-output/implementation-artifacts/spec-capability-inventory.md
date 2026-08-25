---
status: approved
story: 2.4
date: 2026-08-26
---

# Capability Inventory

## Frozen Intent

### Always

- `morva_core::capabilities()` returns a deterministic, structured `CapabilityInventory` with `version` (`CAPABILITY_INVENTORY_VERSION`, currently 1) and, in stable field order: semantic declarations, clause kinds, expression forms, comparison operators, assignment operators, literals, builtin types, builtin type aliases, simulation value types, simulation phases, compatibility containers, soft behaviors, and explicitly unsupported categories. It uses no CLI, filesystem, network, clock, or randomness.
- Container kinds, soft behaviors, clause kinds, operators, and simulation phases are derived from the same constants the parser, AST, and simulator execute (`SEMANTIC_DECLARATION_KINDS`, `COMPATIBILITY_CONTAINER_KINDS`, `SoftBehaviorKind::ALL`, `ClauseKind::ALL`, `BinaryOperator::ALL`, `AssignmentOperator::ALL`, `SimulationPhase::ALL`, builtin name consts). No second executable capability table exists in CLI or docs.
- `morva capabilities` takes no path argument, reads or writes no `.morva` file, prints the same inventory as stable human-readable text on stdout, and exits 0; repeat runs are byte-identical. Any extra argument is a usage error (exit 2).
- Public tests prove each listed container kind parses into exactly one `MORVA5001`, each listed soft behavior into exactly one `MORVA5002` with a matching structured kind, each listed operator/clause/builtin (including aliases) is accepted by the real parser and checker, and a real simulation's phase sequence equals the listed phases.
- Adding or removing a language capability must update the inventory and its exact tests in the same delivery.
- The inventory version bumps only on reviewed contract changes to the inventory shape or content semantics.

### Never

- Do not read model files in `capabilities`; do not change `check`/`parse`/`inspect`/`simulate` semantics, warnings, or exit codes.
- Do not claim compatibility containers or soft behaviors are validated or executed; they are listed under parsed-only headings.
- Do not add JSON or machine-readable serialization (Epic 4 will serialize this same structure), MCP, or new language syntax.

### Ask First

- Renaming inventory fields, changing the CLI text shape, or extending the unsupported list semantics.

## Verification

- `crates/morva-core/tests/language.rs::capability_inventory_matches_public_language_behavior` — per-item drift checks against the real parser/checker.
- `crates/morva-core/tests/simulation.rs::simulation_phases_match_the_capability_inventory` — phase parity with a real simulation report.
- `crates/morva-cli/tests/cli.rs::capabilities_prints_the_stable_inventory_without_reading_models` — stable text, determinism, exit codes.
