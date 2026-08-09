---
title: 'v0.1 最小语义闭环'
type: 'feature'
created: '2026-08-10'
status: 'done'
baseline_commit: '6e2509fc53d40a927418c17b6da5520953a82b46'
context:
  - '{project-root}/docs/language-design.md'
  - '{project-root}/docs/cli.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** 当前 Morva 只提取声明树，字段、参数、行为条款与表达式都会被跳过；语义检查无法验证引用，诊断只有字节偏移，`inspect` 也尚不存在。

**Approach:** 在保持 `Morva / morva / .morva` 和双 crate workspace 不变的前提下，建立覆盖 `system`、`entity`、`action`、`requires`、`effects`、`ensures`、`invariant` 的小型强类型 AST、解析与引用检查，并对现有 `enum` 只建模成员，使裸枚举值必须由表达式上下文中的枚举类型解析；`check`、`parse`、`inspect` 通过同一模型提供确定性输出。

## Boundaries & Constraints

**Always:** `.morva` 源码是 Source of Truth；行为须显式、确定、可测试；诊断携带稳定代码、span 及行列；沿用现有 workspace、命名和退出码；新增行为必须有测试；不引入无必要依赖。

**Ask First:** 改变现有示例含义、移除已支持构造、引入 parser/CLI 框架、破坏 CLI 输出或扩展本次语法集合。

**Never:** 不实现 `simulate`、AI review、代码生成、LSP、模块语义、完整表达式类型系统、形式化验证或格式化器；除解析现有枚举成员外不扩展本次语法；不为未来功能预建抽象；不重命名项目、命令或后缀。

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|---------------|----------------------------|----------------|
| 合法模型 | system 含 entity、action 及四类条款 | 完整 AST；三个 CLI 命令输出确定 | N/A |
| 语法错误 | 缺名称、分隔符或闭合符 | 拒绝输入 | 诊断含代码、行列及源码指示 |
| 重复或未知引用 | 重名声明/字段/参数，未知类型/路径，或未在期望枚举中声明的裸标识符 | `check` 失败 | 指向具体名称；上下文匹配的枚举成员合法 |
| 非法 effects | 左侧不是参数字段路径 | `check` 失败 | 报告不可写目标 |

</frozen-after-approval>

## Code Map

- `crates/morva-core/src/lib.rs` -- 公共 API；当前全部核心实现集中于此。
- `crates/morva-core/src/ast.rs` -- 强类型声明、枚举成员、表达式与 span 模型。
- `crates/morva-cli/src/main.rs` -- `check`、`parse` 命令实现及文本输出。
- `crates/morva-core/tests/`, `crates/morva-cli/tests/` -- 新增公开 API 和 CLI 集成测试。
- `examples/order.morva` -- 现有名称与语法兼容基线。
- `README.md`, `docs/` -- 当前承诺与实现状态。

## Tasks & Acceptance

**Execution:**
- [x] `crates/morva-core/src/` -- 拆成最少必要模块，定义带 span 的声明、枚举成员、表达式、token 和 diagnostic。
- [x] `crates/morva-core/src/lexer.rs`, `parser.rs` -- 解析核心语法及现有 enum；兼容无参 `action Name {}`；仅白名单跳过已声明的软行为项；拒绝拼错条款、缺块容器、保留字名称、不完整输入和越界整数。
- [x] `crates/morva-core/src/semantic.rs` -- 检查单一且仅顶层的 system、重名、歧义短类型名、类型/字段路径、上下文枚举成员及 effect 写目标；不实现模块查找规则。
- [x] `crates/morva-cli/src/main.rs` -- 保留行为与退出码，新增 `inspect`，安全且精确地渲染含 tab/控制字符的源码位置诊断。
- [x] `crates/morva-core/tests/language.rs`, `crates/morva-cli/tests/cli.rs` -- 覆盖矩阵、现有示例及审查发现的 parser/diagnostic 回归路径。
- [x] `README.md`, `docs/` -- 只记录已落地能力及枚举值解析边界。

**Acceptance Criteria:**
- Given `examples/order.morva`，when 分别执行 `morva check`、`morva parse` 和 `morva inspect`，then 三者成功且输出反映实际字段、参数与条款。
- Given `order.status == Pending` 或 `order.status = Confirmed`，when 对应成员属于 `OrderStatus`，then 检查成功；若拼错或属于其他 enum，then 返回未知枚举成员诊断。
- Given 拼错 action 条款、缺块兼容容器、嵌套 system 或歧义短类型名，when 执行 `check`，then 稳定失败而非丢弃源码或依赖声明顺序。
- Given 任一语法或语义错误，when 执行 `check`，then 返回 1 并输出带稳定代码和行列的诊断；用法错误仍返回 2。
- Given workspace 全部改动，when 执行标准质量命令，then formatting、clippy 和所有测试均通过且无 warning。

## Spec Change Log

- 2026-08-10 intent clarification：审查发现裸标识符既可能是未知路径，也可能是现有示例中的枚举值。用户选择最小建模 enum members；规格加入上下文枚举解析，避免把所有未绑定名称当符号值。KEEP：强类型带 span AST、无第三方依赖、兼容现有示例、确定性 CLI 与稳定诊断。

## Design Notes

只实现首批构造所需表达式。裸标识符若不绑定字段/参数，只能在比较或赋值的期望类型明确为 enum 时解析为该 enum 的成员；无上下文或成员不存在即报错，不增加通用类型推断。兼容容器内同名类型若会造成短名歧义则拒绝，而非实现模块作用域。语义节点使用枚举/结构体；span 保存字节范围，渲染时计算行列。`inspect` 输出文本摘要，不承诺 JSON 协议。

## Verification

**Commands:**
- `cargo fmt --check` -- 全 workspace 格式正确。
- `cargo clippy --workspace --all-targets -- -D warnings` -- 所有 target 无 lint warning。
- `cargo test --workspace` -- 核心单测、公开 API 测试与 CLI 集成测试全部通过。
- `cargo run -p morva-cli -- check examples/order.morva` -- 示例通过语义检查。
- `cargo run -p morva-cli -- parse examples/order.morva` -- 输出完整声明及行为结构。
- `cargo run -p morva-cli -- inspect examples/order.morva` -- 输出稳定语义摘要。

## Suggested Review Order

**核心入口与语义边界**

- 公共 API 保持 parse/check 两步闭环。
  [`lib.rs:14`](../../crates/morva-core/src/lib.rs#L14)

- 单一 system、全局歧义类型和递归检查从这里汇合。
  [`semantic.rs:46`](../../crates/morva-core/src/semantic.rs#L46)

- 裸标识符只在明确 enum 上下文中解析成员。
  [`semantic.rs:577`](../../crates/morva-core/src/semantic.rs#L577)

- effects 目标提供赋值值所需的 enum 类型上下文。
  [`semantic.rs:608`](../../crates/morva-core/src/semantic.rs#L608)

**语法模型与兼容边界**

- 声明分派只为首批构造和 enum 建立强类型节点。
  [`parser.rs:98`](../../crates/morva-core/src/parser.rs#L98)

- action 只白名单兼容已声明软行为，拒绝未知内容。
  [`parser.rs:238`](../../crates/morva-core/src/parser.rs#L238)

- clause 支持单行或跨行块，同时保持表达式分隔明确。
  [`parser.rs:381`](../../crates/morva-core/src/parser.rs#L381)

- 兼容容器缺块时停止，不能吞掉下一声明。
  [`parser.rs:127`](../../crates/morva-core/src/parser.rs#L127)

**CLI 与诊断**

- 三条命令共享同一解析和语义检查路径。
  [`main.rs:71`](../../crates/morva-cli/src/main.rs#L71)

- 诊断安全渲染 span、tab、控制字符和文件路径。
  [`main.rs:99`](../../crates/morva-cli/src/main.rs#L99)

- inspect 提供确定性文本语义摘要。
  [`main.rs:331`](../../crates/morva-cli/src/main.rs#L331)

**回归证据**

- 核心测试覆盖 enum 上下文与引用失败路径。
  [`language.rs:85`](../../crates/morva-core/tests/language.rs#L85)

- parser 回归覆盖软行为、跨行块和缺块容器。
  [`language.rs:180`](../../crates/morva-core/tests/language.rs#L180)

- CLI 集成测试锁定输出、退出码及诊断位置。
  [`cli.rs:34`](../../crates/morva-cli/tests/cli.rs#L34)
