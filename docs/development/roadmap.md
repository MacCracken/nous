# Roadmap

## Completed

### v0.1.0

- Single-package resolution from marketplace, system, and Flutter sources
- Configurable resolution strategies (MarketplaceFirst, SystemFirst, OnlySource, SearchAll)
- Cross-source unified search with marketplace-priority dedup
- System package database via apt/dpkg
- Marketplace registry via local filesystem (stub)
- Heuristic source detection from package name format
- Full serde support on all public types
- Structured logging via tracing
- P(-1) scaffold hardening: `#[non_exhaustive]`, `#[must_use]`, `PartialEq`/`Eq`, benchmark suite, documentation

## Backlog (prioritized)

### P1 — Dependency Resolution (Critical)

- Transitive dependency resolution
- Topological sort for install order
- Cycle detection for circular dependencies
- Version constraint matching (>=, ^, ~, =)
- Conflict detection (incompatible versions)
- Diamond dependency handling (deduplicated)

### P2 — Zugot Integration (High)

- Recipe parsing from `.toml` files
- Build-order awareness from `build-order.txt`
- Source URL resolution for `github_release` shorthand
- SHA256 verification

### P3 — Caching & Performance (Medium)

- Resolution cache (skip re-resolution if unchanged)
- Index caching for marketplace/system packages
- Incremental resolution

### P4 — Mela Integration (Medium)

- Replace registry_stub with real mela marketplace API
- Package metadata sync
- Trust integration with sigil

### P5 — Error Quality (Medium-Low) — partially complete

- ~~Replace anyhow with dedicated error types (thiserror)~~ — done (v0.1.0 unreleased)
- ~~Package name validation errors~~ — done (v0.1.0 unreleased)
- Conflict explanation (which constraints conflict)
- Suggestion engine for typos
- Resolution trace with --verbose mode

## Future

- Lockfile generation and consumption
- Parallel resolution across sources
- Plugin system for additional package sources
- Resolver constraint language specification

## v1.0 Criteria

- [ ] P1 complete — full dependency graph resolution with cycle detection
- [ ] P2 complete — zugot recipe awareness
- [x] P5 partial — dedicated error types via thiserror, anyhow removed
- [ ] P5 complete — conflict explanations, suggestion engine, resolution trace
- [ ] All public API documented with examples
- [ ] Integration tests against real apt on CI
- [ ] Benchmark regressions gated in CI
- [ ] MSRV tested in CI
