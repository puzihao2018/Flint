# Build for Flint

[简体中文](../CN/AI-INTEGRATION.md)

Use this prompt with a coding agent to add Flint support to an existing project. It targets Flint 0.1.9 or later and manifest version 0.2. See the [command contract](COMMAND-CONTRACT.md) for field definitions and the [JSON Schema](../../schema/launcher.schema.json) for editor validation.

## Prompt for a coding agent

> Inspect the project’s entry point, lockfile, and actual command-line options. Add a `launcher.yaml` with schemaVersion "0.2", a stable application ID, the application's release version and name, environment.mode set to existing, parameters.port.default, service.protocol, and service.run.
>
> Represent the startup command and each optional installation step with program, args, and optional env. Each argument is a separate string. Pass `${port}` to an existing port option, map it to the environment variable the project reads, or read LAUNCHER_PORT directly in a new project. Bind to loopback. Exit if the chosen port is busy.
>
> Keep installation separate from startup. Commit the lockfile. Put any required build in install and make each step repeatable and finite. For Python, install with uv sync --frozen and run with uv run --frozen --no-sync. For Node, use npm ci and add npm run build only if the project defines and needs it. Document prerequisites such as Node or uv.
>
> Run one foreground service without opening a browser or prompting for input. Handle interruption, release the port and child processes, and flush logs to stdout/stderr. Omit a stop command. Keep credentials out of the manifest and logs.
>
> Use http-api for a web app or HTTP API. The default health path is / and a successful check needs a 2xx response. Use mcp-http only for a real Streamable HTTP MCP endpoint and declare its path. Flint probes initialization with protocol version 2025-03-26. A stdio server needs a separate transport implementation before it can use this mode.
>
> Use LAUNCHER_DATA_DIR for persistent data and LAUNCHER_CACHE_DIR for rebuildable files. Provide defaults for standalone use. Custom projects use their existing directory. Keep extensions for optional tool metadata. Declare platforms you have tested and record the scope of validation.
>
> Deliver the manifest, the smallest necessary code changes, setup instructions, and a verification record. In a temporary instance, install, start, make an HTTP or MCP request, change ports, stop, and verify port release. Check that an installation failure prevents startup. Use a separate data directory for any test engine.

## Minimal example

For an app that reads LAUNCHER_PORT directly:

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

If the program reads PORT, use this run block under service:

```yaml
run:
  program: node
  args: [server.js]
  env:
    PORT: "${port}"
```

Omit install for a ready-to-run project. For a project with an npm build step, add:

```yaml
install:
  - program: npm
    args: [ci]
  - program: npm
    args: [run, build]
```

Installation and startup share service.cwd, which defaults to the manifest directory. Only `${port}`, `${host}`, `${dataDir}`, and `${cacheDir}` expand in args and env.

## Verify the integration

Run `npm run manifest:check -- /absolute/path/to/launcher.yaml` from the Flint checkout. This offline validator does not execute the manifest. Then check the following in Flint:

- The name and version match the project. The stable application ID is independent of the local instance ID.
- Flint can find the executable. Document local overrides without committing personal paths.
- The reported address follows the configured port. HTTP readiness returns 2xx without credentials.
- Removing the registration preserves the external project, including node_modules, .venv, and project-owned models.
- Each import creates a new instance. Editing a manifest does not update previously saved instances.

See the working [Node](NODE-EXAMPLE.md) and [Python/uv](PYTHON-EXAMPLE.md) examples. Custom manifests do not download releases, provision a managed runtime, orchestrate dependencies between services, or expose a Flint control API.
