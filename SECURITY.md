# Security Policy

## Supported Versions

Slopity is pre-release, source-available software under active development. Security fixes are applied exclusively to the latest revision on the `main` branch.

| Version / Branch | Supported | Notes |
| ---------------- | --------- | ----- |
| `main` (trunk)   | Yes       | Actively maintained; receive security patches |
| Pre-release tags (`v0.1.x`) | Best effort | Users should update to latest `main` or latest tag |
| Older revisions  | No        | Not supported |

## Pre-Release Disclaimer

Slopity is currently pre-production software intended for evaluation, development, and non-commercial local server orchestration. It is not yet certified for hostile production environments or multi-tenant hosting.

## Reporting a Vulnerability

We take security vulnerabilities seriously. Please do **not** report security vulnerabilities through public GitHub issues or discussions.

Instead, please report security vulnerabilities privately using one of the following methods:

1. **GitHub Private Vulnerability Reporting (Recommended):**
   Navigate to the repository's **Security** tab and click **Report a vulnerability** to open an encrypted private advisory draft.

2. **Direct Contact to the Maintainer:**
   Contact **MousaXD** via GitHub profile: [https://github.com/MousaXD](https://github.com/MousaXD).

### What to Include in Your Report

To help us investigate and triage the issue quickly, please provide:

- A description of the vulnerability and its potential impact.
- Affected component(s) (`slopity-core`, `slopity-runtime-http`, Tauri IPC commands, frontend, etc.).
- Target platform and OS architecture (e.g. Linux x86_64, Android ARM64).
- Exact Git commit SHA or version tag tested.
- Step-by-step reproduction instructions or a minimal proof-of-concept.
- Any suggested remediations or mitigations if known.

### Response Process

1. **Acknowledgment:** We will acknowledge receipt of your report within 48 hours.
2. **Assessment:** We will validate and determine the severity and scope of the issue.
3. **Remediation:** A fix will be developed and tested in a private branch before public release.
4. **Disclosure:** Once the fix is released, a security advisory will be published crediting the reporter (unless anonymity is requested).

## Responsible Testing Boundaries

When testing Slopity for vulnerabilities:

- Only test against your own local devices, networks, and test installations.
- Do not attempt denial-of-service attacks or disruption of public services or infrastructure.
- Do not attempt to access, modify, or compromise accounts, devices, or data belonging to others.
- Do not execute destructive actions or distribute malicious payloads.

## Security Architecture and Threat Model

For details on Slopity's security boundaries, process execution safety, network scoping, and isolation architecture, see [docs/SECURITY_MODEL.md](docs/SECURITY_MODEL.md) and [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).
