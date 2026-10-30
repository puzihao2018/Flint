#!/usr/bin/env python3
"""Fetch the pinned uv executable without running an upstream install script."""
import hashlib
import io
import json
import os
from pathlib import Path
import platform
import tarfile
import urllib.request

ROOT = Path(__file__).resolve().parents[1]


def prepare():
    if platform.system() != "Darwin" or platform.machine() != "arm64":
        raise SystemExit("This release supports macOS Apple Silicon only.")
    lock = json.loads((ROOT / "scripts/uv.lock.json").read_text())
    target = ROOT / "src-tauri/resources/uv"
    digest = lambda data: hashlib.sha256(data).hexdigest()
    if target.exists() and digest(target.read_bytes()) == lock["binarySha256"]:
        target.chmod(0o755)
        print("Pinned uv verified:", lock["version"])
        return
    with urllib.request.urlopen(lock["url"], timeout=120) as response:
        archive = response.read()
    if digest(archive) != lock["archiveSha256"]:
        raise SystemExit("uv archive checksum mismatch")
    with tarfile.open(fileobj=io.BytesIO(archive), mode="r:gz") as tar:
        entries = [m for m in tar.getmembers() if m.isfile() and Path(m.name).name == "uv"]
        if len(entries) != 1:
            raise SystemExit("Expected exactly one uv executable")
        data = tar.extractfile(entries[0]).read()
    if digest(data) != lock["binarySha256"]:
        raise SystemExit("uv binary checksum mismatch")
    target.parent.mkdir(parents=True, exist_ok=True)
    staged = target.with_suffix(".download")
    staged.write_bytes(data)
    staged.chmod(0o755)
    os.replace(staged, target)
    print("Prepared uv", lock["version"], lock["target"])


if __name__ == "__main__":
    prepare()
