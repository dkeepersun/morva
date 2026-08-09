---
title: 'v0.1 最小静态表达式类型检查'
type: 'feature'
created: '2026-08-10'
status: 'done'
baseline_commit: 'd71bb804401285ed6f1370d0b244b14bc33a11e3'
context:
  - '{project-root}/docs/project-context.md'
  - '{project-root}/docs/requirements.md'
  - '{project-root}/_bmad-output/implementation-artifacts/spec-v0-1-minimal-semantic-core.md'
  - '{project-root}/_bmad-output/implementation-artifacts/spec-v0-1-minimal-simulate.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** checker 把所有 builtin 折叠为同一类型，只检查引用和 enum member；非 Boolean 谓词、错误比较和 effect 类型不匹配只能在部分模拟路径失败，其他模型会被静态接受。

**Approach:** 不改变 grammar、AST 或 simulator 值域，为现有表达式增加 canonical 类型和确定性检查，把可证明的错误前移到 `check`，保留 runtime guards 作为纵深防御。

## Boundaries & Constraints

**Always:** 规范化 `Bool/Boolean`、`Int/Integer`、`ID/Id`；谓词必须为 Boolean；`==/!=` 要求同 canonical scalar 或同 enum；有序比较只接受 Integer 或 Decimal；明确 Decimal 上下文中的非负 Integer 字面量是精确常量，以保持 `balance: Decimal` 与 `balance >= 0` 合法；effect `=` 必须同型，`+=/-=` 仅接受 Integer；引用失败后抑制级联类型诊断，并保持源码诊断顺序。

**Ask First:** 改变现有示例、取消 Decimal 上下文整数常量、允许 Entity 整体比较/赋值、增加通用数值转换或改变 CLI 契约。

**Never:** 不新增语法、字面量、运算符、AST、通用推断、模块作用域、数据流分析、常量折叠或矛盾证明；不扩展 simulator 值域或删除 runtime guards；Entity 只能作为字段路径中间类型。

## I/O & Edge-Case Matrix

| Scenario | Valid | Invalid diagnostic |
|---|---|---|
| Predicate | Boolean literal/path、合法 binary | 非 Boolean → `MORVA2013` |
| Equality | 同 builtin family、同 enum/member | 异型或 Entity → `MORVA2014` |
| Ordered | Integer；Decimal/Decimal-context Integer literal | Boolean、enum、混合字段 → `MORVA2015` |
| Set effect | target 与 RHS 同型 | 不兼容终值 → `MORVA2016` |
| Compound effect | Integer target/RHS | 非 Integer → 单个 `MORVA2017` |
| Resolution failure | 既有 `MORVA2007/2009/2010/2012` | 不追加 `MORVA2013-2017` |

</frozen-after-approval>

## Code Map

- `crates/morva-core/src/semantic.rs` -- builtin identity、表达式类型、比较与 effect 规则。
- `crates/morva-core/src/simulate.rs` -- 保留现有 runtime 类型守卫。
- `crates/morva-core/tests/language.rs`, `simulation.rs` -- 静态矩阵和失败层级回归。
- `docs/language-reference.md`, `docs/implementation-status.md`, `docs/architecture.md`, `docs/project-context.md` -- 保证与限制。

## Tasks & Acceptance

**Execution:**
- [x] `semantic.rs` -- 实现 canonical builtin、表达式类型结果、五类诊断及级联抑制。
- [x] `language.rs` -- 先覆盖正常/失败、别名、Decimal、Entity、enum 上下文、message 和 span。
- [x] `simulation.rs` -- 调整静态前移后的公开 API 预期，保留 runtime guard 回归证据。
- [x] `README.md`, `docs/*.md` -- 记录静态保证、Decimal 上下文、Entity 边界和剩余限制。
- [x] workspace -- 通过格式、严格 Clippy、全部测试和四命令示例闭环。

**Acceptance Criteria:**
- Given builtin aliases，when 同型比较/赋值，then 无类型诊断。
- Given `Decimal balance >= 0` 或 `balance = 0`，when check，then 合法；Decimal path 与 Integer path 仍失败。
- Given 非 Boolean 条款或 expect，when check，then 在谓词 span 返回 `MORVA2013`。
- Given effect mismatch 或非 Integer 复合赋值，then 只返回 `MORVA2016` 或 `MORVA2017`。
- Given 引用/enum 错误，then 只保留原诊断；Given 现有合法 fixture，then API、CLI 和七阶段模拟不变。

## Spec Change Log

- 2026-08-10 review hardening：限定排序单个 scenario 新增诊断以保持源码顺序；为保留的 runtime equality/effect guards 增加单元回归，其中 equality 直接执行 `evaluate_binary` 的 RuntimeError 分支并锁定 message/span，effect 直接覆盖期望字段类型 guard；收敛 builtin 识别为 canonical resolver 单一事实源；补齐合法谓词、`!=`、Integer 复合 effect、Decimal 路径负例、诊断 message/span 和解析失败主诊断证据。README 保留既有引用与 effect target 能力表述。KEEP：不改变 AST、grammar、公开 API、simulator 值域或 runtime 实现。

## Design Notes

Integer 字面量只在明确 Decimal 操作数或目标下获得 Decimal 上下文；Integer path 与 Decimal path 不发生隐式转换。内部 invalid 类型传播负责抑制派生错误。

## Verification

**Commands:**
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo run -p morva-cli -- check examples/order.morva`
- `cargo run -p morva-cli -- parse examples/order.morva`
- `cargo run -p morva-cli -- inspect examples/order.morva`
- `cargo run -p morva-cli -- simulate examples/order.morva NormalConfirmation`

2026-08-10 review hardening 最终验证：formatting、严格 Clippy、72 个自动化测试（70 integration + 2 runtime guard unit，其中 equality 直接执行 evaluator 错误分支）及四命令示例闭环全部通过。

## Suggested Review Order

**类型模型与语义入口**

- Canonical builtin family 是全部兼容判断的单一事实源。
  [`semantic.rs:31`](../../crates/morva-core/src/semantic.rs#L31)

- 谓词比较集中生成稳定类型诊断并处理 Decimal 上下文。
  [`semantic.rs:670`](../../crates/morva-core/src/semantic.rs#L670)

- Effect 检查区分普通赋值与 Integer-only 复合赋值。
  [`semantic.rs:767`](../../crates/morva-core/src/semantic.rs#L767)

- Scenario 局部稳定排序保持新旧诊断源码顺序。
  [`semantic.rs:248`](../../crates/morva-core/src/semantic.rs#L248)

**防御边界**

- Runtime evaluator 守卫保持不变并获得直接分支测试。
  [`simulate.rs:833`](../../crates/morva-core/src/simulate.rs#L833)

- 静态失败通过公开 simulate seam 阻止进入执行阶段。
  [`simulation.rs:265`](../../crates/morva-core/tests/simulation.rs#L265)

**行为证据**

- 五类诊断的 message 与 span 从谓词矩阵开始锁定。
  [`language.rs:181`](../../crates/morva-core/tests/language.rs#L181)

- Decimal 上下文常量与路径不转换在此形成对照。
  [`language.rs:445`](../../crates/morva-core/tests/language.rs#L445)

- Scenario 多诊断按 span 单调排列，防止阶段式乱序。
  [`language.rs:576`](../../crates/morva-core/tests/language.rs#L576)

**用户契约**

- 语言参考精确定义 comparison、Decimal 与 Entity 边界。
  [`language-reference.md:111`](../../docs/language-reference.md#L111)

- 实现状态区分已落地静态保证与剩余类型限制。
  [`implementation-status.md:12`](../../docs/implementation-status.md#L12)

- AI 上下文保护 canonical family 与 simulator 分界。
  [`project-context.md:33`](../../docs/project-context.md#L33)
