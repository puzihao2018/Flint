# Flint 0.1.9 · macOS alpha

Flint 将本地服务的安装、启动、端口配置和日志放到桌面应用里。

- 新增 launcher.yaml 0.2：结构化命令、应用版本、显式安装步骤、离线校验、JSON Schema 与 AI / Node / Python 接入文档。
- 删除服务时可勾选卸载独立环境、模型和服务数据，显示文件大小及清理位置；自动停止后执行。共享运行时、下载缓存及其他版本保留。
- 修复 IPv6 端口冲突漏检，提示占用进程和 PID，并识别启动阶段的 EADDRINUSE。
- 首发支持 macOS Apple Silicon（arm64）；声明最低 macOS 13，尚未覆盖所有系统和芯片。
- MinerU 4.0.12 与 4.1.2 独立安装，保留不同 API 协议与环境。
- 支持已有 HTTP API / MCP HTTP 服务的自定义启动命令和 launcher.yaml。
- 关闭主窗口后只留菜单栏，后台服务继续运行；重新打开恢复 Dock 图标。
- 修复 macOS 受管 MinerU 冷启动的异步权重加载锁等待。

## 下载

`Flint-0.1.9-macOS-arm64.dmg`：打开后拖到 Applications。
`Flint-0.1.9-macOS-arm64.zip`：同一应用的 ZIP 备选包。
`SHA256SUMS.txt`：发行附件校验值。

安装包使用 ad-hoc 签名，**尚未经过 Apple Developer ID 公证**，macOS 可能阻止首次打开。请核对下载来源与校验值。本 alpha 不含自动更新。更新前先等解析任务完成并正常退出旧版 Flint。

首次部署 MinerU 需要下载 Python、依赖和模型；这些内容不包含在安装包中。客户端须支持所选 MinerU 版本的 API，4.0 和 4.1 不可互换。

Windows、Linux、Intel Mac、端口自动发现和 AI 配方适配尚未发布。源码采用 MIT；第三方软件及模型各自适用上游许可证。
