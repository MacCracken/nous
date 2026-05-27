# Roadmap

## Completed (v1.2.0) — Cyrius 6.0.1 modernization

Opens the 1.2.x arc: align nous with the Cyrius 6.0.1 library conventions
already adopted by sibling AGNOS libraries (patra 1.9.5, sigil 3.4.3).

- Cyrius toolchain pin 5.7.29 → 6.0.1; clean smoke build (pin-drift +
  stdlib-shadow warnings eliminated)
- `lib/` de-vendored (24 committed stdlib files removed); stdlib now
  resolved from the version-pinned `~/.cyrius/versions/6.0.1/lib/` snapshot
- `[lib] modules = [...]` added; `cyrius distlib` produces the
  consumer-facing `dist/nous.cyr` bundle (the 14 src modules, ~82 KB)
- `.gitignore` de-Rust-ified (Cyrius shape: ignore `/build/` + `/lib/`,
  track `dist/`)
- CI/release hardened to the patra pattern: release-asset HTTP pre-flight
  + gzip verify, versioned toolchain layout, stdlib-via-symlink, a
  distlib-sync gate, and `nous-<tag>.cyr` shipped as a release artifact
- Worked around a Cyrius 6.0.1 codegen bug (typed `vec_get` nested as a call
  argument / re-evaluated returns a wrong value) that the stdlib bump exposed in
  cycle detection, topo sort, and `resolver_resolve_all`. Fixed by binding the
  accessor to a local; suite back to 271/0. Reported upstream with a
  self-contained reproducer +
  [issue 0001](issues/0001-cyrius-6.0.1-vec-get-recompute.md). Compiler not
  touched — treated as an external dependency.

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

## 1.2.x Modernization Arc

Continued alignment to Cyrius 6.0.1 conventions, sequenced after 1.2.0.
Each is an independent, releasable bite.

### 1.2.1 — Hygiene + codegen-workaround sweep

- **Harden the remaining at-risk `vec_get` nesting sites** flagged in
  [issue 0001](issues/0001-cyrius-6.0.1-vec-get-recompute.md): `src/sysdb.cyr`
  (~lines 33, 83, 85, 97, 100, 101, 172), `src/source.cyr` (~133),
  `src/recipe.cyr` (~284, 513). They pass the 271-test suite today (incl.
  `integration_apt`), so they're a defensive cleanup, not a known break.
  Revert all workarounds (and this item) if a Cyrius release fixes the bug —
  re-run the issue's reproducer to confirm.
- Run a one-shot `cyrius fmt` over `src/` + `tests/`. `cyrius fmt --check`
  reports drift on `src/recipe.cyr`, `src/registry.cyr`,
  `src/resolver.cyr`, `tests/nous.tcyr`, `tests/nous.bcyr`.
- Clear the per-module `cyrius lint` cosmetic warnings on `src/error.cyr`,
  `src/recipe.cyr`, `src/resolver.cyr`, `src/source.cyr` (multiple
  consecutive blank lines). The barrel `src/nous.cyr` already lints clean.
- Promote the CI fmt step from advisory to **fail-on-drift**, and switch
  lint from barrel-only to per-module fail-on-warn.

### 1.2.2 — Policy gates

- Add a `cyrius deny src/main.cyr` project policy and wire it into CI.
- Add `cyrius doc --check src/nous.cyr` (doc-warnings-as-errors) to CI.
- Completes the CLAUDE.md cleanliness-check order (fmt → lint → vet →
  deny → doc).

### 1.2.3 — Consumer migration (ark)

- Migrate ark to consume `modules = ["dist/nous.cyr"]` (one bundle) in
  place of the 14 individually-listed `src/*.cyr` it pins today; bump
  ark's nous `tag` to the 1.2.x release. Consumer-side change, executed
  in the ark repo and coordinated from here.

### 1.2.4 — Docs & instructions

- De-Rust-ify `CLAUDE.md` — it still describes the project in Rust-crate
  terms ("flat library crate"; cargo fmt/clippy/audit/deny lineage).
- Refresh `docs/architecture/overview.md` for the `dist/nous.cyr`
  bundle-consumption model.
- Template the `src/main.cyr` smoke version string so the hardcoded
  no-VERSION-file fallback banner (`"nous 1.2.0 …"`) tracks `VERSION`
  automatically (standing item; revisit when cyrius gains compile-time
  string-from-file templating).

## Future

- Lockfile generation and consumption
- Parallel resolution across sources
- Plugin system for additional package sources
- Resolver constraint language specification
