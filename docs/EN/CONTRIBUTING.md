# Contributing

[简体中文](../CN/CONTRIBUTING.md)

Start with the development commands in the [README](../../README.md) and the [user guide](USER-GUIDE.md). In a pull request, describe the problem, the resulting behavior, and the checks you actually ran. For service lifecycle changes run npm test. For UI changes run npm run build and check the affected interface. Use the locked dependencies.

For a service recipe, include the official upstream source, exact version, installation and startup commands, port argument, shutdown behavior, health check, and tested platforms. Exercise generated recipes before listing them as supported. See [versioned releases](RELEASES.md) and the [command contract](COMMAND-CONTRACT.md).

Keep environments, models, personal service configurations, API keys, documents, and private logs out of commits. Redact diagnostics. Generate installable packages through the release workflow. Verify installation, startup, parsing, stop, and tray behavior on each newly supported platform.

Contributions are provided under the project's MIT license. Retain the applicable notices and permissions for third-party code and assets.

Keep the matching English and Chinese documents under docs/EN and docs/CN up to date. For website changes run npm run site:build and inspect both languages on desktop and mobile. See [website maintenance](WEBSITE.md).
