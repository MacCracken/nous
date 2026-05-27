# 0001 — Cyrius 6.0.1: typed `vec_get` miscompiles when nested as a call argument / re-evaluated

- **Status:** **CLOSED — NOT A DEFECT (misdiagnosis).** Withdrawn 2026-05-27.
- **Filed:** 2026-05-26 · **Closed:** 2026-05-27 (nous 1.2.5)
- **Resolution:** There is no Cyrius codegen bug. The nested / re-evaluated
  typed `vec_get` form compiles **correctly on both Cyrius 6.0.1 and 6.0.3**.
  The original "proof" was a self-defective reproducer; the real CI failure was
  entirely [issue 0002](0002-sysdb-exec-bare-name-path.md) (sysdb bare-`argv[0]`
  PATH bug). The de-nesting workaround was unnecessary and was fully reverted in
  nous 1.2.5.

> Kept for the record so the misdiagnosis is traceable. The original report
> claimed Cyrius 6.0.1 miscompiled a typed `vec_get(): i64` when used as a
> nested call argument (`str_from(vec_get(…))`) or re-evaluated for the same
> index in one scope, producing silently-wrong cycle detection / topo sort /
> resolution. **That claim does not hold up.**

## Why it was wrong

### 1. The reproducer was self-defective

The minimal reproducer's `detect_cycle` ended its success path with
`return load64(result_holder)`, but its `dfs` **never wrote `result_holder`** —
the back-edge branch did `vec_pop(path); return 1;` with no `store64`. So
`detect_cycle` returned `0` **unconditionally**, on every compiler and every
stdlib, printing `RESULT: NO cycle found (WRONG)` regardless of codegen. (The
real `src/graph.cyr:detect_cycle_dfs` *does* `store64(result_cycle, rev)` on the
back edge — the reproducer dropped exactly that line.)

Consequently the published bisection table —

| `lib/vec.cyr` source | claimed reproducer result |
|----------------------|---------------------------|
| `5.7.29` (untyped)   | `cycle detected (correct)` |
| `6.0.1`  (`: i64`)   | `NO cycle found (WRONG)`   |

— is **not reproducible**: a reproducer that always returns `0` cannot print
`cycle detected (correct)` under any toolchain. Adding the missing `store64`
makes the reproducer print `cycle detected (correct)` on **both** 6.0.1 and
6.0.3, nested form and de-nested form alike.

### 2. The real nous suite passes nested on 6.0.1 *and* 6.0.3

Re-nesting every site the issue blamed — `dep_graph_detect_cycle`,
`dep_graph_topo_sort`, `resolver_resolve_all`, `mpkg_to_resolved`,
`recipe_db_load`, and the sysdb parsers — and running `cyrius test
tests/nous.tcyr`:

| compiler | source form | suite |
|----------|-------------|-------|
| cycc 6.0.1 (6.0.1 lib) | de-nested (workaround) | 271 passed, 0 failed |
| cycc 6.0.1 (6.0.1 lib) | **nested (original)**  | **271 passed, 0 failed** |
| cycc 6.0.3 (6.0.3 lib) | nested (original)      | 271 passed, 0 failed |

No SIGSEGV, no truncated topo order, no dropped deps in any cell. The claimed
"SIGSEGV in `cycle_detection` on 6.0.1" never occurred from nesting.

### 3. The CI failures were issue 0002, not codegen

The two CI assertion failures (`search found results`, `coreutils installed`)
were in the **apt-gated sysdb path** and are fully explained by
[issue 0002](0002-sysdb-exec-bare-name-path.md): `exec_capture` execve's a bare
`argv[0]` with an empty envp, so `dpkg-query` / `apt-cache` never launched and
sysdb returned empty. The `tool_path` fix (resolve to an absolute path in the
parent) is the genuine repair. The 1.2.0 de-nesting of those same sysdb
functions was incidental and did nothing — issue 0002 alone greened CI.

## What was actually a (project) bug

[Issue 0002](0002-sysdb-exec-bare-name-path.md) — sysdb's bare-`argv[0]` exec.
It is a nous bug (our usage of `exec_capture`), not a compiler bug, and it is
fixed. That is the bug the 0001 narrative was obscuring.

## Lesson

A reproducer must be checked end-to-end before a defect is attributed to a
third-party toolchain. A green-vs-red bisection table that the pasted
reproducer cannot actually produce is the tell. When a downstream failure looks
like a compiler bug, first confirm the downstream code is correct in isolation —
here, the sysdb exec path (issue 0002) was the real cause the whole time.
