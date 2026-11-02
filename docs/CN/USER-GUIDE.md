<div align="center">
  <img src="../../branding/app-icon.svg" width="104" alt="Flint logo" />
  <h1>Flint · 火石</h1>
  <p>点一下，让本地服务运行起来。</p>
  <p>A local service launcher with isolated installs and a macOS menu bar.</p>
</div>

[English](../EN/USER-GUIDE.md)

Flint 把命令行服务的安装、启动、端口配置和日志放到一个桌面应用里。首个内置配方是 MinerU；也可以添加已有的 HTTP API、MCP HTTP 服务或 Web App 启动命令。

**当前版本：0.1.12 alpha · macOS Apple Silicon。** Windows、Linux 和 Intel Mac 尚未发布。Flint 使用 Tauri 2、Rust、React 和 TypeScript，源码采用 [MIT](../../LICENSE)。

## 安装与使用

1. [下载 macOS DMG](https://github.com/puzihao2018/Flint/releases/download/v0.1.12/Flint-0.1.12-macOS-arm64.dmg)，打开后将 Flint 拖入「应用程序」。ZIP 与 SHA-256 校验文件见[发布页](https://github.com/puzihao2018/Flint/releases/tag/v0.1.12)。
2. 打开 Flint，在「应用市场 → MinerU」选择版本并部署。首次部署需要网络下载 Python、依赖和模型，耗时和磁盘占用取决于配置。
3. 打开启动开关，等待检查通过，复制服务地址给需要它的客户端。
4. 改端口时先停止服务，修改并保存，再启动。

首发安装包使用 **ad-hoc 签名，尚未经过 Apple Developer ID 公证**，macOS 可能阻止首次打开。这不是 App Store 安装包；请核对来源和 Release 提供的 SHA-256。Apple 公证的发布步骤见[macOS 发布指南](MACOS-RELEASE.md)。

应用声明最低 macOS 13；真实安装与解析验证来自 Apple Silicon 开发机，尚未覆盖所有 macOS 版本、芯片及内存容量。

### 后台运行

关闭主窗口后只保留菜单栏图标，Dock 图标隐藏，服务继续运行。点击菜单栏可打开快捷面板，再打开主窗口时恢复 Dock 图标。「停止服务并退出」会停止 Flint 启动的服务；下一次打开不会自动启动服务。更新 Flint 前请先完成正在进行的任务并正常退出。

### 界面语言

在「设置与存储 → 语言」选择跟随系统、简体中文或 English。切换立即应用到主窗口和菜单栏，并保存到数据目录下的 `preferences.json`。中文系统使用简体中文，其他系统语言回退到 English。服务名称、命令和原始日志保持原样。新增语言说明见 [本地化维护](LOCALIZATION.md)。

### 删除服务

在服务详情页点击「删除服务」，可分别勾选卸载独立环境、删除此版本的模型，以及清理服务数据/日志/专用缓存；均不勾选则只移除登记。运行或安装中的服务会先自动停止。面板显示所选内容的文件大小和路径，实际释放空间可能受共享文件或磁盘快照影响。选中文件直接删除，不进入废纸篓；失败时保留登记以供重试。另一版本的 MinerU、共享 Python、下载缓存不会删除；自定义服务只清理 Flint 管理的实例数据，不删除外部项目目录。存在旧进程中断记录时仍需先确认并清除。

### MinerU 版本

| Release | API | 使用条件 |
|---|---|---|
| 4.0.12 | `/v1/uploads`、`/v1/parse/jobs`、ZIP 结果 | 已验证 llm-for-zotero 3.9.15 的本地解析流程 |
| 4.1.2 | `/v1/tasks` | 客户端必须支持新的 Tasks API |

两个版本分别安装到独立环境，保留各自配置、模型目录和端口。以 Flint 显示的服务地址为准；端口不是固定的 8000 或 8001。MinerU 这里提供的是 **HTTP API**，不是 MCP server。Flint 当前配方没有为此本地 API 设置密钥。

macOS 受管 MinerU 默认使用串行权重加载，避开已观察到的模型冷启动锁等待。已验证的档位和模型源是 **standard + Hugging Face**；其他组合不能视为同等验证。

## 自己的服务

在「添加服务」填写名称、已有项目目录、前台启动命令、端口与健康检查路径，例如：

```sh
/path/to/uv run --frozen --no-sync myserver --host 127.0.0.1 --port ${port}
```

`${port}` 会替换为界面中的端口。程序本身需要支持该参数。Flint 通过中断信号停止服务，无需另写 stop command。不要使用 `nohup`、`&` 或自行 daemonize。

也可导入 [`launcher.yaml` 0.2](../../examples/node-webapp/launcher.yaml)，支持应用 ID/版本、结构化参数与环境变量、显式安装步骤；0.1 清单继续兼容。导入和保存不执行代码，有安装步骤时点击「安装项目依赖」，全部成功后才能启动。自定义服务使用已有项目目录，项目内的 node_modules/.venv 不由 Flint 卸载。

开发者参考：[接入规范](COMMAND-CONTRACT.md) · [给 AI 的任务说明](AI-INTEGRATION.md) · [JSON Schema](../../schema/launcher.schema.json) · [Node 示例](NODE-EXAMPLE.md) · [Python/uv 示例](PYTHON-EXAMPLE.md)。`npm run manifest:check -- /path/to/launcher.yaml` 可只校验清单。MCP 支持范围为 Streamable HTTP 初始化探测，暂不支持 stdio、旧式 SSE 或带鉴权的探测配置。

## 开发

需要 Apple Silicon Mac、Xcode Command Line Tools、Python 3.9+、Node.js 20.19+（建议 `nvm use` 使用 Node 24）和 rustup。仓库通过 `rust-toolchain.toml` 固定 Rust 版本。

```sh
npm ci
npm run desktop      # 下载并校验 uv，然后启动开发窗口
npm test             # 运行本地化检查和串行 Rust 测试
npm run release:mac  # 构建 .app，打包 DMG / ZIP 和校验文件
```

打包产物位于 `release/0.1.12/`，不会提交到 Git。`uv` 从官方固定 Release 下载，并验证压缩包及二进制 SHA-256；不要求用户预先安装 Python 或 uv，不向源码仓库提交 uv 二进制。`Cargo.lock`、`package-lock.json` 和各 MinerU 依赖锁随源码提交。

开发测试请使用独立目录，例如 `FLINT_DATA_DIR=/tmp/flint-dev npm run desktop`。安装版运行时不要让第二个实例或 headless 验证器写同一个数据目录；不同开发数据目录仍共享同一应用标识的单实例限制。

## 目录

| 位置 | 内容 |
|---|---|
| `src/` | 主窗口、菜单栏快捷面板与共享样式 |
| `src-tauri/src/` | 服务生命周期、受管安装、健康检查、清单导入 |
| `registry/` | 已适配 MinerU Release 元数据和依赖锁 |
| `scripts/` | 构建资源准备、许可证清单和 macOS 打包 |
| `examples/` | 可导入的示例服务 |
| `.github/` | CI、Release 草稿和问题模板 |
| `docs/` | 命令规范、Release 配方、发布说明与验证范围 |

macOS 数据目录为 `~/Library/Application Support/local.flint.services/`。应用、模型、配置、缓存和日志保存在这里，替换应用程序不会删除这些内容。详情见[架构与数据](ARCHITECTURE.md)。

## 当前支持范围

已包含：内置 MinerU 多版本安装、隔离环境、启停与端口、HTTP/MCP HTTP 健康检查、自定义命令、日志、菜单栏控制。应用市场目前是随包维护的配方列表。

尚不支持开机启动、自动重启、应用内更新、多服务编排、任意 GitHub Release 自动安装。

[验证范围](VALIDATION.md) · [发布记录](CHANGELOG.md) · [贡献指南](CONTRIBUTING.md) · [第三方声明](THIRD-PARTY-NOTICES.md)
