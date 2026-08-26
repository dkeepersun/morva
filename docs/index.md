# Morva 项目文档索引

这是人类与 AI 进入 Morva 项目的主入口。

## 项目速览

- 类型：单一产品、四 crate Rust workspace（core library + 共享机器输出 + CLI + 只读 MCP 服务器）
- 主语言：Rust 2024 edition
- 架构：lexer → parser → typed AST → semantic check → CLI/simulator
- Source of Truth：经过人工审核的 `.morva` 源码
- 当前状态：v0.1 semantic core、多文件装配、Boolean 取反/析取与三态矛盾分析、四命令 `--format json`、checked-semantics 协议单文件切片与只读 MCP 服务器已完成；CI 由 GitHub 托管运行成功，branch protection 尚未启用

## 必读文档

1. [产品与需求基线](requirements.md) — 定位、功能/非功能需求、非目标、变更控制
2. [语言演进决策](language-evolution-policy.md) — 表达力阶梯、不可跨越的红线、`opaque` 逃生舱；约束后续每一份规格
3. [实现状态](implementation-status.md) — 已实现、仅兼容、已知风险和候选下一步
4. [架构](architecture.md) — crate/module 职责、数据流、约束和技术债务
5. [语言参考](language-reference.md) — 当前可依赖的具体语法和语义
6. [AI 开发交接协议](ai-handoff.md) — 读取优先级、工作方式和禁止漂移
7. [Project Context](project-context.md) — 给 AI 的短小强制规则

## 开发与验证

- [开发指南](development-guide.md)
- [测试与质量策略](testing-strategy.md)
- [CI 质量门禁](ci.md)
- [历史项目扫描快照](project-scan-report.json) — 数量与验证结果仅适用于其 `source_commit`
- [源码树与职责](source-tree-analysis.md)
- [CLI 设计与契约](cli.md)
- [MCP 集成](mcp.md)

## 产品设计与规划

- [项目概览](project-overview.md)
- [语言设计草案](language-design.md)
- [Roadmap](roadmap.md)
- [命名审查](naming.md)

## 已批准实现规格

- [v0.1 最小语义闭环](../_bmad-output/implementation-artifacts/spec-v0-1-minimal-semantic-core.md)
- [v0.1 最小场景模拟](../_bmad-output/implementation-artifacts/spec-v0-1-minimal-simulate.md)
- [最小静态表达式类型](../_bmad-output/implementation-artifacts/spec-minimal-static-expression-types.md)
- [有界诊断窗口与安全路径输出](../_bmad-output/implementation-artifacts/spec-bounded-diagnostic-rendering.md)
- [通用换行契约](../_bmad-output/implementation-artifacts/spec-universal-newline-contract.md)
- [明显状态转换矛盾](../_bmad-output/implementation-artifacts/spec-obvious-transition-contradictions.md)

规格中的 `<frozen-after-approval>` 内容是人工拥有的意图边界，其他 AI 不得为适配代码而自行修改。

## 快速开始

```sh
cargo run -p morva-cli -- check examples/order.morva
cargo run -p morva-cli -- parse examples/order.morva
cargo run -p morva-cli -- inspect examples/order.morva
cargo run -p morva-cli -- simulate examples/order.morva NormalConfirmation
```

## 文档可信度规则

文档发生冲突时，按 [AI 开发交接协议](ai-handoff.md) 的优先级处理。Roadmap 只表达方向；“支持”必须能由当前强类型 AST、公开 API 和自动化测试证明。
