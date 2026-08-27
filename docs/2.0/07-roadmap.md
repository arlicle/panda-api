# Panda API 2.0 路线图

## 当前里程碑

### M0 — Repository Reset / Architecture Foundation

归档 Panda 0.7，建立 2.0 文档、13-crate Rust 2024 workspace、锁文件和三平台 CI。
M0 不实现业务能力。

## 后续里程碑

| 里程碑 | 交付内容 |
| --- | --- |
| M1 | SourceMap、JSON5 adapter、Typed AST、Canonical Contract、Compiler、Diagnostics、Snapshot |
| M2 | Axum/Tower、Matcher、Scenario Mock、预算、watcher、ArcSwap、last-known-good |
| M3 | Reqwest test runner、assertions、diff、Terminal/JSON/JUnit reports |
| M4 | React/TypeScript Web Workbench、版本化 DTO、Monaco、TanStack |
| M5 | OpenAPI 3.2 import/export、validation、round-trip fixtures |
| M6 | 官方 RMCP、MCP tools/resources、Agent test loop |
| M7 | Panda 0.7 loader、migrator、报告和 golden migration tests |
| M8 | AsyncAPI/Event Contract、WebSocket/SSE 与后续 broker adapters |
| M9 | Core 稳定后的 WASM Component Model plugin boundary/runtime |

## Definition of Done 边界

[架构基线第 96 节](01-architecture.md#96-definition-of-done)同时列出了 Compiler、Mock、
Test、Web、OpenAPI、MCP 和 legacy migration，因此它是 **M0–M7 合并后的 2.0 HTTP
MVP Definition of Done**，不是 M0 单阶段验收条件。

每个后续里程碑必须独立完成 Plan、Implement、Test、Review、Documentation Update、
Commit；禁止一次性实现 M1–M9。
