# Compiler 与 Runtime

**状态：计划中（M1–M2）**

目标数据流为：

```text
Source Files → Parser → AST → Resolver → Validator → Compiler
             → Immutable WorkspaceSnapshot → Runtime
```

`panda-core` 保持纯内存、无框架依赖。HTTP 请求只读取已编译的 immutable snapshot，
不得在请求期间重新解释 JSON5。M2 才会引入 Axum/Tower、文件 watcher、后台编译、
ArcSwap 和 last-known-good snapshot。

M0 的 `panda-runtime` 与 `panda-workspace` 仅是无依赖 crate skeleton，不提供服务、
文件监听、HTTP DTO 或健康端点。
