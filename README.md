# Morva

Morva is a compact, structured semantic language for describing software systems: requirements, business rules, flows, constraints, and architectural intent. It is aimed at people who already understand programming or architecture. Natural language is an AI-assisted input, not the language itself.

> Status: experimental v0.1 skeleton. Syntax will change.

## Why Morva?

Source code answers *how a system is implemented*. Morva aims to state *what must be true* precisely enough for people, analyzers, and AI implementation tools to inspect the same model.

The v0.1 scope is intentionally small:

- structured declarations for systems, modules, data, behavior, flows, and scenarios;
- parsing and basic semantic checks;
- a future lightweight model-level `simulate`, not a production runtime;
- soft `implementation_hint` guidance;
- a stable semantic model that later tools such as AI review, LSP, Tree-sitter, diagrams, Skills, and MCP can consume.

Formal verification, code generation, infrastructure configuration, and a large keyword taxonomy are explicitly out of scope for v0.1.

## Quick start

```sh
cargo run -p morva-cli -- check examples/order.morva
cargo run -p morva-cli -- parse examples/order.morva
```

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

See [the language draft](docs/language-design.md), [CLI design](docs/cli.md), [roadmap](docs/roadmap.md), and [naming review](docs/naming.md).

## Repository layout

```text
crates/morva-core  minimal lexer, parser, AST, and checks
crates/morva-cli   command-line entry point
docs               language, CLI, and roadmap decisions
examples           small executable design examples
```

## License

MIT
