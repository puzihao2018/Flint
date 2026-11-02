# 给 AI：为项目提供 Flint 接入

[English](../EN/AI-INTEGRATION.md)

目标客户端：Flint 0.1.9+，清单规范 0.2。本文件可直接交给代码生成 AI；完整定义见 [COMMAND-CONTRACT.md](COMMAND-CONTRACT.md)，机器可读结构见 [launcher.schema.json](../../schema/launcher.schema.json)。

## 可直接复制的任务指令

> 请检查当前项目的实际启动方式，为它添加可导入 Flint 的 `launcher.yaml`。使用 schemaVersion 字符串 "0.2"，提供稳定的应用 id、Release version、name、environment.mode: existing、parameters.port.default、service.protocol 和 service.run。不要臆造项目不存在的 CLI 参数。
>
> `service.run` 和可选的 `install` 步骤用 `{program, args, env}`，args 是字符串数组，不把整条 shell 命令塞进 program。已有程序支持 `--port` 就传 `${port}`；只支持 PORT 环境变量就用 `env: {PORT: "${port}"}`。新项目可直接读取 LAUNCHER_PORT/LAUNCHER_HOST。默认绑定 127.0.0.1，端口冲突时退出，不自动换号。
>
> 安装和运行分开，提交依赖锁。有构建步骤时写入 install；启动不安装或升级依赖。安装步骤需要有限时间内退出且能重复执行。Python 示例用 `uv sync --frozen` 安装、`uv run --frozen --no-sync ...` 运行；Node 按项目实际脚本使用 npm ci 和 npm run build（如果确实存在），不凭空添加 build。清单不会自动下载 Node/uv，注明前置条件。
>
> 运行进程保持前台，不打开浏览器、不要求交互输入、不使用 nohup、后台 &、tmux 或守护器。响应中断并释放端口/子进程，不需要 stop 命令。日志及时写 stdout/stderr，不打印密钥。
>
> Web App/HTTP API 使用 http-api，healthPath 可以是首页 /，默认就是 /；已有健康检查更合适时填写实际路径，不强制新增 /healthz。MCP 使用 mcp-http 并显式声明真实 Streamable HTTP 端点；Flint 当前探测仅支持 2025-03-26，不能把普通 HTTP API 标记成 MCP，也不要把 stdio MCP 当成 HTTP 服务。
>
> 持久数据优先使用 LAUNCHER_DATA_DIR，缓存使用 LAUNCHER_CACHE_DIR；不在清单里放密码或 API Key。缺少这些环境变量时项目仍能独立启动。
>
> 当前没有通用 managed 环境、GitHub Release 自动下载、依赖服务编排、Flint 外部控制 API；不要写这些尚未实现的字段。不要替用户创建受管环境的声明然后忽略它。extensions 仅存工具元数据，不能放执行逻辑。针对不同平台实测后再声明支持。
>
> 交付：launcher.yaml、必要的最小代码改动、README 接入步骤和验证记录。先校验清单；在临时实例里完成安装、启动、HTTP/MCP 请求、修改端口、停止及端口释放；安装失败应阻止启动。明确区分实测平台和推测兼容。不要启动第二个 Flint 引擎去修改用户正在使用的数据目录。

## 最小参考

应用直接读取 LAUNCHER_PORT 时：

```yaml
schemaVersion: "0.2"
id: example.my-app
version: "1.0.0"
name: My App
environment: {mode: existing}
parameters: {port: {default: 3000}}
service:
  protocol: http-api
  run:
    program: node
    args: [server.mjs]
```

应用只读取 PORT 时，将 run 改为：

```yaml
run:
  program: node
  args: [server.js]
  env:
    PORT: "${port}"
```

仅需要启动已部署项目：省略 install。需要安装/构建：按实际情况加入，例如：

```yaml
install:
  - program: npm
    args: [ci]
  - program: npm
    args: [run, build]
```

`service.cwd` 默认清单目录；安装和启动共享这个目录。变量占位符仅支持 `${port}`、`${host}`、`${dataDir}`、`${cacheDir}`，只在 args/env 中展开，不执行 shell 表达式。

## 自检

- 名称、应用版本与仓库实际版本一致；清单 ID 不使用本机随机实例 ID。
- program 可被 Flint 的运行环境找到；有问题时用绝对可执行路径，不能把个人路径提交给所有用户。
- 不依赖某个固定端口。命令打印的 URL 与 Flint 配置一致。
- HTTP 检查返回 2xx；返回 401/404 的路径不能当作已就绪依据。
- `existing` 的 node_modules/.venv/模型归项目所有，删除 Flint 登记不会移除它们。
- 每次导入新增实例，修改清单不会同步旧实例；更新演示时处理好旧实例，避免端口重复。

仓库校验命令：`npm run manifest:check -- /absolute/path/to/launcher.yaml`。该工具只校验，不接管运行中的 Flint。实际项目模板见 [Node](NODE-EXAMPLE.md) 和 [Python](PYTHON-EXAMPLE.md)。
