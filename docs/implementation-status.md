# Morva 实现状态

最后核对：2026-08-10。当前能力以下列已完成规格、对应源码/公开测试和实际质量门禁结果为证据；提交锚定的历史数字不代表当前总量。

## 已实现且有自动化测试

- Rust workspace：`morva-core` + `morva-cli`。
- 带字节 span 的 lexer、parser、强类型 AST 和 diagnostic。
- `system`、`entity`、`enum`、`action`、`scenario` 核心模型。
- 字段、参数、enum member、四类行为条款和有限表达式。
- 单一顶层 system、重复名称、未知/歧义类型、路径、enum member、effect target 检查。
- Canonical builtin、Boolean 谓词、比较操作数及 effect 值类型检查；解析失败后抑制派生类型诊断。
- Action 前/后态的有限精确字面量事实一致性检查，以及顺序 effects 最终已知直接赋值与后置约束检查（`MORVA2018/2019`）。
- Scenario 结构、action 选择、arity、entity 参数绑定和 given 值检查。
- `check`、`parse`、`inspect`、`simulate` CLI 与稳定退出码。
- 四个命令接受单文件或平铺项目目录；core 以同名 system 装配跨文件声明，用 source map 将语法、语义和模拟失败定位回原文件本地 span。
- GitHub Actions 质量门禁：格式、严格 Clippy、职责分片测试、四命令示例与最终汇总检查。工作流已推送并由 GitHub 托管运行成功（最近核对为 `bc81fac`，2026-08-11）；`Quality gate` 尚未设为 branch protection 必需检查。
- 单 action、纯内存、enum/Boolean/Integer 状态模拟及七个阶段。
- 自动化测试覆盖 core 语法、语义、模拟公开 API、runtime guard 与 CLI 进程级契约；当前总数以 `cargo test --workspace --locked` 的实际结果为准。
- LF、CRLF、CR 统一作为单个逻辑换行；注释、parser 分隔、原源 byte span 与 CLI 混合换行行列保持一致。
- 保留 `//` 行注释（含 EOF 终止），并支持 token 间可嵌套的 `/* ... */` 块注释；块内逻辑换行仍分隔语法，未闭合时以 `MORVA1024` 标记最外层 opener。无换行块注释在任何 lexer 内容中切分 identifier/Integer 时以 `MORVA1025` 标记触发 opener。
- 普通诊断和模拟失败共享有界源码窗口；excerpt 与 marker 分别不超过 160 个渲染字符，换行终止符不回显，所有 UTF-8 路径输出安全转义控制字符。
- Additive core analysis report 为每个兼容容器生成结构化 `MORVA5001`，并为每个 action soft behavior 生成结构化 `MORVA5002`；Project 将 keyword span 单次回映到责任源，CLI `check` 安全渲染 warning 且 warning-only 仍退出 0，旧 `check()` 与 simulator 保持 error-only。
- CLI `inspect` 消费同一 analysis notices，在存在未建模内容时追加确定的 `unmodeled:` 摘要：容器 kind/name 与 action/soft behavior 按源码顺序（项目按文件顺序）列出，只显示结构化名称、不回显正文；无未建模内容时不输出摘要，重复运行 byte-identical。
- Core 提供版本化 `capabilities()` 能力清单（v1）：语义声明、clause、表达式形态、操作符、字面量、builtin 类型与别名、模拟值类型与七阶段、兼容容器、soft behavior 和明确不支持类别；容器/soft behavior/操作符/阶段直接复用 parser、AST 与 simulator 的同一常量，公开测试逐项验证清单与实际解析、检查行为一致。`morva capabilities` 命令不读文件、输出确定文本并退出 0。

## 兼容解析但没有专有语义

- `module`、`service`、`event`、`flow`、`lifecycle`、`policy`。
- `atomic`、`idempotent`、`timeout`、`retry`、`implementation_hint`：AST 只保留 kind 与 keyword span provenance；payload/body 不保留、不验证、不执行，但显式 analysis 会报告 `MORVA5002`。

这些构造不得出现在“已验证能力”清单中。修改兼容行为前应先保护已有示例。

## 已知限制与风险

### 静态类型检查边界

当前 checker 已区分 canonical builtin family，并检查现有谓词、比较和 effect 类型。它仍不是完整类型系统：没有通用推断或转换、逻辑/算术表达式、Entity 整体值、数据流分析或形式化证明。Decimal 上下文只接受非负 Integer 字面量作为精确常量；受限 simulator 仍只执行 enum、Boolean 和 Integer 值，并保留运行时守卫。

明显矛盾检查只处理已解析的 Boolean、Integer、enum 与 Decimal-context Integer 精确字面量事实。它不做有序区间求解、entity invariant 参数实例化、scenario/action 内联、未写路径跨阶段推理或 compound effect 折叠；非字面量写入和 compound effect 会把相应最终值降为 Unknown。

### 词法与诊断边界

- 标识符只支持 ASCII。
- 没有负整数字面量；负值只可能由模拟中的 `-=` 产生。
- 有界窗口不是总 stderr 上限；诊断 message 和安全转义后的路径保持完整。

### 作用域与协议

- 没有模块作用域或限定名，同名短类型直接报歧义。
- 多文件项目不递归、不支持 manifest/import/container reopening；目录发现仅限直接子级小写 `.morva` 普通文件。
- 项目读取拒绝 symlink、目录外 canonical target 和发现后身份变化；Unix 以 device/inode 复核打开句柄。标准库无法跨平台提供原子 `nofollow`，非 Unix 身份校验较弱，且并发原地改写同一文件不承诺快照隔离。
- `parse`/`inspect` 是稳定文本而非 JSON 协议。
- 没有增量分析、formatter、Tree-sitter 或 LSP。

## 已批准并完成的规格

- `_bmad-output/implementation-artifacts/spec-v0-1-minimal-semantic-core.md`
- `_bmad-output/implementation-artifacts/spec-v0-1-minimal-simulate.md`
- `_bmad-output/implementation-artifacts/spec-minimal-static-expression-types.md`
- `_bmad-output/implementation-artifacts/spec-bounded-diagnostic-rendering.md`
- `_bmad-output/implementation-artifacts/spec-universal-newline-contract.md`
- `_bmad-output/implementation-artifacts/spec-obvious-transition-contradictions.md`

冻结块记录人类批准意图，其他 AI 不得自行修改其边界或把未完成方向标为已完成。

## 下一阶段候选（尚未批准实现）

可考虑 AI `grill/challenge/review`。Tree-sitter、LSP、图导出和 flow/lifecycle 模拟只能按真实用例独立立项。

任何候选进入开发前都需要：问题陈述、边界、I/O 矩阵、验收标准、测试计划和人工批准。
