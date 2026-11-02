# Flint manifest contract 0.2

[简体中文](../CN/COMMAND-CONTRACT.md)

Flint 0.1.9 and later can import a project's launcher.yaml or equivalent JSON. Flint 0.1.8 and earlier accept version 0.1. Existing registrations and 0.1 manifests remain usable. A web app, HTTP API, or MCP HTTP server can use this interface without a Flint SDK or management API.

## Minimal manifest

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

The service must read LAUNCHER_PORT or map the port into args or env. Import the file, review it, and save the instance. If it declares install steps, choose **Install dependencies** before starting. Import and save do not execute commands. Successful installation leaves the service stopped. Manual shell commands remain available in the UI.

## Fields

| Field | Required | Meaning |
|---|---|---|
| schemaVersion | Yes | String "0.2", independent of the app version |
| id | Yes | Stable application ID, up to 120 bytes. Start with a lowercase letter and use lowercase letters, digits, dots, hyphens, or underscores. |
| version | Yes | Release string, up to 80 bytes. A prefixed tag is allowed. This value is metadata. |
| name | Yes | Nonempty display name, up to 120 bytes |
| environment.mode | Yes | existing runs in the project directory with the project's own dependencies |
| parameters.port.default | Yes | Integer from 1024 to 65535. The user can change it. A conflict fails startup. |
| platforms | No | List of macos, linux, windows. An empty or missing list permits import on any platform. |
| install | No | Up to 32 structured commands executed in order. Every step must succeed. Defaults to empty. |
| service.protocol | Yes | http-api (including web apps) or mcp-http |
| service.cwd | No | Defaults to the manifest directory. Relative and absolute paths are accepted. The directory must exist at import. |
| service.run | Yes | One structured startup command |
| service.healthPath | For MCP | HTTP defaults to / if omitted or empty. MCP requires an explicit nonempty endpoint, such as /mcp. |
| extensions | No | Optional tool metadata. Flint ignores it and does not persist it in the instance. |

The parser rejects unknown fields, schema versions, protocols, and environment modes. Keep executable behavior in the versioned fields. An import creates a local instance UUID separate from the application ID. You can register the same app on different ports or at different versions. Changes to a file do not update existing instances. Reimport to change run/install commands. Name, directory, port, and health path are editable in the UI.

## Structured commands

Use the same format for service.run and each install step:

```yaml
program: node
args: [server.mjs]
env:
  PORT: "${port}"
  HOST: "${host}"
```

- program is one executable name or path. Pass arguments separately. Placeholders are invalid in program.
- args is an array of literal strings, empty by default. Spaces, quotes, semicolons, and shell substitutions inside an argument remain literal.
- env maps names to strings and overrides inherited values. Names use ASCII letters, digits, and underscores, with a nonnumeric first character. LAUNCHER_ names are reserved. Keep secrets in local configuration or the inherited environment.
- args and env accept only `${port}`, `${host}`, `${dataDir}`, and `${cacheDir}`. Unknown or unclosed placeholders fail. There is no recursive, shell, or tilde expansion.
- On macOS/Linux a fixed login-shell wrapper resolves the runtime and invokes the program with positional arguments. Use an executable path to select a specific runtime. Interactive aliases and functions are unreliable.
- Windows execution is unverified. Shell scripts, POSIX assignments and .cmd wrappers are platform-specific. The example manifests declare macOS and Linux.
- Put complex setup in a project script and call it. Explicitly invoking a shell gives that shell responsibility for interpreting its input.

## Installation and ownership

```yaml
install:
  - program: npm
    args: [ci]
  - program: npm
    args: [run, build]
```

For Python with uv:

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

Commit the lockfile. Each install step runs in service.cwd with the instance environment, shared log, and cancellation support. Each has a two-hour timeout. Failure skips later steps and blocks startup. A retry starts from step one. Changing the directory or install steps invalidates installation. If installation is unnecessary, omit install and document prerequisites.

An existing project's node_modules, .venv, and models belong to that project. Removing a Flint instance can clear its own data and cache while preserving the external directory. Custom manifests cannot request managed Node, automatic release downloads, or platform-specific installation assets. MinerU uses a separate built-in adapter. Commands execute with the user's permissions.

## Runtime contract

1. Stay in the foreground without interaction, browser launch, nohup, or daemonization.
2. Bind to loopback and honor the configured port through arguments or environment.
3. Flint sends an interrupt on stop and escalates if needed. Release child processes and the port. Detached daemons may escape process-group cleanup.
4. Flush logs to stdout/stderr, omit secrets, and keep dependency upgrades out of startup.
5. Return HTTP 2xx on the readiness path when able to accept requests. A web app may use its home page. Test application behavior separately.
6. For MCP, expose Streamable HTTP. Flint probes initialization with protocol 2025-03-26 and accepts JSON/SSE responses. It does not probe stdio, legacy SSE transport, or authenticated endpoints.

| Variable available during install and run | Value |
|---|---|
| LAUNCHER_PORT | Configured port |
| LAUNCHER_HOST | 127.0.0.1 |
| LAUNCHER_INSTANCE_ID | Local UUID |
| LAUNCHER_DATA_DIR | Persistent instance directory, created before execution |
| LAUNCHER_CACHE_DIR | Rebuildable instance cache, created before execution |

For standalone use, prefer explicit CLI flags, then LAUNCHER_* values, then application defaults. Custom projects inherit the environment without Flint's MinerU-specific overrides.

## MCP example

This assumes the project implements the stated flags and an MCP endpoint:

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

## Validation and migration

The [JSON Schema](../../schema/launcher.schema.json) covers 0.1 and 0.2. The importer also checks directory existence, platform match, and UTF-8 byte limits for names and versions. JSON Schema counts characters.

From a Flint checkout with Rust installed:

```sh
npm run manifest:check -- /absolute/path/to/launcher.yaml
```

The checker parses the manifest offline. Verify installation, requests, port changes, and shutdown separately. In 0.1, service.command uses shell syntax and expands only `${port}`. In 0.2, replace command with run, remove category: service and add id and version. See the [legacy example](../../examples/http-service/launcher.yaml), [Node](NODE-EXAMPLE.md), [Python/uv](PYTHON-EXAMPLE.md), and [coding-agent guide](AI-INTEGRATION.md).
