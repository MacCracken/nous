# 0003 — Cyrius 6.0.1 `vec_get` de-nesting workaround

**Status:** **Superseded / Withdrawn (2026-05-27, nous 1.2.5).** The premise was a
misdiagnosis — there is no Cyrius codegen bug. The workaround was reverted.

## Context (original, 2026-05-26)

The 1.2.0 stdlib bump to Cyrius 6.0.1 was believed to expose a codegen defect:
a typed `vec_get(): i64` used as a nested call argument (`str_from(vec_get(…))`)
or re-evaluated for the same index in one scope appeared to return a wrong
value, producing silently-incorrect resolution. The decision (this ADR) was to
bind every typed accessor to a local before re-use, across the `src/` tree, and
to document each site inline with a pointer to
[issue 0001](../development/issues/0001-cyrius-6.0.1-vec-get-recompute.md).

## Why it was withdrawn

[Issue 0001](../development/issues/0001-cyrius-6.0.1-vec-get-recompute.md) was
**closed as not-a-defect**. The nested form compiles correctly on both Cyrius
6.0.1 and 6.0.3; the original reproducer was self-defective (its `dfs` omitted
the `store64` that the real `detect_cycle_dfs` performs, so it returned `0`
unconditionally regardless of compiler). Re-nesting every blamed site and
running the suite gives **271/0 on cycc 6.0.1 _and_ 6.0.3**. The CI failures
that triggered the investigation were entirely
[issue 0002](../development/issues/0002-sysdb-exec-bare-name-path.md) (sysdb
bare-`argv[0]` PATH bug), which is a real nous bug and is fixed independently.

## Decision (revised)

Revert the de-nesting. The 1.2.0/1.2.1 workaround sites in `src/graph.cyr`
(`dep_graph_detect_cycle`, `dep_graph_topo_sort`, `resolver_resolve_all`),
`src/resolver.cyr` (`mpkg_to_resolved`), `src/recipe.cyr` (`recipe_db_load`),
and `src/sysdb.cyr` (the apt/dpkg parsers) are restored to their original
nested forms, and every "Cyrius 6.0.1 miscompile" comment is removed (1.2.5).
nous treats the Cyrius compiler as an external dependency and does not adopt a
defensive idiom for a defect that does not exist.

## Consequences

- No behaviour change: 271/0 unchanged on 6.0.3; benchmarks unchanged.
- The codebase no longer asserts a non-existent compiler bug — future
  maintainers and the upstream Cyrius team are not misled.
- ADR [0002](0002-flat-modules-dist-bundle.md) (modules + bundle) and
  [0001](0001-result-error-handling.md) (Result error handling) are unaffected.
- The genuine project-side fix from the same arc —
  [issue 0002](../development/issues/0002-sysdb-exec-bare-name-path.md)'s
  `which_path`/`tool_path` absolute-path resolution — **stands**; it is the real
  repair the 0001 narrative had obscured.
