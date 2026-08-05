# ADR 0001: runtime adapter boundary

**Status:** Accepted  
**Date:** 2026-08-05

## Context

The product must host more than Minecraft and adapt to phone capability. Different workloads require different runtime families, packaging rules, licenses, health checks, command channels, and shutdown behavior.

## Decision

Keep profile management and orchestration independent from runtime execution. Every execution family implements a `RuntimeAdapter` and reports availability explicitly. The initial repository registers unavailable placeholder adapters.

## Consequences

- The application can evolve without coupling all behavior to Java or Termux.
- Unsupported runtimes fail honestly during preflight.
- Runtime-specific security and licensing decisions remain local.
- More design and testing is required before the first real server starts.
