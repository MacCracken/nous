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

## Completed (v1.2.1) — Hygiene + codegen-workaround sweep

- Completed the issue-0001 `vec_get`-nesting de-nest sweep (source, recipe,
  json, sort — incl. the sort inner-loop double `vec_get(v, j)` re-eval). No
  `outer(vec_get(…))` sites remain in `src/`.
- `cyrius fmt` applied across `src/` + `tests/` (whole tree canonical).
- Cleared per-module lint warnings (consecutive blank lines in
  error/recipe/resolver/source); every module lints clean.
- CI: fmt check promoted advisory → **fail-on-drift** (and fixed — `--check` is
  exit-code based, not stdout-diff); lint promoted barrel-only → **per-module
  fail-on-warn**.

## Completed (v1.2.2) — Policy gates

- CI now runs `cyrius deny src/main.cyr` (0 violations) and
  `cyrius doc --check src/nous.cyr` (barrel; passes), completing the CLAUDE.md
  cleanliness order: fmt → lint → vet → deny → doc.
- Re-reviewed the arc against vidya + cyrius; added the 1.2.3 return-type sweep.

## Completed (v1.2.3) — Return-type annotation sweep

- Annotated all 206 `src/*.cyr` functions `: i64` (was 0% typed), matching the
  cyrius stdlib (100% typed) and patra/sigil (uniform `: i64`). Behaviour-
  preserving idiom alignment — 271/0 unchanged; fmt/lint/vet/deny/doc clean;
  codegen verified. Independent of the issue-0001 de-nesting.

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
Each is an independent, releasable bite. (1.2.0–1.2.3 shipped — see Completed.)
Scope re-confirmed 2026-05-26 against sibling **vidya** and the **cyrius**
stdlib/docs. The final nous-side arc item remains:

### 1.2.4 — Docs modernization + P(-1) arc closeout

Docs / instructions:
- De-Rust-ify `CLAUDE.md` — "flat library crate", cargo fmt/clippy/audit/deny
  lineage, and the `#[must_use]` / `#[non_exhaustive]` Rust attributes (map to
  cyrius `#must_use` and exhaustive `match`).
- Start `docs/adr/` (vidya has one; nous's own Documentation Standards require
  ADRs) — record the `Result<T, E>` adoption, the 14-module flat layout, and the
  issue-0001 de-nest workaround.
- Add `docs/guides/getting-started.md` (vidya model).
- Refresh `docs/architecture/overview.md` for the `dist/nous.cyr` bundle model.
- Add `#`-block doc comments to public functions; once documented, promote the
  CI doc gate from the barrel to per-module (`cyrius doc --check` exits =
  undocumented count). Optional doctest examples via `# >>>` / `# ===`.
- Template the `src/main.cyr` version banner so the hardcoded no-VERSION-file
  fallback tracks `VERSION` (revisit if cyrius gains compile-time string-from-
  file templating).

P(-1) Scaffold Hardening closeout (per CLAUDE.md P(-1)) — closes the 1.2.x arc:
- Full test + benchmark sweep; baseline vs post-review deltas (never skip benches).
- Cleanliness pass (fmt/lint/vet/deny/doc) + internal deep review (gaps, perf,
  memory, security, errors/logging) + external research (resolver completeness /
  world-class accuracy).
- `@unsafe` audit — the cyrius guide wraps unchecked `load*`/`store*` in
  `@unsafe { }`; deny/lint don't currently require it for nous, so this is a
  judgment call to make during the review.
- Documentation audit; mark the arc complete.

## Consumer coordination (ark — separate repo, not a nous version)

- Migrate ark to consume `modules = ["dist/nous.cyr"]` (one bundle) in place of
  the 14 `src/*.cyr` it pins today; bump ark's nous `tag` to the latest 1.2.x.
  Executed in the ark repo; coordinated from here. Not gating the nous arc
  closeout.

## Future

- Lockfile generation and consumption
- Parallel resolution across sources
- Plugin system for additional package sources
- Resolver constraint language specification
