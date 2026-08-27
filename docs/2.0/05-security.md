# 安全模型

**状态：架构已定义，实现计划中**

Panda API 2.0 的安全边界从 M1/M2 实现开始即为必需条件：

- 所有文件访问必须经过 `WorkspacePathResolver` 和显式 capability；
- 默认 local mode 只监听 loopback；public/team mode 必须显式启用安全策略；
- 上传不得用原 filename 构造磁盘路径，并必须有字节、数量、时间和磁盘配额；
- secret 只能来自受控环境或 secret source，且不得进入 DTO、日志、MCP、报告或缓存；
- parser、schema、Mock 和请求 body 都必须有资源预算；
- 诊断和 health response 不得泄露绝对路径或秘密。

0.7 归档中的路径穿越、上传和认证风险不会在 compatibility package 内修补。该 package
只作为历史证据，不属于 2.0 安全边界。
