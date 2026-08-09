# AI 开发交接协议

本文用于把 Morva 交给新的 AI 会话或编码代理。它不能替代源码和测试，但规定读取顺序与决策边界。

## 1. 必读顺序

1. `docs/project-context.md`：短小的强制工作规则。
2. `docs/requirements.md`：产品目标、v0.1 需求、非目标和变更门槛。
3. `docs/implementation-status.md`：已实现、兼容、未实现和技术债务。
4. `docs/architecture.md`：模块职责和依赖方向。
5. 与任务相关的语言/CLI/测试文档。
6. `_bmad-output/implementation-artifacts/` 中相关冻结规格。
7. 实际源码、测试和 `examples/order.morva`。

如果文档冲突，优先级为：人工最新明确指令 → 冻结规格中的 human-owned intent → 自动化测试与源码事实 → 需求基线 → 其他说明文档 → roadmap。

## 2. 开始任务前

- 检查 Git 状态、当前分支和现有未提交改动。
- 用源码和测试验证任务所依赖的当前行为，不凭文档标题推断。
- 明确本次是诊断、设计还是实现；诊断请求不自动授权修改。
- 判断是否触发人工确认门槛，特别是语法扩张、命名、CLI 契约、依赖和模拟边界。
- 把任务收缩到一个小而可验收的增量。

## 3. 实现规则

- `.morva` 源码是 Source of Truth；自然语言和 AI 输出是候选输入。
- 不把兼容容器或软行为项当作强类型语义。
- 不在 CLI 复制 parser/checker/simulator 规则。
- 不宣称完整类型安全、形式化验证或代码生成，除非对应实现与测试真实存在。
- 不为未来 LSP、插件、MCP、AI review 或 codegen 预建抽象。
- 新行为必须包含测试、文档和完整质量验证。
- 保留稳定 diagnostic code、span、CLI 退出码与现有示例含义。

## 4. 交付格式

每次交付应报告：

- 实际改动文件和用户可见行为；
- 新增/修改测试覆盖什么；
- 运行过的格式、lint、测试和示例结果；
- 未解决风险和明确未实现内容；
- 下一步最小建议，但不擅自开始未授权阶段。

## 5. 禁止的漂移模式

- 因 parser 接受某段文本就声称该能力已支持。
- 用 `dict`/JSON 临时模型绕开现有 AST。
- 在 simulator 中猜测 checker 没有定义的语言规则。
- 把 `implementation_hint` 当执行指令。
- 一次加入多种声明、完整表达式系统或通用 runtime。
- 修改冻结规格的 intent/boundaries/checkbox 来匹配实现。
- 为让测试通过而放宽未知项、歧义或未初始化读取。

## 6. 可直接复制的新会话提示

```text
请先读取 docs/index.md 和 docs/project-context.md，再按 docs/ai-handoff.md 的优先级核对需求、冻结规格、源码与测试。保持 Morva/morva/.morva 命名，结构化源码是 Source of Truth，核心语义只在 morva-core，优先简单、准确、低复杂度。不要把兼容解析当成已支持语义，不要扩张未批准范围。所有实现必须补测试并通过 fmt、严格 clippy、workspace tests 和相关示例闭环。交付时列出实际改动、验证结果、已知限制和下一最小步骤。
```
