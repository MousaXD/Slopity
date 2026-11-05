# Slopity release packaging

Slopity packages installable previews through `.github/workflows/release.yml`.

## Current release status

The release workflow is a **preview** pipeline, not a final production-signing pipeline.

Current published assets are:

- Linux x86_64 `.deb`, built by Tauri in release mode.
- Linux x86_64 AppImage, built by Tauri in release mode.
- Android ARM64 **debug** APK.
- Android ARM64 **debug** AAB.
- `SHA256SUMS.txt` covering all four package files.

The Android artifacts are intentionally named `*-debug.*`. They are not production-signed, Play-ready, or evidence of physical-device durability. GitHub Releases created by this workflow are therefore prereleases.

## Version source

All three version declarations must match before packaging starts:

- `apps/slopity/src-tauri/tauri.conf.json`
- `apps/slopity/package.json`
- `[workspace.package].version` in the root `Cargo.toml`

The release tag must be exactly `v<version>`. For application version `0.1.0`, the only accepted tag is `v0.1.0`.

## Workflow behavior

### Pull requests

Release-related pull requests build Linux and Android packages and upload temporary workflow artifacts. `publish` is resolved to false in the metadata job, so the GitHub Release job cannot run from a pull-request context.

This validates `.deb`, AppImage, APK, and AAB creation before merge without granting the validation jobs write permission.

### Pushes to `main`

A `main` push validates the synchronized application version and checks for `v<version>`.

- If the tag does not exist, packages are built and the workflow publishes a new GitHub prerelease for that version.
- If the tag already exists, the workflow intentionally skips duplicate packaging and publication. Existing tags/releases are not mutated by ordinary `main` pushes.

### Version-tag pushes

A `v*` tag must match the synchronized application version and its commit must be reachable from `main`. If a GitHub Release with the tag already exists, the workflow fails before packaging instead of overwriting it.

### Manual dispatch

Manual publication is allowed only when the workflow is dispatched from `main` with a new `v<version>` tag that matches application metadata. If that tag already exists, the workflow fails clearly and requires a version bump rather than reusing or moving the tag.

This is the deliberate duplicate policy: release tags and published releases are immutable to this workflow. A failed release should be diagnosed before creating a replacement version; the workflow does not silently rewrite an existing release.

## Creating the next release

1. Change the version in all three declarations above.
2. Open a pull request and require normal CI plus release-package validation to pass.
3. Review the generated Linux and Android workflow artifacts if desired.
4. Merge the validated version change into `main`.
5. The first `main` commit carrying a version whose `v<version>` tag does not yet exist publishes that version as a prerelease.

An explicit new tag push or a manual dispatch from `main` is also supported, subject to the immutable-tag rules above.

## Asset naming

Published files are normalized deterministically to:

```text
Slopity_<version>_linux-amd64.deb
Slopity_<version>_linux-x86_64.AppImage
Slopity_<version>_android-arm64-debug.apk
Slopity_<version>_android-arm64-debug.aab
SHA256SUMS.txt
```

The publication job discovers exactly one package of each expected type, renames it, and generates SHA-256 checksums immediately before publication. Missing or duplicate package outputs fail the job.

## Permissions and secrets

The workflow defaults to `contents: read`. Only the final publication job receives `contents: write`. Build jobs do not receive write permission, signing secrets are not required or invented, and the workflow never prints secret values.

## What is still missing for production releases

Before Slopity can call the release pipeline production-signed, it still needs Android signing-key handling, signed Android release builds, physical-device installation testing, a Linux signing/distribution policy decision, and documented key rotation/recovery procedures.

Signing keys must never be committed to the repository.
