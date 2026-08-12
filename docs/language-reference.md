# Morva v0.1 语言参考

本文记录当前实现可依赖的语言表面。设计动机见 [language-design.md](language-design.md)，实现限制见 [implementation-status.md](implementation-status.md)。

## 1. 文件与词法

- 单文件输入不强制检查后缀。目录输入只选择直接子级中扩展名精确为小写 `.morva` 的普通文件，忽略其他文件、子目录和 symlink，并按 UTF-8 文件名字节序装配。
- 标识符：ASCII 字母或 `_` 开头，后续可含 ASCII 数字。
- LF、CRLF、CR 各表示一个逻辑换行；CRLF 虽占两个 byte，但只作为一个分隔符。混合使用时也按每个序列一行计数，源码 span 不经过换行规范化。
- 行注释：`//` 到任一受支持换行或 EOF 之前。
- 块注释：`/* ... */`，可跨行并嵌套，例如 `/* outer /* inner */ outer */`。块内的 LF、CRLF、CR 仍作为语法分隔换行，CRLF 不双计；注释正文不进入 AST。所谓与空白模型等价，是把非换行正文替换为空白并保留这些内部逻辑换行，不是无条件删除整个注释。
- 注释只能出现在 token 之间，不能把 identifier 写成 `Or/*...*/der`、把 Integer 写成 `1/*...*/0`，也不能把现有复合 operator `==`、`!=`、`>=`、`<=`、`+=`、`-=` 写成由无换行注释隔开的两部分；连续无换行块注释同样不能绕过。该规则在 parser 会跳过的 compatibility、soft behavior 和 implementation hint 内容中同样生效；失败报告 `MORVA1025: comment cannot split a token` 并标记触发的 `/*` 两个 byte。块内有逻辑换行或两侧本来不会形成同一个现有 token 时不报此错，而是保留该换行的语法分隔作用。
- 未闭合块注释报告 `MORVA1024`，span 是最外层 `/*` 的两个原始 byte；嵌套层也未闭合时仍只指向最外层 opener。
- 语句主要以逻辑换行分隔；部分位置允许 `;`。
- Integer 字面量必须在非负 `i64` 范围（`0..=i64::MAX`）内；当前没有一元负数字面量。
- Boolean 字面量为 `true`、`false`。
- 路径由 `.` 连接，例如 `order.status`。

## 2. 文档结构

```morva
system Shop {
  // declarations
}
```

较长的人工评审说明可以使用块注释：

```morva
/* Why this transition exists.
   /* Nested detail stays inside the outer comment. */
*/
system Shop { /* declarations */ }
```

必须恰好有一个顶层 `system`。system 内可以嵌套声明；任何嵌套 system 都是错误。

一个系统也可平铺到多个文件；每个文件保留一个同名 system 外壳：

```morva
// 10-types.morva
system Shop { entity Order { status: OrderStatus } }

// 20-actions.morva
system Shop { action Confirm(order: Order) {} }
```

装配只合并根 system 的子声明，之后继续使用全局短名解析。文件之间没有 import、模块作用域或 container reopening 语义。

## 3. 强类型声明

### Enum

```morva
enum OrderStatus {
  Pending
  Confirmed
  Cancelled
}
```

成员名在 enum 内唯一。裸成员只在表达式另一侧或赋值目标提供明确 enum 类型时解析。

### Entity

```morva
entity Order {
  id: ID
  status: OrderStatus
  attempts: Integer
  invariant attempts >= 0
}
```

当前内建类型名：`Bool`/`Boolean`、`Decimal`、`ID`/`Id`、`Int`/`Integer`、`String`。静态检查把 `Bool`/`Boolean`、`Int`/`Integer` 和 `ID`/`Id` 分别视为同一 canonical 类型；模拟值仍只支持 Boolean 和 Integer 两组。

### Action

```morva
action Confirm(order: Order) {
  requires order.status == Pending
  effects order.status = Confirmed
  ensures order.status == Confirmed
}
```

无参 action 可写成 `action Refresh {}` 或 `action Refresh() {}`。

谓词条款：`requires`、`ensures`、`invariant`。effect 条款只接受赋值。条款可写单行或块：

```morva
requires {
  order.status == Pending
  order.attempts < 3
}

effects {
  order.status = Confirmed
  order.attempts += 1
}
```

赋值操作符为 `=`、`+=`、`-=`。effect 目标必须从 action 参数开始并指向字段。`=` 要求目标与值属于同一 canonical 标量或同一 enum；`+=/-=` 的目标和值都必须是 Integer。

### Scenario

```morva
scenario NormalConfirmation {
  given order.status = Pending
  given order.attempts = 0
  run Confirm(order)
  expect order.status == Confirmed
}
```

结构必须严格为 `given* → 恰好一个 run → expect+`。given 只支持 `=`；run 实参按位置绑定 action 的 entity 参数且必须互异。scenario 值仅支持 enum member、Boolean、Integer。

## 4. 表达式

当前表达式形态：

```text
integer
boolean
path
left == right
left != right
left > right
left >= right
left < right
left <= right
```

没有算术表达式、逻辑连接、括号优先级、字符串字面量、调用表达式或集合。二元表达式不递归组合；不要根据未来预期自行扩展 grammar。

谓词必须产生 Boolean。`==/!=` 要求两侧是同一 canonical 标量或同一 enum；`<`、`<=`、`>`、`>=` 只接受 Integer 或 Decimal。非负 Integer 字面量在明确的 Decimal 比较或赋值上下文中是精确 Decimal 常量，例如 `balance >= 0` 和 `effects account.balance = 0` 合法；Integer 路径与 Decimal 路径之间没有隐式转换。Entity 只能作为字段路径中间类型，不能整体比较或赋值。

### 明显矛盾检查

引用和类型检查成功后，checker 会保守识别 action 中无需符号执行即可确定的字面量矛盾：恒假的 Boolean/Integer 常量比较、同一状态阶段中同一路径的互斥 `==`/`!=` 精确值事实，以及最终已知直接字面量 `=` effect 与后置事实的冲突。`requires + action invariant` 形成前态事实组，`action invariant + ensures` 形成独立的后态事实组；合法的前后状态变化不会被合并。

Effects 仍按源码顺序解释。每个路径最后一次直接字面量 `=` 可形成已知最终值；非字面量 `=` 或 `+=/-=` 会把该路径降为未知，后续直接字面量 `=` 可以恢复已知。当前不做有序区间求解、entity invariant 参数实例化、scenario/action 内联、未写路径跨阶段推理或 compound effect 折叠，因此未报告不代表已证明可满足。

## 5. 兼容容器

Parser 识别下列声明关键字，但除 `system/entity/enum/action/scenario` 外都只作为兼容容器：

```text
module service event flow lifecycle policy
```

兼容容器保留名称、span 和嵌套声明；其其他内容可能被跳过，不参与专有语义检查或模拟。因此“能解析”不等于“已支持”。

显式 analysis 和 CLI `check` 会为每个兼容容器报告 `MORVA5001` warning，指向容器名称并说明其未被专有语义验证。warning 不改变旧 `check()` error API、模拟语义或成功退出码；被跳过的正文仍不会回显或进入求值。

## 6. 软行为与实现提示

Action 中可出现：

```morva
atomic
idempotent by order.id
timeout 10
retry 2
implementation_hint {
  storage: relational
  consistency: strong
}
```

这些内容只以 `SoftBehavior { kind, span }` provenance 进入 AST；参数、路径和 block 正文仍被丢弃，不进入行为语义、静态证明或模拟。显式 analysis 和 CLI `check` 会为每个源码项报告一个 `MORVA5002` warning，span 只覆盖其 keyword，消息说明该项未被语义验证或模拟执行。旧 `check()` API、`parse`/`inspect`/`simulate` 输出和成功退出码不变；拼写错误或其他未知 action 项仍会失败。

Line soft item 从行首白名单 keyword 延续到逻辑换行或 action `}`，其中再次出现白名单词不会产生额外 warning。每个 `implementation_hint` block（包括嵌套 block）是一个 item。

## 7. 名称规则

- 声明、字段、参数和 enum member 在各自检查范围内必须唯一。
- 类型短名当前全局解析；跨容器同名类型是歧义。
- action 与 scenario 名必须各自在全局唯一，以支持无模块限定的模拟选择。
- 语言关键字、行为关键字和 Boolean 字面量不能用作普通声明名。
- `given`、`run`、`expect` 是 scenario 项开头的上下文关键字，在其他名称位置可合法使用。

## 8. 模拟语义

模拟仅解释直接 entity 字段状态，值为 enum、Boolean 或 Integer。状态不跨 scenario 保存。所有读取都要求此前由 given 或 effect 初始化。

Effects 按源码顺序生效，Integer `+=/-=` 使用 checked arithmetic。失败不回滚内存报告中的已发生变化，但不会提交到任何外部系统。

## 9. 不受支持的语法/语义

当前不支持负整数字面量、Decimal/String/ID 字面量、逻辑运算、函数调用、嵌套对象构造、模块限定名、import、递归项目发现、manifest、flow/lifecycle 执行、事件、时间、重试执行、IO 或通用代码块。

## 10. 稳定性说明

v0.1 仍属实验阶段。现有示例、测试覆盖的语法、诊断代码和 CLI 契约应视为受保护行为；未被强类型 AST 和测试覆盖的兼容文本不能视为稳定语言承诺。
