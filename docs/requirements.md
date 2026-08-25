# Morva 产品与需求基线

本文是 Morva 的长期需求基线。具体版本的已实现状态以 [implementation-status.md](implementation-status.md) 为准，已批准实现意图以 `_bmad-output/implementation-artifacts/` 下的冻结规格为准。

## 1. 产品定位

Morva 是面向有编程或架构经验用户的高层结构化语义描述语言。它描述系统必须满足的业务事实、约束、状态变化与架构意图，不描述所有实现细节。

Morva 源码是 Source of Truth。自然语言仅作为 AI 辅助生成、解释和评审入口；任何 AI 产物都必须回到结构化源码，由人审核后才成为模型事实。

## 2. 核心目标

1. 用比自然语言更准确、比通用编程语言更紧凑的形式表达系统语义。
2. 让人、静态分析器、模拟器和 AI 使用同一份模型。
3. 尽早发现引用、类型、约束和状态变化中的明显错误。
4. 支持人工审核、修改和版本控制，不让 AI 的隐式判断成为事实。
5. 在语义稳定后，为 AI 生成目标语言实现提供高质量上下文。

## 3. 目标用户与主要任务

- 软件架构师：描述实体、动作、约束、状态变化和实现偏好。
- 资深开发者：验证设计，审查边界，并把模型用于实现生成。
- 业务技术人员：在不编写完整应用代码的前提下维护可检查规则。
- AI 工具：生成候选模型、提出 challenge/review 意见、读取已审核模型生成实现。

核心工作流：

```text
自然语言或人工设计
        ↓
候选 .morva 源码
        ↓
人工审核与修改
        ↓
parse → semantic check → inspect → simulate
        ↓
grill / challenge / review（规划中）
        ↓
带 implementation_hint 的目标实现生成（规划中）
```

## 4. 不可变产品原则

- 简单优先：没有真实用例支撑时不增加语法或抽象。
- 准确优先：不把兼容解析、软提示或运行时行为误称为已验证语义。
- 确定性：相同源码和版本必须产生相同 AST、诊断和模拟结果。
- 显式性：可执行语义必须出现在结构化源码中。
- 人在回路：AI 可以建议，不能静默修改 Source of Truth。
- 低复杂度：核心默认不引入依赖；新增依赖必须带来明确、可测收益。
- 可审计：诊断、阶段、状态变化和失败位置必须可追踪到源码 span。

## 5. v0.1 功能需求

### FR-01 源码与命名

- 项目、语言和命令统一使用 `Morva` / `morva`；`.morva` 是源码文件约定，当前 CLI 不按后缀拒绝输入。
- 文档必须恰好包含一个顶层 `system`；禁止嵌套 `system`。
- CLI 输入可为单文件或项目目录。目录只装配直接子级中小写 `.morva` 普通文件；忽略其他文件、子目录和 symlink，候选文件名必须是有效 UTF-8 并按其字节序排序。每个候选文件必须独立解析并恰有一个同名顶层 `system`，装配只合并 system 子声明。
- 多文件项目沿用全局短名语义，不引入 import、module scope、递归目录或跨项目依赖；诊断与模拟失败必须回映到责任文件的本地 span 和行列。
- 标识符当前只支持 ASCII。
- LF、CRLF、CR 各表示一个逻辑换行；CRLF 只产生一个换行 token，span 保留原文件 byte offset。
- `//` 行注释在任一逻辑换行或 EOF 前结束；`/* ... */` 块注释可在 token 之间跨行并嵌套。块内每个 LF、CRLF、CR 仍产生一个逻辑换行；注释正文不进入 AST 或语义。这里的等价是把非换行注释正文视为空白、保留内部逻辑换行，不是无条件删除全部注释 byte。
- 无换行块注释不得切分一个现有 identifier、Integer 或复合 operator token（`==`、`!=`、`>=`、`<=`、`+=`、`-=`），包括 compatibility、soft behavior 和 implementation hint 等 parser 跳过区；违反时以 `MORVA1025` 标记触发的 `/*` 两个 byte。未闭合块注释以 `MORVA1024` 精确标记最外层 `/*` 两个 byte。

### FR-02 强类型核心模型

- 强类型建模 `system`、`entity`、`enum`、`action` 和 `scenario`。
- `entity` 支持字段与 `invariant`。
- `action` 支持参数以及 `requires`、`effects`、`ensures`、`invariant`。
- `scenario` 支持 `given* → run → expect+`。
- 所有语义节点保留源码 span。

### FR-03 解析兼容边界

- `module`、`service`、`event`、`flow`、`lifecycle`、`policy` 当前仅是兼容容器，不承诺内部语义。
- 每个兼容容器在显式分析和 CLI `check` 中产生 `MORVA5001` 非致命 warning；旧 error-only `check()` API 与 simulator 不把 warning 当失败。
- action 中仅 `atomic`、`idempotent`、`timeout`、`retry`、`implementation_hint` 可作为白名单软项被接受；每个已解析源码项在显式分析和 CLI `check` 中产生一个 `MORVA5002`，指向 keyword，并说明该项未被语义验证或模拟执行。
- 未知 action 项必须报错，不能静默吞掉拼写错误。

### FR-04 静态语义检查

- 检查重复声明、字段、参数和枚举成员。
- 检查未知类型、全局短类型歧义、内建类型冲突。
- 检查字段路径、effect 写目标和上下文枚举成员。
- 规范化 builtin 别名；要求谓词为 Boolean、比较操作数兼容、普通 effect 同型且复合 effect 仅用于 Integer。
- Decimal 比较或赋值上下文可把非负 Integer 字面量作为精确常量；Integer 路径与 Decimal 路径不隐式转换。
- 在引用和类型检查成功后，检查 action 同一状态阶段内由 Boolean、Integer、enum 和 Decimal-context Integer 精确字面量事实直接证明的明显矛盾。
- `requires + action invariant` 属于前态，`action invariant + ensures` 属于后态；effects 按源码顺序摘要，只有最后已知的直接字面量 `=` 可用于检查后置约束。
- 检查 scenario 顺序、action 选择、参数数量、实参唯一性与可模拟类型。
- 当前检查器不得被描述为形式化证明或完整静态类型系统。

### FR-05 诊断

- 语法、语义和模拟选择错误使用稳定的 `MORVA1xxx`、`MORVA2xxx/3xxx`、`MORVA4xxx` 代码空间。
- 非致命语义覆盖提示使用稳定的 `MORVA5xxx` warning 空间；warning-only 检查成功退出 0。
- CLI 诊断包含代码、消息、行列和源码标记。
- tab、控制字节和文件路径必须安全显示，不能污染终端输出。
- 每条源码 excerpt 和 marker 的显示内容分别不超过 160 个渲染字符；窗口必须保留错误起点和至少一个 caret，裁剪不得切断转义片段。

### FR-06 CLI

- `morva check <file-or-directory>`：解析、装配并执行当前语义检查。
- `morva parse <file-or-directory>`：对通过当前语义检查的模型，输出强类型 AST 中已建模的声明内容；不回显被忽略的兼容文本。
- `morva inspect <file-or-directory>`：输出稳定的语义摘要文本。
- `morva simulate <file-or-directory> <scenario>`：执行一个已检查的最小场景。
- `morva capabilities`：不读取任何模型文件，输出 core 权威能力清单的稳定文本。
- 退出码固定为：成功 `0`，模型无效或模拟失败 `1`，用法/文件错误 `2`。

### FR-07 最小模型级模拟

- 仅运行一个 action，run 参数必须是互异的 entity 实例名。
- 状态仅存在内存中，值限 enum、Boolean、Integer。
- 所有读取必须已初始化；effects 按源码顺序执行；Integer 复合赋值检查溢出。
- 固定阶段为 givens、initial invariants、requires、effects、final invariants、ensures、expects。
- 失败立即停止，保留失败前状态和已发生的变化。
- 不执行实现代码、软行为项或 `implementation_hint`，不访问外部系统。

### FR-08 实现提示

- `implementation_hint` 表达实现偏好，不改变模型真假。
- 每个已解析提示产生结构化非致命 `MORVA5002`，不得单独造成语义错误；提示正文仍不进入语义求值或模拟。

## 6. 非功能需求

- NFR-01 正确性：每项新增语义必须有正常、失败和边界测试。
- NFR-02 稳定性：诊断代码、CLI 退出码和已承诺输出变更必须显式评审。
- NFR-03 安全性：解析和渲染不因不可信源码 panic，不执行模型携带的代码或 IO。
- NFR-04 性能：v0.1 以输入总量上线性或接近线性的处理为目标；不先行建设增量框架。
- NFR-05 可移植性：核心使用 Rust 标准库，CLI 可构建为单二进制。
- NFR-06 可维护性：语义只在 `morva-core` 实现；CLI 不复制检查规则。
- NFR-07 可复现性：格式、lint、测试与示例命令必须在交付前全部通过。

## 7. 明确非目标

v0.1 不包含：形式化验证、完整表达式类型系统、模块作用域、包管理、宏、格式化器、LSP、Tree-sitter、持久化、网络、并发、时间语义、事件运行时、flow/lifecycle 模拟、多 action 调用链、通用脚本运行时或生产代码生成。

这些项目只能在真实用例、独立规格和验收标准齐备后进入实现。

## 8. 变更控制

以下变更必须先获得人工确认并更新需求/规格：

- 改名、改后缀或改变现有示例含义；
- 扩大强类型语法集合或把兼容容器升级为语义节点；
- 改变 CLI 命令、输出契约、诊断代码或退出码；
- 引入第三方 parser/CLI/runtime 框架；
- 执行 `implementation_hint`、访问外部状态或生成/运行应用代码；
- 支持多 action、标量参数、别名或新的模拟值类型。

每次变更至少更新：需求条目、语言参考、实现状态、测试和对应实现规格。

## 9. v0.1 完成定义

- 已批准规格的验收标准全部满足。
- `cargo fmt --check`、严格 clippy、workspace tests 全部通过。
- `examples/order.morva` 的 check/parse/inspect/simulate 闭环通过。
- README、语言参考、实现状态和 AI 上下文与代码一致。
- 未实现能力清晰标注，不以 roadmap 代替当前承诺。
