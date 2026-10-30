#!/usr/bin/env python3
"""Collect dependency metadata and upstream license text for the macOS bundle.

Includes build dependencies too; this is an inventory, not a legal compatibility audit.
"""
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "src-tauri/resources/licenses"
OVERRIDES = json.loads((ROOT / "licenses/overrides.json").read_text())


def license_files(directory):
    found = []
    for p in sorted(directory.iterdir()):
        if p.is_file() and p.name.upper().startswith(("LICENSE", "LICENCE", "COPYING", "NOTICE", "COPYRIGHT")):
            found.append(p)
    return found


def main():
    entries, texts = [], ["Flint and third-party license texts\nIncludes runtime and build dependencies.\n", (ROOT / "LICENSE").read_text()]
    def add(kind, name, version, license_id, repository, directory, extra=None):
        files = license_files(directory)
        if extra and extra.is_file() and extra not in files:
            files.append(extra)
        override = OVERRIDES.get(f"{kind}:{name}@{version}", {})
        if not files and override.get("file"):
            files = [ROOT / override["file"]]
        if not files and override.get("parentPackage"):
            parent = ROOT / "node_modules" / override["parentPackage"]
            if json.loads((parent / "package.json").read_text())["version"] != version:
                raise SystemExit(f"License parent version mismatch: {name}")
            files = license_files(parent)
        if not files:
            raise SystemExit(f"Missing license text for {kind}:{name}@{version}; add a versioned override")
        source = f"https://crates.io/api/v1/crates/{name}/{version}/download" if kind == "cargo" else f"https://www.npmjs.com/package/{name}/v/{version}"
        entries.append(dict(ecosystem=kind, name=name, version=version, license=license_id, source=source,
                            repository=repository, licenseFiles=[p.name for p in files]))
        texts.append(f"\n{'=' * 72}\n{kind}: {name} {version}\nDeclared license: {license_id or 'not declared'}\nCorresponding source: {source}\n")
        for p in files:
            texts.append(f"\n--- {p.name} ---\n" + p.read_text(errors="replace") + "\n")
    metadata = json.loads(subprocess.check_output([
        "cargo", "metadata", "--locked", "--format-version", "1", "--filter-platform", "aarch64-apple-darwin",
        "--manifest-path", str(ROOT / "src-tauri/Cargo.toml")], cwd=ROOT))
    nodes = {node["id"] for node in metadata["resolve"]["nodes"]}
    for item in sorted(metadata["packages"], key=lambda x: (x["name"], x["version"])):
        if item["id"] not in nodes or item["name"] == "service-launcher":
            continue
        directory = Path(item["manifest_path"]).parent
        extra = Path(item["license_file"]) if item.get("license_file") else None
        add("cargo", item["name"], item["version"], item.get("license"), item.get("repository"), directory, extra)
    lock = json.loads((ROOT / "package-lock.json").read_text())
    for location, item in sorted(lock["packages"].items()):
        directory = ROOT / location
        if not location or not (directory / "package.json").is_file():
            continue  # Optional packages for other platforms are not bundled.
        package = json.loads((directory / "package.json").read_text())
        add("npm", package["name"], package["version"], package.get("license"), package.get("repository"), directory)
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "THIRD-PARTY-LICENSES.txt").write_text("\n".join(texts))
    (OUT / "dependencies.json").write_text(json.dumps(entries, ensure_ascii=False, indent=2) + "\n")
    print(f"Collected {len(entries)} dependency records and available license texts.")


if __name__ == "__main__":
    main()
