# Flint 0.1.12 · macOS alpha

[简体中文](https://github.com/puzihao2018/Flint/blob/main/docs/CN/RELEASE-NOTES.md)

Flint installs and runs local services from the macOS menu bar. This release targets Apple Silicon and requires macOS 13 or later.

- The main window now uses functional page titles and removes redundant labels and the unused Workspace breadcrumb.
- Switch between English, Simplified Chinese, and the system language. Flint saves the selection and applies it to the main window and menu bar. Service logs retain their original text.
- Import launcher.yaml 0.2 with structured commands, an application version, explicit install steps, and offline validation. Examples cover Node and Python/uv.
- Remove a service registration or also uninstall its managed environment, models, and instance data. Review the paths and sizes before removal. Other versions and shared runtimes and downloads are preserved.
- Port conflict checks cover IPv4 and IPv6 and report the occupying process and PID on macOS. Startup diagnostics recognize EADDRINUSE.
- Install MinerU 4.0.12 and 4.1.2 side by side with separate environments and APIs. Serial weight loading addresses an observed macOS cold-start hang.
- Add existing HTTP API and MCP HTTP services with a startup command or manifest. Closing the main window leaves Flint in the menu bar and keeps managed services running.

## Downloads

Open Flint-0.1.12-macOS-arm64.dmg and drag Flint into Applications. The ZIP contains the same app. SHA256SUMS.txt lists asset checksums.

The app is ad-hoc signed and has not been notarized by Apple. macOS may block the first launch. Verify the source and checksums. This alpha has no automatic updater. Wait for active jobs to finish and quit the old version before replacing it.

MinerU installation downloads Python, dependencies, and models separately. Check that your client supports the API of the selected MinerU version. Windows, Linux, Intel Mac builds, automatic port discovery, and AI recipe adaptation are unavailable in this release. Flint source uses MIT. Third-party software and models retain their upstream licenses.
