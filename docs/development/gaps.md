# Nous — Known Gaps & Hardening Targets

> **Status**: Active | **Last Updated**: 2026-04-16
>
> Nous is currently a single-package resolver ported to Cyrius 5.1.7.
> This document tracks the gaps between what exists and what a production resolver needs.

---

## Current State

Single library file `src/nous.cyr` (1,276 lines) with manual struct layout.

**What works:**
- Single-package resolution across 4 sources (System, Marketplace, FlutterApp, Community)
- Configurable resolution strategy (MarketplaceFirst, SystemFirst, OnlySource, SearchAll)
- Cross-source search with marketplace-priority dedup
- Installed package listing with deterministic ordering
- Available update checking
- Heuristic source detection
- Manifest dependency parsing from JSON
- Registry reload for cache refresh
- JSON serialization/deserialization for all public types
- Error types with full display for all 7 variants
- Package name validation

**What's missing for production use:**

---

## P1 — Dependency Resolution (Critical)

The resolver currently resolves packages **individually**. It does not resolve dependency graphs.

| Gap | Description | Priority |
|-----|-------------|----------|
| **Version constraint parsing** | Parse SemVer versions and constraint operators (>=, ^, ~, =, >) | Critical |
| **Transitive dependency resolution** | `ark install A` where A depends on B which depends on C — resolve the full graph | Critical |
| **Topological sort** | Install order must respect dependency ordering (Kahn's algorithm) | Critical |
| **Cycle detection** | Detect and report circular dependencies (DFS with coloring) | Critical |
| **Conflict detection** | A needs foo>=2.0 but B needs foo<2.0 — report clearly | Critical |
| **Diamond dependency handling** | A→C and B→C with compatible constraints — deduplicate | Critical |

---

## P2 — Zugot Integration

Nous currently doesn't read zugot recipes directly. It should.

| Gap | Description | Priority |
|-----|-------------|----------|
| **Recipe parsing** | Read `.toml` recipes from zugot and extract dependency/version info | High |
| **Build-order awareness** | Respect `build-order.txt` from zugot for base system packages | High |
| **Source URL resolution** | Map `github_release` shorthand to actual download URLs | High |
| **SHA256 verification** | Validate downloaded artifacts against recipe SHA256 | High |

---

## P3 — Caching & Performance

| Gap | Description | Priority |
|-----|-------------|----------|
| **Persistent resolution cache** | Don't re-resolve packages whose recipes/versions haven't changed | Medium |
| **Index caching** | Cache marketplace and system package indices locally | Medium |
| **Incremental resolution** | Only re-resolve the subgraph affected by a change | Medium |

---

## P4 — Mela Integration

Registry stub is a placeholder. Needs real mela integration.

| Gap | Description | Priority |
|-----|-------------|----------|
| **Replace registry stub** | Connect to actual mela marketplace API | Medium |
| **Package metadata sync** | Pull package metadata from mela registry | Medium |
| **Trust integration** | Verify package signatures via sigil during resolution | Medium |

---

## P5 — Error Quality

| Gap | Description | Priority |
|-----|-------------|----------|
| **Conflict explanation** | When resolution fails, explain WHY clearly (which constraints conflict) | Medium |
| **Suggestion engine** | "Did you mean X?" for typos | Low |
| **Resolution trace** | `--verbose` mode showing the decision path | Low |

---

## Cyrius-Specific Notes

- Struct constructors (`StructName { ... }`) only work in `main()`. All library code uses `alloc`+`store64` manual constructors.
- `is_dir()` from fs.cyr is broken. Nous uses `our_is_dir()` via direct stat syscall.
- `dir_list()` from fs.cyr requires Str arguments, not C strings.
- `file_read_all()` and `file_exists()` from io.cyr require C strings.
- All struct field access uses `load64(ptr + offset)` accessor functions.

---

*This document should be consumed by any agent hardening nous or ark.*
