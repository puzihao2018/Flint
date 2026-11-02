# Python and uv web app example

[简体中文](../CN/PYTHON-EXAMPLE.md)

[Example project and manifest](../../examples/python-webapp). Use Flint 0.1.9+ with uv available to the login environment and Python 3.9+. Depending on its settings, uv may download Python. The example includes uv.lock and has no third-party dependencies. It was verified on macOS. Linux is declared but untested, and Windows is not declared.

1. Import the example's launcher.yaml, review the install and run commands, and save.
2. Choose **Install dependencies**. The command uv sync --frozen --no-dev creates a project-local .venv.
3. Start the service. The default port is 18374. The home page reads “Python Web App is running” and /healthz returns JSON with status 200.
4. Stop, change the port, restart, and open the new address. Stop again and check that the port was released.

The manifest maps `${host}` and `${port}` to CLI flags. The startup command uses uv run --frozen --no-sync to avoid syncing dependencies. For standalone use, run `python3 server.py --port 3000`.

Persistent data uses LAUNCHER_DATA_DIR or the standalone fallback .data. Removing the registration preserves the external project's .venv. Supply a usable uv executable independently of Flint's built-in MinerU installer.
