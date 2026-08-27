# Panda API 文档中心

Panda API 2.0 的当前文档与 Panda API 0.7 历史事实在这里严格分离。

## Panda API 2.0

当前仓库只完成 **M0 架构奠基**。除 workspace、包边界、版本入口和 CI 外，文档中的
产品能力均仍为设计，不代表已经可运行。

1. [产品概览与当前状态](2.0/00-overview.md)
2. [架构设计基线](2.0/01-architecture.md)
3. [Panda DSL 2.0](2.0/02-dsl.md)
4. [Compiler 与 Runtime](2.0/03-runtime.md)
5. [Mock 与 Test](2.0/04-mock-and-test.md)
6. [安全模型](2.0/05-security.md)
7. [MCP 与 Agent](2.0/06-mcp.md)
8. [路线图](2.0/07-roadmap.md)
9. [版本核验与勘误](2.0/version-errata.md)

## Panda API 0.7

[0.7.4 历史文档](legacy/0.7/README.md)绑定代码基线 `7deff42`，用于行为审计、
fixtures 和未来迁移器，不会被改写成 2.0 已实现能力。

## 状态约定

| 状态 | 含义 |
| --- | --- |
| **M0 已建立** | 仓库、文档或构建边界已经存在。 |
| **计划中** | 架构已经定义，但没有可执行实现。 |
| **Legacy** | 只描述 Panda API 0.7.4 历史行为。 |

架构正文是带日期的不可变设计基线；依赖或标准出现变化时，只在勘误和对应里程碑
ADR 中记录，不静默重写历史判断。
