#!/usr/bin/env python3
"""Reject inconsistent package versions and tags before building a release."""
import json
import os
from pathlib import Path
import re

root = Path(__file__).resolve().parents[1]
version = json.loads((root / "package.json").read_text())["version"]
assert re.fullmatch(r"\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?", version), "Invalid version"
lock = json.loads((root / "package-lock.json").read_text())
assert lock["version"] == lock["packages"][""]["version"] == version, "npm lock version mismatch"
assert json.loads((root / "src-tauri/tauri.conf.json").read_text())["version"] == version, "Tauri version mismatch"
cargo = (root / "src-tauri/Cargo.toml").read_text()
assert re.search(r'(?m)^version = "([^"]+)"', cargo).group(1) == version, "Cargo version mismatch"
cargo_lock = (root / "src-tauri/Cargo.lock").read_text()
assert re.search(r'name = "service-launcher"\nversion = "([^"]+)"', cargo_lock).group(1) == version, "Cargo lock version mismatch"
if os.environ.get("GITHUB_REF_TYPE") == "tag":
    assert os.environ.get("GITHUB_REF_NAME") == "v" + version, "Tag must match package version"
print("Version verified:", version)
