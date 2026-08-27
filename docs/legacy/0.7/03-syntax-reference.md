# Panda API 0.7.4 语法参考

## 1. 适用范围

本文描述提交 `7deff42` 的实际 DSL、目录、CLI 和运行时行为。示例以 `.json5` 为主；若旧教程与本文冲突，以本文和当前代码为准。

状态含义：

- **已实现**：当前存在完整执行路径；
- **部分实现**：可使用，但配置或边界不完整；
- **仅设计**：旧文章提过，当前没有实现；
- **已知缺陷**：实现存在明显错误或与文档不一致。

## 2. 最小可用示例

在一个独立目录中创建 `auth.json5`：

```json5
{
  name: "用户接口",
  desc: "登录和退出",
  order: 1,
  apis: [
    {
      name: "用户登录",
      method: "POST",
      url: "/login/",
      body_mode: "json",
      body: {
        username: { name: "用户名" },
        password: { name: "密码" },
      },
      response: {
        code: { name: "状态码", type: "int", enum: [-1, 1] },
        msg: { name: "消息", type: "csentence" },
        token: { name: "令牌", type: "string", required: false, length: 32 },
      },
      test_data: [
        {
          name: "密码错误",
          body: { username: "edison", password: "bad" },
          response: { code: -1, msg: "密码错误" },
        },
        {
          name: "登录成功",
          body: { username: "edison", password: "123456" },
          response: {
            code: 1,
            msg: "登录成功",
            token: { $mock: true, required: true },
          },
        },
      ],
    },
  ],
}
```

在该目录运行服务后，访问 `http://127.0.0.1:9000/`。请求值完全匹配 `test_data` 时返回对应固定 response；否则按 `response` 字段模型生成随机 Mock。

> 当前仓库源码在现代工具链上无法直接构建，安装命令和二进制发布状态需先按[改进路线图](04-improvement-roadmap.md)恢复。本节只说明工作区与 DSL 的运行方式。

## 3. JSON5 基础语法

Panda API 通过 `json5` crate 解析文档。相较严格 JSON，常用能力包括：

```json5
{
  // 单行注释
  unquoted_key: '允许单引号',
  hex: 0x10,
  leading_decimal: .5,
  trailing_decimal: 5.,
  list: [1, 2, 3,], // 允许尾逗号
}
```

### 3.1 字符串换行

标准 JSON5 的多行字符串应在换行前使用反斜杠：

```json5
{
  desc: "第一行\
第二行",
}
```

当前 Loader 还会在解析前用正则改写冒号后的引号字符串，尝试兼容旧教程中的原始换行。这个预处理无法正确理解所有转义、嵌套和引号组合，属于**部分实现**；新文档应优先使用 `\n`、反斜杠续行，或把长说明放入 Markdown。

### 3.2 常见错误

- JSON5 中的正则反斜杠仍需转义，例如 `"^1[3-9]\\d{9}$"`；
- 示例中的 key 若包含 `/`、`-`、空格或其他非标识符字符，应使用引号；
- `NaN` 和 `Infinity` 是 JSON5 语法，但转换为 JSON HTTP 响应时并不适合作为可移植值；
- 字段类型错误经常触发 `unwrap`，当前不会稳定地产生友好的配置诊断。

## 4. 工作区目录

Panda API 把启动命令所在目录作为工作区根目录。

```text
api-docs/
├── README.md                  # 项目首页说明
├── _settings.json5            # 项目与全局配置
├── _auth.json5                # Mock 认证与权限
├── auth.json5                 # API 文档
├── $2文章/
│   ├── $_folder.md            # 可选的父菜单说明
│   ├── $1文章说明.md
│   └── $2article.json5
├── _data/
│   ├── models.json5           # 被 $ref 的模型
│   └── _upload/               # 运行时上传文件
├── media/                     # Markdown 可访问媒体
└── _theme/                    # 可选的工作区主题覆盖
```

| 名称 | 状态 | 用途与限制 |
| --- | --- | --- |
| `README.md` | **已实现** | 项目首页；不会进入普通 Markdown 菜单 |
| `_settings.json5` | **已实现** | 优先配置文件；代码也尝试 `_settings.json` |
| `_auth.json5` | **已实现** | 推荐且实际可靠的认证文件名 |
| `_auth.json` | **已知缺陷** | 代码列出此 fallback，但 `_auth.json5` 不存在时会提前返回，实际不会加载 |
| `*.json5` | **已实现** | 递归扫描为 API 文档，隐藏目录与 `_data/` 除外 |
| `*.md` | **部分实现** | 形成菜单；无文首配置时内容 filename 可能为空，建议始终写配置块 |
| `_data/` | **已实现** | 模型和上传数据，不作为 API 文档扫描 |
| `media/` | **已实现** | 通过 `/media/...` 访问；旧文章所称 `/static/...` 路由未注册 |
| `.gitignore` | **部分实现** | 扫描和 watcher 使用的忽略来源；解析错误缺少统一诊断 |

不要在不可信网络中使用当前文件接口；路径约束问题见[项目分析](01-project-analysis.md#6-安全分析)。

## 5. `_settings.json5`

```json5
{
  project_name: "My API",
  project_desc: "项目简述",
  global: {
    apis: {
      base_path: "/v1/",
      auth: true,
      response: {
        code: { name: "状态码", type: "int" },
        msg: { name: "消息" },
      },
    },
  },
  servers: {
    dev: {
      name: "开发服务器",
      desc: "本地开发",
      url: "http://127.0.0.1:8080",
    },
  },
}
```

### 5.1 根配置

| 字段 | 类型 | 默认值 | 当前用途 |
| --- | --- | --- | --- |
| `project_name` | string | `Panda api docs` | 在线文档标题 |
| `project_desc` | string | `""` | 项目简介 |
| `global` | object | `null` | API 全局默认配置 |
| `servers` | object | 无 | CLI `test --server` 和内置前端的环境选择 |

根 `README.md` 存在时优先作为首页内容；不存在时回退到 `project_desc`，再回退到 `Panda api docs`。

### 5.2 `global.apis` 支持范围

**已实现**的全局标量：`name`、`desc`、`url`、`base_path`、`method`、`body_mode`、`auth`。

**已实现**的全局结构：`response`。全局 response 字段先加载，接口级 `$ref` 和当前接口可以覆盖或删除它们。

`body`、`query`、`url_param`、`request_headers`、`response_headers`、`test_data` 不从 `global.apis` 加载。不要把旧文章中的“所有字段都可全局继承”理解成通用深合并。

### 5.3 `base_path`

```json5
global: {
  apis: {
    base_path: "/v2/",
  },
}
```

最终 URL 由 `base_path.trim_end_matches("/") + url` 直接拼接。为了得到 `/v2/login/`，接口 `url` 应以 `/` 开头。接口可通过自己的 `base_path` 覆盖全局值；设置字符串 `"$del"` 可回退为空默认值。

### 5.4 `servers`

`servers.<name>.url` 被 `panda test --server <name>` 使用。`name`、`desc` 主要供前端展示。Rust 服务不会把未匹配的业务请求反向代理到该 server。

## 6. API 文档根对象

```json5
{
  name: "文章接口",
  desc: "文章增删改查",
  order: 20,
  define: {
    model: "./_data/models.json5",
    article: "./_data/models.json5:Article",
  },
  apis: [
    // API 对象
  ],
}
```

| 字段 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `name` | string | 文件路径 | 文档和菜单名称；空字符串也会回退到文件路径 |
| `desc` | string | `""` | 支持 Markdown 展示 |
| `order` | integer | `0` | 菜单排序；非整数会 panic |
| `define` | object | 无 | 为 `$ref` 路径定义短名称 |
| `apis` | array | `[]` | API 列表；非数组会得到空列表 |

接口文件必须是 `.json5` 才会被工作区扫描。模型引用文件可以是 JSON5 内容且路径可以使用其他扩展名，但推荐统一使用 `.json5`。

## 7. API 对象

```json5
{
  name: "查看文章",
  desc: "按 id 查看",
  method: "GET",
  url: "/posts/{id:\\d+}/",
  url_param: {
    id: { name: "文章 ID", type: "posint" },
  },
  auth: false,
  body_mode: "json",
  body: null,
  query: {
    preview: { name: "预览模式", type: "bool", required: false },
  },
  request_headers: null,
  response_headers: {
    status_code: 200,
    content_type: "application/json",
  },
  response: {
    id: { name: "文章 ID", type: "posint" },
    title: { name: "标题" },
  },
  test_data: [],
}
```

| 字段 | 类型 | 默认值 | 状态与行为 |
| --- | --- | --- | --- |
| `name` | scalar | 文档文件名 | **已实现**，非字符串会格式化为文本 |
| `desc` | scalar | `""` | **已实现** |
| `url` | scalar | `""` | **已实现**；应避免空 URL |
| `base_path` | scalar | `""` | **已实现**；与 url 直接拼接 |
| `method` | string/array | `GET` | **已实现**；加载后统一大写，`*` 表示任意 method |
| `url_param` | model | `null` | **已实现**为文档数据；不参与 path 值校验 |
| `auth` | bool | `false` | **部分实现**；详见[认证](#15-认证与权限) |
| `body_mode` | string | `json` | **部分实现**；配置值不被强制校验 |
| `body` | model | `null` | **已实现**为文档/Mock schema 和 JSON 测试数据模型 |
| `query` | model | `null` | **已实现**；还控制 query 字符串的类型转换 |
| `request_headers` | model | `null` | **已知缺陷**；缺少 header 时仍可能匹配 |
| `response_headers` | model | `null` | **部分实现**；只识别 status 与 content type |
| `response` | 任意 JSON 值 | 空对象 | **已实现**；固定数据未匹配时作为 Mock schema |
| `test_data` | array | `null` | **已实现**；非数组记录错误但继续加载 |
| `$ref` | string | 无 | **已实现**；可以复用整个 API 对象 |

### 7.1 `method`

```json5
method: "POST"
method: ["GET", "POST", "PATCH"]
method: "*"
method: "WEBSOCKET"
```

Loader 不限制 HTTP method 名称，只统一转成大写并与实际请求比较。`WEBSOCKET` 还有特殊启动行为，见[WebSocket](#17-websocket)。

### 7.2 URL pattern

URL pattern 交给旧版 Actix `ResourceDef`：

```json5
url: "/posts/{id:\\d+}/"
```

精确 URL 优先；精确查找失败才遍历 pattern。多个 pattern 同时命中时没有稳定优先级，应避免重叠模式。

### 7.3 `body_mode`

历史文档列出 `json`、`form-data`、`text`、`html`、`xml`、`javascript`、`binary`。当前实际行为：

| mode | 状态 |
| --- | --- |
| `json` | JSON extractor 可提供 `Value`，但配置值本身不参与验证 |
| `form-data` | 实际请求 Content-Type 是 multipart 时进入上传解析 |
| `text`/`html`/`xml`/`javascript` | 能识别部分 Content-Type 字符串，但没有读取原始 body 的 extractor |
| `binary` | 没有实现 |

运行时根据请求的实际 `Content-Type` 决定解析方式，并不检查它是否等于 API 配置的 `body_mode`。

### 7.4 `request_headers`

两种值写法都能读取：

```json5
request_headers: {
  Accept: "application/vnd.example+json; version=2",
  "x-tenant": { value: "demo" },
}
```

请求中存在 header 但值不相等时会跳过该 API；请求完全缺少 header 时，当前代码不会跳过，属于**已知缺陷**。不要把它当成可靠的访问控制。

### 7.5 `response_headers`

```json5
response_headers: {
  status_code: 202,
  content_type: "application/json",
}
```

- `status_code` 支持整数或 `{value: 202}`，有效范围 100–599，否则回退 200；
- `content_type` 支持字符串或 `{value: "..."}`；
- 默认 status 200、content type `application/json`；
- 旧教程中的 `"content-type"` 不会被读取；
- `Location` 等任意响应 header 没有实现。

## 8. 字段模型基础

简单字段通常写成：

```json5
title: {
  name: "标题",
  desc: "文章标题",
  type: "ctitle",
  required: true,
  default: null,
  value: null,
  enum: [],
}
```

| 属性 | 当前行为 |
| --- | --- |
| `name` | 文档名称；存在 string `name` 时该对象会被识别为简单字段 |
| `desc` | 文档说明 |
| `type` | 简单字段类型；不区分大小写 |
| `required` | 对简单 response Mock，`false` 时约 50% 概率省略；不验证请求必填 |
| `default` | 只作为文档数据传递，Mock 和请求匹配不使用 |
| `value` | Mock 固定返回该值，优先于 enum 和类型生成 |
| `enum` | 从数组中随机选择原始元素 |
| `$enum` | 从二维数组中随机选择元素的第一个值 |

优先级：

```text
required 随机省略检查 → value → $value → enum → $enum → 按 type 生成
```

`required:false` 对 object、array、map、rec 等复杂节点不会执行相同的随机省略逻辑。`default` 不会在字段缺失时自动补值。

### 8.1 `enum` 与 `$enum`

```json5
code: { type: "int", enum: [-1, 1] }
code: { type: "int", $enum: [[-1, "失败"], [1, "成功"]] }
```

两者生成值都是 `-1` 或 `1`。若写成 `enum: [[-1, "失败"], [1, "成功"]]`，生成值则是整个子数组。格式错误的 `$enum` 可能返回 null 或 panic，当前没有 schema 校验。

### 8.2 `$value`

`$value` 可以把请求数据复制到 Mock response：

```json5
response: {
  username: { name: "用户名", $value: "$body/username" },
  page: { name: "页码", $value: "$query/page" },
  upload_url: { name: "上传地址", $value: "$body/file:url" },
}
```

路径使用 JSON Pointer 风格，以 `/` 分隔。不存在时该 response 字段被省略。`:url` 是 multipart 文件上传的内部特殊值，只对上传字段有效。

## 9. 数字、布尔和时间类型

所有类型名加载后转成小写。

| type | 生成值 | 特有属性 | 当前默认/限制 |
| --- | --- | --- | --- |
| `number` / `int` | i64 整数 | `min_value`, `max_value` | 两端未设置时使用 i64 范围；上界为随机区间排除端 |
| `posint` | 非负整数 | 同上 | 默认最小值 0，因此并非严格正整数 |
| `negint` | 负整数 | 同上 | 默认最大上界 0 |
| `float` | f64 | 数值范围与小数位 | 默认固定 2 位 |
| `posfloat` | 非负小数 | 同上 | 最小值 0 |
| `negfloat` | 负小数 | 同上 | 最大上界 0 |
| `timestamp` | Unix 秒数 | `min_value`, `max_value` | 默认当前时间之前约两年到现在 |
| `bool` | boolean | 无 | 随机 true/false |
| `date` | string | `min_value`, `max_value` | 输出 `%Y-%m-%d` |
| `datetime` | string | `min_value`, `max_value` | 输出 `%Y-%m-%d %H:%M:%S` |

### 9.1 浮点小数位

```json5
money: {
  name: "金额",
  type: "float",
  min_value: 0,
  max_value: 1000,
  decimal_places: 2,
}
```

- `decimal_places` 设置固定小数位；
- 未设置任何小数位属性时默认 `2`；
- 若使用随机位数，`min_decimal_places` 缺省为 `2`，`max_decimal_places` 缺省为 `16`；
- 极大小数位可能发生幂运算溢出或精度问题。

### 9.2 date/datetime 已知缺陷

旧文档声明支持 `format`，当前 API 生成器没有读取字段中的 `format`。更严重的是，当 min/max 都缺省或最大值不大于最小值时，函数回退到固定的 `2020-02-02 14:16:18 UTC`，而不是文档声称的相对当前时间随机值。

因此 date/datetime 是**已知缺陷**；若业务依赖精确日期，优先用 `value`、`enum` 或 `test_data`。

## 10. 字符串与专用类型

| type | 别名 | 预期输出 | 代码中的默认范围/行为 |
| --- | --- | --- | --- |
| `string` | 无 | 字母、数字和符号 | 长度默认随机 5–29；显式 `length` 有效 |
| `word` | 无 | 小写英文单词 | 默认 3–12 个字符 |
| `cword` | `cw` | 常用汉字 | 默认 1–4 个汉字 |
| `title` | 无 | 英文标题 | 目标长度默认 20–89 |
| `ctitle` | `ct` | 中文标题 | 目标长度默认 5–49 |
| `sentence` | 无 | 英文句子 | 目标长度默认 20–89 |
| `csentence` | `cstring`, `cs` | 中文句子 | 目标长度默认 5–49 |
| `summary` | 无 | 英文短段 | 目标长度默认 120–299 |
| `csummary` | `cm` | 中文短段 | 目标长度默认 60–249 |
| `paragraph` | 无 | 英文长段 | 目标长度默认 300–1599 |
| `cparagraph` | `cp` | 中文长段 | 目标长度默认 300–1599 |
| `name` | 无 | 英文姓名 | first + 可选 middle + last |
| `cname` | 无 | 中文姓名 | 姓 + 名 |
| `uuid` | 无 | UUID v4 | 标准连字符字符串 |
| `regex` | 无 | 正则生成字符串 | 属性 `regex`；无表达式时返回空串 |
| `url` | 无 | 随机 HTTP(S) URL | 内置英文词和域名组成 |
| `domain` | 无 | 域名 | 当前 response 生成总是带 `www.` |
| `ip` | 无 | IPv4 字符串 | 每段实际为 0–254，不保证可路由 |
| `email` | 无 | 随机邮箱 | 不保证真实域或可投递 |
| `image` | 无 | dummyimage.com URL | 支持尺寸、前景色、背景色和文字 |

### 10.1 长度属性

```json5
token: { type: "string", length: 64 }
title: { type: "ctitle", min_length: 10, max_length: 30 }
```

通用属性是 `length`、`min_length`、`max_length`。但当前派生文本函数会在正常 min/max 区间内重新随机选择长度，导致显式 `length` 对 `word/title/sentence/summary/paragraph` 及中文变体不可靠；普通 `string` 的 `length` 才可靠。这是**已知缺陷**。

中文生成函数内部部分地方按 UTF-8 字节长度近似目标值，所以“长度”也不总等于最终 Unicode 字符数。

### 10.2 paragraph `content_type`

```json5
content: { type: "cparagraph", content_type: "html" }
```

- `html`：每段包裹 `<p>...</p>`；
- `markdown`：使用空行分段，是默认值；
- 不做 HTML 转义或内容安全处理。

### 10.3 image

```json5
avatar: {
  name: "头像",
  type: "image",
  size: "300x300",
  foreground: "#fff",
  background: "#007bff",
  text: "Panda API",
}
```

`format` 会被读取并传入函数，但当前函数没有使用它；旧文档中的 `accept` 也没有参与 Mock 生成或上传校验。

### 10.4 未知类型

任何未被识别的简单 `type` 都回退到普通 `string` 生成，不会报错。例如拼错 `postint` 不会得到整数，也不会得到明确诊断。

## 11. object

对象无需显式 `type`：

```json5
category: {
  $name: "分类",
  $desc: "文章所属分类",
  id: { name: "分类 ID", type: "posint" },
  name: { name: "分类名称", type: "cword" },
}
```

复杂对象的元数据使用 `$` 前缀，以免与真实字段冲突：`$name`、`$desc`、`$type`、`$required`、`$order`。生成 Mock 时这些私有 key 被跳过。

对象类型推断较脆弱：对象中存在字符串 `name` 时会被识别为简单 string 字段；否则只要任意值是 object/array，就识别为 object。建议简单字段始终有 `name`，复杂对象的说明始终写 `$name`。

## 12. array

数组推荐只放一个模板元素：

```json5
items: [
  {
    $name: "文章列表",
    $min_length: 3,
    $max_length: 10,
    id: { name: "文章 ID", type: "posint" },
    title: { name: "标题", type: "ctitle" },
  },
]
```

| 属性 | 行为 |
| --- | --- |
| `$length` | 固定元素个数 |
| `$min_length` | 随机最小个数，默认 3 |
| `$max_length` | 随机最大个数，默认 10，生成时包含 10 |
| `$name` / `$desc` | 数组字段说明 |

简单元素数组：

```json5
ids: [
  {
    $name: "ID 列表",
    $length: 3,
    name: "ID",
    type: "posint",
  },
]
```

若数组包含多个模板，解析器不会像单模板那样递归处理引用，Mock 生成器则会为每个模板分别生成一批元素，行为不直观；0.7.4 文档应坚持单模板。

## 13. map

```json5
menu: {
  $type: "map",
  $name: "菜单映射",
  $key: { name: "菜单路径", type: "string" },
  $value: {
    name: { name: "菜单名称", type: "cword" },
    order: { name: "排序", type: "int" },
  },
  $min_length: 0,
  $max_length: 7,
}
```

- 必须显式 `$type:"map"`，并同时提供 `$key` 与 `$value`；
- key 生成结果只接受 string、bool 和 number，其他类型被丢弃；
- `$length` 固定数量；默认随机范围的代码是 0 到 6 个尝试项；
- 随机 key 可能重复，最终 map 实际长度可能小于请求长度。

## 14. `$ref`、继承与递归

### 14.1 模型文件

```json5
// _data/models.json5
{
  Article: {
    id: { name: "文章 ID", type: "posint" },
    title: { name: "标题", type: "ctitle" },
    category: {
      $name: "分类",
      id: { name: "分类 ID", type: "posint" },
      name: { name: "分类名称" },
    },
    tags: [
      {
        id: { name: "标签 ID", type: "posint" },
        name: { name: "标签名称" },
      },
    ],
  },
}
```

### 14.2 引用语法

```json5
body: {
  $ref: "./_data/models.json5:Article",
}
```

冒号前是文件路径，冒号后是 JSON Pointer 风格路径；嵌套字段用 `/`：

```json5
title: { $ref: "./_data/models.json5:Article/title" }
```

路径解析限制：

- `./_data...` 相对当前 API 文档所在目录；
- `/_data...` 相对工作区根目录；
- 其他相对路径实际相对进程工作目录；
- 实现用 `split(":")`，不适合含冒号的 Windows drive path；
- 引用必须能得到 object；引用 scalar/array 时会退化为空对象或产生异常行为。

### 14.3 `define`

```json5
{
  define: {
    model: "./_data/models.json5",
    article: "./_data/models.json5:Article",
  },
  apis: [
    {
      body: { $ref: "$model:Article" },
      response: { $ref: "$article" },
    },
  ],
}
```

实现只替换 `$ref` 中匹配到的第一个 `$单词`。未定义名称不会报出结构化错误。

### 14.4 `$include` 与 `$exclude`

```json5
body: {
  $ref: "$article",
  $include: ["id", "title", "category"],
  $exclude: ["category/id"],
}
```

- `$include` 先执行，只支持引用对象的顶层 key；
- `$exclude` 后执行，支持 `/` 表示嵌套 object/array index；
- 两者格式不正确会因 `unwrap` 失败。

### 14.5 覆盖和新增嵌套字段

```json5
body: {
  $ref: "$article",
  "category/id": { name: "新的分类 ID", type: "int" },
  "category/order": { name: "分类排序", type: "int" },
  "tags/0/order": { name: "标签排序", type: "int" },
}
```

当前对象中的普通字段覆盖引用字段；含 `/` 的 key 在解析后写入嵌套路径。数组 index 必须是无前导零的非负整数。

### 14.6 删除字段

```json5
response: {
  code: "$del",
  msg: { $del: true },
}
```

两种写法都可删除继承字段。`{$del:true}` 也会让 Mock 跳过当前字段。

### 14.7 复用整个 API

```json5
// _data/common-api.json5
{
  upload: {
    name: "上传文件",
    method: "POST",
    url: "/upload/",
    body_mode: "form-data",
    response: { url: { name: "文件地址" } },
  },
}
```

```json5
apis: [
  {
    $ref: "./_data/common-api.json5:upload",
    name: "上传文章附件",
  },
]
```

API 自身的标量和结构字段可以覆盖整个 API `$ref` 的值。

### 14.8 `rec` 递归模型

```json5
response: {
  node: {
    $name: "节点树",
    id: { name: "节点 ID", type: "posint" },
    children: [
      {
        $type: "rec",
        $ref: "/node",
        $min_length: 0,
        $max_length: 3,
        $min_count: 0,
        $max_count: 3,
      },
    ],
  },
}
```

| 属性 | 当前默认 |
| --- | --- |
| `$ref` | 必填，必须以 `/` 开头且只指向当前原始 response |
| `$length` | 固定当前递归数组长度 |
| `$min_length` / `$max_length` | 默认有效范围 0–3 |
| `$count` | 固定递归层数 |
| `$min_count` / `$max_count` | 默认有效范围 0–3 |
| `$empty_value` | 自定义尾节点值 |

旧文档中的 `$mac_count` 是拼写错误，代码只读取 `$max_count`。递归会复制完整模型，配置过大可能快速放大输出；当前没有响应大小或深度硬上限。

## 15. 认证与权限

### 15.1 配置示例

```json5
// _auth.json5
{
  name: "Mock 认证",
  desc: "仅用于本地接口联调",
  auth_type: "Bearer",
  auth_place: "headers",
  no_perm_response: {
    code: -1,
    msg: "没有权限",
  },
  groups: [
    {
      name: "管理员",
      users: [
        { id: 1, username: "root", token: "local-demo-token" },
      ],
      has_perms: ["*"],
      no_perms: [["/admin/users/", "DELETE"]],
      no_perm_response: {
        code: -1,
        msg: "当前管理员不能删除用户",
      },
    },
  ],
}
```

只有 API 设置 `auth:true` 时才调用验证器。权限顺序是：找到 token 所在 group → 先检查 `no_perms` → 再检查 `has_perms`。

### 15.2 权限表达式

```json5
has_perms: ["*"]
has_perms: ["/settings/", ["/users/", "GET", "POST"]]
has_perms: [
  { url: "/posts/", methods: ["GET", "POST"] },
]
has_perms: [
  "$article.json5",
  { $ref: "$admin.json5", methods: ["GET"], $exclude: ["/admin/secret/"] },
]
```

- 单个 URL string 默认所有 method；
- 数组第一个元素是 URL，后续元素是 method；
- object 使用 `url` 与 `methods`；
- 以 `$` 开头的文档文件名会展开其中所有 API URL；
- `no_perms` 和 `has_perms` 的表达式相同；deny 优先于 allow。

### 15.3 当前限制与风险

| 配置/行为 | 状态 |
| --- | --- |
| `auth_type` | **部分实现**：会展示，但运行时忽略，固定按 Bearer 语义处理 |
| `auth_place` | **仅设计**：body/query 不读取，只看 Authorization header |
| Bearer prefix | **已知缺陷**：代码移除精确 `Bearer `；没有该前缀时仍把整个 header 当 token |
| `auth:true` 但无 `_auth` | **已知缺陷/P0**：验证器返回通过，属于 fail open |
| 无权限响应 | **部分实现**：返回配置 JSON，但 HTTP status 通常仍是 200 |
| 凭据可见性 | **已知风险**：在线文档接口会返回用户和 token |

这些是 Mock 凭据，不应写入真实生产 token，也不应把当前服务直接暴露到公网。

## 16. `test_data` 与 Mock 匹配

### 16.1 完整结构

```json5
test_data: [
  {
    name: "用例名称",
    desc: "说明，不参与匹配",
    method: "POST",
    url: "/login/",
    body: { username: "root", password: "123456" },
    query: { source: "web" },
    response: { code: 1, msg: "成功" },
    delay: 100,
  },
]
```

当前参与匹配的字段是 `url`、`method`、`body`、`query`。`name`、`desc` 仅用于说明，`response` 是匹配结果，`delay` 单位为毫秒。

### 16.2 匹配顺序

1. 按数组从前到后遍历；
2. 用例有 `url` 时必须等于实际请求 path；没有则忽略 URL 条件；
3. 用例有 `method` 时必须匹配实际大写 method 或 `*`；没有则忽略 method 条件；
4. `body` 与请求 JSON 必须完全相等；用例未写 body 时等价于 null/空对象；
5. query 先按 API `query` 模型转换类型，再与用例完全相等；
6. 返回第一个通过全部条件的用例，不再继续搜索。

因此，宽泛用例应放在更具体用例之后。

### 16.3 query 类型转换

HTTP query 原本都是字符串。当 API 定义了 query object 时：

- `number/int/posint/negint/timestamp` 尝试转为 i64；
- `float/posfloat/negfloat` 尝试转为 f64；
- 其他类型保留 string；
- 请求中未在 API query schema 定义的 key 会被丢弃；
- 转换失败的已定义 key 也会被丢弃。

API 没有 query schema 时，使用 extractor 得到的原始值。这个规则会影响 `test_data.query` 应写数字还是字符串。

### 16.4 `$mock`

固定 response 中可以只对某个字段重新使用模型生成：

```json5
test_data: [
  {
    body: { username: "root", password: "123456" },
    response: {
      code: 1,
      token: { $mock: true, required: true, length: 64 },
    },
  },
]
```

运行时从 API `response` 的同一路径读取原字段属性，再用当前对象中的属性覆盖并生成值。模型中不存在该路径时无法生成。只应使用 `$mock:true`；`$mock:false` 当前会得到意外空对象。

### 16.5 `delay`

`delay` 使用 `thread::sleep(Duration::from_millis(...))`，会阻塞 Actix worker。它适合本机少量联调，不适合并发或大延迟。

### 16.6 form-data

旧文章给出的 `form_data:{...}` **当前不会参与匹配**。multipart 字段和文件名会被放入内部 body，因此可尝试用 `body` 匹配普通字段，但文件处理还伴随写磁盘和路径安全问题，不能视为稳定契约。

### 16.7 未匹配行为

- `response` 有模型：生成随机 Mock；
- `response` 为 null、空 object 或空 array：返回 `code:-1` 的说明 JSON；
- method 没有候选：返回说明 JSON；
- URL 没有候选：返回说明 JSON；
- 上述错误通常使用 HTTP 200。

## 17. WebSocket

```json5
{
  name: "WebSocket",
  apis: [
    {
      name: "聊天室",
      method: "WEBSOCKET",
      url: "/ws/",
    },
  ],
}
```

method 包含 `WEBSOCKET` 时，该 API 的 URL 被注册为 WebSocket 路由。当前只保存一个 WebSocket API，多个配置时后加载值覆盖前值且顺序不稳定。

固定文本协议：`/list`、`/join <room>`、`/name <name>` 和房间广播。body/response/test_data 不参与消息 Mock，因此状态是**部分实现**。

## 18. Markdown 文档与菜单

### 18.1 文件和目录排序

```text
$1开发前必读/
├── $_folder.md
├── $1首页.md
└── $2更新日志.md
```

文件或目录开头的 `$数字` 用作 `order`，剩余名称用作菜单标题；没有数字时 order 为 0。JSON5 和 Markdown 共用菜单树。

### 18.2 文首配置

建议每个 Markdown 文件都使用下面的文首配置：

````markdown
```
{
  menu_title: "接口更新日志",
  order: 7,
  desc: "更新记录",
}
```

# 接口更新日志
````

文首 object 必须位于文件开头，可以被三反引号包裹；当前正则实际要求结束三反引号。配置成功后 object 会从展示正文移除。

普通 `.md` 若没有可解析的配置块，Loader 没有设置其 `filename`，前端可能只显示菜单而无法拉取正文，属于**已知缺陷**。因此 0.7.4 建议始终写配置块。

### 18.3 `$_folder.md`

```json5
{
  menu_title: "开发前必读",
  order: 1,
  desc: "接入前需要阅读",
  show_content: true,
}
```

- `show_content:true` 时父菜单关联该 Markdown 内容；
- 缺省为 true；
- `show_content:false` 时只作为不可加载正文的目录菜单。

### 18.4 图片

把图片放在工作区 `media/`：

```markdown
![架构图](/media/architecture.png)
```

当前只注册 `/media/*`，没有旧文档提到的 `/static/*` 工作区路由。

## 19. 文件上传

multipart 请求的处理逻辑：

1. 从 `Content-Disposition` 提取只包含 `\w+` 的字段名；
2. 有 filename 时写入 `./_data/_upload/<原始文件名>`；
3. body 中保存字段值为原始 filename；
4. 额外保存 `$___<field>:url`，供 `$value:"$body/<field>:url"` 使用；
5. 上传结果通过 `/_upload/<filename>` 访问。

当前没有文件名净化、路径约束、覆盖策略、类型/大小限制或配额，属于 P0 风险。只应在可信本机工作区测试，不要接收不可信上传。

## 20. CLI

CLI 定义位于 [`src/main.rs`](../../../compatibility/panda-0.7/src/main.rs)。安装器历史上把命令暴露为 `panda`；Cargo binary target 名称是 `panda-api`，StructOpt 内部 name 仍写成 `classify`，三者需要在未来统一。

### 20.1 服务参数

```text
--host / -h       默认 127.0.0.1，也可用 PANDA_API_HOST
--port / -p       默认 9000，也可用 PANDA_API_PORT
```

由于当前项目无法构建，本表由源码推导，未重新验证生成的 help 文本及 `-h` 与 help flag 的冲突。

### 20.2 token

```shell
panda token --num 20 --length 64
```

`num`/`-n` 默认 10，`length`/`-l` 默认 64。字符集包含大小写字母、数字和部分符号；它不是 JWT 或密码学凭据签发器。

### 20.3 test

```shell
panda test --server dev --url /login/
panda test --server-url http://127.0.0.1:8080 --docs auth.json5 article.json5
```

| 参数 | 状态 |
| --- | --- |
| `--server` / `-s` | **已实现**：读取 `_settings.servers.<name>.url` |
| `--server-url` | **已实现**：优先于 `--server` |
| `--url` / `-u` | **已实现**：测试一个 URL 的全部候选 API |
| `--docs` / `-d` | **已实现**：测试指定文档列表 |
| `--all` / `-A` | **仅设计**：参数未被执行逻辑读取 |
| `--cron` / `-c` | **仅设计**：参数未被执行逻辑读取 |
| minute/hour/day/month | **仅设计**：结构存在，但没有调度逻辑 |

测试客户端的真实能力：

- 遍历 API 的 `test_data`；
- API method 包含 POST 时发送 JSON POST；包含 GET 时发送 query GET；
- 其他 HTTP method、headers、auth、form-data 和 delay 不支持；
- `response` 参数传入 `get/post` 后没有被读取；
- 只打印响应或解析错误，不比较预期、不生成报告、业务不匹配不会形成可靠失败退出码；
- GET query 中的 bool 被错误序列化为字面量 `bool`。

因此这个命令当前是“批量请求器”，不是可用于 CI 的接口测试器。

## 21. 当前未实现或不应依赖的语法

| 历史文档中的说法 | 0.7.4 实际状态 |
| --- | --- |
| `test_data.form_data` 匹配 | 未读取 |
| `body_mode: binary` | 未实现 |
| text/html/xml/javascript body Mock 匹配 | 未读取原始 body |
| date/datetime 自定义 `format` | 字段被忽略 |
| image `accept` / `format` | 不参与上传校验或 URL 生成 |
| 任意 response header | 只支持 status_code/content_type |
| `auth_place: body/query` | 未实现 |
| 自定义 `auth_type` | 运行时忽略 |
| 接口后端响应断言 | 未实现 |
| `test --all`、cron、测试报告 | 未实现 |
| Mock 模糊测试、边界外测试、性能测试 | 仅设计 |
| `$sha256(...)`、`$command(...)`、`$request(...)` | 仅旧测试设计稿，无解析执行代码 |
| CSV 外部数据 | 旧文章写明“正在搞”，未实现 |
| 同 URL/method 后定义覆盖前定义 | 代码实际按不稳定候选顺序选择 |
| `/static/...` 工作区资源 | 未注册，只支持 `/media/...` |

## 22. 编写建议

- 所有工作区文档统一使用 `.json5`，设置与认证使用下划线保留文件名；
- API、模型和用例保持小而明确，避免重叠 URL pattern 和重复 URL/method/header 组合；
- 数组只放一个模板，递归显式设置较小的最大长度和层数；
- `test_data` 从具体到宽泛排列，并写 `name`/`desc` 说明意图；
- 对当前有缺陷的动态日期、复杂文本长度、上传和认证行为，使用固定 `value`/`enum`/`test_data`；
- 只使用 Mock token，不存真实秘密；默认只监听 `127.0.0.1`；
- 每次升级 parser 或 matcher 前，用已有文档建立 golden test，确保兼容变化可见。
