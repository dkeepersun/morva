# Morva 实现状态

最后核对：2026-08-10，基于提交 `ae90f8d` 之后的工作树。

## 已实现且有自动化测试

- Rust workspace：`morva-core` + `morva-cli`。
- 带字节 span 的 lexer、parser、强类型 AST 和 diagnostic。
- `system`、`entity`、`enum`、`action`、`scenario` 核心模型。
- 字段、参数、enum member、四类行为条款和有限表达式。
- 单一顶层 system、重复名称、未知/歧义类型、路径、enum member、effect target 检查。
- Canonical builtin、Boolean 谓词、比较操作数及 effect 值类型检查；解析失败后抑制派生类型诊断。
- Scenario 结构、action 选择、arity、entity 参数绑定和 given 值检查。
- `check`、`parse`、`inspect`、`simulate` CLI 与稳定退出码。
- 单 action、纯内存、enum/Boolean/Integer 状态模拟及七个阶段。
- 72 个自动化测试：62 个 core 语法/语义/模拟公开 API 集成测试、2 个 core runtime guard 单元测试、8 个 CLI 进程级测试。

## 兼容解析但没有专有语义

- `module`、`service`、`event`、`flow`、`lifecycle`、`policy`。
- `atomic`、`idempotent`、`timeout`、`retry`、`implementation_hint`。

这些构造不得出现在“已验证能力”清单中。修改兼容行为前应先保护已有示例。

## 已知限制与风险

### 静态类型检查边界

当前 checker 已区分 canonical builtin family，并检查现有谓词、比较和 effect 类型。它仍不是完整类型系统：没有通用推断或转换、逻辑/算术表达式、Entity 整体值、数据流分析或形式化证明。Decimal 上下文只接受非负 Integer 字面量作为精确常量；受限 simulator 仍只执行 enum、Boolean 和 Integer 值，并保留运行时守卫。

### 词法与诊断边界

- 标识符只支持 ASCII。
- 没有负整数字面量；负值只可能由模拟中的 `-=` 产生。
- CR-only 换行未作为单独语言契约验证。
- CLI 当前会渲染完整源码行，极端超长单行需要后续增加窗口上限。

### 作用域与协议

- 没有模块作用域或限定名，同名短类型直接报歧义。
- `parse`/`inspect` 是稳定文本而非 JSON 协议。
- 没有增量分析、formatter、Tree-sitter 或 LSP。

## 已批准并完成的规格

- `_bmad-output/implementation-artifacts/spec-v0-1-minimal-semantic-core.md`
- `_bmad-output/implementation-artifacts/spec-v0-1-minimal-simulate.md`
- `_bmad-output/implementation-artifacts/spec-minimal-static-expression-types.md`

冻结块记录人类批准意图，其他 AI 不得自行修改其边界或把未完成方向标为已完成。

## 下一阶段候选（尚未批准实现）

优先建议先处理诊断资源边界，再考虑 AI `grill/challenge/review`。Tree-sitter、LSP、图导出和 flow/lifecycle 模拟只能按真实用例独立立项。

Roadmap 中的状态转换与明显条款矛盾检查也属于后续候选，不阻塞上述已批准规格的完成状态。

任何候选进入开发前都需要：问题陈述、边界、I/O 矩阵、验收标准、测试计划和人工批准。
