# Roadmap

## Completed

### v0.1.0 (Rust)

- Single-package resolution from marketplace, system, and Flutter sources
- Configurable resolution strategies (MarketplaceFirst, SystemFirst, OnlySource, SearchAll)
- Cross-source unified search with marketplace-priority dedup
- System package database via apt/dpkg
- Marketplace registry via local filesystem (stub)
- Heuristic source detection from package name format
- Full serde support on all public types
- Structured logging via tracing
- P(-1) scaffold hardening: non_exhaustive, must_use, PartialEq/Eq, benchmark suite, documentation

### v0.1.0-cyrius (Cyrius port)

- Full port from Rust to Cyrius 5.1.7 (2143 → 2174 lines)
- Manual struct layout with alloc/store64 constructors and load64 accessors
- JSON serialization and deserialization for all public types
- All 7 error variants with complete display formatting
- Manifest dependency parsing from JSON
- Registry reload() and install_package() stub
- 140 tests, 11 benchmarks, 3 fuzz harnesses
- 115KB standalone x86_64 ELF binary

## Backlog (prioritized)

### P1 — Dependency Resolution (Critical) — COMPLETE

- [x] Version constraint parsing (SemVer: >=, ^, ~, =, >, <, <=, *) — `constraint_parse`, `constraint_matches`
- [x] Dependency graph construction (transitive resolution) — `dep_graph_new`, `dep_graph_add`, `resolver_resolve_all`
- [x] Topological sort for install order (Kahn's algorithm) — `dep_graph_topo_sort`
- [x] Cycle detection (DFS with 3-color marking) — `dep_graph_detect_cycle`
- [x] Conflict detection (incompatible version constraints) — `constraints_compatible`
- [x] Diamond dependency handling (deduplicated via hashmap) — tested in `test_diamond_deps`

### P2 — Zugot Integration (High) — COMPLETE

- [x] Recipe parsing from `.cyml` files — `recipe_parse_file`, CYML parser with `[section]` support
- [x] Build-order awareness from `build-order.txt` — `read_build_order`, `recipe_db_order`
- [x] Source URL resolution for `github_release` shorthand — parsed in `recipe_parse_file`
- [x] SHA256 field extraction — stored in Recipe, verification delegated to ark/takumi

### P3 — Caching & Performance (Medium)

- [ ] Persistent resolution cache (skip re-resolution if unchanged)
- [ ] Index caching for marketplace/system packages
- [ ] Incremental resolution

### P4 — Mela Integration (Medium)

- [ ] Replace registry stub with real mela marketplace API
- [ ] Package metadata sync
- [ ] Trust integration with sigil

### P5 — Error Quality (Medium-Low) — partially complete

- [x] Dedicated error types (7 variants, full display)
- [x] Package name validation errors
- [x] Conflict explanation (which constraints conflict)
- [x] Suggestion engine for typos (Levenshtein distance)
- [x] Resolution trace with `resolver_with_trace(r)`

## Future

- Lockfile generation and consumption
- Parallel resolution across sources
- Plugin system for additional package sources
- Resolver constraint language specification

## v1.0 Criteria

- [x] P1 complete — full dependency graph resolution with cycle detection
- [x] P2 complete — zugot recipe awareness
- [x] P5 complete — conflict explanations, suggestion engine, resolution trace
- [x] Security audit — internal (P0-P2 fixed) + external (8 attack categories)
- [x] `cyrius lint` clean on all source (0 warnings)
- [x] 271 tests, 18 benchmarks, 3 fuzz harnesses — all pass
- [x] All public API documented with examples (`docs/api.md`)
- [ ] Integration tests against real apt on CI
- [ ] Benchmark regressions gated in CI
