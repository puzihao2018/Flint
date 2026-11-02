# Changelog

[简体中文](../CN/CHANGELOG.md)

## 0.1.13 — 2026-11-02

Fixed the alignment of service icons in the menu bar panel, including the MinerU “M”.

## 0.1.12 — 2026-10-30
Removed promotional subtitles, decorative labels, and duplicate hints. Pages now use functional titles and retain configuration, status, errors, and actions.

## 0.1.11 — 2026-10-30
Removed the Workspace breadcrumb that lacked a corresponding overview page. The header shows the current service or page.

## 0.1.10 — 2026-10-30
Added system, Simplified Chinese, and English preferences across the main window, menu bar panel, and native tray menu. Centralized translations for service state, installation, removal, configuration, catalog copy, and common diagnostics. Tests cover keys, placeholders, dynamic errors, preference persistence, and failed writes.

## 0.1.9 — 2026-10-30
Added launcher.yaml 0.2 with stable app identity, version, structured commands, environment mappings, platforms, and explicit installation. Version 0.1 remains supported. HTTP checks default to the home page. MCP requires an explicit endpoint. Added offline validation, JSON Schema, integration guidance, and runnable Node and Python examples. Custom projects no longer receive MinerU-specific overrides. Failed or cancelled installation blocks startup.

## 0.1.8 — 2026-10-30
Port preflight checks IPv4 and IPv6 loopback and wildcard listeners. macOS reports the process, PID, and address. Startup identifies port races and EADDRINUSE without attributing errors from an earlier log to a new run.

## 0.1.7 — 2026-10-30
Added separate uninstall choices for managed environments, models, and instance data, with sizes and paths. Flint stops managed operations before cleanup, respects version boundaries, and preserves shared runtimes and downloads. Cleanup avoids following symlinks into external files. A failure leaves the registration available for retry.

## 0.1.6 — 2026-10-30
Added registration removal for MinerU and custom services with native confirmation. This version blocked removal of running or interrupted instances and preserved the record on save failure. An empty list persisted after deleting the final instance. Services could be added again from the catalog.

## 0.1.5 — 2026-10-30
Closing the macOS window hides the Dock icon while keeping the menu and services active. Reopening restores the window and icon. Retained the two isolated MinerU releases and serial loading. Added the standalone MIT repository, DMG/ZIP packaging, and a draft release workflow.

## 0.1.4 — 2026-10-30
Set HF_DEACTIVATE_ASYNC_LOAD=1 for managed MinerU processes to address an observed macOS cold-start lock wait.

## 0.1.3 — 2026-10-30
Added MinerU 4.0.12 alongside 4.1.2 with version-scoped environments, models, configuration, and ports.

## 0.1.2 — 2026-10-30
Unified the main window, app icon, and menu bar around the flat three-bar F mark.

## 0.1.0 — 2026-10-29
Initial local alpha with MinerU installation, HTTP and MCP HTTP service management, custom commands, manifest import, and menu bar controls.
