# Panda API 0.7.4 构建基线

本记录冻结 M0 开始前的旧实现构建状态，不代表 Panda API 2.0 的构建结果。

## 环境与命令

```text
date: 2026-08-27
code baseline: 7deff42
rustc: 1.95.0 (59807616e 2026-04-14)
cargo: 1.95.0 (f2d3ce0bd 2026-03-21)
command: cargo test --locked --all-targets
result: failed, exit code 101
```

旧 binary 项目在 `.gitignore` 中排除了 `Cargo.lock`。本地曾存在一个被忽略的解析结果，
但它不是可复现历史制品，因而没有纳入 compatibility archive。

## 首个阻塞错误

构建 `actix-http 1.0.1` 时，其 cookie 实现尝试在 `time::Tm` 与 Chrono
`TimeDelta` 之间执行加减法，产生 `E0277` 和 `E0308` 类型错误。旧代码、旧依赖声明和
这一失败事实均保持不变；Panda API 2.0 的根 workspace 不构建该 package。

更完整的成因、测试覆盖和安全审计见[项目分析](01-project-analysis.md)。
