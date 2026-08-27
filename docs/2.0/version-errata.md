# 版本核验与勘误

本页记录 2026-08-27 架构基线发布后的事实核验与候选变化。它不修改
[原始架构正文](01-architecture.md)，也不代表尚未开始的里程碑已经完成依赖选型。

## M0 已锁定

| 项目 | 选择 | 核验 |
| --- | --- | --- |
| Rust | `1.98.0` | 2026-08-20 正式发布；M0 `rust-toolchain.toml` 精确固定 |
| Rust Edition | `2024` | 所有 2.0 workspace package 统一使用 |
| OpenAPI target | `3.2.0` | 仍是正式规范；M5 才实现 adapter |

参考：[Rust 1.98.0 发布公告](https://blog.rust-lang.org/releases/latest/)、
[OpenAPI 3.2.0 规范](https://spec.openapis.org/oas/v3.2.0.html)。

## 已发现但本阶段不选择

| 项目 | 架构正文记录 | 2026-08-27 registry 状态 | 处理 |
| --- | --- | --- | --- |
| `jsonschema` | `0.50.x` | `0.52.0` stable | 0.x minor 可能包含 breaking change；M1 通过独立 ADR、PoC、MSRV/license/advisory 检查后选择 |
| TypeScript | `6.0` | `7.0.2` latest | major version 已变化；M4 比较迁移成本后通过独立 ADR 选择 |

参考：[jsonschema crate](https://crates.io/crates/jsonschema)、
[TypeScript package](https://www.npmjs.com/package/typescript)。

M0 不引入上述依赖，也不创建前端 package，因此没有隐式锁定未来选择。
