# 源码树与职责

```text
morva/
├── Cargo.toml                         # workspace 与共享 package 元数据
├── Cargo.lock                         # 可复现依赖锁；当前无第三方依赖
├── README.md                          # 人类入口与快速开始
├── crates/
│   ├── morva-core/                    # 唯一语言语义实现
│   │   ├── src/
│   │   │   ├── lib.rs                 # 公共 parse/check/simulate API 与 re-export
│   │   │   ├── ast.rs                 # 强类型 AST、表达式、scenario、span
│   │   │   ├── lexer.rs               # token 化与词法诊断
│   │   │   ├── parser.rs              # 语法与兼容解析边界
│   │   │   ├── project.rs             # 同名 system 装配、虚拟 span 与来源回映射
│   │   │   ├── semantic.rs            # 索引、名称、路径、scenario 静态检查
│   │   │   ├── diagnostic.rs          # 稳定诊断数据结构
│   │   │   └── simulate.rs            # 单 action 内存模拟器
│   │   └── tests/                      # language、project、simulation 集成测试
│   │       ├── language.rs             # 语言与静态诊断回归
│   │       ├── project.rs              # 多文件装配、source map 与 API parity
│   │       └── simulation.rs           # 模拟语义与阶段回归
│   └── morva-cli/                     # 可执行适配层
│       ├── src/main.rs                 # 命令、IO、呈现与退出码
│       └── tests/cli.rs                # 单文件与目录输入的进程级 CLI 契约
├── examples/
│   ├── order.morva                    # 单文件四命令基线模型
│   └── order-project/                 # 平铺多文件四命令基线模型
├── docs/                              # 项目知识与 AI 检索入口
└── _bmad-output/
    └── implementation-artifacts/      # 已批准且冻结的实现规格
```

## 关键入口

- 面向 Rust 调用方：`crates/morva-core/src/lib.rs`
- 面向最终用户：`crates/morva-cli/src/main.rs`
- 面向 AI 或新贡献者：`docs/index.md`，随后读取 `docs/project-context.md`
- 当前真实行为证据：`crates/*/tests/`、`examples/order.morva` 和 `examples/order-project/`

## 修改路由

| 变更 | 首要位置 | 必须同步 |
|---|---|---|
| 新声明/表达式结构 | `ast.rs`, `parser.rs` | semantic、language tests、语言参考 |
| 新静态规则 | `semantic.rs` | 诊断测试、实现状态 |
| 新模拟语义 | `simulate.rs` | simulation tests、需求/架构/语言参考 |
| 新命令或输出 | CLI `main.rs` | CLI tests、`cli.md` |
| 多文件装配或发现 | core `project.rs`、CLI `main.rs` | project/CLI tests、架构与 CLI 文档 |
| 产品范围与阶段 | `requirements.md`, `roadmap.md` | 独立冻结规格 |

`.idea/` 是本地 IDE 元数据，不属于产品架构或交付接口。
