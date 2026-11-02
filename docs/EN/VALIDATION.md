# Validation record

[简体中文](../CN/VALIDATION.md)

## Prototype on Apple Silicon, October 29–30, 2026

MinerU 4.1.2 was installed in isolation and used to parse text and scanned PDFs. Tests changed its port from 8000 to 3000, stopped the service, and verified cleanup on quit. A separate 4.0.12 instance passed file upload and ZIP retrieval through its jobs API.

Setting HF_DEACTIVATE_ASYNC_LOAD=1 in managed macOS processes addressed an observed safetensors loading stall. Zotero subsequently parsed an 18-page paper and its batch queue advanced. Completion of the entire library was not established. In 0.1.5 the user verified that closing the installed app's window hid the Dock icon and reopening restored it. Personal paths, titles and raw logs are excluded.

## Repository checks

npm test uses temporary data directories and loopback ports. It covers configuration, port conflicts, repeated startup, stop, process-group cleanup, crashes, failed health checks, interruption records, MCP initialization with JSON/SSE responses, installation cancellation, and manifest parsing. Tests run serially. Other system processes can still claim a released port.

npm run release:mac validates version consistency, pinned uv checksums, app identity and architecture, third-party notices, signing, and the DMG. CI does not download models. Real MinerU checks require an explicitly supplied fixture. A headless verifier must have its own data directory.

Coverage excludes Intel, Windows, Linux, the full macOS hardware and OS matrix, every MinerU profile and model source, and Developer ID notarization.

## Release checks

| Version or milestone | What was verified |
|---|---|
| Standalone repository, Oct 30 | Clean npm install, pinned uv, Rust/TypeScript release build. Local Node 20.20.2 and Rust 1.99.1, CI configured for Node 24. One manifest and nine lifecycle tests passed. All 380 dependency notices included license text. The DMG contained the Applications shortcut, app signature and image were checked, and ZIP/DMG checksums were generated. Tag mismatch rejection and workflow syntax were checked locally. |
| 0.1.6 | One unit and 11 lifecycle tests covered persistent deletion, empty-list restart, file retention, version isolation, active/interrupted guards, and rollback on save failure. A native test instance verified confirmation, cancellation, empty restart, and adding from the catalog. |
| 0.1.7 | One unit and 17 lifecycle tests covered version-scoped optional uninstall, shared caches, automatic stop, external projects, duplicate versions, symlink boundaries, and persistence before deletion. The native UI showed three choices, sizes from 0 to 60 KiB, paths, and cancellation. File deletion was exercised in test data. Installed app, ZIP/DMG, and checksums were checked. |
| 0.1.8 | Two unit and 19 lifecycle tests included IPv4/IPv6 loopback and wildcard listeners, PID/address reporting, EADDRINUSE, and stale-log isolation. With an existing IPv6 Node listener on port 3000, preflight reported the conflict and the original site continued returning HTTP 200. Packages passed verification. |
| 0.1.9 | Four unit and 22 lifecycle tests passed, including 0.2 parser errors, installation failure/cancellation, persistence, port changes, and literal arguments. Node and Python examples installed in temporary directories, returned HTTP 200 on two ports, and released them on stop. Python used bundled uv and an existing interpreter offline. Schema, examples, and complete documentation manifests passed Draft 2020-12 validation. Native import preview, signature, DMG, and ZIP were checked on Apple Silicon. |
| 0.1.10 | Five unit and 22 lifecycle tests passed. Checks covered locale fallback, persistence, invalid preferences, failed writes, 263 bilingual resource entries, placeholders, and nested errors. The installed app switched languages immediately in both windows and retained English across restart. Native file pickers followed macOS. TypeScript, signing, and DMG checks passed. |
| 0.1.11 | Localization, frontend build, version consistency, arm64 packaging, and signing passed. The installed app showed only the current service title with the unused Workspace level removed. |
| 0.1.12 | Checks passed for 264 bilingual messages, TypeScript/Vite, versions, signed app, DMG, and ZIP. The installed app's service detail, catalog, and settings showed functional text without decorative subtitles. |

## Website and public distribution

The static bilingual site was tested at desktop and 390 px mobile widths. Checks covered overflow, keyboard navigation, simulated service controls, menu bar preview, persistent language selection, copy feedback, and the displayed YAML. No browser errors were observed in those checks.

The v0.1.12 tagged macOS workflow and Pages deployment subsequently succeeded. The public DMG and ZIP were downloaded without signing in and checked against the release SHA-256 list.
