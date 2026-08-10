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
cargo run -p morva-cli -- check examples/order-project
cargo run -p morva-cli -- parse examples/order-project
cargo run -p morva-cli -- inspect examples/order-project
cargo run -p morva-cli -- simulate examples/order-project NormalConfirmation
```

## 2. 测试层级

| 层级 | 位置 | 目标 |
|---|---|---|
| Core language integration | `crates/morva-core/tests/language.rs` | AST、语法、静态诊断和回归边界 |
| Core project integration | `crates/morva-core/tests/project.rs` | 多文件装配、跨文件引用、source map、模拟与单文件 API parity |
| Simulation integration | `crates/morva-core/tests/simulation.rs` | 阶段顺序、状态变化、失败与公开 API |
| CLI process integration | `crates/morva-cli/tests/cli.rs` | stdout/stderr、位置、退出码、文件与用法错误 |
| Executable examples | `examples/order.morva`, `examples/order-project/` | 单文件与平铺多文件最小闭环基线 |

## 3. 新语义的最低测试矩阵

每增加一种语法或语义，至少覆盖：

1. 合法输入能解析且 AST 与预期一致。
2. 语法错误在准确 span 失败，不 panic、不吞掉后续声明。
3. 引用或类型错误产生稳定 code 和可行动消息。
4. CLI 映射到正确退出码并安全渲染输入。
5. 与已有示例和兼容边界不冲突。

诊断呈现变更还必须通过真实 CLI 进程覆盖：超长行的起点/中间/末尾/EOF、159/160/161 渲染宽度、左上下文 72/73 宽度、长或跨行 span、tab/control/non-ASCII 转义片段、模拟失败，以及成功/读取失败/模型失败/运行时失败中的 UTF-8 路径控制字符。源码 excerpt 和 marker 的 160 字符上限应分别断言；这不等同于总 stderr 上限。

换行契约必须通过公开 core parse/check 和真实 CLI 进程分别覆盖：等价 LF/CRLF/CR 模型、原源 byte span、三种注释终止、混合序列、CRLF 单计数，以及换行后 EOF 的行列、空 excerpt 和 caret。

注释契约必须覆盖：`//` 的显式 EOF/三种换行终止、token 间块注释、跨行与嵌套块、行/块模式隔离、块内 LF/CRLF/CR Newline、混合换行后的原始 byte span；typed 和 parser-skipped 内容中的 identifier、Integer 及六种复合 operator 切分、连续 comment run、不会合并 token 或含换行的反例必须验证 `MORVA1025` code/message/span。outer/inner 均未闭合时只返回 `MORVA1024`，单/多文件 CLI 都须精确标记责任文件的最外层 `/*` 两个 byte。四命令测试还应比较注释模型与“正文为空白、内部换行保留”模型的 parse/inspect 输出及七阶段模拟最终状态，而非只断言 success。

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
