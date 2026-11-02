# Architecture and storage

[简体中文](../CN/ARCHITECTURE.md)

The React main window and menu bar WebView call the same Rust engine through Tauri commands. The engine owns configuration, installation jobs, process groups, health checks, and logs. Closing a window keeps it running. Quitting Flint stops its managed services.

On macOS, the data root is `~/Library/Application Support/local.flint.services/`. Set FLINT_DATA_DIR to use a separate development directory.

| Path under the data root | Contents |
|---|---|
| services.json | Service configuration, written atomically |
| apps/mineru/&lt;version&gt;/env/ | Version-specific Python environment |
| runtimes/python/ | Python managed by uv |
| models/mineru/ and models/mineru-4.0.12/ | Models. The original 4.1 location is retained for compatibility. |
| instances/&lt;id&gt;/data/ | Application settings and persistent data |
| instances/&lt;id&gt;/service.log | Installation and runtime logs |
| cache/ | Reusable downloads |

Flint checks the selected port before startup and verifies that the listener belongs to the managed process group. Stop sends SIGINT, escalating to SIGTERM and SIGKILL after waiting. An unexpected exit leaves an interruption record. Flint never uses a stale PID to take ownership of another process. A daemon that leaves the group may escape cleanup.

Custom commands run with the current user's permissions. In a legacy shell command, Flint substitutes only the validated integer port for `${port}`. The shell interprets the remaining syntax. Import previews the settings before execution.

There is no separate supervisor or automatic restart. Logs rotate on the next start if they exceed 8 MiB. The UI reads the final 64 KiB. Model sources are not pinned to immutable commits.

For 0.2 manifests, Flint stores the app ID and version separately from the instance ID, alongside the structured run command and install steps. Existing registrations retain their original shell-command behavior. Structured arguments and environment values expand four documented variables without constructing shell code. Installation uses process groups, logs, and cancellation. A failed installation keeps installed false. Changing the working directory or install steps invalidates the installation state. The importer and offline validator share a parser.
