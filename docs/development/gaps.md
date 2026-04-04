# Nous — Known Gaps & Hardening Targets

> **Status**: Active | **Last Updated**: 2026-04-04
>
> Nous is currently a minimal resolver. This document tracks the gaps between
> what exists and what a production resolver needs. Use this as the hardening roadmap.

---

## Current State

Two source files: `lib.rs` (everything) and `registry_stub.rs` (placeholder for mela).

**What works:**
- Single-package resolution across 4 sources (System, Marketplace, FlutterApp, Community)
- Configurable resolution strategy (MarketplaceFirst, SystemFirst, etc.)
- Flat search across sources
- Installed package listing
- Available update checking
- Heuristic source detection

**What's missing for production use:**

---

## P1 — Dependency Resolution (Critical)

The resolver currently resolves packages **individually**. It does not resolve dependency graphs.

| Gap | Description | Priority |
|-----|-------------|----------|
| **Transitive dependency resolution** | `ark install A` where A depends on B which depends on C — nous must resolve the full graph | Critical |
| **Topological sort** | Install order must respect dependency ordering | Critical |
| **Cycle detection** | Detect and report circular dependencies | Critical |
| **Version constraint matching** | Support `>=`, `^`, `~`, `=` version constraints from recipes | Critical |
| **Conflict detection** | A needs foo>=2.0 but B needs foo<2.0 — report clearly | Critical |
| **Diamond dependency handling** | A→C and B→C with compatible constraints — deduplicate | Critical |

**Suggested module**: `src/graph.rs` — dependency graph construction, topological sort, cycle detection, conflict resolution.

---

## P2 — Zugot Integration

Nous currently doesn't read zugot recipes directly. It should.

| Gap | Description | Priority |
|-----|-------------|----------|
| **Recipe parsing** | Read `.toml` recipes from zugot and extract dependency/version info | High |
| **Build-order awareness** | Respect `build-order.txt` from zugot for base system packages | High |
| **Source URL resolution** | Map `github_release` shorthand to actual download URLs | High |
| **SHA256 verification** | Validate downloaded artifacts against recipe SHA256 | High |

**Suggested module**: `src/recipe.rs` — zugot recipe reader, integrates with `TakumiBuildSystem::load_recipe`.

---

## P3 — Caching & Performance

| Gap | Description | Priority |
|-----|-------------|----------|
| **Resolution cache** | Don't re-resolve packages whose recipes/versions haven't changed | Medium |
| **Index caching** | Cache marketplace and system package indices locally | Medium |
| **Incremental resolution** | Only re-resolve the subgraph affected by a change | Medium |

**Suggested module**: `src/cache.rs` — resolution cache backed by serde_json or SQLite.

---

## P4 — Mela Integration

`registry_stub.rs` is a placeholder. Needs real mela integration.

| Gap | Description | Priority |
|-----|-------------|----------|
| **Replace registry_stub** | Connect to actual mela marketplace API | Medium |
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

## Structural Recommendations

Current structure (everything in `lib.rs`) should split into:

```
src/
├── lib.rs              — re-exports, NousResolver
├── types.rs            — PackageSource, ResolvedPackage, etc.
├── graph.rs            — dependency graph, topological sort, cycle detection
├── strategy.rs         — ResolutionStrategy logic
├── system.rs           — SystemPackageDb (extracted from lib.rs)
├── recipe.rs           — zugot recipe parsing
├── cache.rs            — resolution cache
├── registry.rs         — mela marketplace client (replaces registry_stub.rs)
└── error.rs            — dedicated error types (replace anyhow in public API)
```

---

*This document should be consumed by any agent hardening nous or ark.*
