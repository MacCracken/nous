# Roadmap

## Completed (v1.1.1)

- Single-package and transitive dependency resolution across system, marketplace, Flutter, and community sources
- 4 resolution strategies (MarketplaceFirst, SystemFirst, OnlySource, SearchAll)
- Dependency graph with topological sort (Kahn's), cycle detection (DFS 3-color), conflict detection
- Version constraint matching: 8 operators (>=, >, <=, <, =, ^, ~, *)
- Zugot recipe integration: CYML parser, recipe DB (428 recipes, 11 categories), build-order reader
- Cross-source unified search with marketplace-priority dedup
- System package database via apt/dpkg (array-based exec, no shell injection)
- Marketplace registry with reload and install stub
- Heuristic source detection, typo suggestions (Levenshtein), resolution trace
- JSON serialization/deserialization for all public types
- 7 error variants with full display, conflict explanations
- Security audit: 5 internal fixes (P0-P2), 8 external attack categories documented
- 14-module split matching ark's architecture pattern
- Dead code elimination (CYRIUS_DCE=1) in CI/release
- 271 tests, 18 benchmarks, 3 fuzz harnesses, full API docs
- Ported from Rust to Cyrius 5.1.7; tracked toolchain forward to 5.7.29
- CI/release modernized to the argonaut/daimon pattern (manifest-driven
  toolchain pin, `git archive` source tarball, SHA256SUMS, changelog
  extraction)

## Backlog

### P3 — Caching & Performance

- Persistent resolution cache (skip re-resolution if unchanged)
- Index caching for marketplace/system packages
- Incremental resolution (only re-resolve affected subgraph)

### P4 — Mela Integration

- Replace registry stub with real mela marketplace API
- Package metadata sync
- Trust integration with sigil (package signing/verification)

## Housekeeping

- Per-module `cyrius lint` reports cosmetic warnings on `src/error.cyr`,
  `src/recipe.cyr`, `src/resolver.cyr`, `src/source.cyr` (multiple
  consecutive blank lines). The barrel `src/nous.cyr` lints clean,
  which is what CI checks. Sweep these so per-file lint can be
  promoted in CI.
- `cyrius fmt --check` reports drift on `src/recipe.cyr`,
  `src/registry.cyr`, `src/resolver.cyr`, `tests/nous.tcyr`,
  `tests/nous.bcyr`. Run a one-shot `cyrius fmt` over the tree, then
  promote the CI step from advisory to fail-on-drift.
- `CLAUDE.md` still describes the project in Rust-crate terms
  (cargo fmt/clippy/audit/deny; "flat library crate"). Sweep when
  the next P(-1) audit comes around.

## Future

- Lockfile generation and consumption
- Parallel resolution across sources
- Plugin system for additional package sources
- Resolver constraint language specification
