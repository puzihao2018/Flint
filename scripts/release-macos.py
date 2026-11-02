#!/usr/bin/env python3
"""Build a macOS arm64 alpha release: signed app ZIP, drag-install DMG, hashes."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import plistlib
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def run(*args, **kwargs):
    return subprocess.run(list(args), cwd=ROOT, check=True, **kwargs)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--skip-build", action="store_true", help="Package an already built app; still verify identity, version, architecture and signature")
    parser.add_argument("--signing", choices=["adhoc", "preserve"], default="adhoc", help="preserve keeps an existing Developer ID signature instead of re-signing")
    args = parser.parse_args()
    if platform.system() != "Darwin" or platform.machine() != "arm64":
        parser.error("Only macOS Apple Silicon is supported by this release recipe")
    run("python3", "scripts/check-version.py")
    if not args.skip_build:
        run("npm", "run", "package")
    metadata = json.loads(subprocess.check_output([
        "cargo", "metadata", "--no-deps", "--format-version", "1", "--manifest-path", "src-tauri/Cargo.toml"], cwd=ROOT))
    app = Path(metadata["target_directory"]) / "release/bundle/macos/Flint.app"
    version = json.loads((ROOT / "package.json").read_text())["version"]
    info = plistlib.loads((app / "Contents/Info.plist").read_bytes())
    if info["CFBundleShortVersionString"] != version or info["CFBundleIdentifier"] != "local.flint.services":
        parser.error("Built app identity/version does not match source")
    arch = subprocess.check_output(["lipo", "-archs", str(app / "Contents/MacOS/service-launcher")], text=True).strip()
    if arch != "arm64":
        parser.error("Expected an arm64 app, got " + arch)
    if not (app / "Contents/Resources/resources/licenses/THIRD-PARTY-LICENSES.txt").is_file():
        parser.error("Build must include third-party license notices")
    destination = ROOT / "release" / version
    destination.mkdir(parents=True, exist_ok=True)
    stem = f"Flint-{version}-macOS-arm64"
    # A clean copy avoids Finder/iCloud metadata invalidating code signing.
    with tempfile.TemporaryDirectory(prefix="flint-release-") as temporary:
        temporary = Path(temporary)
        layout = temporary / "layout"
        layout.mkdir()
        clean = layout / "Flint.app"
        run("ditto", "--norsrc", "--noextattr", str(app), str(clean))
        if args.signing == "adhoc":
            run("codesign", "--force", "--deep", "--sign", "-", str(clean))
        else:
            signature = subprocess.run(["codesign", "-dv", "--verbose=4", str(clean)], capture_output=True, text=True, check=True)
            if "Authority=Developer ID Application:" not in signature.stderr:
                parser.error("--signing preserve requires a Developer ID Application signature")
        run("codesign", "--verify", "--deep", "--strict", str(clean))
        run("ditto", "-c", "-k", "--norsrc", "--noextattr", "--keepParent", str(clean), str(temporary / f"{stem}.zip"))
        (layout / "Applications").symlink_to("/Applications")
        (layout / "INSTALL.txt").write_text("Flint: drag Flint.app to Applications.\nQuit an older Flint before replacing it; quitting stops its services.\nThis alpha is not Apple-notarized. See the GitHub release notes before installing.\n")
        run("hdiutil", "create", "-volname", "Flint", "-srcfolder", str(layout), "-format", "UDZO", str(temporary / f"{stem}.dmg"))
        run("hdiutil", "verify", str(temporary / f"{stem}.dmg"))
        for extension in ("zip", "dmg"):
            shutil.copy2(temporary / f"{stem}.{extension}", destination / f"{stem}.{extension}")
    shutil.copy2(ROOT / "LICENSE", destination / "LICENSE")
    shutil.copy2(ROOT / "docs/EN/RELEASE-NOTES.md", destination / "RELEASE-NOTES.md")
    build_info = {"version": version, "platform": "macos-arm64", "minimumSystemVersion": info.get("LSMinimumSystemVersion"),
                  "signing": args.signing, "notarization": "not verified by this script",
                  "uv": json.loads((ROOT / "scripts/uv.lock.json").read_text()),
                  "rust": subprocess.check_output(["rustc", "--version"], text=True).strip(),
                  "node": subprocess.check_output(["node", "--version"], text=True).strip()}
    (destination / "BUILD-INFO.json").write_text(json.dumps(build_info, indent=2) + "\n")
    artifacts = sorted(p for p in destination.iterdir() if p.is_file() and p.name != "SHA256SUMS.txt")
    (destination / "SHA256SUMS.txt").write_text("".join(hashlib.sha256(p.read_bytes()).hexdigest() + "  " + p.name + "\n" for p in artifacts))
    print("Release prepared:", destination)


if __name__ == "__main__":
    main()
