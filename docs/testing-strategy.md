# 测试与质量策略

## 1. 质量门槛

每次交付必须通过：

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

涉及端到端语言行为时还必须运行：

```sh
cargo run -p morva-cli -- check examples/order.morva
cargo run -p morva-cli -- parse examples/order.morva
cargo run -p morva-cli -- inspect examples/order.morva
cargo run -p morva-cli -- simulate examples/order.morva NormalConfirmation
```

## 2. 测试层级

| 层级 | 位置 | 目标 |
|---|---|---|
| Core language integration | `crates/morva-core/tests/language.rs` | AST、语法、静态诊断和回归边界 |
| Simulation integration | `crates/morva-core/tests/simulation.rs` | 阶段顺序、状态变化、失败与公开 API |
| CLI process integration | `crates/morva-cli/tests/cli.rs` | stdout/stderr、位置、退出码、文件与用法错误 |
| Executable example | `examples/order.morva` | 最小闭环的人工可读基线 |

## 3. 新语义的最低测试矩阵

每增加一种语法或语义，至少覆盖：

1. 合法输入能解析且 AST 与预期一致。
2. 语法错误在准确 span 失败，不 panic、不吞掉后续声明。
3. 引用或类型错误产生稳定 code 和可行动消息。
4. CLI 映射到正确退出码并安全渲染输入。
5. 与已有示例和兼容边界不冲突。

诊断呈现变更还必须通过真实 CLI 进程覆盖：超长行的起点/中间/末尾/EOF、159/160/161 渲染宽度、左上下文 72/73 宽度、长或跨行 span、tab/control/non-ASCII 转义片段、CRLF 与 EOF 前孤立 CR、模拟失败，以及成功/读取失败/模型失败/运行时失败中的 UTF-8 路径控制字符。源码 excerpt 和 marker 的 160 字符上限应分别断言；这不等同于总 stderr 上限。CR-only 仍是独立语言契约，不得由呈现测试隐式定义。

可执行语义还必须覆盖：成功、每个失败阶段、未初始化状态、顺序性、溢出/边界值、类型不匹配和失败后的状态报告。

## 4. 回归原则

- 修复 bug 时先添加能复现问题的失败测试。
- 不只断言“有错误”，应尽量锁定 code、阶段、关键消息和 span。
- 输出契约测试应锁定必要稳定部分，避免对无承诺格式过度耦合。
- 不以 snapshot 掩盖语义断言；关键 AST 和 report 字段应直接断言。
- 测试夹具不得执行网络、外部存储或用户代码。

## 5. 评审检查

- 需求是否已有对应测试，测试是否能证明而非仅运行代码。
- Parser 是否误把拼写错误当兼容文本跳过。
- Semantic checker 是否在 CLI 和 simulator 之间出现重复或不一致规则。
- 不可信输入、整数边界、空集合、重复名称和歧义是否安全。
- 诊断 code、CLI 退出码和已有示例是否保持兼容。
- 文档是否把 runtime guard 错写为静态保证。
