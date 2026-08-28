# ADR 0002: desired runtime intent and restart recovery

- Status: Accepted for the Step 023 foundation
- Date: 2026-08-29

## Context

Slopity distinguishes what the user wants from what the current process can actually observe. A runtime process, listener, thread, PID, foreground service, or last observed `running` state is process-local evidence. Persisting that evidence and replaying it after an app restart would create a false claim that the same workload is still alive.

At the same time, future restart and crash-loop policy needs a durable representation of user intent that can be migrated independently from transient runtime observations.

## Decision

Slopity defines a future desired-state document with these rules:

1. The document is versioned independently from the profile document schema.
2. A record contains only a stable `serverId` and desired intent (`stopped` or `running`).
3. It never persists PIDs, socket handles, observed runtime state, last-known bind success, foreground-service state, or other process-local proof.
4. Loading desired intent after process start does not automatically launch a runtime. Recovery must first reload and validate the current profile, re-check runtime availability, run current admission policy, and apply a future explicit restart/backoff policy.
5. Unknown schema versions fail closed. Missing records mean `stopped` unless a later migration explicitly defines another rule.
6. A runtime that exits unexpectedly while desired intent remains `running` may contribute bounded failure evidence, but Step 023 does not automatically restart it.

A prospective JSON shape is:

```json
{
  "schemaVersion": 1,
  "servers": [
    { "serverId": "example", "desiredState": "running" }
  ]
}
```

This ADR defines the persistence contract only. Step 023 deliberately does not write or restore this file.

## Consequences

- A fresh Slopity process never treats stale observed state or a persisted PID as proof of a live server.
- Future restart recovery can be added without changing profile semantics or weakening admission checks.
- Automatic restart, retry timing, crash-loop suppression, process-death recovery, and Android reboot recovery remain separate work that requires explicit policy and platform proof.
