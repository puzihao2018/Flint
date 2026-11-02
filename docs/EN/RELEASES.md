# Installing versioned service releases

[简体中文](../CN/RELEASES.md)

The catalog stores the recipe ID separately from the upstream software version. Selecting an existing release opens its instance. Selecting a new release creates an independent instance. Installation always uses the exact version stored on that instance.

## Built-in catalog

- The releases array in registry/mineru.json records version, label, python, api, releaseUrl, and notes.
- Each supported release has a full Python 3.12 dependency lock for macOS arm64. Both 4.0.12 and 4.1.2 have their own lock.
- The versioned apps directory contains the environment and requirements.lock. Instance IDs, data, and models are isolated. Version 4.1.2 retains the original paths to reuse existing installations.
- Each catalog release has one instance. A new instance receives an available port. Subsequent starts fail if its assigned port is busy.
- Stop before editing configuration. Versions cannot overwrite each other in place. Health checks verify version, API characteristics, and model ID.
- Legacy MinerU registrations without a version load as 4.1.2, preserving installation state, settings, and port.

## Adding a supported release

1. Match the official tag to the package version. Record the exact version and protocol differences in the catalog.
2. Generate a dependency lock with the recipe's Python on the target platform. Perform an isolated installation and check system prerequisites and CLI options.
3. Register the embedded lock in the installer. Adapt the command, model settings, or API checks where they differ.
4. Verify installation, cancellation and recovery, startup, port configuration, shutdown, real document parsing, and an intended client's protocol before shipping the recipe.

The current installer accepts explicitly supported releases. It rejects unknown versions. It does not discover arbitrary GitHub releases, update recipes remotely, or select binary assets across platforms.

## MinerU compatibility

4.0.12 exposes /v1/uploads and /v1/parse/jobs with ZIP results, used by llm-for-zotero 3.9.15. Version 4.1.2 uses /v1/tasks. Both use a v1 prefix but their request formats differ. Changing a port or API key does not change that protocol.

Use scripts/verify-mineru-4.0.py or scripts/verify-mineru.py. Each submits only the explicitly supplied test file to the selected local service.
