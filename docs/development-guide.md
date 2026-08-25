# Morva 开发指南

## 前置条件

- 支持 Rust 2024 edition 的稳定 Rust toolchain。仓库根 `rust-toolchain.toml` 把质量门禁钉在一个具体版本（rustup 环境自动采用）；workspace `rust-version = "1.85"` 是经完整测试套件验证的 MSRV。升级钉住版本前须先在新版本上本地跑通 fmt、严格 Clippy 与全部测试，并在同一提交中更新钉住文件。
- Cargo。
- 不需要数据库、服务账号、环境变量或网络依赖。

## 常用命令

```sh
cargo build --workspace
cargo test --workspace
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
```

运行 CLI：

```sh
cargo run -p morva-cli -- check examples/order.morva
cargo run -p morva-cli -- simulate examples/order.morva NormalConfirmation
```

## 修改语言的推荐顺序

1. 在需求或独立实现规格中明确语义、非目标和兼容影响。
2. 添加失败测试和成功示例。
3. 更新 `ast.rs`，只建模本次所需结构。
4. 更新 lexer/parser，保持 span 和明确错误。
5. 更新 semantic checker，集中实现跨节点规则。
6. 如属已批准可执行语义，再更新 simulator。
7. 最后更新 CLI 呈现、语言参考和实现状态。
8. 执行完整质量门槛和示例闭环。

## 代码约定

- 语义模型使用 Rust enum/struct 和穷尽 match。
- 不用 `HashMap<String, unknown>` 一类弱结构替代 AST。
- Parser 只负责结构；名称/类型规则放在 semantic 层。
- CLI 只负责适配和呈现，不复制 core 规则。
- 错误必须返回 `Diagnostic` 或结构化 simulation failure，不对用户输入 `unwrap`。
- 默认不引入第三方依赖；需要引入时在规格中记录权衡。
- 保持确定性顺序；需要稳定输出的集合优先显式排序或使用 `BTreeMap`。

## 文档同步规则

- 当前行为变化：更新 `language-reference.md` 和 `implementation-status.md`。
- 产品边界变化：更新 `requirements.md`。
- 模块、依赖或执行流程变化：更新 `architecture.md`。
- CLI 契约变化：更新 `cli.md` 和 CLI 测试。
- 下一阶段方向变化：更新 `roadmap.md`，但不要用 roadmap 宣称已实现。
- AI 工作规则变化：更新 `project-context.md` 和 `ai-handoff.md`。

## Git 与交付

- 保留用户已有工作树改动，不进行破坏性重置。
- 一个提交应表达一个可验证意图，并包含相应测试和文档。
- 未经明确请求不 push、不发布 crate、不创建远端资源。

同一组质量命令已写入 [GitHub Actions CI](ci.md)，在 workflow 推送后才会按 push、pull request、手动和每周定时触发自动执行；当前远程状态以 CI 文档中的核对说明为准。仓库仍没有部署配置。
