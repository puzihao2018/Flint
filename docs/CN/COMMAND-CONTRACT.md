# Flint 项目接入标准 0.2

[English](../EN/COMMAND-CONTRACT.md)

适用 Flint **0.1.9 起**。`launcher.yaml` 是项目随代码提交的启动清单；也接受同结构 JSON。Flint 0.1.8 及更早只支持 0.1 清单。旧清单与已登记服务继续兼容。

目标是让一个 Web App、HTTP API 或 MCP HTTP 服务可以导入、安装、启动、检查和停止。应用不需要 Flint SDK，也不需要回调 Flint。当前没有外部管理 HTTP API。

## 最小清单

```yaml
schemaVersion: "0.2"
id: example.my-webapp
version: "1.0.0"
name: My Web App
environment:
  mode: existing
parameters:
  port:
    default: 3000
service:
  protocol: http-api
  run:
    program: node
    args: [server.mjs]
```

服务须读取 `LAUNCHER_PORT`，或在 `args`/`env` 中映射端口。默认工作目录为清单所在目录，HTTP 默认检查 `/`。已有项目仍可在 UI 手动填写 shell 命令。

导入清单 → 检查配置 → 保存 → 有 `install` 时点击「安装项目依赖」→ 启动。导入和保存均不运行命令；安装成功也不会自动启动。每次导入创建独立实例，不覆盖同名或相同应用 ID 的旧实例。

## 字段

| 字段 | 必填 | 含义 |
|---|---|---|
| `schemaVersion` | 是 | 字符串 `"0.2"`，独立于应用版本 |
| `id` | 是 | 稳定应用 ID，最多 120 字节；小写字母开头，仅小写字母、数字、`.`、`-`、`_` |
| `version` | 是 | 应用 Release 版本字符串，最多 80 字节；建议 `1.0.0`，也允许带前缀的 tag；不用于下载或执行路径 |
| `name` | 是 | 显示名称，非空，最多 120 字节 |
| `environment.mode` | 是 | 当前仅 `existing`：在已有项目目录执行，不复制项目、不自动创建隔离环境 |
| `parameters.port.default` | 是 | 1024–65535；可在 UI 改，端口占用时失败，不自动换号 |
| `platforms` | 否 | `macos`、`linux`、`windows` 列表；省略/空列表不限制导入；不代表 Flint 已验证这些系统 |
| `install` | 否 | 最多 32 个结构化命令，按顺序执行，默认空；全部成功才标记安装完成 |
| `service.protocol` | 是 | `http-api`（含 Web App）或 `mcp-http` |
| `service.cwd` | 否 | 默认 `.`，相对清单目录；也接受绝对路径；导入时目录必须存在 |
| `service.run` | 是 | 单个结构化启动命令 |
| `service.healthPath` | 否/是 | HTTP 省略或空字符串时检查 `/`；MCP 必须显式提供非空端点，如 `/mcp` |
| `extensions` | 否 | 工具专用元数据对象，Flint 忽略且不存入服务配置；不可放必需的运行逻辑 |

未认识的普通字段、命令字段、清单版本、协议和环境模式会明确报错，不静默忽略。`extensions` 是唯一允许自由扩展的元数据位置。需要新的可执行语义时升级规范版本，旧客户端明确拒绝。

`id` 标识应用，Flint 另外生成本机实例 UUID；同一应用可登记多个版本或多个端口。此版不会自动升级、合并或重新导入覆盖。修改清单不会自动更新已保存实例；修改启动/安装命令后重新导入。UI 可编辑实例名称、工作目录、端口和检查路径。

## 结构化命令

`service.run` 和 `install` 中的每一步使用相同结构：

```yaml
program: node
args: [server.mjs]
env:
  PORT: "${port}"
  HOST: "${host}"
```

- `program` 是可执行文件名或路径，不是一整条命令，不接受占位符；不需要手动加引号。
- `args` 是字符串数组，每项就是一个参数；空格、引号、`;`、`$()` 不会变成 shell 代码。默认空数组。
- `env` 是字符串映射，覆盖继承环境中的对应变量，默认空对象。变量名使用英文字母、数字和下划线，不能以数字开头；`LAUNCHER_` 前缀不允许覆盖。不要在可分享清单里填写密钥，使用项目自己的本地配置或继承环境。
- `args` 和 `env` 仅替换 `${port}`、`${host}`、`${dataDir}`、`${cacheDir}`；未知/未闭合占位符报错，不支持模板表达式、`${HOME}` 或递归替换。`~` 不自动展开。
- macOS/Linux 使用固定的登录 shell 包装器寻找运行时，再通过位置参数 `exec` 程序；不把清单参数拼成 shell 源码。需要确定的 Node/uv 版本时填写可执行文件路径。依赖只在交互式 shell 中定义的 alias/function 不可靠。
- Windows 尚未验证；请使用实际可执行程序，不能假定 `.cmd` 包装器、POSIX 环境赋值或 shell 脚本跨平台兼容。两份可运行示例当前声明 macOS/Linux。
- 复杂安装逻辑放在项目里的 `scripts/setup.mjs` / `setup.py`，清单只调用它。可以显式调用 shell，但那段脚本由作者负责，参数安全保证不会覆盖作者主动执行的 shell 源码。

## 安装与环境归属

```yaml
install:
  - program: npm
    args: [ci]
  - program: npm
    args: [run, build]
```

或 Python：

```yaml
install:
  - program: uv
    args: [sync, --frozen, --no-dev]
service:
  protocol: http-api
  run:
    program: uv
    args: [run, --frozen, --no-sync, python, -u, server.py, --port, "${port}"]
```

作者应提交 `package-lock.json` / `uv.lock`，让安装可重复。安装命令在 `service.cwd` 执行，使用同样的实例环境变量、日志和取消机制；失败时后续步骤不执行，启动保持禁用。重新执行从第一步开始，不是续传事务，脚本应可安全重复运行。每一步最长两小时；修改工作目录或安装步骤后重新要求安装。有意跳过安装时省略 `install`，并在 README 写明前置条件。

`existing` 允许 `npm ci` 在项目内生成 `node_modules`，或 `uv sync` 创建项目 `.venv`；这些仍归外部项目所有。Flint 删除服务时不会删除项目目录、其中的环境或模型。Flint 只可清理自己实例的数据和缓存。

通用 `managed`、自动下载 GitHub Release、受管 Node、按平台选择安装资产尚未实现，此版拒绝相关字段。MinerU 仍使用内置、经过验证的受管适配器。独立环境不等于安全沙箱。

## 服务运行约定

1. 前台常驻、不弹浏览器、不交互输入，不用 `nohup`、后台 `&` 或自行 daemonize。
2. 绑定回环地址；读取端口参数或环境变量。Flint 设置变量本身不会改变不读取它们的应用。
3. 停止由 Flint 发中断信号并等待，超时升级终止；应用释放端口及子进程。不需要 `stop` 子命令。主动脱离进程组的 daemon 不保证被清理。
4. 日志写 stdout/stderr，及时刷新、不打印密钥。运行时不升级依赖。
5. HTTP 检查路径返回 2xx，表示可以接受请求；普通 Web App 可用首页，不强制 `/healthz`。检查成功不等于全部业务功能已验证。
6. MCP 使用真实 Streamable HTTP 端点。当前探测仅支持协议版本 `2025-03-26` 的初始化及 JSON/SSE 响应；不支持 stdio、旧 SSE transport、鉴权探测，不把普通 HTTP API 自动包装成 MCP。

Flint 注入以下变量，安装与启动均可用：

| 环境变量 | 用途 |
|---|---|
| `LAUNCHER_PORT` | 实例端口 |
| `LAUNCHER_HOST` | `127.0.0.1` |
| `LAUNCHER_INSTANCE_ID` | 本机实例 UUID |
| `LAUNCHER_DATA_DIR` | 持久数据目录，执行前创建 |
| `LAUNCHER_CACHE_DIR` | 可重建实例缓存目录，执行前创建 |

新项目建议采用 CLI 参数 > `LAUNCHER_*` > 默认值。应用不依赖这些变量也能独立运行。Flint 保留继承环境；MinerU 专用环境设置不再注入自定义项目。

## MCP 清单示意

以下要求项目确实提供 `server.mjs`、支持这些参数及对应 MCP 协议；不是可运行的 MCP 实现：

```yaml
schemaVersion: "0.2"
id: example.my-mcp
version: "1.0.0"
name: My MCP Server
environment: {mode: existing}
parameters: {port: {default: 3001}}
service:
  protocol: mcp-http
  run:
    program: node
    args: [server.mjs, --host, "${host}", --port, "${port}"]
  healthPath: /mcp
```

## 验证与迁移

[JSON Schema](../../schema/launcher.schema.json) 同时描述 0.1/0.2，可配置到编辑器。它检查结构；工作目录是否存在、当前平台是否匹配由实际导入器检查。运行时对名称/版本限制按 UTF-8 字节计数，Schema 按字符计数。

仓库开发工具（需要 Rust；不是安装版的远程控制 API）：

```sh
npm run manifest:check -- /absolute/path/to/launcher.yaml
```

只解析/验证，输出简短结果，不安装、不启动、不修改服务登记。通过校验不代表启动成功或项目可信；还需实际安装、请求、改端口、停止测试。

0.1 的 `service.command` 仍按原来 shell 语义执行，唯一 Flint 占位符仍为 `${port}`。0.2 使用 `service.run`，不同时接受 `command`；移除原 `category: service`，新增应用 `id` / `version`。旧版本示例保留在 [http-service](../../examples/http-service/launcher.yaml)。

可运行示例：[Node Web App](NODE-EXAMPLE.md)、[Python / uv Web App](PYTHON-EXAMPLE.md)。给代码生成 AI 的独立参考：[AI-INTEGRATION.md](AI-INTEGRATION.md)。
