# 0001 — Cyrius 6.0.1: typed `vec_get` miscompiles when nested as a call argument / re-evaluated

- **Status:** Open — reported upstream (Cyrius), worked around in nous 1.2.0
- **Filed:** 2026-05-26
- **Affected toolchain:** Cyrius `6.0.1` (`cycc`), x86_64-linux
- **Last good toolchain (for nous):** the `5.7.29` stdlib snapshot
- **Severity:** High — produces silently wrong results (no diagnostic, exit 0
  in isolation; downstream null-deref → SIGSEGV in the full suite)
- **Component:** code generation for typed stdlib accessors (`lib/vec.cyr`)

> This is an **upstream Cyrius** report. nous treats the Cyrius compiler as an
> external dependency and does **not** patch it. The entry exists so the
> workaround in nous source is traceable and so the bug can be verified against
> a published Cyrius release.

## Summary

Under Cyrius `6.0.1`, a typed stdlib accessor — most clearly `vec_get(v, i): i64`
— returns a **wrong value** when it is used as a **nested argument** to another
call (e.g. `str_from(vec_get(v, i))`, `map_get(m, vec_get(v, i))`) or
**re-evaluated for the same index** within one scope while other typed calls and
heap allocation happen in between. Binding the accessor result to a local
variable first makes the result correct.

The defect is **context-sensitive**: trivial reproductions pass; it manifests in
realistic code that mixes recursion, `map_new()`-keyed lookups, `map_keys()`
(which builds a vec), and `to_cstr` allocation churn.

## Proof — bisection

`lib/vec.cyr` is **byte-identical** between the `5.7.29` and `6.0.1` stdlib
snapshots **except** for added explicit return-type annotations
(`fn vec_get(v, idx)` → `fn vec_get(v, idx): i64`, and likewise for every other
`vec_*`). The function *bodies* do not change.

With nous's source unchanged, swapping only `lib/vec.cyr`:

| `lib/vec.cyr` source | reproducer result | nous test suite |
|----------------------|-------------------|-----------------|
| `5.7.29` (untyped sigs) | `cycle detected (correct)` | 271 passed, 0 failed |
| `6.0.1`  (`: i64` sigs) | `NO cycle found (WRONG)`   | SIGSEGV in `cycle_detection` |

Every other stdlib module held constant. Overlaying *only* the 6.0.1 `vec.cyr`
onto an otherwise-5.7.29 lib reproduces the failure; reverting *only* `vec.cyr`
to 5.7.29 fixes it.

## Minimal reproducer

Self-contained (stdlib only). Build against a 6.0.1 stdlib and run.
Mirrors nous's 3-colour DFS cycle detection over a cstr-keyed `map_new()` whose
node list comes from `map_keys()`. Graph `A → B → C → A` (a cycle).

```cyrius
include "lib/syscalls.cyr"
include "lib/string.cyr"
include "lib/alloc.cyr"
include "lib/str.cyr"
include "lib/vec.cyr"
include "lib/hashmap.cyr"

fn to_cstr(s) {
    if (s == 0) { return ""; }
    var data = str_data(s);
    var len = str_len(s);
    var buf = alloc(len + 1);
    memcpy(buf, data, len);
    store8(buf + len, 0);
    return buf;
}

fn dfs(edges, name, color, path, result_cycle) {
    var cname = to_cstr(name);
    map_set(color, cname, 1);
    vec_push(path, name);
    if (map_has(edges, cname) == 1) {
        var deps = map_get(edges, cname);
        var i = 0;
        while (i < vec_len(deps)) {
            var dep = vec_get(deps, i);
            var cdep = to_cstr(dep);
            var dep_color = map_get(color, cdep);
            if (dep_color == 1) { vec_pop(path); return 1; }   # back edge
            if (dep_color == 0) {
                if (dfs(edges, dep, color, path, result_cycle) == 1) {
                    vec_pop(path); return 1;
                }
            }
            i = i + 1;
        }
    }
    map_set(color, cname, 2);
    vec_pop(path);
    return 0;
}

fn detect_cycle(nodes_map, edges) {
    var color = map_new();
    var nodes = map_keys(nodes_map);
    var i = 0;
    while (i < vec_len(nodes)) {
        map_set(color, vec_get(nodes, i), 0);
        i = i + 1;
    }
    var result_holder = alloc(8);
    store64(result_holder, 0);
    i = 0;
    while (i < vec_len(nodes)) {
        # BUG TRIGGER: vec_get(nodes, i) evaluated twice in this scope.
        var name = str_from(vec_get(nodes, i));
        if (map_get(color, vec_get(nodes, i)) == 0) {
            if (dfs(edges, name, color, vec_new(), result_holder) == 1) {
                return load64(result_holder);
            }
        }
        i = i + 1;
    }
    return 0;
}

fn main() {
    alloc_init();
    var nodes_map = map_new();
    var edges = map_new();
    var da = vec_new(); vec_push(da, str_from("B"));
    var db = vec_new(); vec_push(db, str_from("C"));
    var dc = vec_new(); vec_push(dc, str_from("A"));
    map_set(nodes_map, to_cstr(str_from("A")), 1);
    map_set(nodes_map, to_cstr(str_from("B")), 1);
    map_set(nodes_map, to_cstr(str_from("C")), 1);
    map_set(edges, to_cstr(str_from("A")), da);
    map_set(edges, to_cstr(str_from("B")), db);
    map_set(edges, to_cstr(str_from("C")), dc);
    var r = detect_cycle(nodes_map, edges);
    if (r != 0) { println("RESULT: cycle detected (correct)"); }
    else { println("RESULT: NO cycle found (WRONG)"); }
    return 0;
}

var ec = main();
syscall(60, ec);
```

### Expected
```
RESULT: cycle detected (correct)
```
### Actual (Cyrius 6.0.1)
```
RESULT: NO cycle found (WRONG)
```

The back edge `C → A` is missed: the colour lookup for the gray node `A`
returns a value other than `1`, so a real cycle reads as "no cycle".

### The one-line fix that flips it back to correct
```cyrius
        # cache the accessor in a local instead of re-evaluating it
        var nd = vec_get(nodes, i);
        if (map_get(color, nd) == 0) {
            var name = str_from(nd);
            ...
```

## Impact on nous

When nous moves onto the 6.0.1 stdlib (1.2.0 de-vendor), the bug surfaced as:

- `dep_graph_detect_cycle` — a cyclic graph read as acyclic → caller did
  `vec_len(0)` → **SIGSEGV** in the test suite.
- `dep_graph_topo_sort` — emitted a 1-element order for a 3-node chain.
- `resolver_resolve_all` — resolved only the seed package (deps lost), because
  `mpkg_to_resolved` built the dependency list with `str_from(vec_get(dk, i))`.

All produced **silently wrong** results — exactly the class of failure a package
resolver must never ship (resolution must be deterministic and cycle detection
is mandatory).

## Workaround (applied in nous 1.2.0)

Bind the typed accessor to a local before using it in another call or re-using
it. Sites fixed:

| File | Function | Pattern removed |
|------|----------|-----------------|
| `src/graph.cyr` | `dep_graph_detect_cycle` | double `vec_get(nodes, i)` |
| `src/graph.cyr` | `dep_graph_topo_sort` | double `vec_get(nodes, i)`; `to_cstr(vec_get(deps, j))` ×2 |
| `src/graph.cyr` | `resolver_resolve_all` | `map_has(visited, to_cstr(dep))` |
| `src/recipe.cyr` | `recipe_db_load` | `str_from(vec_get(categories, ci))` |
| `src/resolver.cyr` | `mpkg_to_resolved` | `str_from(vec_get(dk, i))` |

## Remaining at-risk sites (tracked, not yet hardened)

These use the same `outer(vec_get(...))` nesting shape but currently pass the
full 271-test suite (including `integration_apt`, which exercises sysdb against
real `dpkg` output). Scheduled for the 1.2.1 hardening sweep
(`docs/development/roadmap.md`); de-nesting is a safe, behaviour-preserving
transform:

- `src/sysdb.cyr`: lines ~33, 83, 85, 97, 100, 101, 172 (`str_trim`/`str_split`/
  `str_to_int`/`str_contains` wrapping `vec_get`)
- `src/source.cyr`: line ~133 (`str_from(vec_get(rkeys, i))`)
- `src/recipe.cyr`: lines ~284, 513 (`str_trim`/`to_cstr` wrapping `vec_get`)

## Verification

```sh
# reproduce (against a 6.0.1 stdlib): prints "NO cycle found (WRONG)"
cyrius build repro.cyr repro && ./repro
# nous suite, post-workaround: 271 passed, 0 failed
cyrius test tests/nous.tcyr
```

When a Cyrius release fixes the codegen, re-run the reproducer; once it prints
`cycle detected (correct)` the nous workarounds can be reverted (revert the
commits referencing this file) and the pin bumped.
