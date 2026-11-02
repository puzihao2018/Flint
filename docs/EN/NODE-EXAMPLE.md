# Node web app example

[简体中文](../CN/NODE-EXAMPLE.md)

[Example project and manifest](../../examples/node-webapp). Use Flint 0.1.9+ with Node.js 20+ and npm available to the login environment. This example has no third-party dependencies and includes a lockfile. It was verified on macOS. Linux is declared but untested, and Windows is not declared.

1. Import the example's launcher.yaml into Flint, review the settings, and save.
2. Choose **Install dependencies**, then start the service.
3. Open the displayed URL. The home page reads “Node Web App is running” and /healthz returns JSON with status 200.
4. Stop, change the port in settings, and restart. Check the new address, stop again, and verify the port is free.

The default port is 18373. The app reads LAUNCHER_PORT and LAUNCHER_HOST. The manifest runs node server.mjs directly. For standalone use on macOS/Linux, run `PORT=3000 npm start`.

The app creates Flint's configured data directory or falls back to .data. Removing the registration preserves the project and its dependencies. This example uses npm ci --ignore-scripts. Projects that require installation scripts should use their actual setup command.
