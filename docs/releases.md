# Slopity release packaging

Slopity publishes installable preview packages through `.github/workflows/release.yml`.

## Current release status

The release pipeline is intentionally a **preview** pipeline, not the final signed-production pipeline from Phase 6.

Current assets:

- Linux x86_64 `.deb`, built by Tauri in release mode.
- Linux x86_64 AppImage, built by Tauri in release mode.
- Android ARM64 debug APK.
- Android ARM64 debug AAB.
- `SHA256SUMS.txt` covering all four packages.

The Android files are debug builds and are not production-signed or Play-ready. GitHub Releases created by this workflow are therefore marked as prereleases.

## Version source

Three version declarations must match before any release build can proceed:

- `apps/slopity/src-tauri/tauri.conf.json`
- `apps/slopity/package.json`
- `[workspace.package].version` in the root `Cargo.toml`

The release tag must be exactly `v<version>`. For version `0.1.0`, the only accepted release tag is `v0.1.0`.

## Workflow behavior

### Pull requests

When release-related source or workflow files change, the release workflow builds the Linux and Android packages and uploads them as temporary workflow artifacts. It does **not** create a GitHub Release from a pull request.

This makes `.deb`, AppImage, APK, and AAB creation part of review-time validation instead of discovering bundle failures after merge.

### Pushes to `main`

When a commit reaches `main`, the workflow reads the application version and checks for the matching tag.

If the tag does not yet exist, the workflow builds all packages and creates the matching GitHub prerelease. GitHub creates the version tag at that commit as part of release creation.

If the matching tag already exists, the main-branch release workflow skips the duplicate package build and publication.

### Version tag pushes

Pushing a `v*` tag runs the same packaging workflow. The tag must exactly match the version declared by the application files or the workflow fails before packaging.

### Manual dispatch

The workflow can also be started manually with a `tag` input such as `v0.1.0`. The supplied tag must match the application version. If that tag already exists, it must point at the selected workflow commit.

## Creating the next release

1. Change the version in all three version declarations listed above.
2. Open a pull request and let both the normal CI and release-package validation finish.
3. Review the generated Linux and Android workflow artifacts if desired.
4. Merge the validated change into `main`.
5. The first `main` commit carrying a version whose `v<version>` tag does not exist publishes that version as a GitHub prerelease automatically.

A manual or explicit tag flow remains available when an automatic `main` release is not desired.

## Asset naming

Published release files are normalized to:

```text
Slopity_<version>_linux-amd64.deb
Slopity_<version>_linux-x86_64.AppImage
Slopity_<version>_android-arm64-debug.apk
Slopity_<version>_android-arm64-debug.aab
SHA256SUMS.txt
```

The workflow discovers Linux bundles from the clean Cargo workspace after Tauri finishes instead of assuming a package-local `target` directory. Android files use the exact generated paths already proven by the existing Android CI.

## What is still missing for production releases

Before the roadmap item for a reproducible signed release pipeline can be marked complete, Slopity still needs production signing and release validation. At minimum this includes Android signing-key handling, signed Android release builds, physical-device installation testing, a decision on Linux signing/distribution policy, and documented key rotation/recovery procedures.

Signing keys must never be committed to the repository.
