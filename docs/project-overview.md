# Morva 项目概览

Morva 是一个实验性高层结构化语义语言，用于描述软件系统的实体、动作、规则、状态变化和架构意图。目标用户已具备编程或架构经验；自然语言只作为 AI 辅助入口，经过人工审核的 `.morva` 源码才是 Source of Truth。

## 快速事实

| 项目 | 当前事实 |
|---|---|
| 版本 | 0.1.0，实验阶段 |
| 语言 | Rust 2024 edition |
| 结构 | 单一产品、双 crate Cargo workspace |
| 核心 | `morva-core`：lexer/parser/AST/checker/simulator |
| 用户入口 | `morva` CLI |
| 外部依赖 | 无 |
| 当前闭环 | source(s) → per-file parse → project assembly → AST → check → diagnostics → inspect/simulate |
| 当前模拟 | 单 action、纯内存、enum/Boolean/Integer |

## 当前价值

Morva 已能让同一份设计模型被人阅读、被静态检查、被结构化检查和进行受限场景模拟。它还不是完整规格语言、形式化验证器或代码生成器。

## 文档入口

- 产品需求与边界：[requirements.md](requirements.md)
- 当前实现与风险：[implementation-status.md](implementation-status.md)
- 架构：[architecture.md](architecture.md)
- 语言参考：[language-reference.md](language-reference.md)
- AI 交接：[ai-handoff.md](ai-handoff.md)
- 开发和测试：[development-guide.md](development-guide.md)、[testing-strategy.md](testing-strategy.md)
