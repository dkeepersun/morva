# Morva

Morva is a compact, structured semantic language for describing software systems: requirements, business rules, flows, constraints, and architectural intent. It is aimed at people who already understand programming or architecture. Natural language is an AI-assisted input, not the language itself.

> Status: experimental v0.1 semantic core. Syntax may change.

## Why Morva?

Source code answers *how a system is implemented*. Morva aims to state *what must be true* precisely enough for people, analyzers, and AI implementation tools to inspect the same model.

The current v0.1 scope is intentionally small:

- strongly typed `system`, `entity`, `enum`, `action`, and `scenario` syntax models;
- fields, parameters, enum members, and `requires`, `effects`, `ensures`, and `invariant` clauses;
- `//` line comments and nested `/* ... */` block comments whose newlines still separate syntax;
- duplicate-name, type, field-path, enum-member, effect-target, canonical expression type, comparison, and effect-value checks;
- deterministic `check`, `parse`, and `inspect` commands with source diagnostics;
- file-or-directory input, so one system can be split across independently maintained `.morva` files;
- pure in-memory simulation for one-action scenarios using enum, Boolean, and Integer state;
- compatibility parsing for broader declarations and documented soft behavior already used by the example;
- explicit non-fatal `check` warnings when compatibility containers or action soft behavior are parsed without corresponding validation or execution.

Formal verification, code generation, infrastructure configuration, and a large keyword taxonomy are explicitly out of scope for v0.1.

## Quick start

```sh
cargo run -p morva-cli -- check examples/order.morva
cargo run -p morva-cli -- parse examples/order.morva
cargo run -p morva-cli -- inspect examples/order.morva
cargo run -p morva-cli -- simulate examples/order.morva NormalConfirmation
```

All four commands parse and semantically check the model first. `parse` prints the
currently modeled AST surface; compatibility-only text that the parser deliberately
ignores is not reproduced.

`check` reports compatibility containers as `MORVA5001` warnings and each parsed
action soft behavior as `MORVA5002` on stderr while remaining successful when no
semantic errors exist. The bundled single-file example therefore emits three warnings:
one for `module Orders`, plus one each for `idempotent` and `implementation_hint`.
These warnings do not change its semantics or simulation.

Each command also accepts a directory. A project directory contains direct-child,
lowercase `.morva` regular files, each with the same `system` wrapper. Morva sorts
the filenames by UTF-8 bytes, merges only those systems' child declarations, and
reports errors against the original file and local line/column. Other extensions,
subdirectories, and symlinks are ignored.

```text
examples/order-project/
  10-types.morva
  20-actions.morva
  30-scenarios.morva
```

```sh
cargo run -p morva-cli -- check examples/order-project
cargo run -p morva-cli -- simulate examples/order-project NormalConfirmation
```

Example:

```morva
/* Reviewable design rationale can span lines.
   /* Nested details are supported. */
*/
system Shop {
  module Orders {
    entity Order {
      status: OrderStatus
    }

    action Confirm(order: Order) {
      requires order.status == Pending
      effects order.status = Confirmed
      ensures order.status == Confirmed

      implementation_hint {
        storage: relational
      }
    }
  }
}
```

Start with the [project documentation index](docs/index.md). The [requirements baseline](docs/requirements.md), [implementation status](docs/implementation-status.md), [architecture](docs/architecture.md), and [language reference](docs/language-reference.md) distinguish current guarantees from compatibility parsing and future plans.

## Repository layout

```text
crates/morva-core  spanned lexer, parser, AST, diagnostics, and semantic checks
crates/morva-cli   command-line entry point
docs               language, CLI, and roadmap decisions
examples           small executable design examples
```

## License

MIT
