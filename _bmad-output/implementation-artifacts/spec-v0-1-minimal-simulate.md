---
title: 'v0.1 最小场景模拟'
type: 'feature'
created: '2026-08-10'
status: 'done'
baseline_commit: 'abde484041e986ef68ba6e463c241356e862920d'
context:
  - '{project-root}/docs/language-design.md'
  - '{project-root}/docs/cli.md'
  - '{project-root}/_bmad-output/implementation-artifacts/spec-v0-1-minimal-semantic-core.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Morva 已能验证 action 的结构和引用，但 `scenario` 仍是 opaque 兼容容器，无法用显式初始状态验证 requires、effects、ensures、invariant 和期望结果。

**Approach:** 将现有 `given → run → expect` 场景提升为强类型 AST，以单个 action 和纯内存状态执行确定性的模型级模拟，并提供 `morva simulate <file> <scenario>`；模拟只解释已建模语义，不运行实现代码。

## Boundaries & Constraints

**Always:** 每个 scenario 按 `given* → 恰好一个 run → expect+` 排列；run 实参按位置绑定 action 的 entity 参数且名称互异；given 只能用 `=` 初始化实参字段；值仅为 enum member、Boolean、Integer；所有读取必须已初始化；按源码顺序执行并输出阶段、状态变化与稳定诊断；复用现有 span、表达式和语义模型；无新依赖。

**Ask First:** 支持多 action、标量 action 参数、Decimal/String/ID 值、跨 scenario 状态、别名实参、改变既有 scenario 含义或公开 CLI/退出码。

**Never:** 不实现 flow/lifecycle 模拟、调用链、事件、并发、时间、重试、存储、网络、应用代码、脚本语言、通用运行时或完整静态类型系统；不执行 `implementation_hint` 和其他软行为项。

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|---------------|----------------------------|----------------|
| 成功转换 | `NormalConfirmation` 初始化 Pending，运行 Confirm | 显示 Pending → Confirmed，各阶段 PASS，退出 0 | N/A |
| 前置失败 | given 状态使 requires 或初始 invariant 为 false | effects 不执行，指出失败阶段，退出 1 | 保留初始状态与表达式 span |
| 后置失败 | effect 后 invariant、ensures 或 expect 为 false | 显示已发生变化及失败阶段，退出 1 | 不访问或提交外部状态 |
| 非法场景 | 缺/多 run、无 expect、未知/歧义 action、arity 错误、非 entity 参数 | `check` 失败 | 稳定 diagnostic 指向 scenario 项 |
| 状态错误 | 非法 given 目标/值、未知字段或 enum、未初始化读取、算术溢出 | 模拟失败 | 指出路径、阶段及原因，不 panic |

</frozen-after-approval>

## Code Map

- `crates/morva-core/src/ast.rs` -- 当前 declaration、expression、assignment；新增 Scenario/Run 节点。
- `crates/morva-core/src/parser.rs` -- 当前 scenario 作为兼容容器；改为解析 given/run/expect。
- `crates/morva-core/src/semantic.rs` -- 复用 action、entity、enum 和路径解析，增加场景结构/绑定检查。
- `crates/morva-core/src/simulate.rs` -- 新增小型值、内存状态、阶段结果和 evaluator。
- `crates/morva-cli/src/main.rs` -- 新增 simulate 路由、确定性报告和既有诊断渲染。
- `examples/order.morva` -- 首个端到端成功场景。

## Tasks & Acceptance

**Execution:**
- [x] `crates/morva-core/src/ast.rs`, `parser.rs` -- 建模并严格解析 given/run/expect，保留源码顺序和 span。
- [x] `crates/morva-core/src/semantic.rs` -- 验证场景结构、全局唯一 action/scenario、entity 参数绑定、given/expect 路径与 enum 成员；拒绝不支持的参数和值。
- [x] `crates/morva-core/src/simulate.rs`, `lib.rs` -- 实现 `Value`、扁平内存状态、表达式求值、顺序 effects、阶段失败和结构化报告；整数 `+=/-=` 使用 checked arithmetic。
- [x] `crates/morva-cli/src/main.rs` -- 实现 `simulate <file> <scenario>`，成功/模型失败返回 0/1，用法和文件错误保持 2。
- [x] `crates/morva-core/tests/simulation.rs`, `crates/morva-cli/tests/cli.rs` -- 覆盖矩阵、每个失败阶段、未初始化读取、溢出、选择错误、输出和退出码。
- [x] `README.md`, `docs/language-design.md`, `docs/cli.md`, `docs/roadmap.md` -- 只记录已落地的单 action scenario 模拟及延期边界。

**Acceptance Criteria:**
- Given `examples/order.morva`，when 执行 `morva simulate examples/order.morva NormalConfirmation`，then 输出 Confirm、`order.status: Pending -> Confirmed`、全部 PASS 并返回 0。
- Given requires 或初始 invariant 为 false，when 模拟，then 不执行 effects，返回 1 并报告准确阶段和 span。
- Given effects 已执行但最终 invariant、ensures 或 expect 为 false，when 模拟，then 返回 1，报告变化后的内存状态和失败阶段。
- Given 未初始化读取、错误 enum、整数复合赋值溢出或不支持值，when 模拟，then 稳定失败且不 panic。
- Given workspace 变更，when 执行标准质量命令，then formatting、严格 clippy 和全部测试通过。

## Spec Change Log

## Design Notes

状态使用按路径排序的映射，例如 `order.status → Enum(OrderStatus.Pending)`；不构造对象或持久化。run 实参从 action 参数获得 entity 类型，使此前出现的 given 也可在完整 scenario 解析后验证。执行顺序固定为：应用 givens、绑定 entity 初始 invariants、requires（含 action invariant）、顺序 effects、最终 action/entity invariants、ensures、expects。`=` 支持三种值；`+=/-=` 只支持 Integer 并检查溢出。失败 report 保留截至失败点的内存状态。

## Verification

**Commands:**
- `cargo fmt --check` -- workspace 格式正确。
- `cargo clippy --workspace --all-targets -- -D warnings` -- 所有 target 无 warning。
- `cargo test --workspace` -- 核心、模拟与 CLI 测试全部通过。
- `cargo run -p morva-cli -- check examples/order.morva` -- 场景静态检查通过。
- `cargo run -p morva-cli -- simulate examples/order.morva NormalConfirmation` -- 输出 Pending → Confirmed 和 PASS。

## Suggested Review Order

**模拟入口与执行语义**

- 单一入口验证模型后按七个阶段执行场景。
  [`simulate.rs:110`](../../crates/morva-core/src/simulate.rs#L110)

- 表达式求值限制 equality 和有序比较的运行时类型。
  [`simulate.rs:577`](../../crates/morva-core/src/simulate.rs#L577)

- effect 写入前统一守卫目标字段类型。
  [`simulate.rs:728`](../../crates/morva-core/src/simulate.rs#L728)

**Scenario 语言与静态约束**

- AST 只建模 Given、Run、Expect 三种场景项。
  [`ast.rs:134`](../../crates/morva-core/src/ast.rs#L134)

- parser 严格保留场景项源码顺序和位置。
  [`parser.rs:316`](../../crates/morva-core/src/parser.rs#L316)

- semantic 固定结构、action 绑定与 entity 参数边界。
  [`semantic.rs:231`](../../crates/morva-core/src/semantic.rs#L231)

- given 只允许受支持字段和值初始化。
  [`semantic.rs:375`](../../crates/morva-core/src/semantic.rs#L375)

**CLI 与报告**

- CLI 在既有 parse/check 后调用模拟器并保持退出码。
  [`main.rs:29`](../../crates/morva-cli/src/main.rs#L29)

- 报告确定性输出阶段、变化、状态和最终结果。
  [`main.rs:433`](../../crates/morva-cli/src/main.rs#L433)

**回归证据**

- 端到端测试锁定 order 示例的 Pending → Confirmed。
  [`simulation.rs:19`](../../crates/morva-core/tests/simulation.rs#L19)

- 阶段测试证明前置失败不执行 effects，后置失败保留变化。
  [`simulation.rs:36`](../../crates/morva-core/tests/simulation.rs#L36)

- 审查回归覆盖重复 given、类型写入和 enum invariant。
  [`simulation.rs:60`](../../crates/morva-core/tests/simulation.rs#L60)

- CLI 测试锁定成功、失败 span、选择错误和退出码。
  [`cli.rs:120`](../../crates/morva-cli/tests/cli.rs#L120)
