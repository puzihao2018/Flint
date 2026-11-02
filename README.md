<div align="center">
  <img src="branding/app-icon.svg" width="88" alt="Flint" />
  <h1>Flint</h1>
  <p>A home for your local services.</p>
  <p>Install MinerU. Launch web apps. Manage HTTP and MCP servers from your menu bar.</p>
  <p><a href="https://puzihao2018.github.io/Flint/">Website</a> · <a href="docs/USER-GUIDE.md">User guide</a> · <a href="docs/AI-INTEGRATION.md">Build for Flint</a> · <a href="https://github.com/users/puzihao2018/projects/5">Roadmap</a></p>
</div>

**English** | [简体中文](readme_cn.md)

[![macOS build](https://github.com/puzihao2018/Flint/actions/workflows/macos.yml/badge.svg)](https://github.com/puzihao2018/Flint/actions/workflows/macos.yml)
[![Website](https://github.com/puzihao2018/Flint/actions/workflows/pages.yml/badge.svg)](https://github.com/puzihao2018/Flint/actions/workflows/pages.yml)
[![MIT](https://img.shields.io/badge/license-MIT-203e35)](LICENSE)

## What Flint does

- **Installs MinerU** in isolated environments, with 4.0.12 and 4.1.2 available side by side.
- **Manages your services** with start/stop controls, configurable ports, health checks, and logs.
- **Runs from the menu bar** when the main window is closed.
- **Works with your projects** through a foreground command or a portable `launcher.yaml`.
- **Keeps cleanup explicit**: remove a registration, or also uninstall its managed environment and models.
- **Speaks English and Chinese**, with a saved language preference.

## Get Flint

**macOS 13+ · Apple Silicon · alpha.** Windows, Linux, and Intel Mac builds are not available yet.

[Download Flint 0.1.12 for macOS (DMG)](https://github.com/puzihao2018/Flint/releases/download/v0.1.12/Flint-0.1.12-macOS-arm64.dmg). Open the DMG and drag Flint into Applications. The [release page](https://github.com/puzihao2018/Flint/releases/tag/v0.1.12) includes the ZIP and SHA-256 checksums.

Builds are ad-hoc signed and **not Apple-notarized**. First-time MinerU setup downloads dependencies and models. See the [user guide](docs/USER-GUIDE.md) for installation, compatibility, and removal details.

## Bring your app

Add an existing project directory and startup command, or import a [`launcher.yaml`](examples/python-webapp/launcher.yaml). Flint handles the process lifecycle; no separate stop command is required.

[Integration contract](docs/COMMAND-CONTRACT.md) · [Guide for coding agents](docs/AI-INTEGRATION.md) · [Node example](examples/node-webapp) · [Python/uv example](examples/python-webapp)

MinerU exposes an HTTP API, not MCP. Custom MCP support currently covers Streamable HTTP, not stdio or legacy SSE. Automatic port discovery and broader platform support are on the roadmap.

## Develop and contribute

On an Apple Silicon Mac, install Xcode Command Line Tools, Python 3.9+, Node.js 20.19+ (Node 24 recommended), and rustup. Rust is pinned in `rust-toolchain.toml`.

```sh
npm ci
FLINT_DATA_DIR=/tmp/flint-dev npm run desktop
npm test
npm run release:mac
```

[Contributing](CONTRIBUTING.md) · [Architecture](docs/ARCHITECTURE.md) · [Validation](docs/VALIDATION.md) · [Changelog](CHANGELOG.md) · [Website development](docs/WEBSITE.md)

Flint is [MIT licensed](LICENSE). Third-party software and models retain their respective licenses; see [third-party notices](THIRD-PARTY-NOTICES.md).
