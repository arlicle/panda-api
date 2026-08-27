# Mock 与 Test

**状态：计划中（M2–M3）**

Scenario 将统一承担文档示例、固定 Mock 响应、后端 API 测试、前端 fixture 和回归测试。
Matcher 必须与 Axum 解耦且结果确定；Mock generator 必须支持 seed 和资源预算。

M2 计划实现 matcher、Scenario matching 和 Mock runtime。M3 计划实现真实 HTTP 请求、
`exact`、`subset`、`schema` assertions，以及 Terminal、JSON、JUnit 报告和可靠退出码。

M0 的 `panda-mock` 与 `panda-test` 不包含 matcher、generator、runner 或 assertion API。
