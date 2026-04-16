# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-04-16

### Added

- **P5: Error Quality** — complete
  - Conflict explanation engine: detailed error messages identify which packages impose conflicting constraints ("A requires foo>=2.0 conflicts with B requires foo<1.0")
  - Typo suggestion engine: Levenshtein distance matching against known packages ("Package 'ngins' not found. Did you mean 'nginx'?")
  - Resolution trace: `resolver_with_trace(r)` enables step-by-step logging of resolution decisions ("marketplace: foo ... hit", "system: bar ... miss"). Trace accessible via `resolver_get_trace(r)`.

- **P(-1) Scaffold Hardening**
  - `cyrius lint` clean on all source files (0 warnings)
  - Shared `finalize_graph()` function — deduplicated cycle/conflict/topo logic between `resolver_resolve_all` and `resolver_resolve_all_with_recipes`

- **Security Audit** — internal + external research
  - Fixed P0: command injection in sysdb_* functions (replaced shell strings with array-based exec)
  - Fixed P0: NULL dereference in accessor chains (added NULL checks at every level)
  - Fixed P1: path traversal in scan_installed (reject ".." and "/" in entries)
  - Fixed P1: integer overflow in semver_parse (MAX_SAFE bound check)
  - Fixed P2: TOCTOU in scan_installed (removed file_exists before read)
  - External research: 8 attack categories analyzed (dependency confusion, supply chain, etc.)
  - Full audit report: `docs/audit/2026-04-16.md`

### Changed

- All sysdb_* functions now use array-based `exec_capture()` instead of shell string concatenation
- `finalize_graph()` extracts shared logic from both resolve_all variants
- 233 tests total, 13 benchmarks, 3 fuzz harnesses

## [0.3.0] - 2026-04-16

### Added

- **P2: Zugot Recipe Integration** — parse and resolve from .cyml recipes
  - CYML parser (`cyml_parse`, `cyml_parse_file`) — handles `[section]` headers (stdlib toml.cyr only handles `[[arrays]]`)
  - Recipe struct with 12 fields: name, version, description, license, arch, groups, release, url, sha256, patches, runtime_deps, build_deps
  - Recipe parser (`recipe_parse_file`) — reads .cyml files, extracts all sections
  - Recipe database (`recipe_db_load`, `recipe_db_get`, `recipe_db_count`) — scans zugot directory tree (11 categories), indexes by name
  - Build-order reader (`read_build_order`) — parses build-order.txt for tier-sorted install order
  - `resolver_with_recipes(r, rdb)` — attach recipe DB to resolver
  - `resolver_resolve_all_with_recipes(r, names)` — transitive resolution with recipe fallback
  - `recipe_to_resolved(recipe)` — convert Recipe to ResolvedPkg for graph integration
  - `parse_toml_array(s)` — parse TOML inline arrays `["a", "b"]` to vec
  - Tests against real zugot recipes (curl, ark), synthetic dep chains
  - `recipe_parse` benchmark (19us avg per recipe)
  - 218 tests total, 13 benchmarks

- **P1: Dependency Resolution** — full transitive dependency graph resolution
  - SemVer parsing (`semver_parse`, `semver_cmp`, `semver_to_str`) — supports `major.minor.patch`
  - Version constraint parsing (`constraint_parse`) — supports `>=`, `>`, `<=`, `<`, `=`, `^` (caret), `~` (tilde), `*` (any)
  - Constraint matching (`constraint_matches`) — evaluates whether a version satisfies a constraint
  - Dependency graph (`dep_graph_new`, `dep_graph_add`) — tracks packages, edges, and version constraints
  - Cycle detection (`dep_graph_detect_cycle`) — DFS with 3-color marking, returns cycle path
  - Topological sort (`dep_graph_topo_sort`) — Kahn's algorithm for install order
  - Conflict detection — checks constraint compatibility for diamond dependencies
  - `resolver_resolve_all(r, names)` — resolves a list of packages transitively, returns `ResPlan` with ordered package list
  - 12 new test groups (52 new assertions): semver_parse, semver_cmp, semver_to_str, constraint_parse, constraint_matches, dep_graph_basic, topo_sort, cycle_detection, diamond_deps, resolve_all, resolve_all_cycle
  - `graph_resolve` benchmark (7us avg for single-package transitive resolution)
  - 192 tests total, 12 benchmarks

- **Cyrius port** — full port from Rust to Cyrius 5.1.7
  - 2143 lines Rust → 2174 lines Cyrius (src/nous.cyr + src/main.cyr + tests + benchmarks + fuzz)
  - Manual struct layout: 12 struct types with alloc/store64 constructors, load64 accessors
  - JSON serialization/deserialization for all public types (resolved_to_json, installed_to_json, update_to_json, strategy_to_json, manifest_to_json, error_kind_to_json, agent_info_to_json, search_result_to_json + from_json variants)
  - Manifest dependency parsing (json_extract_deps parses "dependencies":{} objects)
  - registry_reload() — public cache refresh for marketplace registry
  - registry_install_package() — stub for mela interface parity
  - nous_error_to_kind() — serializable error companion
  - Complete error display for all 7 NousError variants (was 3, now 7)
  - source_from_json() — deserialize PackageSource from JSON
  - our_is_dir() — workaround for broken fs.cyr is_dir (direct stat syscall)
  - 140 tests across 40 test groups (up from 87/26)
  - 11 benchmarks including 4 strategy comparisons and serde roundtrip (up from 6)
  - 3 fuzz harnesses (validate_name, detect_source, json_extract)
  - 115KB standalone x86_64 ELF binary

### Changed

- **Breaking**: Language changed from Rust to Cyrius 5.1.7. All APIs renamed to snake_case function style (e.g., `NousResolver::resolve()` → `resolver_resolve(r, name)`).
- **Breaking**: Serde derives replaced with manual JSON serialization functions.
- **Breaking**: Struct field access via accessor functions (e.g., `rp_name(pkg)` instead of `pkg.name`).
- Rust source preserved in `rust-old/` for reference.

### Removed

- Rust dependencies (thiserror, serde, serde_json, tracing, chrono)
- Cargo.toml, Cargo.lock, rust-toolchain.toml, deny.toml, codecov.yml
- Criterion benchmarks (replaced with Cyrius bench harness)

### Previous (Rust era)

- P(-1) scaffold hardening pass (round 3)
  - `LocalRegistry` cached inside `NousResolver` at construction time — eliminates per-call filesystem scans. Benchmarks: resolve_marketplace_hit 687µs→163ns (~4200x), search_100 1.41ms→25µs (~56x), list_installed_100 1.40ms→11µs (~127x)
  - `NousResolver::new()` now returns `Result` — propagates registry errors instead of silently returning empty results
  - `Serialize`/`Deserialize` on `NousResolver` and `SystemPackageDb` — serializes configuration, reconstructs live state on deserialize
  - `Debug` derive on `NousResolver`, `SystemPackageDb`, `LocalRegistry`
  - `LocalRegistry::base_dir()` accessor, `LocalRegistry::reload()` for cache refresh
  - `LocalRegistry::search()` and `list_installed()` now return borrowed data (no cloning)
  - 3 additional tests (56 total): registry error propagation (1), serde roundtrips for NousResolver and SystemPackageDb (2)
- P(-1) scaffold hardening pass (round 2)
  - Package name validation (`InvalidPackageName` error) — rejects empty, whitespace, and special characters
  - Deterministic ordering for `list_installed()` and `search()` results (sorted by name)
  - Replaced `which` shell-out with native PATH scanning in `which_exists()`
  - `dir_size()` → `dir_size_recursive()` — now counts nested directory contents
  - `installed_at` now uses manifest file mtime instead of `Utc::now()`
  - Fixed `bench-history.sh` — `--save` now works with current criterion version
  - 7 additional tests (53 total): name validation (4), deterministic ordering (1), InvalidPackageName error display + serde (2)
  - README: fixed stale `anyhow` → `thiserror` in dependency table
  - Updated architecture/overview.md, roadmap.md, gaps.md to reflect current state
- P(-1) scaffold hardening pass (round 1)
  - `#[non_exhaustive]` on all public enums (`PackageSource`, `ResolutionStrategy`)
  - `#[must_use]` on all pure functions and constructors
  - `PartialEq` + `Eq` derives on all public data types (`ResolvedPackage`, `InstalledPackage`, `AvailableUpdate`, `UnifiedSearchResult`, registry stub types)
  - Criterion benchmark suite (`benches/resolver.rs`) covering detect_source, resolve, search, list_installed, strategy variants, and serde roundtrip
  - `scripts/bench-history.sh` for benchmark archival
  - 9 additional tests (40 total, up from 31): serde roundtrip for `PackageSource`, `ResolutionStrategy`, `UnifiedSearchResult`, `MarketplaceManifest`; community variant coverage; dedup verification; flutter runtime guard; multi-package listing
  - CHANGELOG.md, CONTRIBUTING.md, SECURITY.md, CODE_OF_CONDUCT.md
  - `docs/architecture/overview.md`
  - `docs/development/roadmap.md`
- Replaced `anyhow` with `thiserror` for typed error handling (P5 partial)
  - `NousError` enum with variants: `CommandExec`, `RegistryIo`, `InvalidManifest`, `InvalidVersionConstraint`, `DependencyCycle`, `VersionConflict`
  - `NousErrorKind` — serializable error snapshot for logging and wire transport
  - `error::Result<T>` type alias replaces `anyhow::Result<T>` in all public API
  - 6 error-specific tests (46 total): Display formatting, serde roundtrip on `NousErrorKind`, error-to-kind conversion

### Changed

- **Breaking**: `NousResolver::new()` now returns `Result<Self>` instead of `Self`. The marketplace directory must exist.
- **Breaking**: `LocalRegistry::search()` returns `Vec<&InstalledMarketplacePackage>` (borrowed) instead of `Vec<InstalledMarketplacePackage>` (owned).
- **Breaking**: `LocalRegistry::list_installed()` returns `&[InstalledMarketplacePackage]` (borrowed slice) instead of `Vec<InstalledMarketplacePackage>` (owned).
- **Breaking**: All public `Result` types are now `Result<T, NousError>` instead of `anyhow::Result<T>`. Consumers can now match on specific error variants.

### Removed

- `anyhow` dependency — no longer needed

### Fixed

- `cargo fmt` formatting violations in test assertions
- 3 clippy `collapsible_if` warnings (registry_stub.rs, lib.rs)
- Dead code warning on `LocalRegistry::install_package` stub

## [0.1.0] - 2026-04-04

### Added

- Initial release
- `NousResolver` — multi-source package resolver with configurable strategies
- `SystemPackageDb` — apt/dpkg wrapper for system package queries
- `LocalRegistry` — filesystem-based marketplace package registry (stub)
- Resolution strategies: `MarketplaceFirst`, `SystemFirst`, `OnlySource`, `SearchAll`
- Heuristic source detection from package name format
- Cross-source unified search with marketplace-priority dedup
- Installed package listing across all sources
- System update checking via `apt list --upgradable`
- Full serde support on all public types
- Structured logging via `tracing`
- 31 unit tests with serde roundtrip coverage

[Unreleased]: https://github.com/MacCracken/nous/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/MacCracken/nous/releases/tag/v0.1.0
