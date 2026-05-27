# 0003 — Cyrius 6.0.1 `vec_get` de-nesting workaround

**Status:** Accepted (revisit when the upstream codegen bug is fixed)

## Context

Moving to the Cyrius 6.0.1 stdlib (1.2.0) exposed a compiler codegen defect:
a typed `vec_get(): i64` (and similar accessors) used as a **nested call
argument** (`str_from(vec_get(…))`, `map_get(m, vec_get(…))`) or **re-evaluated
for the same index** within one scope can return a **wrong value**. It is
context-sensitive (heap/recursion dependent) and produced silently-incorrect
resolution — a cyclic graph read as acyclic (→ SIGSEGV), topo sort truncated,
deps dropped. Full reproduction + bisection (only `lib/vec.cyr`'s added `: i64`
annotations trigger it; bodies are byte-identical) is in
[`docs/development/issues/0001-cyrius-6.0.1-vec-get-recompute.md`](../development/issues/0001-cyrius-6.0.1-vec-get-recompute.md).

The Cyrius compiler is an **external dependency**; nous does not patch it.

## Decision

Bind every typed accessor to a **local variable** before using it as a call
argument or re-using it in a scope — `var x = vec_get(v, i);` then use `x`.
Applied across the whole `src/` tree (1.2.0 + the 1.2.1 sweep): cycle detection,
topo sort, the resolver BFS, recipe loading, sort inner loops, JSON serializers,
and sysdb. The defect is reported upstream; the workaround is documented inline
at each site with a pointer to the issue.

A separate but related sysdb exec/PATH defect (bare `argv[0]` + empty envp) is
recorded in
[`docs/development/issues/0002-sysdb-exec-bare-name-path.md`](../development/issues/0002-sysdb-exec-bare-name-path.md).

## Consequences

- Correctness restored on 6.0.1: 271/0 tests, deterministic resolution, working
  cycle detection.
- A small, uniform idiom (`var x = accessor(...);`) is now the house style in
  nous and is cheap to keep consistent; the 1.2.3 return-type sweep did not
  reintroduce the bug.
- The workaround is **reversible**: when a Cyrius release fixes the codegen, re-run
  the issue-0001 reproducer; once it passes, the de-nesting can be relaxed and the
  inline notes removed.
- Annotating nous's own functions (1.2.3) is **independent** of this — it does
  not substitute for the de-nesting.
