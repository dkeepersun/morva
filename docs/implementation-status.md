# Morva 实现状态

最后核对：2026-08-10，基于提交 `ae90f8d` 之后的工作树。

## 已实现且有自动化测试

- Rust workspace：`morva-core` + `morva-cli`。
- 带字节 span 的 lexer、parser、强类型 AST 和 diagnostic。
- `system`、`entity`、`enum`、`action`、`scenario` 核心模型。
- 字段、参数、enum member、四类行为条款和有限表达式。
- 单一顶层 system、重复名称、未知/歧义类型、路径、enum member、effect target 检查。
- Scenario 结构、action 选择、arity、entity 参数绑定和 given 值检查。
- `check`、`parse`、`inspect`、`simulate` CLI 与稳定退出码。
- 单 action、纯内存、enum/Boolean/Integer 状态模拟及七个阶段。
- 50 个集成测试：42 个 core 语法/语义/模拟公开 API 测试，8 个 CLI 进程级测试。

## 兼容解析但没有专有语义

- `module`、`service`、`event`、`flow`、`lifecycle`、`policy`。
- `atomic`、`idempotent`、`timeout`、`retry`、`implementation_hint`。

这些构造不得出现在“已验证能力”清单中。修改兼容行为前应先保护已有示例。

## 已知限制与风险

### 静态类型检查不完整

当前 checker 能解析类型和路径，但没有完整地区分所有 builtin 身份，也没有全面强制谓词结果为 Boolean、比较操作符适用性或普通 effect RHS 类型兼容。受限 simulator 在运行路径上有额外类型守卫，但这不能替代完整静态分析。

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

冻结块记录人类批准意图，其他 AI 不得自行修改其边界或把未完成方向标为已完成。

## 下一阶段候选（尚未批准实现）

优先建议先补强静态表达式类型模型和诊断资源边界，再考虑 AI `grill/challenge/review`。Tree-sitter、LSP、图导出和 flow/lifecycle 模拟只能按真实用例独立立项。

Roadmap 中的状态转换与明显条款矛盾检查也属于后续候选，不阻塞上述两份已批准 v0.1 规格的完成状态。

任何候选进入开发前都需要：问题陈述、边界、I/O 矩阵、验收标准、测试计划和人工批准。
