<div align="center">
  <img src="../branding/app-icon.svg" width="104" alt="Flint logo" />
  <h1>Flint</h1>
  <p>A local service launcher with isolated installs and a macOS menu bar.</p>
</div>

**English** | [简体中文](USER-GUIDE_CN.md)

Flint brings installation, startup, port configuration, and logs for command-line services into a desktop app. MinerU is the first built-in recipe. You can also add startup commands for existing HTTP APIs, MCP HTTP servers, and web apps.

**Current version: 0.1.12 alpha · macOS Apple Silicon.** Windows, Linux, and Intel Mac builds have not been released. Flint uses Tauri 2, Rust, React, and TypeScript and is licensed under [MIT](../LICENSE).

## Installation and usage

1. [Download the macOS DMG](https://github.com/puzihao2018/Flint/releases/download/v0.1.12/Flint-0.1.12-macOS-arm64.dmg), open it, and drag Flint into Applications. The [release page](https://github.com/puzihao2018/Flint/releases/tag/v0.1.12) includes a ZIP and SHA-256 checksums.
2. Open Flint, go to **App market → MinerU**, choose a version, and install it. The first installation downloads Python, dependencies, and models; time and disk usage depend on the configuration.
3. Turn on the service, wait for its checks to pass, and copy its address into your client.
4. To change a port, stop the service, edit and save the port, then start it again.

The initial packages use **ad-hoc signing and are not notarized with Apple Developer ID**. macOS may block the first launch. This is not an App Store package; verify the source and the SHA-256 checksums supplied with the release. See the [macOS release guide](MACOS-RELEASE.md) for notarization steps.

The app declares macOS 13 as its minimum version. Installation and parsing have been tested on an Apple Silicon development machine, not across all macOS versions, chips, or memory configurations.

### Running in the background

Closing the main window hides the Dock icon and leaves Flint in the menu bar while services keep running. Click the menu bar icon for quick controls; reopening the main window restores the Dock icon. **Stop services and quit** stops services launched by Flint. Services do not start automatically the next time Flint opens. Finish ongoing tasks and quit normally before updating Flint.

### Interface language

Under **Settings & storage → Language**, choose Follow system, 简体中文, or English. Changes apply immediately to the main window and menu bar and are saved in `preferences.json` in the data directory. Chinese system locales use Simplified Chinese; other locales fall back to English. Service names, commands, and raw logs remain unchanged. See [localization maintenance](LOCALIZATION.md) to add a language.

### Removing a service

Click **Remove service** on the service detail page. You can separately select the isolated environment, models for that version, and service data/logs/dedicated cache for removal. Leave all options unchecked to remove only the registration. Running services or installation tasks are stopped first.

The dialog shows sizes and paths for the selected files. Actual reclaimed space can vary because of shared files or filesystem snapshots. Selected files are deleted directly, without going to Trash; if removal fails, the registration is retained for retry. Other MinerU versions, shared Python runtimes, and download caches are preserved. For custom services, Flint removes only its own instance data, never the external project directory. An unresolved record from an interrupted process must still be acknowledged and cleared first.

### MinerU versions

| Release | API | Compatibility |
|---|---|---|
| 4.0.12 | `/v1/uploads`, `/v1/parse/jobs`, ZIP results | Local parsing verified with llm-for-zotero 3.9.15 |
| 4.1.2 | `/v1/tasks` | Requires a client that supports the new Tasks API |

Each version has its own environment, configuration, model directory, and port. Use the service address shown in Flint; ports are not fixed to 8000 or 8001. MinerU provides an **HTTP API**, not an MCP server. Flint's current recipe does not set an API key for this local API.

On macOS, managed MinerU instances use sequential weight loading by default to avoid an observed lock wait during cold model startup. The verified configuration is **standard + Hugging Face**; other combinations have not received equivalent validation.

## Add your own service

Under **Add service**, enter a name, an existing project directory, a foreground startup command, a port, and a health-check path. For example:

```sh
/path/to/uv run --frozen --no-sync myserver --host 127.0.0.1 --port ${port}
```

`${port}` is replaced with the port configured in Flint. Your program must support the corresponding argument. Flint stops services by sending an interrupt signal, so no separate stop command is needed. Do not use `nohup`, `&`, or daemonize the process.

You can also import a [`launcher.yaml` 0.2 manifest](../examples/node-webapp/launcher.yaml), with an app ID/version, structured arguments and environment variables, and explicit installation steps. Version 0.1 manifests remain compatible. Importing and saving do not execute code. If the manifest includes installation steps, click **Install dependencies**; all steps must succeed before the service can start. Custom services use an existing project directory, and Flint does not uninstall that project's `node_modules` or `.venv`.

Developer resources: [integration contract](COMMAND-CONTRACT.md) · [instructions for AI](AI-INTEGRATION.md) · [JSON Schema](../schema/launcher.schema.json) · [Node example](../examples/node-webapp/README.md) · [Python/uv example](../examples/python-webapp/README.md).

Run `npm run manifest:check -- /path/to/launcher.yaml` to validate a manifest without executing it. MCP support currently covers Streamable HTTP initialization checks; stdio, legacy SSE, and authenticated probe configuration are not supported.

## Development

Requirements: an Apple Silicon Mac, Xcode Command Line Tools, Python 3.9+, Node.js 20.19+ (Node 24 via `nvm use` is recommended), and rustup. The repository pins its Rust version in `rust-toolchain.toml`.

```sh
npm ci
npm run desktop      # Download and verify uv, then open the development app
npm test             # Run localization checks and serial Rust tests
npm run release:mac  # Build the app, DMG, ZIP, and checksum files
```

Build artifacts go into `release/0.1.12/` and are not committed to Git. `uv` is downloaded from a pinned official release, with SHA-256 checks for both its archive and binary. End users do not need to install Python or uv in advance, and the uv binary is not committed to the source repository. `Cargo.lock`, `package-lock.json`, and the MinerU dependency locks are committed.

Use a separate data directory for development, for example `FLINT_DATA_DIR=/tmp/flint-dev npm run desktop`. Do not let a second instance or headless validator write to the installed app's data directory while it is running. Separate development data directories still share the single-instance restriction associated with the same application identifier.

## Repository layout

| Location | Contents |
|---|---|
| `src/` | Main window, menu bar quick controls, and shared styles |
| `src-tauri/src/` | Service lifecycle, managed installs, health checks, and manifest import |
| `registry/` | Supported MinerU release metadata and dependency locks |
| `scripts/` | Build resources, license notices, and macOS packaging |
| `examples/` | Importable example services |
| `.github/` | CI, draft releases, and issue templates |
| `docs/` | Command contract, release recipes, release notes, and validation scope |

On macOS, data is stored in `~/Library/Application Support/local.flint.services/`. This includes apps, models, configuration, caches, and logs. Replacing the application does not delete this data. See [architecture and storage](ARCHITECTURE.md).

## Status and roadmap

Available now: multiple built-in MinerU versions, isolated environments, start/stop controls, port configuration, HTTP/MCP HTTP health checks, custom commands, logs, and menu bar controls. The app market is currently a recipe list maintained and shipped with Flint.

Planned: more verified service recipes, AI-assisted recipe adaptation, dedicated web app workflows, automatic port discovery, and other desktop platforms. Launch at login, automatic restart, in-app updates, multi-service orchestration, and automatic installation of arbitrary GitHub releases are not currently supported.

[Validation scope](VALIDATION.md) · [Changelog](../CHANGELOG.md) · [Contributing](../CONTRIBUTING.md) · [Third-party notices](../THIRD-PARTY-NOTICES.md)
