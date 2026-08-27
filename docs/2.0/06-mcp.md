# MCP 与 Coding Agent

**状态：计划中（M6）**

Panda MCP 将通过官方 RMCP 把 canonical contract、schemas、scenarios、diagnostics 和
test execution 暴露给 Codex 等 coding agents。Panda 不自行实现 MCP wire protocol，
也不硬编码协议协商规则。

M6 才会定义并实现 tools、resources、transport 和 conformance tests。M0 的
`panda-mcp` 只是 crate skeleton，没有 MCP server 或网络能力。
