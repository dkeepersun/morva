# Morva

Morva is a compact, structured semantic language for describing software systems: requirements, business rules, flows, constraints, and architectural intent. It is aimed at people who already understand programming or architecture. Natural language is an AI-assisted input, not the language itself.

> Status: experimental v0.1 semantic core. Syntax may change.

## Why Morva?

Source code answers *how a system is implemented*. Morva aims to state *what must be true* precisely enough for people, analyzers, and AI implementation tools to inspect the same model.

The current v0.1 scope is intentionally small:

- strongly typed `system`, `entity`, `enum`, and `action` syntax models;
- fields, parameters, enum members, and `requires`, `effects`, `ensures`, and `invariant` clauses;
- duplicate-name, type, field-path, enum-member, and effect-target checks;
- deterministic `check`, `parse`, and `inspect` commands with source diagnostics;
- pure in-memory simulation for one-action scenarios using enum, Boolean, and Integer state;
- compatibility parsing for broader declarations and documented soft behavior already used by the example.

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

Example:

```morva
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
