# Panda DSL 2.0

**状态：计划中（M1）**

Panda JSON5 是原生 authoring format；JSON/YAML 是交换格式，OpenAPI 是行业交换标准。
所有 Panda 2 源文件必须显式声明：

```json5
{
  panda: "2.0",
}
```

M1 将实现 SourceMap、parser adapter、Typed AST、resolver、diagnostics 和 canonical
contract。Schema 语义对齐 JSON Schema 2020-12，Panda 专属 Mock 行为通过 extension
表达，不再把 `posint`、`csentence` 等生成器混入基础 Schema 类型。

当前 `fixtures/panda-2/` 只是设计样本；M0 没有 parser，不能把它的存在视为 DSL 已可用。
字段、Scenario 和工作区示例的权威设计仍以[架构基线](01-architecture.md)为准。
