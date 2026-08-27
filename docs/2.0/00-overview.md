# Panda API 2.0 概览

**状态：M0 架构奠基**

**版本：2.0.0-alpha.1**

**设计基线：2026-08-27**

Panda API 2.0 的产品定位是 **API Contract Workbench**。Panda Contract 将成为开发者、
前后端、测试、CI、文档系统和 AI Coding Agent 共同使用的 API 事实来源。

目标闭环包括：

- Panda JSON5 Contract 与 JSON Schema 2020-12；
- 文档、Scenario、Mock、真实接口测试和诊断；
- OpenAPI/AsyncAPI 交换；
- Web Workbench 与 MCP Agent Interface；
- Files + Git 作为 Source of Truth，不引入核心数据库。

## 当前可用能力

M0 只建立仓库分层、Rust crate 边界、文档治理、历史归档和 CI。`panda` 目前只支持
`--version`，不能 serve、validate、test、import、export 或启动 MCP。

完整且不可变的设计决策见[架构设计基线](01-architecture.md)，实际版本偏差见
[版本核验与勘误](version-errata.md)，交付顺序见[路线图](07-roadmap.md)。
