# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

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
