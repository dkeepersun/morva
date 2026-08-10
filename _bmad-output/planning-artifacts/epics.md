---
stepsCompleted: ['step-01-validate-prerequisites', 'step-02-design-epics', 'step-03-create-stories', 'step-04-final-validation']
inputDocuments:
  - docs/requirements.md
  - docs/architecture.md
  - docs/implementation-status.md
  - docs/testing-strategy.md
  - docs/project-context.md
  - _bmad-output/implementation-artifacts/deferred-work.md
  - next.md
  - docs/language-evolution-policy.md
---

# Morva - Epic Breakdown

## Overview

本文把 Morva 的长期需求、当前架构、真实实现状态和下一阶段评估整理为可连续执行的 Epic/Story 输入。`next.md` 用于缺口与优先级证据；尚未提交的 `docs/language-evolution-policy.md` 仅作为待复审草案，不是已批准约束。

## Requirements Inventory

### Functional Requirements

FR1: Morva 必须以结构化 `.morva` 源码作为 Source of Truth，支持单文件或平铺目录项目，并把多文件诊断映射回责任文件的本地位置。

FR2: 语言必须保持确定的 UTF-8 byte span、LF/CRLF/CR 换行契约，以及 `//` 和可嵌套 `/* ... */` 注释行为。

FR3: 语言必须强类型建模 `system`、`entity`、`enum`、`action`、`scenario` 及其字段、参数、条款和场景步骤。

FR4: Checker 必须验证声明唯一性、类型与路径解析、谓词和比较类型、effect 赋值、scenario 结构及当前有限精确事实矛盾。

FR5: CLI 必须提供 `check`、`parse`、`inspect`、`simulate`，保持模型错误 1、输入/用法错误 2、成功 0 的退出码契约。

FR6: Simulator 必须保持单 action、内存 enum/Boolean/Integer 状态和固定七阶段，失败位置及已发生状态变化可追踪。

FR7: `implementation_hint` 和所有未建模内容不得改变语义真假、执行代码或访问外部状态。

FR8: `module`、`service`、`event`、`flow`、`lifecycle`、`policy` 及被忽略 soft behavior 必须向使用者明确显示为“已解析但未验证”，不能继续产生无提示的假成功。

FR9: 非致命 notice/warning 必须与 error 区分；只有 warning 时 `check` 保持退出 0，单/多文件输出都携带稳定来源位置。

FR10: `inspect` 必须列出或统计未建模声明，使人和 AI 能判断模型中多少内容未被语义覆盖。

FR11: L1 表达式必须支持 `!`、`||`、括号和嵌套 Boolean 谓词，并为优先级、类型、模拟及保守矛盾分析提供明确契约。

FR12: Morva 必须提供稳定、带版本的机器可读输出，至少覆盖诊断的 code、message、source、line/column、byte span 和 severity。

FR13: Morva 必须在 core 中提供唯一权威的结构化能力清单，明确当前支持与不支持的表达式、字面量、声明和模拟边界，并可分别渲染为人类与机器输出。

FR14: AI 接口必须以只读检查、解析、查看和模拟为边界，不得绕过人工审核直接修改 Source of Truth。

### NonFunctional Requirements

NFR1: 每项新增语法或语义必须有成功、失败、边界、公开 core seam 和真实 CLI seam 证据。

NFR2: 相同源码和版本必须产生确定的 AST、诊断、排序和模拟结果；稳定输出变更必须显式评审。

NFR3: 不可信源码、路径和极端输入不得造成 panic、终端控制字符污染或无界源码摘录。

NFR4: v0.1 处理目标为随输入总量线性或接近线性；性能证据应可重复，不使用易受 CI 抖动影响的严格墙钟比率作为首个阻断门禁。

NFR5: `morva-core` 默认保持标准库、无外部作用；新增依赖必须证明维护、二进制和测试收益。

NFR6: 所有语言语义只能存在于 core；CLI 只负责文件发现、命令路由和呈现。

NFR7: 每次交付必须通过 fmt、严格 Clippy、workspace tests、单/多文件四命令和 diff check。

NFR8: Rust 工具链和 MSRV 必须形成显式、可复现的支持策略；非阻断前瞻 lane 与阻断门禁分离。

NFR9: GitHub 质量门禁状态、远程运行证据和 branch protection 文档必须与实际仓库状态一致。

NFR10: 公开 API、诊断 code、退出码、示例含义及人机输出协议必须保持兼容或通过版本化迁移。

### Additional Requirements

- 先完成交付与治理收口：修正 CI 已托管成功运行但文档仍称未推送的漂移，补齐完成规格和 `MORVA1025` 文档，并单独评审语言演进草案。
- GitHub 已存在成功的 hosted `Test Pipeline`；`main` 当前未启用 branch protection。启用保护属于明确的远程仓库变更，实施时需单独执行和验证。
- 工具链固定必须基于实际可构建版本和 API 使用情况决定，不能只由 edition 2024 的最低版本推导完整 MSRV。
- 兼容内容可见性应优先采用 additive 分析接口或独立 notice 模型，避免破坏现有 `Diagnostic` struct、`check()` 和退出码。
- 多文件 notice/diagnostic 必须沿用 `SourceId + LocalSourceSpan`，不得在 CLI 复制语言判断。
- L1 应先定义表达式优先级和 AST 递归边界，再扩展 checker、simulator 和矛盾分析；不借机加入算术、字符串或集合。
- JSON 工作先冻结 schema、版本和确定排序，再决定手写序列化或隔离依赖；人类文本输出继续兼容。
- MCP 或其他 AI 接入依赖稳定机器协议和能力清单；工具只读且不直接写 `.morva`。
- NFR-04 首先建立可重复的规模生成与趋势基线；是否升级为阻断阈值由实测稳定性决定。
- 并发项目文件快照、原子跨平台 `nofollow`、递归目录、manifest/import、模块作用域和增量分析维持延期。
- 语言演进草案中关于 SAT、非线性算术、覆盖率与 `opaque` 的结论必须经过单独事实审查和人工冻结后才能约束实现。
- Deferred Candidates 不属于当前功能承诺：Decimal 字面量、负 Integer、String、线性算术必须逐项立证和评审，且算术边界必须先明确有限机器值与数学值模型。
- `lifecycle` 状态机、多步 scenario、集合/量词、Tree-sitter 和 LSP 仅在真实模型证据与独立规格齐备后生成新的 Epic。

### UX Design Requirements

无。Morva 当前是 core library + CLI 产品，没有 UI/UX 设计合同；CLI 诊断可读性和机器协议要求已分别计入 FR 与 NFR。

### FR Coverage Map

FR1: Epic 1 — 单/多文件 Source of Truth 与本地位置映射。

FR2: Epic 1 — 原始 byte span、通用换行与注释契约。

FR3: Epic 1 — 强类型核心声明与条款模型。

FR4: Epic 1 — 当前静态语义和有限矛盾检查。

FR5: Epic 1 — 四命令与稳定退出码。

FR6: Epic 1 — 单 action 七阶段模拟。

FR7: Epic 1 — 实现提示与未建模内容不进入求值。

FR8: Epic 2 — 兼容构造与 soft behavior 显式提示。

FR9: Epic 2 — 非致命 warning/notice 与位置契约。

FR10: Epic 2 — inspect 未建模声明清单和统计。

FR11: Epic 3 — `!`、`||`、括号和嵌套 Boolean 谓词。

FR12: Epic 4 — 版本化机器可读验证协议。

FR13: Epic 2 — core 权威 capability inventory；Epic 4/5 只消费并暴露它。

FR14: Epic 5 — 只读 AI 校验接口和人工审核边界。

## Epic List

### Epic 1: 已验证的 v0.1 模型基础

用户已经可以编写、检查、查看和模拟确定、可追踪的单文件或多文件 Morva 模型；本 Epic 记录既有完成证据，不重复开发。

**FRs covered:** FR1, FR2, FR3, FR4, FR5, FR6, FR7
**Current state:** done

### Epic 2: 诚实的语义覆盖反馈

模型作者可以明确看到哪些内容已被验证、哪些只是解析兼容，并从同一权威能力清单理解当前边界。

**FRs covered:** FR8, FR9, FR10, FR13

### Epic 3: 常见 Boolean 业务规则

模型作者可以用析取、取反、分组和嵌套谓词准确表达高频业务条件，同时保留确定检查与有限模拟。

**FRs covered:** FR11

### Epic 4: 稳定的机器可读验证协议

自动化工具可以通过版本化、确定的 schema 消费四命令结果和权威能力清单，而不抓取人类文本。

**FRs covered:** FR12（并消费 FR13）

### Epic 5: 只读 AI 校验集成

AI 客户端可以通过标准工具调用检查、解析、查看和模拟 Morva，同时保持人工审核和无外部作用边界。

**FRs covered:** FR14（并消费 FR13）

## Epic 1: 已验证的 v0.1 模型基础

用户已经可以编写、检查、查看和模拟确定、可追踪的单文件或多文件 Morva 模型。本 Epic 记录既有完成证据，不重复开发。

### Story 1.1: 编写并检查强类型业务模型

As a 软件架构师或资深开发者,
I want 使用结构化声明、类型、路径、条款和场景描述业务模型,
So that 我可以在进入实现前发现明确的结构、引用、类型和约束错误。

**Acceptance Criteria:**

**Given** 一个包含 `system`、`entity`、`enum`、`action` 和 `scenario` 的合法 Morva 模型
**When** 用户执行 core `parse/check` 或 CLI `morva check`
**Then** 模型成功构造带原始 byte span 的强类型 AST
**And** 合法模型不产生语义错误。

**Given** 模型包含重复声明、未知或歧义类型、无效字段路径或 enum member
**When** 执行语义检查
**Then** 返回稳定 code、message 和责任源码 span
**And** 独立错误按确定顺序报告。

**Given** action 谓词、比较或 effect 使用不兼容类型
**When** 执行检查
**Then** checker 在模拟前拒绝模型
**And** 引用失败不会产生派生类型诊断噪声。

**Given** action 的前态或后态包含可由精确字面量直接证明的矛盾
**When** 主引用和类型检查成功
**Then** checker 报告保守且无误报的矛盾诊断
**And** 未报告不得被解释为已证明可满足。

**Given** 模型包含 `implementation_hint` 或当前白名单 soft behavior
**When** 执行检查
**Then** 这些内容不改变模型真假、不被执行且不访问外部状态。

### Story 1.2: 查看并模拟单操作业务契约

As a 模型评审者,
I want 查看模型语义摘要并执行一个受控场景,
So that 我可以确认 action 的前置条件、状态变化和后置结果符合预期。

**Acceptance Criteria:**

**Given** 一个通过静态检查的合法模型
**When** 用户执行 `morva parse` 或 `morva inspect`
**Then** 输出只包含当前强类型 AST 和已建模语义
**And** 被忽略的兼容文本或实现提示不被误写为已验证行为。

**Given** 一个结构为 `given* → run → expect+` 的合法 scenario
**When** 用户执行 `morva simulate <input> <scenario>`
**Then** 模拟按 givens、initial invariants、requires、effects、final invariants、ensures、expects 七阶段运行
**And** 报告每个阶段、状态变化和最终状态。

**Given** action 包含多个 effect
**When** 执行模拟
**Then** effect 按源码顺序生效
**And** enum、Boolean、Integer 状态变化保持确定顺序。

**Given** 某个阶段失败、读取未初始化字段或 Integer 运算溢出
**When** 执行模拟
**Then** 模拟在责任阶段停止
**And** 报告失败 span、失败前状态及此前已经发生的变化。

**Given** 用户选择未知 scenario 或模型未通过检查
**When** 调用模拟
**Then** 返回稳定诊断和退出码 1
**And** 不执行任何实现代码、IO 或外部状态操作。

### Story 1.3: 获得安全且精确的源码诊断

As a Morva 模型作者,
I want 错误始终指向真实源码位置并安全显示上下文,
So that 我可以快速修复模型，而不会被换行、超长输入或控制字符干扰。

**Acceptance Criteria:**

**Given** 源码使用 LF、CRLF、CR 或混合换行
**When** lexer、parser、checker 或 simulator 产生位置
**Then** 每种序列只计一个逻辑换行
**And** span 始终引用未规范化原始源码的 UTF-8 byte offset。

**Given** 源码包含 `//` 或可嵌套 `/* ... */` 注释
**When** 解析模型
**Then** 注释正文不进入 AST 或语义
**And** 块内逻辑换行继续分隔语法。

**Given** 块注释未闭合或被插入现有 token 内部
**When** 执行解析
**Then** 返回稳定的 `MORVA1024` 或 `MORVA1025`
**And** marker 精确覆盖责任 `/*` 两个 byte。

**Given** 诊断位于超长源码行、EOF、tab、控制字符或非 ASCII byte 附近
**When** CLI 渲染诊断
**Then** excerpt 和 marker 分别不超过 160 个渲染字符
**And** 保留错误起点、至少一个 caret，且不切断安全转义片段。

**Given** 输入路径包含 UTF-8 控制字符
**When** 输出成功、读取错误、模型错误或模拟失败
**Then** 路径被安全转义
**And** 原始控制字符不能污染终端。

**Given** 相同源码和 Morva 版本被重复检查
**When** 产生多个诊断
**Then** code、message、顺序、位置和必要输出保持确定。

### Story 1.4: 按文件组织同一系统模型

As a 维护大型 Morva 模型的开发者,
I want 把同一系统的类型、动作和场景分布在多个文件中,
So that 我可以按关注点独立修改，而不失去全局检查和模拟能力。

**Acceptance Criteria:**

**Given** CLI 输入是一个目录
**When** 发现项目源码
**Then** 只选择直接子级中小写 `.morva` 普通文件
**And** 忽略其他文件、子目录和 symlink，并按 UTF-8 文件名字节序确定顺序。

**Given** 多个文件各自包含一个同名顶层 `system`
**When** core 装配项目
**Then** 只合并根 system 的子声明
**And** 跨文件类型、action 和 scenario 引用沿用现有全局短名语义。

**Given** 某个后排序文件包含语法、语义或运行期失败
**When** CLI 渲染结果
**Then** code、message、行列、excerpt 和 marker 指向责任文件的本地 span
**And** 不能错误映射到其他文件的相同 offset。

**Given** 项目为空、文件不可读、候选文件名不是 UTF-8 或目录发现失败
**When** 执行任一命令
**Then** 返回退出码 2
**And** 不输出部分模型结果。

**Given** 文件包含不同 system 名、多个 system 或跨文件重复声明
**When** 执行检查
**Then** 返回模型错误和责任文件位置
**And** 退出码为 1。

**Given** 用户继续传入单个普通文件或无后缀文件
**When** 执行四个命令
**Then** 旧单文件 API、AST、诊断、输出和模拟行为保持兼容。

## Epic 2: 诚实的语义覆盖反馈

模型作者可以明确看到哪些内容已验证、哪些只是解析兼容，并从同一权威能力清单理解当前边界。

### Story 2.1: 识别已解析但未验证的兼容容器

As a Morva 模型作者,
I want `check` 明确提示只被兼容解析的声明,
So that 我不会把 `module`、`policy` 或其他空壳误认为已被语义验证。

**Acceptance Criteria:**

**Given** 模型包含 `module`、`service`、`event`、`flow`、`lifecycle` 或 `policy`
**When** core 分析模型
**Then** 每个兼容容器产生一个稳定、非致命 warning
**And** warning 包含构造类型、名称和声明来源 span。

**Given** 现有调用方继续使用 `check(&Document) -> Vec<Diagnostic>`
**When** 模型只有兼容容器 warning 而没有错误
**Then** `check()` 的错误返回契约保持兼容
**And** 新 warning 通过 additive analysis/report API 获取。

**Given** CLI 对只包含 warning 的模型执行 `morva check`
**When** 渲染分析结果
**Then** 显示“已解析但未验证”的 warning 和安全源码 marker
**And** 命令仍返回退出码 0。

**Given** 模型同时包含 warning 和真实语法或语义错误
**When** 执行 `check`
**Then** warning 与 error 按确定的来源顺序呈现
**And** 退出码仍由 error 决定为 1。

**Given** 兼容容器位于多文件项目的后排序文件
**When** CLI 显示 warning
**Then** 文件路径、行列、excerpt 和 marker 指向责任文件的本地 span。

**Given** 模型不包含任何兼容容器
**When** 执行现有示例四命令
**Then** stdout、stderr、退出码和 core error API 保持现有行为。

**Given** 兼容容器内部包含未建模文本
**When** warning 被报告
**Then** 这些文本仍不进入 checker 或 simulator
**And** warning 不得暗示其内部行为已被验证。

### Story 2.2: 识别未执行的 action soft behavior

As a Morva 模型作者,
I want `check` 明确提示 action 中只被接受但未被验证或执行的行为项,
So that 我不会把 `atomic`、重试设置或实现提示误认为模拟器已经兑现的保证。

**Acceptance Criteria:**

**Given** action 包含 `atomic`、`idempotent`、`timeout`、`retry` 或 `implementation_hint`
**When** core 分析模型
**Then** 每个 soft behavior 产生一个稳定、非致命 warning
**And** warning 说明该项已解析但未被语义验证或模拟执行。

**Given** soft behavior 带有参数、路径或 block 内容
**When** parser 接受该项
**Then** core 保留生成 warning 所需的种类和原始 span
**And** 不把未建模正文升级为可执行 AST 语义。

**Given** 同一 action 包含多个 soft behavior
**When** 执行检查
**Then** warning 按源码顺序确定呈现
**And** 每个责任项只报告一次。

**Given** soft behavior 位于多文件项目中的 action
**When** CLI 渲染 warning
**Then** warning 映射到 action 所在文件的本地行列和 marker
**And** 仅有 warning 时退出码保持 0。

**Given** action 包含白名单之外的未知项或拼写错误
**When** parser 处理 action
**Then** 继续返回现有语法 error
**And** 不能把未知项降级成 warning 或静默忽略。

**Given** 用户执行 `simulate`
**When** action 含有 soft behavior
**Then** soft behavior 仍不改变七阶段、状态或结果
**And** 模拟输出不能暗示重试、超时、原子性或实现提示已被执行。

**Given** 现有模型不包含 soft behavior
**When** 执行 core API 和 CLI 回归
**Then** 不新增 warning
**And** 现有输出和退出码保持兼容。

### Story 2.3: 在 inspect 中查看未建模内容

As a 模型评审者,
I want `inspect` 汇总模型中尚未获得语义覆盖的声明和行为项,
So that 我可以判断当前模型有多少内容不能被 checker 或 simulator 证明。

**Acceptance Criteria:**

**Given** 模型包含兼容容器或 action soft behavior
**When** 用户执行 `morva inspect`
**Then** 输出包含明确的“未建模内容”摘要
**And** 分别列出构造种类、名称或所属 action，以及确定的数量统计。

**Given** 模型包含多个文件中的未建模内容
**When** 执行 inspect
**Then** 清单按项目文件顺序和源码顺序稳定排列
**And** 同一输入重复运行产生 byte-identical 输出。

**Given** 兼容容器内部包含任意被跳过文本
**When** inspect 生成摘要
**Then** 只显示安全、结构化的容器种类与名称
**And** 不回显未建模正文或把正文解释成语义。

**Given** action 包含多个 soft behavior
**When** inspect 生成摘要
**Then** 每项与所属 action 建立明确关联
**And** 不声称其原子性、重试、超时或实现提示已经验证。

**Given** 模型只有 warning 而没有 error
**When** 执行 inspect
**Then** 命令成功并返回退出码 0
**And** 摘要与 `check` warning 使用同一 core 分析事实。

**Given** 模型不包含任何未建模内容
**When** 执行现有示例的 inspect
**Then** 不新增空摘要或噪声
**And** 已承诺的现有 inspect 文本保持兼容。

**Given** 用户执行 `parse` 或 `simulate`
**When** 模型含有未建模内容
**Then** 本 Story 不改变 parse 的强类型输出或 simulator 的执行边界
**And** 未建模内容仍不进入求值。

### Story 2.4: 查询 Morva 的权威能力清单

As a 模型作者或工具集成者,
I want 从 Morva 本身查询当前支持和不支持的语言能力,
So that 我不需要猜测语法边界，也不会依赖可能漂移的手写列表。

**Acceptance Criteria:**

**Given** core library 被调用
**When** 调用 additive capability inventory API
**Then** 返回确定、结构化且有版本标识的能力模型
**And** 该模型不依赖 CLI、文件系统或网络。

**Given** capability inventory 被查询
**When** 查看其内容
**Then** 明确列出强类型声明、表达式形态、比较与赋值操作符、字面量、模拟值类型和七阶段
**And** 同时列出兼容容器、soft behavior 及明确不支持的主要类别。

**Given** 用户执行新增的 `morva capabilities` 命令
**When** 不提供模型文件
**Then** CLI 以稳定的人类可读文本呈现 core 的同一能力模型
**And** 成功退出 0，不读取或修改任何 `.morva` 文件。

**Given** `check` warning 和 `inspect` 未建模摘要需要描述支持边界
**When** 生成消息或分类
**Then** 使用 capability inventory 的相同构造分类
**And** 不在 CLI 或文档中复制另一份可执行判断表。

**Given** 后续新增或移除语言能力
**When** 修改 parser、checker 或 simulator
**Then** 同一交付必须更新 capability inventory 和精确测试
**And** 测试能发现声明能力与实际公开行为的明显漂移。

**Given** Epic 4 增加 JSON 输出或 Epic 5 增加 AI resource
**When** 暴露能力清单
**Then** 序列化本 Story 的同一结构化模型
**And** 不建立第二套能力来源。

**Given** 相同 Morva 版本重复查询能力
**When** 调用 core API 或 CLI 命令
**Then** 字段、顺序和人类文本保持确定
**And** 能力版本只在经评审的契约变化时更新。

## Epic 3: 常见 Boolean 业务规则

模型作者可以用析取、取反、分组和嵌套谓词准确表达高频业务条件，同时保留确定检查与有限模拟。

### Story 3.1: 分组并取反 Boolean 谓词

As a 业务规则作者,
I want 使用括号和 `!` 对 Boolean 条件进行分组和取反,
So that 我可以准确表达“不是某状态”或“整个条件不成立”。

**Acceptance Criteria:**

**Given** 谓词包含 Boolean literal、Boolean 路径或比较表达式
**When** 用户写入 `!predicate` 或 `(predicate)`
**Then** parser 构造带完整原始 byte span 的递归 Boolean AST
**And** 任意支持深度的嵌套括号与连续 `!` 按确定规则解析。

**Given** 表达式同时包含比较和 `!`
**When** 没有额外括号
**Then** 比较先形成 Boolean proposition，再由 `!` 取反
**And** `!order.status == Pending` 等价于 `!(order.status == Pending)`。

**Given** 用户需要覆盖默认结合方式
**When** 写入显式括号
**Then** 括号内容作为一个完整 Boolean predicate
**And** 多余但平衡的嵌套括号不改变结果。

**Given** `!` 的操作数不是 Boolean，或括号未闭合/为空
**When** 执行 parse/check
**Then** 返回稳定、可行动的语法或类型诊断
**And** marker 指向责任操作符、括号或表达式 span。

**Given** 合法的取反谓词出现在 entity/action invariant、requires、ensures 或 scenario expect
**When** 执行检查和模拟
**Then** 所有位置使用同一 core 类型与求值规则
**And** 模拟按原七阶段在内存中确定求值。

**Given** 取反表达式读取未初始化路径
**When** 执行模拟
**Then** 保留现有未初始化读取失败契约
**And** 不因 `!` 隐藏责任路径或 span。

**Given** lexer 同时处理 `!` 和现有 `!=`
**When** 解析旧模型与新谓词
**Then** `!=` 行为完全兼容
**And** `!` 只在新递归 Boolean 语法中作为取反操作符。

**Given** 语言能力清单已存在
**When** 本 Story 交付
**Then** 同步声明 parentheses 和 Boolean negation
**And** 仍明确标记 `||`、算术、字符串、集合和函数调用的当前支持状态。

### Story 3.2: 使用析取表达替代业务条件

As a 业务规则作者,
I want 使用 `||` 表达多个可接受条件中的任意一个,
So that 我可以描述“VIP 或金额超过阈值”这类没有现有 workaround 的高频规则。

**Acceptance Criteria:**

**Given** 两个合法 Boolean predicate
**When** 用户写入 `left || right`
**Then** parser 构造带完整 span 的析取 AST
**And** checker 要求左右两侧都产生 Boolean。

**Given** 表达式同时包含比较、`!` 和 `||`
**When** 没有额外括号
**Then** 默认优先级为比较高于 `!`、`!` 高于 `||`
**And** 连续 `||` 按左结合、确定地构造 AST。

**Given** 用户写入括号嵌套的析取和取反
**When** 执行 parse/check
**Then** 括号覆盖默认优先级
**And** 所有分支使用 Story 3.1 的同一递归 Boolean 类型规则。

**Given** `||` 任一侧不是 Boolean、缺少操作数或包含无效引用
**When** 执行静态检查
**Then** 返回稳定语法、引用或类型诊断
**And** 即使左侧为常量 `true`，静态检查仍检查右侧模型合法性。

**Given** 合法析取出现在 invariant、requires、ensures 或 expect
**When** 执行模拟
**Then** 运行时从左到右进行确定的短路求值
**And** 左侧为 `true` 时不读取右侧运行时状态。

**Given** 左侧为 `false` 且右侧读取未初始化路径
**When** 执行模拟
**Then** 保留右侧未初始化读取失败
**And** failure span 指向真正被求值的右侧路径。

**Given** 注释被插入 `||` 两个字符之间
**When** lexer 处理 `|/*...*/|` 或连续无换行注释变体
**Then** 返回 token-split 诊断
**And** 不能把被拆开的操作符当成两个独立符号或兼容文本。

**Given** 旧模型只使用比较或 `!=`
**When** 运行全部回归
**Then** AST、类型检查、诊断和模拟行为保持兼容
**And** 不新增隐式 `&&` 语法。

**Given** capability inventory 已存在
**When** 本 Story 交付
**Then** 同步声明 Boolean disjunction、优先级和短路规则
**And** 人类与未来机器输出消费同一能力事实。

### Story 3.3: 保守分析嵌套 Boolean 矛盾

As a 模型评审者,
I want checker 识别由取反、析取和精确事实直接证明的明显矛盾,
So that 新表达力不会削弱现有的早期错误发现能力或产生误报。

**Acceptance Criteria:**

**Given** Boolean 公式只包含 literal、精确比较、`!`、`||` 和括号
**When** checker 可以从当前阶段的已知精确事实确定公式真假
**Then** 使用 `True / False / Unknown` 三态进行保守求值
**And** 只有整个谓词可证明为 `False` 时才报告矛盾。

**Given** 公式为 `false || false`、`!(true)` 或等价嵌套形式
**When** 检查 action predicate
**Then** 报告现有稳定的明显矛盾诊断
**And** span 覆盖责任 Boolean 公式。

**Given** 析取任一分支可证明为 `True`
**When** 另一分支为 `False` 或 `Unknown`
**Then** 整个析取不报告矛盾
**And** checker 不因未探索分支产生误报。

**Given** 析取包含未知路径值或无法证明的比较
**When** 没有任何分支足以确定完整公式为 `False`
**Then** 结果保持 `Unknown`
**And** 未报告不能被描述为已证明可满足。

**Given** action 前态包含 requires 与 action invariant
**When** 分析嵌套 Boolean 公式
**Then** 继续只在前态事实组内组合
**And** 不把后态 facts 混入前态。

**Given** action 后态包含 action invariant、ensures 和最终已知直接 literal effect
**When** 最终 effect 值使整个嵌套公式可证明为 `False`
**Then** 报告现有后置/transition 矛盾诊断
**And** 非 literal 写入或 compound effect 继续把对应路径降为 `Unknown`。

**Given** 一个 `||` 的不同分支包含不同路径事实
**When** 退出该析取
**Then** 分支专属 fact 不得无条件泄漏到外部 fact 集
**And** 初始实现不需要进行公式分配、完整 SAT 或区间求解。

**Given** 旧模型不包含 `!`、`||` 或括号
**When** 运行现有矛盾检测测试
**Then** MORVA2018/2019 的 code、message、span 和顺序保持兼容。

**Given** 新公式包含主引用或类型错误
**When** 执行 checker
**Then** 保留主诊断并抑制派生矛盾噪声
**And** 不对无效公式执行事实推导。

## Epic 4: 稳定的机器可读验证协议

自动化工具可以通过版本化、确定性的 JSON 协议读取检查、解析、审视和模拟结果，同时默认人类输出保持兼容。

### Story 4.1: 机器读取 check 结果与诊断

As a 自动化工具开发者,
I want 以稳定的 JSON 格式读取 Morva 检查结果与诊断,
So that 我可以可靠地把模型验证接入脚本、编辑器和持续集成。

**Acceptance Criteria:**

**Given** 一个没有 warning 或 error 的有效单文件或目录项目
**When** 执行 `morva check --format json <file-or-directory>`
**Then** stdout 只包含一个完整 JSON object，含 schema version、command、success 和空 diagnostics
**And** stderr 为空且进程退出 0。

**Given** 模型只产生未建模内容 warning
**When** 执行 JSON check
**Then** diagnostics 为确定顺序的结构化数组，逐项包含 severity、稳定 code、message、source、1-based line/column 和文件本地 byte span
**And** success 为 true、进程退出 0。

**Given** 模型存在语法或语义 error
**When** 执行 JSON check
**Then** 同一诊断结构完整表达错误且顺序确定
**And** success 为 false、进程退出 1，不向 stderr 混入人类渲染文本。

**Given** 输入不存在、不可读、目录发现失败或命令用法错误
**When** CLI 已识别 `--format json`
**Then** stdout 返回一个完整、版本化的 machine error envelope
**And** 进程退出 2，不输出部分 JSON 或人类诊断片段。

**Given** 诊断来自多文件项目
**When** 将 core 结果序列化
**Then** source 和 span 指向真实文件及文件本地 offset
**And** 不暴露内部 virtual offset、合成 document span 或机器绝对路径。

**Given** 路径或消息包含引号、反斜线、控制字符或非 ASCII 文本
**When** 输出 JSON
**Then** 使用合法 JSON escaping 且可被标准 JSON parser 完整读取
**And** 不产生原始终端控制序列或无效 UTF-8。

**Given** 相同输入和 Morva 版本被重复检查
**When** 比较 JSON 输出
**Then** key 组织、数组顺序和字段值保持确定
**And** 不包含时间戳、随机值或环境相关绝对路径。

**Given** 用户未提供 `--format json`
**When** 执行现有 check 命令
**Then** 人类可读输出、marker、路径安全规则和退出码保持兼容
**And** core 不依赖 CLI 参数解析或 JSON 协议实现。

**Given** 已发布的 JSON schema 需要不兼容修改
**When** 评审并交付该修改
**Then** 提升 schema version 并同步协议文档与精确回归测试
**And** 不在原版本下静默改变字段含义。

### Story 4.2: 机器读取结构化 AST

As a 自动化工具开发者,
I want 以稳定 JSON 读取 Morva 的结构化 AST,
So that 我可以构建分析、转换和编辑器工具，而不依赖 Rust Debug 文本。

**Acceptance Criteria:**

**Given** 一个有效单文件或目录项目
**When** 执行 `morva parse --format json <file-or-directory>`
**Then** stdout 使用 Story 4.1 的版本化 envelope 输出完整结构化 AST
**And** 成功退出 0、stderr 为空。

**Given** AST 包含 system、声明、成员、表达式或 scenario item
**When** 序列化任意节点
**Then** 节点使用显式、稳定的 `kind` 和语义字段
**And** 不直接把 Rust enum、内部字段名或 Debug 表示当作协议契约。

**Given** AST 节点来自真实源码
**When** 输出其位置
**Then** 节点携带真实 source identity 和文件本地 byte span
**And** 多文件项目不暴露 virtual offset。

**Given** 输入是多文件项目
**When** 输出合并后的 AST 视图
**Then** 声明按确定性的项目文件顺序和文件内源码顺序排列
**And** 合成的项目或 system 外壳被明确标识，不能伪装成某个真实源码位置。

**Given** AST 包含 Epic 3 的分组、取反或析取表达式
**When** 序列化 Boolean AST
**Then** 使用稳定的递归 JSON 结构表示操作符和操作数
**And** 准确保留操作符及完整表达式的源码 span。

**Given** 源码包含兼容容器或 implementation hint
**When** 输出结构化 AST
**Then** 保留其已识别的构造类别、名称和可用结构
**And** 不输出被跳过的原始正文或把非语义文本描述为已解析语义 AST。

**Given** 消费者读取 JSON AST
**When** 使用其进行只读分析
**Then** 文档明确该协议是结构化视图
**And** 不承诺可以无损反序列化为原始 `.morva` 源码。

**Given** 输入存在语法、项目装配或 IO/usage 错误
**When** 执行 JSON parse
**Then** 复用 Story 4.1 的 machine diagnostic 或 error envelope
**And** 分别保持模型错误退出 1、IO/usage 错误退出 2。

**Given** 相同输入和 Morva 版本被重复解析
**When** 比较 JSON 输出
**Then** 输出逐字节确定，不含时间戳、机器绝对路径或目录遍历噪声
**And** JSON 可由标准 parser 完整读取。

**Given** 用户未提供 `--format json`
**When** 执行现有 parse 命令
**Then** 人类可读 AST 输出与退出码保持兼容
**And** core 不依赖 CLI 或 JSON 序列化实现。

**Given** 协议回归套件运行
**When** 覆盖 parse JSON
**Then** 至少包含单文件、多文件、注释、LF/CRLF/CR、兼容容器和递归 Boolean AST
**And** 精确验证 schema version、节点形状、顺序和本地 span。

### Story 4.3: 机器读取语义摘要与能力清单

As a 工具集成开发者,
I want 以 JSON 查询模型摘要和 Morva 能力边界,
So that 工具可以区分已验证、仅识别和不支持的内容。

**Acceptance Criteria:**

**Given** 一个可检查的单文件或目录项目
**When** 执行 `morva inspect --format json <file-or-directory>`
**Then** stdout 使用统一版本化 envelope 输出结构化模型摘要
**And** 成功时退出 0、stderr 为空。

**Given** 模型包含已建模声明、兼容容器、soft behavior 或 implementation hint
**When** 生成 inspect JSON
**Then** 按构造类别输出稳定的计数和条目列表
**And** 已验证、仅识别和未执行内容具有明确不同的分类。

**Given** inspect 条目对应未建模源码内容
**When** 输出该条目
**Then** 包含稳定分类、名称、真实 source identity 和文件本地 span
**And** 不包含被跳过的原始正文。

**Given** 模型只产生 warning
**When** 执行 JSON inspect
**Then** success 保持 true 且进程退出 0
**And** diagnostics 与 summary 使用不同字段，消费者无需解析人类 message 来重建分类。

**Given** 用户不提供模型文件
**When** 执行 `morva capabilities --format json`
**Then** 序列化 Epic 2 Story 2.4 定义的同一 core capability inventory
**And** 成功退出 0，不读取或修改任何 `.morva` 文件。

**Given** 生成 capability JSON
**When** 输出当前语言边界
**Then** 至少覆盖声明、literal、expression/operator、simulation value、七阶段、兼容识别和明确不支持项
**And** human capabilities、JSON capabilities、warning 分类和 inspect 分类不维护重复判断表。

**Given** 消费者检查版本信息
**When** 读取 capability envelope
**Then** 同时获得 JSON protocol schema version 和 capability model version
**And** 文档分别定义两者的含义与兼容性规则。

**Given** 相同 Morva 版本被重复查询或审视相同输入
**When** 比较 JSON 输出
**Then** 字段、分类和数组顺序逐字节确定
**And** 不包含时间戳、随机值或机器环境噪声。

**Given** 模型、IO 或 usage 错误发生
**When** 执行 JSON inspect 或 capabilities 参数无效
**Then** 复用 Story 4.1 的 diagnostic 或 machine error envelope
**And** 保持既有退出码分类。

**Given** 用户未提供 `--format json`
**When** 执行 inspect 或 capabilities
**Then** 现有人类可读输出保持兼容
**And** 人类与机器视图仍消费同一 core 事实。

**Given** 协议回归套件运行
**When** 覆盖本 Story
**Then** 精确验证 clean model、warning-only model、多文件本地映射和能力清单稳定性
**And** 包含能力声明与代表性公开行为一致性的测试。

### Story 4.4: 机器读取七阶段模拟报告

As a 自动化验证工具开发者,
I want 以稳定 JSON 读取场景模拟过程和结果,
So that 我可以准确展示状态变化、失败阶段及源码责任位置。

**Acceptance Criteria:**

**Given** 一个有效项目和可成功模拟的 scenario
**When** 执行 `morva simulate --format json <file-or-directory> <scenario>`
**Then** stdout 只包含一个统一版本化 envelope 和完整 simulation report
**And** stderr 为空、进程退出 0。

**Given** 模拟成功
**When** 读取 JSON report
**Then** report 包含 system、scenario、总体 success、七个有序阶段、各阶段状态、状态变化和最终内存状态
**And** 不需要解析人类文本即可还原模拟结果。

**Given** 输出模拟阶段
**When** 序列化七阶段执行过程
**Then** 顺序固定为现有公开七阶段
**And** 每阶段明确标记 `passed`、`failed` 或 `not_run`。

**Given** report 包含 Boolean、Integer、Decimal-context 或 enum 值
**When** 序列化值和状态变化
**Then** 使用带明确类型标签的稳定结构
**And** 不依赖 Rust Debug 表示或无类型 JSON scalar 推断语义。

**Given** report 包含 entity、field、state entry 或 change
**When** 相同输入重复模拟
**Then** 所有列表按模型和模拟契约确定排序
**And** 输出逐字节一致，不受 hash、时间或机器环境影响。

**Given** requires、effect、ensures、invariant、given 或 expect 失败
**When** 生成失败 report
**Then** 保留已完成阶段、未运行阶段和失败时可观察状态，标识唯一失败阶段
**And** 总体 success 为 false、进程退出 1。

**Given** simulation failure 有源码责任位置
**When** 输出 failure location
**Then** 包含真实 source identity、1-based line/column 和文件本地 byte span
**And** 多文件报告不暴露 virtual offset 或错误归属到 scenario 文件。

**Given** 模拟发生未初始化读取、整数溢出、类型防御性失败或 Epic 3 短路相关行为
**When** 输出结构化 failure 或成功路径
**Then** 保留现有运行时语义、message 和责任 span
**And** 短路未求值分支不得产生虚假 failure。

**Given** scenario 未知、重复或无法选择
**When** 执行 JSON simulate
**Then** 使用统一 machine diagnostic 结构并退出 1
**And** IO、目录发现或 usage 错误使用 machine error envelope 并退出 2。

**Given** JSON simulate 产生任意成功或失败结果
**When** 写入 stdout
**Then** 不混入人类阶段表、marker 或 `PASS/FAIL` 文本
**And** JSON escaping 可安全表示路径、消息和非 ASCII 名称。

**Given** 用户未提供 `--format json`
**When** 执行现有 simulate 命令
**Then** 人类可读七阶段输出、marker 和退出码保持兼容
**And** core simulation 不依赖 JSON 或 CLI 实现。

**Given** 协议回归套件运行
**When** 覆盖 simulation JSON
**Then** 包含成功、各阶段代表性失败、多文件责任映射、状态变化排序、控制字符 escaping 和重复运行确定性
**And** 精确验证 schema version、退出码及 stdout/stderr 边界。

## Epic 5: 只读 AI 校验集成

AI 客户端可以通过标准工具调用检查、解析、查看和模拟 Morva，同时保持人工审核和无外部作用边界。

### Story 5.1: 发现 Morva MCP 服务与能力边界

As a AI 客户端开发者,
I want 通过标准 MCP 发现 Morva 服务及其能力清单,
So that AI 在调用工具前就能理解支持范围和只读约束。

**Acceptance Criteria:**

**Given** Morva workspace 构建 MCP 集成
**When** 引入服务进程
**Then** 新增独立的 `morva-mcp` workspace 边界，它可以依赖 core 和隔离的协议组件
**And** `morva-core` 不依赖 MCP、CLI、JSON-RPC 或异步运行时。

**Given** MCP 客户端启动服务
**When** 通过标准输入输出进行初始化和能力协商
**Then** stdout 只承载协议消息，日志只写入 stderr
**And** 返回稳定的服务名称、版本及协议能力。

**Given** 客户端请求不受支持的协议版本
**When** 服务处理初始化
**Then** 返回符合所选 MCP 协议的结构化错误
**And** 服务不 panic、不输出部分响应。

**Given** 客户端列出并读取 Morva capability resource
**When** 服务生成资源内容
**Then** 内容来自 Epic 2 Story 2.4 的同一 core capability inventory
**And** 明确区分已支持、仅兼容识别和不支持项，并包含 capability model version。

**Given** Epic 4 已提供 JSON capabilities
**When** 比较 MCP resource 与 CLI 机器输出
**Then** 两者对同一能力事实给出等价分类
**And** MCP 层不维护第三份语言判断表。

**Given** AI 客户端读取服务说明
**When** 判断可执行边界
**Then** 说明明确工具不写文件、不执行外部 action、不运行 shell、不访问网络
**And** 所有模型变更必须由人类审核并另行落盘。

**Given** 本 Story 的 MCP 服务正在运行
**When** 客户端尝试请求任意工作区路径或写入操作
**Then** 服务没有对应 resource/tool 能力
**And** 后续模型工具只接受调用方显式传入的 UTF-8 内存 source 集合。

**Given** 服务收到空请求、畸形 JSON-RPC、未知 method、受限极端输入或客户端断开
**When** 处理协议边界
**Then** 不 panic、不污染协议 stdout
**And** 按协议返回错误或安全终止。

**Given** 实现需要 MCP、JSON 或异步依赖
**When** 评审依赖变更
**Then** 依赖只进入集成 crate，并记录版本、维护理由、锁文件变化和供应链范围
**And** 不把协议依赖扩散到 core。

**Given** MCP 真实进程 seam 测试运行
**When** 覆盖 initialize、资源发现、资源读取、未知资源、畸形请求和重复读取
**Then** 精确验证确定性内容及 stdout/stderr 隔离
**And** 相同版本的 capability resource 内容和顺序保持确定。

### Story 5.2: 让 AI 检查内存中的 Morva 项目

As a AI 助手,
I want 检查调用方显式提供的 Morva 源码集合,
So that 我可以发现模型问题并提出建议，而无需读取或修改用户文件。

**Acceptance Criteria:**

**Given** MCP 客户端提供 Morva source bundle
**When** 调用只读 `morva_check` tool
**Then** 输入是非空 `sources` 数组，每项只包含逻辑 source name 和 UTF-8 source text
**And** 服务不接受隐式工作区或目录参数。

**Given** source item 包含 name
**When** 服务验证输入
**Then** name 只作为诊断标识，不解释为本地文件路径、URL、环境变量或 shell 参数
**And** 名称必须非空且在同一请求内唯一。

**Given** 请求包含多个 source
**When** 装配内存项目
**Then** 按规范化 name 的 UTF-8 byte lexical order 确定排列
**And** 重复名称返回 MCP `invalid params`。

**Given** source bundle 验证通过
**When** 执行检查
**Then** tool 直接调用 core 的 project/check API
**And** 不复制 lexer、parser、类型规则、warning 分类或诊断排序逻辑。

**Given** 检查产生成功、warning 或 error
**When** 返回 tool result
**Then** 使用 Epic 4 Story 4.1 的同一结构化 check payload
**And** MCP transport 只负责包装，不重新定义诊断 schema。

**Given** 项目仅包含 warning
**When** `morva_check` 完成
**Then** 返回语言层成功结果和结构化 warnings
**And** 语法、装配或语义 error 返回语言层失败结果，而不是 MCP transport failure。

**Given** 参数形状错误、重复 source name、非 UTF-8 协议输入或资源超限
**When** 服务验证请求
**Then** 返回 MCP `invalid params`
**And** 不把协议输入错误伪装成 Morva 语言诊断。

**Given** 服务执行资源保护
**When** 接收 source bundle
**Then** 最多接受 256 个 source、每个 source name 最多 1 KiB、所有 source text 合计最多 8 MiB
**And** 超限时不开始 parse 或 check。

**Given** 诊断来自内存多文件项目
**When** 返回 source location
**Then** 包含调用方提供的逻辑 source name、1-based line/column 和文件本地 byte span
**And** 不暴露 virtual offset。

**Given** tool 正在处理请求
**When** 请求完成或失败
**Then** 源码只在内存中使用，完成后不缓存、不写临时文件、不读取工作区、不访问网络
**And** 并发请求的源码、诊断和 source identity 相互隔离。

**Given** AI 收到结构化诊断
**When** 生成候选修复建议
**Then** 服务可提供稳定 code、message 和位置作为依据
**And** 不暴露 apply、write、patch、commit 或自动批准工具。

**Given** 相同 source bundle 被重复调用
**When** 比较 structured result
**Then** 内容和顺序保持确定
**And** 不包含进程、时间或工作区环境信息。

**Given** MCP tool 测试运行
**When** 覆盖本 Story
**Then** 包含单文件、多文件跨引用、warning-only、语法/语义错误、重复名称、各资源上限边界和并发隔离
**And** 证明请求没有文件系统副作用。

### Story 5.3: 让 AI 解析并审视内存模型

As a AI 模型评审助手,
I want 获取源码的结构化 AST 和语义覆盖摘要,
So that 我可以解释模型结构，并指出哪些内容尚未被 Morva 验证或执行。

**Acceptance Criteria:**

**Given** MCP 客户端提供 Story 5.2 定义的 source bundle
**When** 列出可用工具
**Then** 服务暴露只读 `morva_parse` 和 `morva_inspect`
**And** 两者复用同一输入排序、校验和资源上限。

**Given** 客户端调用 parse 或 inspect tool
**When** 验证输入
**Then** 只接受逻辑 source name 与内存 source text
**And** 不接受本地路径、URL、shell command 或工作区句柄。

**Given** source bundle 合法
**When** 调用 `morva_parse`
**Then** tool 使用 core project/parse 能力并返回 Epic 4 Story 4.2 的同一结构化 AST payload
**And** MCP 层不复制 parser 或 AST 分类逻辑。

**Given** AST 节点来自源码
**When** 返回 structured result
**Then** 节点包含稳定 `kind`、语义字段、真实 source name 和文件本地 span
**And** 不暴露 Rust Debug 结构或 virtual offset。

**Given** 模型包含兼容容器、implementation hint 或注释
**When** 返回 AST
**Then** 可保留已识别构造的类别、名称和可用结构
**And** 不返回被跳过的原始正文，也不把注释表示为语义节点。

**Given** source bundle 合法
**When** 调用 `morva_inspect`
**Then** tool 使用 core analysis/inspect 能力并返回 Epic 4 Story 4.3 的同一结构化摘要
**And** MCP 层不复制语义覆盖判断。

**Given** inspect 发现兼容或未执行内容
**When** 返回 summary
**Then** 明确区分已验证、仅兼容识别和未执行内容，分类来自同一 capability inventory
**And** warning 与 summary 分字段表达，AI 无需解析人类 message 重建分类。

**Given** 模型存在语法、项目装配或语义问题
**When** parse 或 inspect 完成
**Then** 问题作为 Morva 结构化结果返回
**And** 只有无效参数、协议错误或资源超限属于 MCP input/transport error。

**Given** 输入包含单文件或多个 source
**When** 重复调用 parse 或 inspect
**Then** 结果按规范化 source name 和源码顺序确定排列
**And** AST 与 summary structured result 保持确定。

**Given** tool 请求完成、失败或并发执行
**When** 服务清理请求状态
**Then** 不缓存源码、不访问文件系统、不写文件、不执行模型 action
**And** 一个请求失败不得污染其他请求。

**Given** MCP 集成回归运行
**When** 覆盖本 Story
**Then** 包含递归 Boolean AST、注释、LF/CRLF/CR、兼容容器、soft behavior、多文件映射、warning-only、错误模型和并发隔离
**And** 与对应 Epic 4 CLI JSON payload 进行代表性等价测试。

### Story 5.4: 让 AI 只读模拟业务场景

As a AI 业务规则评审助手,
I want 模拟调用方提供的 Morva 场景并读取结构化报告,
So that 我可以解释规则执行结果和失败原因，而不触发真实业务副作用。

**Acceptance Criteria:**

**Given** MCP 客户端提供 source bundle 和 scenario name
**When** 调用只读 `morva_simulate` tool
**Then** source bundle 复用 Story 5.2 的输入契约
**And** scenario name 是唯一额外的业务参数。

**Given** 服务验证 scenario name
**When** 名称为空、超过 1 KiB 或参数形状无效
**Then** 返回 MCP `invalid params`
**And** 名称不被解释为路径、命令或外部资源标识。

**Given** tool 输入验证通过
**When** 开始模拟
**Then** 调用 core project/simulate API，完整保留静态检查前置条件和现有七阶段顺序
**And** MCP 层不复制 checker 或 simulator 语义。

**Given** 模拟成功或失败
**When** 返回 structured result
**Then** 复用 Epic 4 Story 4.4 的 simulation payload
**And** MCP 层不重新定义阶段、value、change 或 failure schema。

**Given** scenario 成功完成
**When** AI 读取结果
**Then** 获得 scenario、七阶段状态、状态变化和最终内存状态
**And** 进程不产生任何真实业务副作用。

**Given** requires、effect、ensures、invariant、given 或 expect 失败
**When** AI 读取失败结果
**Then** report 保留已完成、失败和未运行阶段、失败时可观察状态及唯一责任阶段
**And** failure 含稳定 message 与源码位置。

**Given** 模拟发生未初始化读取、整数溢出或防御性类型失败
**When** 返回 failure
**Then** 保留 core 的稳定错误语义和责任 span
**And** 不转换为 MCP transport failure。

**Given** scenario 未知或重复
**When** core 无法选择场景
**Then** 返回 Morva 结构化失败结果
**And** 只有参数错误、协议错误和资源超限返回 MCP input/transport error。

**Given** action、entity 和 scenario 分布在不同 source
**When** 模拟失败
**Then** failure 映射到真正负责的 source name、1-based line/column 和文件本地 byte span
**And** 不默认锚定到 scenario source 或暴露 virtual offset。

**Given** 模型使用 Epic 3 的 `!`、`||`、括号或短路
**When** 通过 MCP 模拟
**Then** 结果与 core 和 CLI 的求值、未初始化读取及矛盾边界一致
**And** 未求值的短路分支不产生虚假 failure。

**Given** 模拟执行 effect 或遇到 compatibility implementation hint
**When** tool 运行
**Then** 只修改请求私有的内存状态
**And** 不调用网络、数据库、shell、文件系统或 hint 中描述的外部行为。

**Given** 请求完成、失败或并发执行
**When** 服务清理请求状态
**Then** 不保留模型或模拟状态，重复与并发请求相互隔离
**And** 服务不自动改写源码、不接受模型变更、不声称候选修复已获批准。

**Given** 相同 source bundle 和 scenario 被重复调用
**When** 比较 structured result
**Then** 内容和顺序保持确定
**And** 不包含时间、进程或工作区环境噪声。

**Given** MCP 模拟回归运行
**When** 覆盖本 Story
**Then** 包含成功七阶段、各阶段代表性失败、unknown/duplicate scenario、多文件映射、Boolean 短路、并发隔离和无外部副作用
**And** 与 Epic 4 CLI JSON simulation payload 进行代表性等价测试。
