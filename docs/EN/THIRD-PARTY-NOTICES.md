# Third-party components

[简体中文](../CN/THIRD-PARTY-NOTICES.md)

Flint source is licensed under MIT. This does not relicense third-party software or models.

- **uv 0.12.5**, Astral, MIT OR Apache-2.0: https://github.com/astral-sh/uv/tree/0.12.5 . The macOS arm64 executable is downloaded at build time from its official release, verified against both archive and binary SHA-256 in `scripts/uv.lock.json`, and bundled with Flint. Upstream license texts are in `src-tauri/resources/licenses/` and included in app resources.
- **MinerU 4.0.12 / 4.1.2**, OpenDataLab: https://github.com/opendatalab/MinerU . Installed separately at the user's request. Its code, dependencies and downloaded models retain their own upstream licenses. Flint's DMG/ZIP does not embed MinerU packages or model weights.
- Tauri, Rust crates, React, Vite, Lucide and other dependencies are pinned in Cargo.lock / package-lock.json. `scripts/generate-notices.py` collects dependency metadata and available upstream license texts into `Contents/Resources/resources/licenses/` in the app. The inventory includes build dependencies and is not a claim that every listed package is linked into the final executable.

Review this generated inventory when updating dependencies. License compatibility and model redistribution rights must be assessed for the intended distribution; no model redistribution is provided by this repository.
