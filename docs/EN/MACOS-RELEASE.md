# Releasing Flint for macOS

[简体中文](../CN/MACOS-RELEASE.md)

The distribution targets Apple Silicon (arm64). Intel requires its own uv executable, dependency locks, and real MinerU testing.

## Local build

On a clean checkout, install Xcode Command Line Tools, Python 3.9+, Node 24, and rustup, then run:

```sh
npm ci
npm test
npm run release:mac
cd release/0.1.12
shasum -a 256 -c SHA256SUMS.txt
```

The DMG contains Flint.app and an Applications shortcut. The ZIP contains the same verified app. The release also includes SHA256SUMS.txt, BUILD-INFO.json, LICENSE, and release notes. User settings, logs, MinerU packages, and models are excluded.

The build checks both the pinned uv archive and executable SHA-256. It gathers third-party notices from locked dependencies, including build dependencies, and ships them in the app resources. Review the inventory when distributing a build.

The default signature is ad hoc. Disclose the absence of Apple Developer ID notarization in the release notes.

## GitHub Actions

.github/workflows/macos.yml tests and packages pull requests and pushes to main on a macos-14 arm64 runner. A version tag also creates a draft prerelease for review.

1. Verify the repository, owner, visibility, and main CI result.
2. Match versions across package.json, package-lock.json, Cargo.toml, Cargo.lock, and tauri.conf.json. Update both languages of the changelog and release notes.
3. Create and push the matching tag, for example v0.1.12.
4. Inspect the draft assets, verify the packages and checksums, and publish the prerelease.

The workflow rejects mismatched tags and refuses to overwrite a public release. GitHub's source archives omit uv and models. The build fetches uv with a pinned checksum. See [GitHub-hosted runners](https://docs.github.com/en/actions/reference/runners/github-hosted-runners).

## Developer ID and notarization

For a Developer ID release, provide the signing identity and notarization credentials through the build environment. Follow [Tauri's macOS signing guide](https://v2.tauri.app/distribute/sign/macos/). Keep credentials out of the repository.

To package an already signed app:

```sh
python3 scripts/release-macos.py --skip-build --signing preserve
```

This mode rejects ad-hoc signatures and preserves the existing Developer ID signature. Submit the final DMG with xcrun notarytool submit --wait. On success, staple and verify it, then regenerate checksums. The ZIP should contain the stapled app. The repository does not configure Developer ID credentials or automatic notarization.

## Installation check

Drag the app into Applications, eject the DMG, and launch. Check that closing the main window leaves Flint in the menu bar, then reopen it. Use a sample service to test startup, health, a port change, stop, and quit. Verify a real PDF with the selected MinerU release separately from CI. Quit the old app after active tasks finish before upgrading. The data directory persists across app replacements.
