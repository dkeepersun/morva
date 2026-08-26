# MCP 集成

`morva-mcp` 是独立 workspace crate 提供的只读 MCP stdio 服务器。`morva-core`
不依赖 MCP、CLI、JSON-RPC 或异步运行时；协议侧完全由集成 crate 承担，且保持
std-only——JSON-RPC 帧解析使用仓库自带的最小 JSON parser（嵌套深度与消息大小
有界），响应复用 core 的 canonical JSON writer 的紧凑单行形式。

## 边界

服务器在 initialize `instructions` 中声明并由实现强制：不读写文件、不执行外部
action、不运行 shell、不访问网络。模型工具只接受调用方显式传入的内存 UTF-8
source bundle；不存在写盘、patch、commit 或自动批准工具，所有模型变更必须由
人类审核并另行落盘。stdout 只承载协议消息。

## 协议表面

- `initialize`：支持 `2024-11-05`、`2025-03-26`、`2025-06-18`；不支持的版本返回
  `invalid params` 结构化错误并列出支持版本。畸形 JSON 返回 `-32700`，未知
  method 返回 `-32601`，未知 resource 返回 `-32002`，参数错误返回 `-32602`；
  notifications 不产生响应，任何协议边界都不 panic、不污染 stdout。
- `resources/list` / `resources/read`：唯一资源 `morva://capabilities`，内容与
  `morva capabilities --format json` 逐字节相同（同一 core 能力清单与
  capability model version，无第三份判断表）。
- `tools/list` / `tools/call`：`morva_check`、`morva_parse`、`morva_inspect`、
  `morva_simulate`。输入为 `sources: [{name, text}]`（simulate 另加
  `scenario`）。`name` 只是诊断标识，不解释为路径、URL 或命令。

## Source bundle 契约

最多 256 个 source；`name` 非空、≤1 KiB、同一请求内唯一；全部 `text` 合计
≤8 MiB；超限或形状错误返回 MCP `invalid params`，不开始 parse。装配前按 name
的 UTF-8 byte 顺序确定排序，然后直接调用 core 的 `Project::parse`/`check`/
analysis/simulate——MCP 层不复制任何语言规则。

## 结果

Tool 结果的 text 内容是与 CLI `--format json` 完全相同的 `morva.cli`
envelope（由共享的 `morva-machine` crate 生成）：语法/装配/语义 error 与模拟
失败都是语言层结果（`success: false`，`isError: false`），只有参数、协议错误
和资源超限才是 MCP 层错误。诊断与失败位置携带调用方的逻辑 source name、
1-based line/column 与文件本地 byte span，不暴露 virtual offset。相同输入的
输出逐字节确定，不包含时间、进程或环境信息；请求之间无状态残留。
