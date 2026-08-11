# Morva 架构

## 1. 架构摘要

Morva 是一个单一产品、双 crate 的 Rust workspace。核心采用管线式架构：源码依次经过 lexer、parser、AST、semantic check；CLI 和 simulator 都只消费这套公共模型。

```text
.morva source(s)
      │
      ▼
per-source lexer/parser ──► local AST + byte spans
      │
      ▼
project assembler ─► merged AST + virtual spans + source map
      │
      ▼
semantic check ─► stable diagnostics
      │
      ├────────► parse / inspect CLI views
      │
      └────────► scenario simulator ─► phases / changes / final state
```

关键依赖方向：

```text
morva-cli ─────► morva-core
presentation     language semantics
```

`morva-core` 不依赖 CLI、文件系统或终端；`morva-cli` 不拥有语言规则。

## 2. Workspace 组成

### `morva-core`

公开能力：

- `parse(&str) -> Result<Document, Vec<Diagnostic>>`
- `Project::parse(sources) -> Result<Project, Vec<ProjectDiagnostic>>`
- `check(&Document) -> Vec<Diagnostic>`
- `analyze(&Document) -> AnalysisReport`（additive errors + non-fatal notices）
- `simulate(&Document, &str) -> Result<SimulationReport, Diagnostic>`
- AST、diagnostic 和 simulation report 类型

内部模块：

| 模块 | 职责 | 不负责 |
|---|---|---|
| `ast.rs` | 带 span 的声明、表达式、赋值和 scenario 模型 | 查找、验证、执行 |
| `lexer.rs` | 把 UTF-8 源码分解为 token，拒绝不支持字符 | 语法和语义恢复 |
| `parser.rs` | 构造强类型 AST，维护兼容解析白名单 | 名称解析、类型检查 |
| `project.rs` | 装配同名 system、分配 checked virtual span、回映射 `SourceId + LocalSourceSpan` | 文件发现、IO、模块语义 |
| `semantic.rs` | 建索引并检查声明、引用、effect、scenario 结构和有限字面量事实一致性 | 形式化证明、完整类型系统 |
| `diagnostic.rs` | 稳定 code/message/span 数据模型 | 行列和终端渲染 |
| `simulate.rs` | runtime `Value`/report 模型；解释一个已检查 scenario 的内存状态变化 | IO、应用代码、持久化、并发 |

### `morva-cli`

职责：读取单文件或发现目录的直接 `.morva` 普通文件、路由命令、调用 core、渲染诊断和报告、返回稳定退出码。目录发现按文件名字节序排序并忽略子目录/symlink；装配与语义仍由 core 负责。CLI 不能独立解释语言语义；未承诺的装饰性格式可以调整，但诊断代码、退出码以及测试或文档明确承诺的字段、顺序和文本属于兼容性边界，变更前必须显式评审。

## 3. 核心数据模型

`Document` 包含声明树。强类型声明为 `System`、`Entity`、`Enum`、`Action`、`Scenario`；其他已识别声明使用 `Container` 保存层级但不提供专有语义。

表达式当前仅包含 Integer、Boolean、Path 和二元比较。effects 使用独立 `Assignment`，从类型层面区分谓词与状态写入。`Span` 使用 UTF-8 字节区间；单文件 AST 保持本地 offset，多文件私有 merged AST 使用互不重叠的 checked virtual base。`SourceMap::locate_virtual_span` 只接受 merged document 的虚拟 span，并在 CLI 呈现前恢复 `SourceId + LocalSourceSpan`；`ProjectDiagnostic::Source` 已明确持有本地 diagnostic，不能再次回映射。公开 `Span` 与 `Diagnostic` 形状不变。

## 4. 名称与类型解析

- 文档必须有且仅有一个顶层 system。
- 当前没有模块作用域。entity、enum、action、scenario 的短名按相应规则全局查找。
- 同名短类型即使位于不同兼容容器中也被视为歧义。
- 内建类型名不能被用户类型覆盖。
- 裸标识符只可在明确 enum 期望类型下解释为 enum member。
- Checker 将 `Bool`/`Boolean`、`Int`/`Integer`、`ID`/`Id` 规范化为相同 builtin family；解析失败使用内部 invalid 结果抑制派生类型诊断。
- Entity 只作为字段路径的中间类型。谓词、比较和 effect 只接受已批准的标量或 enum 终值；Decimal 上下文可精确接收非负 Integer 字面量，但不存在路径间数值转换。
- Action 在主引用/类型检查无误后才运行有限事实分析：前态为 `requires + action invariant`，后态为 `action invariant + ensures`。路径的顺序 effect 摘要只有 `Known(literal)` 与 `Unknown`，不会升级为通用数据流。

这是刻意的 v0.1 限制，不能通过在 CLI 或 simulator 中猜测作用域来绕过。

## 5. 模拟器架构

模拟器先调用 `check`，因此公开 API 也不接受未验证文档。它收集 action、scenario、entity、enum 索引，把 action 参数按位置绑定到 scenario 实参，并使用按字符串路径排序的 `BTreeMap<String, Value>` 表示状态。

状态键示例：`order.status`。这种扁平设计避免引入对象堆、生命周期或存储模型。它也意味着 v0.1 只承诺直接 entity 字段模拟。

阶段顺序不可由调用方更改：

1. givens
2. initial invariants
3. requires（含 action invariant）
4. effects
5. final invariants（action 与 entity）
6. ensures
7. expects

## 6. 错误边界

- Lexer/parser 返回一个或多个 `Diagnostic` 的失败结果。
- Semantic checker 累积可独立发现的诊断。
- Simulator 的静态选择错误返回 `Diagnostic`；执行期失败写入 `SimulationReport.failure`。
- CLI 把语言/模拟失败映射到退出码 1，把用法/文件 IO 映射到 2。
- `check()` 和 simulator 保持 error-only；additive analysis report 承载兼容容器 warning，CLI `check` 只在存在 error 时退出 1。
- 不可信输入不得触发 panic；测试覆盖字符、溢出、未初始化和无效结构等边界。
- CLI 对项目候选执行 symlink metadata、canonical root、打开文件句柄及打开前后文件身份校验，读取绑定到已验证句柄。纯标准库没有跨平台原子 `nofollow` open；Unix 使用 device/inode 强身份，其他平台退化为 length/modified 校验。并发原地改写同一 inode 的内容仍不提供快照隔离，这是无 watch/增量承诺下保留的 TOCTOU 残余。

## 7. 架构约束

- AST 中的语义构造应优先使用 enum/struct，不使用无结构 map 代替。
- 新语义首先进入 core 的 AST/parser/checker，再由 CLI 消费。
- 不为未批准的未来功能预建 trait、插件系统或通用 runtime。
- 新依赖需说明为何标准库不足、二进制/维护成本和测试收益。
- `implementation_hint` 永远不进入语义真假判断或 simulator 执行路径。
- 文档中的“支持”必须能指向公开 API 或自动化测试。

## 8. 已知技术债务

- 当前静态表达式类型检查只覆盖已有 literal/path/binary、谓词和 effect，不包含通用推断、逻辑/算术表达式、数据流分析或形式化证明。
- 当前明显矛盾检查只覆盖同阶段精确字面量事实和最终直接赋值；不实例化 entity invariant，不内联 scenario/action，不做区间推理、跨阶段未写路径推理或 compound folding。
- Simulator 仍保留类型守卫作为纵深防御；其值域仍只含 enum、Boolean 和 Integer，不能由静态 Decimal 规则推导出 Decimal 模拟支持。
- 负整数字面量尚未形成产品契约；有界诊断窗口不限制完整 message 或安全路径的总输出大小。
- 模块作用域、稳定机器可读输出和增量分析均未设计。

处理这些债务时必须先写失败测试和独立规格，不能借机扩大语言范围。
