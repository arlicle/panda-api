# Panda API 改进路线图

## 1. 目标与原则

路线图以 Panda API 0.7.4 / `7deff42` 为基线，目标是在保留现有 JSON5 DSL 和主要用户工作流的前提下，把项目恢复为可构建、安全、可测试、可持续发布的本地开发工具。

实施原则：

1. 先固定现状，再升级依赖；
2. 安全边界优先于新功能；
3. parser、matcher、Mock generator 的行为先有兼容测试，再做拆分；
4. 框架升级、架构重构和行为修正分开进行，避免一次变更无法定位回归；
5. 当前文档中标为“仅设计”的能力，不以补 bug 的名义静默上线，必须单独设计和版本化。

## 2. 优先级与工作量

| 级别 | 定义 |
| --- | --- |
| P0 | 阻止可靠构建、可能越界读写文件、认证失效或泄露凭据；应在对外试用前完成 |
| P1 | 当前功能的契约错误、CLI 名存实亡、并发阻塞或显著误导用户 |
| P2 | 限制测试、维护和演进速度的架构与工程问题 |
| P3 | 技术栈现代化、产品化验证和长期能力建设 |

| 工作量 | 参考含义 |
| --- | --- |
| S | 约 0.5–2 工程日，边界明确 |
| M | 约 3–7 工程日，需要多模块与测试配合 |
| L | 约 1–3 工程周，涉及迁移、兼容或多平台验证 |
| XL | 需要单独项目计划，不应与普通修复合并 |

工作量是当前仓库信息下的相对估计，不包含产品评审和外部发布审核。

## 3. P0：恢复可信基线与封闭安全边界

### P0-1 可复现构建

**证据**

- 项目是 binary crate，但 `.gitignore` 忽略 `Cargo.lock`；
- `Cargo.toml` 有宽松旧依赖和 `json="*"`；
- Rust 1.95 上 `cargo test --all-targets` 在 `actix-http 1.0.1`/Chrono 类型冲突处失败；
- 无 `rust-toolchain.toml`、CI、MSRV 和发布构建定义。

**实施**

1. 在隔离分支上解析一个能够编译 0.7.4 的最小依赖组合，优先精确约束引发失败的传递依赖；
2. 将 binary 的 `Cargo.lock` 纳入版本控制，删除 `.gitignore` 中对应规则；
3. 添加 `rust-toolchain.toml`，先固定“恢复构建”所用 stable 工具链，并在 README/发布产物记录 MSRV；
4. 删除确认未使用的依赖，去掉 wildcard version；
5. 建立 Linux、macOS、Windows 的 `cargo build --locked`、`cargo test --locked --all-targets`；
6. 恢复基线后再创建 Actix 2 → 4、Tokio 0.2 → 1 等独立升级任务，不把大版本升级塞进基线修复。

**兼容性**

第一阶段不改变 DSL、HTTP 或 CLI，只固定曾经可工作的依赖解析。若必须修复新编译器错误，变更应单独提交并有回归测试。

**验收**

- 全新 checkout 可执行 `cargo build --locked` 和 `cargo test --locked --all-targets`；
- 三个平台使用同一锁文件；
- 重复构建不会因为 registry 中的新版本改变结果；
- 发布 job 产出的 binary 能输出统一版本号和 `--help`。

**工作量：M**

### P0-2 工作区路径解析器

**证据**

文档详情、`_data`、theme、media 和 upload 路由把 query/path 直接交给 `fs::read_to_string`、`NamedFile` 或字符串拼接。当前没有 canonicalize 后的根目录确认。

**实施**

建立唯一的 `WorkspacePathResolver`，所有文件读写都必须通过它：

- API 只接收工作区逻辑相对路径，不接收绝对主机路径；
- URL decode 后拒绝绝对路径、`..`、空段、NUL 和平台前缀；
- canonicalize 目标与允许根目录，拒绝符号链接逃逸；
- 文档详情只允许 Loader 索引中已有的文件 ID，不直接信任 filename；
- `_data` 只允许当前快照记录的模型文件；
- media、theme、upload 分别限定自己的根目录和读取扩展策略；
- 找不到、越界和禁止类型使用明确 4xx，不回显主机绝对路径。

**兼容性**

工作区内的规范相对路径保持有效；曾经依赖工作区外文件的隐式行为应先产生迁移诊断，再在安全版本中禁止。若确有外部模型需求，未来用显式只读 allowlist 配置，而不是接受任意路径。

**验收**

- 普通、percent-encoded、混合分隔符和双重编码的 `../` 均不能逃逸；
- Unix symlink 与 Windows junction/prefix 有平台测试；
- 所有文件 handler 均无直接使用用户 path 打开文件的路径；
- 工作区内 Markdown、模型、media 和主题回归测试通过。

**工作量：M**

参考：[Actix 官方静态文件安全警告](https://actix.rs/docs/static-files/)。

### P0-3 安全上传与资源上限

**证据**

multipart 原始 filename 被直接拼入 `_data/_upload`，无大小、数量、类型、配额或覆盖控制；写入发生在请求处理路径。

**实施**

- 服务器生成 UUID 存储名，原 filename 只作展示 metadata；
- 使用安全临时文件写入，完成后原子 rename；
- 设置请求总大小、单文件大小、字段数、文件数、读取超时和磁盘配额；
- 默认拒绝覆盖；失败时清理临时文件；
- MIME/扩展 allowlist 作为工作区配置，默认只允许明确的开发用途类型；
- `$body/file:url` 返回逻辑 URL，不暴露磁盘路径；
- 为清理策略和上传目录生命周期增加命令或文档。

**验收**

- filename 中的绝对路径、`../`、分隔符、控制字符和同名文件不会越界或覆盖；
- 超限请求在写满内存/磁盘前被拒绝；
- 中断上传不残留可访问半文件；
- URL 仍可供 `$value:"$body/file:url"` 使用。

**工作量：M**

### P0-4 认证 fail-closed 与秘密隔离

**证据**

- `auth:true` 且没有 `_auth.json5` 时验证器返回通过；
- `auth_type`、`auth_place` 被展示但运行时忽略；
- 文档接口序列化认证用户与 token；
- 原始 Authorization 值即使没有 `Bearer ` 前缀也可被当 token。

**实施**

1. `auth:true` 但认证配置缺失/无效时，工作区验证失败；服务若允许带诊断启动，相应 API 必须拒绝请求；
2. 0.7.x 兼容模式明确只支持 `Authorization: Bearer <token>`，其他 `auth_type/place` 给出“不支持”诊断；
3. 在线文档 DTO 默认彻底移除 token、password 和 users 原始对象，只展示组和权限摘要；
4. token 比较避免日志输出与不必要复制；
5. 区分 Mock authentication 与生产认证，在文档和 UI 中持续警告；
6. 未授权使用明确的 401/403 与一致错误结构。

**兼容性**

保持现有 Bearer header 正常工作。body/query token 从未真正工作，不作为兼容能力。需要展示测试 token 时，应通过仅 loopback、显式开关的开发视图提供，默认关闭。

**验收**

- `auth:true` 在缺配置、空组、未知 token、错误 prefix、无权限时全部拒绝；
- deny 始终优先 allow；
- 公共文档 JSON 和前端状态不含 token/password；
- 认证测试覆盖多组、重复 token、通配 URL/method 和文档权限展开。

**工作量：M**

## 4. P1：修正运行时与 CLI 契约

### P1-1 确定性路由与 header 匹配

**问题**

- 缺少 required request header 时仍可能选择接口；
- 精确 URL 后的多个 pattern 使用 `HashMap` 遍历，结果不稳定；
- 相同 URL/method/header 合同没有冲突诊断；
- 文档称“后定义覆盖前定义”，与代码不一致。

**建议**

- 编译工作区时构建有序 route table：exact 优先、pattern 按显式 order 和稳定文件顺序；
- 所有声明 header 必须存在且按规范化值匹配；
- 启动时拒绝无法区分的重复 route contract；
- 在文档 DTO 展示实际 route priority；
- matcher 返回可诊断的“为何每个候选不匹配”。

**验收**

同一 fixture 在不同操作系统和重复加载后选中同一 API；header 缺失、错误和值正确分别有集成测试。

**工作量：M**

### P1-2 请求 body mode 和 form-data

**问题**

配置 `body_mode` 没有被强制执行；text/html/xml/javascript/binary 没有有效 body extractor；`test_data.form_data` 未读取。

**建议**

- 定义并文档化每种 mode 的 Content-Type 和内部值类型；
- JSON 使用 typed/limited body，text/html/xml/javascript 使用受限 bytes/string，binary 使用 bytes metadata 而非强制 JSON；
- multipart 产生 `{fields, files}` 的统一 RequestValues；
- 兼容期同时接受 `test_data.body` 匹配 multipart fields，并新增真正的 `form_data` 语义；若两者同时出现则要求两者各自匹配，不设置隐式覆盖；
- Content-Type 不符时返回 415，而不是静默得到 null body。

**验收**

每种支持 mode 至少有成功、错误 Content-Type、超限和 `test_data` 匹配测试；未支持 mode 在加载阶段报错。

**工作量：L**

### P1-3 response status、headers 与错误模型

**问题**

只支持 `status_code`/`content_type`，旧文档却声称可设置任意 header；错误普遍返回 200；header 名还存在 `content-type` 与 `content_type` 偏差。

**建议**

- 保留 `status_code`、兼容 `content_type`，新增明确的 `headers:{...}`；
- 拒绝 hop-by-hop header、换行和非法名称/值；
- loader 对旧 `content-type` 给出可迁移 warning，而非静默忽略；
- 统一错误 DTO：`code`、`message`、`kind`、可选 `details`；
- 未找到路由 404、method 405、Content-Type 415、认证 401/403、配置不可用 503。

**验收**

HTTP status 与错误 kind 一致；Location、自定义安全 header 和 content type 有集成测试；非法 header 在加载时失败。

**工作量：M**

### P1-4 非阻塞延迟与资源控制

**问题**

`test_data.delay` 使用 `thread::sleep` 阻塞 worker；Mock 长数组、深递归、复杂正则和大文档无硬限制。

**建议**

- handler 改为 async delay；
- 工作区配置统一限定最大 delay、响应字节、数组长度、map 尝试数、递归层数、正则复杂度；
- generator 在预算耗尽时返回结构化错误而非 panic；
- 随机源可注入 seed，便于复现失败。

**验收**

一个延迟请求不阻塞同 worker 的健康请求；所有生成器限制都有边界测试；相同 seed 产生相同输出。

**工作量：M**

### P1-5 把 `panda test` 变成真正的测试器

**问题**

当前只发 GET/POST 并打印；预期 response 未比较，`--all`/`--cron` 无效，其他 method/header/auth/form-data 不支持，bool query 序列化错误。

**建议**

第一阶段先收窄并兑现契约：

- 支持所有声明 HTTP method、query、body、headers 和认证；
- 严格比较固定 `test_data.response`，同时允许显式选择 exact/subset/schema 三种断言模式；默认 exact；
- 单用例输出 pass/fail、差异路径、status 和耗时；任意失败以非零退出；
- 实现 `--all`，没有选择 docs/url/all 时直接参数错误；
- 删除当前未实现的 cron/time positional 参数，或在后续独立版本设计调度，不保留“看似能用”的 flag；
- 报告先提供 terminal + JSON，方便 CI；
- 模糊测试另立设计，不与固定用例断言混合。

**兼容性**

增加 `--legacy-print-only` 作为一个过渡版本的显式选项；默认行为在新的 minor/major 版本切换到失败退出。旧无效 `--cron` 先 warning，下一兼容边界删除。

**验收**

错误 status、body、header 和解析失败均导致非零退出；GET/POST/PUT/PATCH/DELETE 与 auth 有端到端 fixture；JSON 报告 schema 固定并版本化。

**工作量：L**

### P1-6 修正 Mock 类型与文档契约

**问题**

日期默认固定在 2020、`format` 被忽略、派生文本 `length` 不可靠、`posint` 包含 0、image format/accept 无效、未知 type 静默退成 string。

**建议**

- 先把当前行为锁成 legacy golden tests；
- 修复 date/datetime 当前时间范围并读取 format；
- 统一所有长度为明确的 inclusive/exclusive 契约，显式 length 优先；
- 决定 `posint` 是严格 `>0` 还是保留非负，提供兼容 warning；
- 未知 type 在 strict mode 作为错误，legacy mode warning + string；
- 删除没有行为的字段，或完整实现后再从“设计”升级为“支持”。

**验收**

每个 type 有固定 seed、最小/最大、固定值、枚举和错误配置测试；语法参考由这些测试生成或校验默认值表。

**工作量：L**

## 5. P2：建立可维护核心

### P2-1 提取 `panda-core` library

**目标**

把当前 binary-only 工程拆成可测试边界：

```text
panda-core       typed DSL、引用、验证、matcher、generator
panda-workspace  安全文件系统、加载、watcher、snapshot
panda-http       Actix adapter、DTO、limits、CORS、WebSocket
panda-cli        serve、token、validate、test
web/             前端源码与可复现构建
```

第一步可以仍在单 crate 内建立 `lib.rs` 和模块边界，再根据构建速度与发布需求决定是否转 Cargo workspace。

**验收**

- core 不依赖 Actix、notify、dirs 或进程当前目录；
- parser/matcher/generator 可用内存输入单测；
- CLI 和 HTTP 只通过 core public API 访问模型；
- `api.rs`、`db.rs` 不再承担多层职责，单文件控制在可审查范围。

**工作量：L**

### P2-2 类型化 DSL 与诊断

**目标**

- 为稳定字段建立 serde structs/enums；
- 保留 unknown extension map，防止兼容迁移丢字段；
- 使用带源文件、行列、JSON Pointer、严重级别和修复建议的 `Diagnostic`；
- 区分 parse、resolve、validate、compile 四阶段；
- 所有用户输入错误返回 Result，不使用 `unwrap`/panic；
- 新增 `panda validate [path]`。

**验收**

错误引用、错误类型、重复路由、未知字段和危险路径都能定位到文件与字段；一个文件错误不会产生半有效的随机状态。

**工作量：L**

### P2-3 不可变 snapshot 与可靠 watcher

**目标**

- 在锁外构建完整 `WorkspaceSnapshot`；
- 成功后原子替换 `Arc<Snapshot>`；
- 失败保留最后有效版本并发布诊断；
- 对保存事件 debounce/coalesce，一次编辑只编译一次；
- 删除不可达的增量更新代码；只有在基准证明需要时再实现依赖图增量编译。

**验收**

重载不长时间阻塞读取；无效编辑不让服务丢失最后有效文档；连续保存、rename、delete、模型引用更新有跨平台测试。

**工作量：M–L**

### P2-4 测试金字塔与 CI

| 层级 | 必要覆盖 |
| --- | --- |
| parser unit | JSON5、根字段、类型错误、Markdown 配置 |
| resolver unit | global/ref/include/exclude/delete/嵌套路径/循环引用 |
| matcher table | URL/method/header/body/query/test_data 顺序 |
| generator property | type、边界、seed、递归和预算 |
| auth table | group、token、deny/allow、文档展开 |
| HTTP integration | 文档 DTO、文件边界、上传、status/header/CORS |
| CLI integration | help、validate、test exit/report、token |
| end-to-end | 示例工作区启动、前端加载、Mock 请求、热更新 |

CI 应执行 format check、clippy、unit/integration、文档链接、依赖安全审计和三平台构建。不要一开始追求单一覆盖率数字；先要求每条公共契约和每个已修复缺陷都有回归用例。

**工作量：L，随后持续维护**

### P2-5 日志、健康与版本一致性

**目标**

- 用结构化 logging 取代散落 `println!`；
- 日志包含 workspace、文件、事件、route、request ID 和耗时，但不含 token/password；
- 增加 `/__api_docs/health` 和加载诊断端点；
- crate version、CLI version、`Panda-Api` header 和前端显示都来自一个构建版本源；
- 开发 CORS 与对外 CORS 使用显式配置，默认 loopback。

**验收**

运维者能区分“进程存活、快照有效、最后加载失败”；所有界面显示相同版本；敏感字段有日志测试。

**工作量：M**

## 6. P3：现代化与产品演进

### P3-1 升级 Rust 与 Actix

在兼容测试稳定后分阶段升级：

1. 更新可独立升级的 serde、regex、uuid、rand 等；
2. 升级 Tokio/Reqwest 调用；
3. 迁移 Actix Web 2 到当前受支持大版本，更新 extractor、app data、middleware、multipart 和 WebSocket API；
4. 先迁移 Rust 2021，再评估 2024 Edition，使用 `cargo fix --edition` 并分离纯格式提交；
5. 每一步运行完整 DSL golden 与 HTTP integration tests。

当前 Actix 官方入门文档使用 `actix-web = "4"`；Rust 官方说明 edition 是 crate 主动选择且可借助迁移工具完成。版本选择应在实施时再次核对官方 MSRV，不在路线图中硬编码未来 patch version。

参考：[Actix Web Getting Started](https://actix.rs/docs/getting-started/)、[Rust Edition Guide](https://doc.rust-lang.org/edition-guide/editions/transitioning-an-existing-project-to-a-new-edition.html)。

**工作量：L**

### P3-2 恢复前端源码与可复现构建

**实施**

- 优先从原始仓库/历史备份恢复 Vue 源码，不以 source map 反编译结果作为长期源码；
- 恢复 `package.json`、lockfile、lint/test/build 配置和许可证清单；
- 先做到同功能可复现构建，再决定 Vue 2 迁移 Vue 3 + Vite；
- 前端只依赖版本化 DTO，不读取认证 token；
- production 默认不发布包含完整 sourcesContent 的 source map；需要调试时作为独立、受控 artifact；
- 增加菜单、Markdown、API schema、请求测试和 server 切换的组件/E2E 测试。

**验收**

全新 checkout 能构建与当前功能等价的 `_theme`；构建产物由 CI 生成且可追溯；依赖和 source map 发布策略明确。

**工作量：XL**

### P3-3 DSL Schema、版本与迁移器

**实施**

- 为文档根、API、字段、auth、settings 建立机器可读 schema；
- 增加 `dsl_version`，未写时解释为 legacy 0.7；
- `panda validate --strict` 检查未知类型、无效字段、未实现配置和危险上限；
- `panda migrate --to <version>` 只执行可逆或有明确 diff 的转换；
- 语法参考的字段表和示例尽量由 schema/test fixture 校验，减少文档漂移。

**验收**

编辑器可以补全和提示；旧工作区加载时有明确兼容模式；每次 DSL 变化都有迁移说明、fixture 和版本边界。

**工作量：L–XL**

### P3-4 发布与供应链

**实施**

- CI 构建签名的 macOS/Linux/Windows artifacts，生成 checksum 与 SBOM；
- 安装器由当前仓库脚本生成并在干净环境验证；
- 依赖许可证、安全 advisory 和 provenance 自动检查；
- 发布前执行 `--locked` 构建、全测试、示例 smoke test 和文档链接检查；
- 版本、tag、binary 名与变更日志遵循统一规则。

Cargo 官方建议 binary 项目使用 lockfile 保证确定性；见 [Cargo.toml 与 Cargo.lock](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html)。

**工作量：L**

## 7. 建议实施里程碑

```mermaid
flowchart LR
    M0[M0 现状冻结<br/>fixture + 构建诊断] --> M1[M1 安全可用<br/>P0 全部完成]
    M1 --> M2[M2 契约可信<br/>P1 matcher/body/CLI]
    M2 --> M3[M3 核心可维护<br/>lib + typed DSL + snapshot]
    M3 --> M4[M4 技术栈现代化<br/>Actix/Rust/前端]
    M4 --> M5[M5 可发布<br/>schema + 多平台供应链]
```

### M0：冻结兼容样本

- 收集当前仓库示例和历史文章中的有效 JSON5；
- 建立期望加载结构、匹配结果和固定 seed Mock 的 fixture；
- 保存当前失败构建日志与依赖树。

### M1：安全可用

- P0-1 至 P0-4 全部完成；
- 默认 loopback、路径不可逃逸、上传有限制、认证 fail closed；
- 三平台可复现构建。

### M2：契约可信

- 确定性 matcher、body mode、headers/status、非阻塞预算；
- `panda test` 有断言和可靠退出；
- 修正文档与 Mock 类型偏差。

### M3：核心可维护

- core/workspace/http/cli 边界完成；
- typed diagnostics、immutable snapshot、关键测试与 CI 完成。

### M4：技术栈现代化

- Actix/Tokio/Reqwest/Rust Edition 分阶段升级；
- 恢复并升级前端工程；
- 保持 legacy DSL fixture 全绿。

### M5：可持续发布

- DSL schema/version/migrator；
- 签名 artifacts、checksum、SBOM、安装验证和发布文档。

## 8. 兼容与发布策略

- `0.7.x-compat`：只恢复构建、安全和诊断；危险行为允许收紧，但必须有迁移说明；
- 下一 minor：加入 validate、确定性 matcher、真正 test assertion，同时保留显式 legacy 选项；
- 下一 major：删除无效 CLI flag、启用 strict DSL 默认、采用版本化文档 DTO；
- 每个版本发布 `Behavior changes`，逐项列出旧输入、旧输出、新输出和迁移命令；
- 不以“bug fix”名义无提示改变 `test_data`、随机范围、route priority 或 auth 行为。

## 9. 完成定义

当以下条件同时满足，可认为兼容式现代化第一阶段完成：

- 全新环境可从源码可复现构建、测试并生成三个桌面平台的 binary；
- 不可信请求无法读写工作区允许根以外的文件；
- `auth:true` 永不 fail open，公共文档不泄露 token；
- parser、resolver、matcher、generator、auth、HTTP 与 CLI 均有契约测试；
- `panda validate` 能在启动前报告带路径的配置错误；
- `panda test` 能比较预期、报告差异并以退出码服务 CI；
- 前端有源码、lockfile、测试和可复现构建；
- 本套分析、架构、语法和路线图文档与发布行为同步更新。
