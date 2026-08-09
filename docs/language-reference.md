# Morva v0.1 语言参考

本文记录当前实现可依赖的语言表面。设计动机见 [language-design.md](language-design.md)，实现限制见 [implementation-status.md](implementation-status.md)。

## 1. 文件与词法

- 项目约定扩展名：`.morva`；当前 CLI 不强制检查文件后缀。
- 标识符：ASCII 字母或 `_` 开头，后续可含 ASCII 数字。
- 注释：`//` 到行尾。
- 语句主要以换行分隔；部分位置允许 `;`。
- Integer 字面量必须在非负 `i64` 范围（`0..=i64::MAX`）内；当前没有一元负数字面量。
- Boolean 字面量为 `true`、`false`。
- 路径由 `.` 连接，例如 `order.status`。

## 2. 文档结构

```morva
system Shop {
  // declarations
}
```

必须恰好有一个顶层 `system`。system 内可以嵌套声明；任何嵌套 system 都是错误。

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

## 5. 兼容容器

Parser 识别下列声明关键字，但除 `system/entity/enum/action/scenario` 外都只作为兼容容器：

```text
module service event flow lifecycle policy
```

兼容容器保留名称、span 和嵌套声明；其其他内容可能被跳过，不参与专有语义检查或模拟。因此“能解析”不等于“已支持”。

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

这些内容当前不会进入 AST 的行为语义、静态证明或模拟。白名单的目的仅是保持已有模型可解析；拼写错误或其他未知 action 项会失败。

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

当前不支持负整数字面量、Decimal/String/ID 字面量、逻辑运算、函数调用、嵌套对象构造、模块限定名、import、flow/lifecycle 执行、事件、时间、重试执行、IO 或通用代码块。

## 10. 稳定性说明

v0.1 仍属实验阶段。现有示例、测试覆盖的语法、诊断代码和 CLI 契约应视为受保护行为；未被强类型 AST 和测试覆盖的兼容文本不能视为稳定语言承诺。
