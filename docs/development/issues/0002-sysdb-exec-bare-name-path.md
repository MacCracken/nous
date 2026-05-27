# 0002 — sysdb: `exec_capture` bare command names never launch (empty envp, no PATH search)

- **Status:** Fixed in nous 1.2.0 (`which_path`/`tool_path` full-path resolution)
- **Filed:** 2026-05-26
- **Severity:** High — the system-package backend silently returned no data
- **Component:** `src/sysdb.cyr` (via `lib/process.cyr` `exec_capture`)
- **Pre-existing:** yes — not introduced by the 6.0.1 bump; `exec_capture` is
  byte-identical on this point between the 5.7.29 and 6.0.1 stdlib. Distinct
  from the codegen bug in [issue 0001](0001-cyrius-6.0.1-vec-get-recompute.md).

## Summary

`lib/process.cyr`'s `exec_capture(args, …)` forks and, in the child, calls
`sys_execve(cmd, argv, envp)` where `cmd = argv[0]` and `envp` is **empty**.
`execve(2)` does **not** search `PATH` (only `execvp`-family does), and there is
no `PATH` in the child env anyway. So a **bare** `argv[0]` like `"dpkg-query"`
or `"apt-cache"` fails to launch — the child hits `sys_exit(127)`, the pipe
yields no bytes, and `exec_capture` returns `0`.

`src/sysdb.cyr` built its argv with bare names:

```cyrius
vec_push(args, "apt-cache");   # sysdb_search
vec_push(args, "dpkg-query");  # sysdb_is_installed / sysdb_get_installed
```

so `sysdb_search` returned an empty result and `sysdb_is_installed` returned 0 —
regardless of whether the packages existed. (`sysdb_list` / `sysdb_updates` were
unaffected: they go through `cmd_run`, which execs `"/bin/sh" "-c" <cmdline>` —
an absolute path to the shell, which then resolves the tool via PATH.)

## Why it only showed up in CI

`test_integration_apt` is gated on `sysdb_available()`, which is true only when
`apt-cache`/`dpkg-query` are on PATH. The dev box is **Arch (no dpkg/apt)**, so
the asserts **skip** locally and the suite was a green 271/0; **CI (Ubuntu)** has
apt, so the asserts ran and failed:

```
FAIL: search found results
FAIL: coreutils installed (got 0, expected 1)
cyrius test exit code: 2
```

(The `sysdb_info` asserts in the same test were already guarded by
`if (info != 0)`, so they didn't fire — which is why exactly two assertions
failed.)

## Fix

Resolve the tool to an absolute path in the **parent** (which has `PATH`) and
hand `exec_capture` the full path. New helpers in `src/command.cyr`:

- `which_path(name)` — walks `getenv("PATH")` and returns the first
  `<dir>/<name>` that exists (cstr), else 0. `which_exists` now delegates to it.
- `tool_path(name)` — `which_path` with a bare-name fallback (preserves old
  behaviour when resolution fails).

`sysdb_search`, `sysdb_is_installed`, `sysdb_get_installed`, and `sysdb_info`
now push `tool_path("apt-cache")` / `tool_path("dpkg-query")` as `argv[0]`. The
array-based exec (no shell, no injection — per CLAUDE.md) is preserved; only
`argv[0]` becomes absolute.

This keeps `exec_capture` (stdlib) untouched — the fix lives entirely in nous.

## Verification

Mechanism confirmed locally (Arch) with a tool that *is* present:

```
tool_path("ls")  ->  /usr/bin/ls
exec_capture(["/usr/bin/ls", "/"], …)  ->  output (works)
exec_capture(["ls", "/"], …)           ->  0 bytes (the bug)
```

`integration_apt`'s apt-backed asserts cannot run on the dev box (no dpkg), so
final confirmation is via CI on Ubuntu, where `/usr/bin/apt-cache` and
`/usr/bin/dpkg-query` resolve. The apt-output **parsing** is unchanged, so once
exec returns data the existing logic applies. (It was briefly de-nested under
the since-retracted [issue 0001](0001-cyrius-6.0.1-vec-get-recompute.md); that
de-nesting was reverted in 1.2.5 and never affected parsing behaviour.)
