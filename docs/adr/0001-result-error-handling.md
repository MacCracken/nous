# 0001 — `Result<T, E>` error handling

**Status:** Accepted

## Context

nous resolves package dependencies across system (apt/dpkg), marketplace, and
Flutter sources. Many operations can fail in ways the caller must handle
explicitly and recover from: an invalid package name, an unreadable registry,
a malformed manifest, a version conflict, or a dependency cycle. A
package resolver must never silently drop a failure (per the project's Key
Principles — resolution is deterministic and conflicts must surface), and the
library must contain zero panic/abort paths.

Cyrius 5.8.28 introduced the generic tagged enum `Result<T, E>` (`Ok(v)` /
`Err(e)`) in `lib/result.cyr`, with `is_ok` / `is_err_result` / `payload`
helpers. Before that, the only options were raw sentinel returns (e.g. `0` /
`-1`), which conflate "absent" with "error" and are easy to ignore at call
sites.

## Decision

Fallible public operations return `Result<T, E>`:

- `Ok(value)` on success; `Err(NousError)` on failure.
- `NousError` (`src/error.cyr`) carries a structured `ErrKind` enum (7 variants:
  command-exec, registry-IO, invalid-manifest, invalid-name, version-conflict,
  dependency-cycle, …), a message, and an optional detail string.
- Callers branch on `is_err_result(res)` and unwrap with `payload(res)`; errors
  propagate up the resolver rather than aborting.
- A genuine "not found" (vs. an error) stays a plain `0` return — distinct from
  `Err` — so absence and failure are never conflated.

## Consequences

- Conflicts, cycles, and IO failures are reported to the caller with an
  actionable kind + message instead of being swallowed.
- Every public type has JSON serialize/deserialize (`src/json.cyr`), including
  the error types, so failures cross the ark boundary intact.
- Slight verbosity at call sites (`is_err_result` / `payload` dance) — accepted
  for explicitness. There is no `?`-style propagation sugar in use.
- nous is ahead of older siblings here (e.g. vidya predates `Result<T, E>` and
  uses bare integer returns); the cost is that nous depends on Cyrius ≥ 5.8.28
  (long satisfied by the 6.0.1 pin).
