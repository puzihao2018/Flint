<div align="center">
  <img src="../../branding/app-icon.svg" width="88" alt="Flint" />
  <h1>Flint</h1>
  <p>在自己的电脑上托管 Web 应用和 HTTP 服务。导入项目提供的 launcher.yaml，就能在 Flint 里一键启停。</p>
  <p><a href="https://zihaopu.cn/Flint/?lang=zh-CN">项目网站</a> · <a href="USER-GUIDE.md">使用指南</a> · <a href="AI-INTEGRATION.md">接入 Flint</a></p>
</div>

[English](../../README.md) | **简体中文**

[![macOS 构建](https://github.com/puzihao2018/Flint/actions/workflows/macos.yml/badge.svg)](https://github.com/puzihao2018/Flint/actions/workflows/macos.yml)
[![网站](https://github.com/puzihao2018/Flint/actions/workflows/pages.yml/badge.svg)](https://github.com/puzihao2018/Flint/actions/workflows/pages.yml)
[![MIT](https://img.shields.io/badge/license-MIT-203e35)](../../LICENSE)

## Flint 能做什么

- **独立安装 MinerU**：4.0.12 与 4.1.2 可并存，各自使用独立环境。
- **管理后台服务**：启动、停止、配置端口、健康检查和日志集中展示。
- **菜单栏快捷控制**：关闭主窗口后，服务继续运行。
- **接入自己的项目**：直接填写前台启动命令，或导入 `launcher.yaml`。
- **按需清理空间**：只删除登记，或选择卸载受管环境和模型。
- **中英文界面**：语言可切换，偏好会保留。

## 获取 Flint

**macOS 13+ · Apple Silicon · alpha。** Windows、Linux 和 Intel Mac 暂未提供。

[下载 Flint 0.1.13（macOS DMG）](https://github.com/puzihao2018/Flint/releases/download/v0.1.13/Flint-0.1.13-macOS-arm64.dmg)，打开后将 Flint 拖入「应用程序」。ZIP 和 SHA-256 校验文件见[发布页](https://github.com/puzihao2018/Flint/releases/tag/v0.1.13)。

安装包采用 ad-hoc 签名，**尚未经过 Apple 公证**。首次部署 MinerU 需要下载依赖和模型。安装、兼容性与卸载说明见[使用指南](USER-GUIDE.md)。

## 接入你的项目

添加已有项目目录与启动命令，或导入 [`launcher.yaml`](../../examples/python-webapp/launcher.yaml)。Flint 管理进程启停，无需单独提供停止命令。

[接入规范](COMMAND-CONTRACT.md) · [给编程 AI 的指南](AI-INTEGRATION.md) · [Node 示例](NODE-EXAMPLE.md) · [Python/uv 示例](PYTHON-EXAMPLE.md)

MinerU 提供 HTTP API，不是 MCP。自定义 MCP 服务目前支持 Streamable HTTP，不支持 stdio 和旧式 SSE。

## 开发与贡献

需要 Apple Silicon Mac、Xcode Command Line Tools、Python 3.9+、Node.js 20.19+（推荐 Node 24）和 rustup。Rust 版本由 `rust-toolchain.toml` 固定。

```sh
npm ci
FLINT_DATA_DIR=/tmp/flint-dev npm run desktop
npm test
npm run release:mac
```

[贡献指南](CONTRIBUTING.md) · [架构](ARCHITECTURE.md) · [验证范围](VALIDATION.md) · [更新记录](CHANGELOG.md) · [网站维护](WEBSITE.md)

源码采用 [MIT](../../LICENSE) 许可证。第三方软件与模型保留各自许可证，详见[第三方声明](THIRD-PARTY-NOTICES.md)。
