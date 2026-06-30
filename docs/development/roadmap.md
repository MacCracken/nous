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
- ~~Worked around a Cyrius 6.0.1 codegen bug~~ **[RETRACTED in 1.2.5 — there was
  no codegen bug; misdiagnosis.](issues/0001-cyrius-6.0.1-vec-get-recompute.md)**
  At the time, a typed `vec_get` nested/re-evaluated was *believed* to miscompile
  in cycle detection, topo sort, and `resolver_resolve_all`, and was de-nested to
  a local. The nested form actually compiles correctly on both 6.0.1 and 6.0.3;
  the real cause of the failure was [issue 0002](issues/0002-sysdb-exec-bare-name-path.md)
  (sysdb PATH). The de-nesting was reverted in 1.2.5.

## Completed (v1.2.1) — Hygiene + codegen-workaround sweep

- Extended the issue-0001 `vec_get` de-nest sweep (source, recipe, json, sort).
  **(The 0001 "codegen bug" was later [retracted in 1.2.5](issues/0001-cyrius-6.0.1-vec-get-recompute.md)
  as a misdiagnosis; the core de-nesting was reverted. These 1.2.1 sweep sites
  carried no false comments and were left as ordinary local-binding style.)**
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

## Completed (v1.2.4) — Docs modernization + P(-1) arc closeout

- De-Rust-ified `CLAUDE.md`; version-less `src/main.cyr` banner fallback.
- Started `docs/adr/` (0001 Result, 0002 modules+bundle, 0003 de-nest workaround);
  added `docs/guides/getting-started.md`; refreshed `architecture/overview.md`
  and `development/gaps.md`.
- P(-1) Scaffold Hardening closeout audit
  ([docs/audit/2026-05-26-1.2.x-closeout.md](../audit/2026-05-26-1.2.x-closeout.md)):
  cleanliness + 271/0 + stable benches; internal review (zero TODO/unwrap/panic;
  both filed toolchain issues worked around); `@unsafe` not adopted (deny/lint
  don't require it); resolver-completeness assessment. **Closes the 1.2.x arc.**

## Completed (v1.2.6) — Cyrius 6.2.11 + bayan adoption

- Toolchain pin 6.0.3 → 6.2.11 (clean: fmt/lint/vet/deny/doc, 271/0, 18 benches
  unchanged; pin-drift warning gone).
- Adapted to the 6.2.x stdlib slim-down, which folded `json`/`toml` (and
  `base64`/`bigint`/`csv`/`cyml`/`linalg`/`matrix`/`u128`) into the consolidated
  `bayan` bundle. Swapped `json` + `toml` → `bayan` in `[deps] stdlib`; nous now
  sources the six `toml_*` section/pair structs from bayan's `_compat` surface
  rather than hand-rolling them. nous keeps its own `src/json.cyr` (domain
  serializers) and its own CYML parser — renamed `cyml_parse` → `nous_cyml_parse`
  to dodge bayan's differently-shaped `cyml_parse(data, len)`.
- **Consumer-facing (breaking):** `dist/nous.cyr` now expects the consumer to
  supply `bayan` in `[deps] stdlib` (it references the `toml_*` structs without
  defining them). ark must add `"bayan"` and may drop `"json"`/`"toml"`.

## Completed (v1.2.5) — Cyrius 6.0.3 + issue-0001 retraction

- Toolchain pin 6.0.1 → 6.0.3 (clean: fmt/lint/vet/deny/doc, 271/0, 18 benches
  unchanged; pin-drift warning gone).
- **Retracted the phantom issue-0001 "codegen bug."** Proved there was never a
  Cyrius `vec_get` miscompile: the nested form passes 271/0 on cycc 6.0.1 *and*
  6.0.3, and the original reproducer was self-defective (omitted the `store64`
  the real `detect_cycle_dfs` does, so it returned 0 unconditionally — its
  bisection table is not reproducible). Reverted the unnecessary de-nesting in
  `graph`/`resolver`/`recipe`/`sysdb` to the original nested forms and removed
  every false "miscompile" comment. Issue 0001 closed as not-a-defect; ADR 0003
  superseded. The genuine project-side bug the narrative obscured —
  [issue 0002](issues/0002-sysdb-exec-bare-name-path.md) (sysdb bare-`argv[0]`
  PATH) — stands as fixed.

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

## 1.2.x Modernization Arc — COMPLETE (1.2.0 → 1.2.6)

All releases shipped (see the Completed sections above). nous is a clean,
fully-typed, bundle-shipping Cyrius 6.2.11 library with hardened CI/release,
current docs + ADRs, and a P(-1) closeout audit
([docs/audit/2026-05-26-1.2.x-closeout.md](../audit/2026-05-26-1.2.x-closeout.md)).
1.2.5 retracted the misdiagnosed issue-0001 "codegen bug" and reverted its
unnecessary workaround — there is no Cyrius defect; the real fix was
issue 0002 (sysdb PATH). 1.2.6 tracked the toolchain to 6.2.11 and absorbed the
6.2.x stdlib slim-down (vendored the six `toml_*` structs nous used; dropped the
unused `json`/`toml` stdlib deps).

Deferred from the arc (not arc-blocking, tracked here):
- **Per-function `#` doc comments** + promoting the CI `doc` gate from the barrel
  to per-module. The structural docs (ADRs, guide, overview) landed in 1.2.4; the
  ~206-function doc-comment pass is a larger, lower-priority follow-on.
- **ark consumer migration** to `modules = ["dist/nous.cyr"]` — see below.

## Consumer coordination (ark — separate repo, not a nous version)

- Migrate ark to consume `modules = ["dist/nous.cyr"]` (one bundle) in place of
  the 14 `src/*.cyr` it pins today; bump ark's nous `tag` to the latest 1.2.x.
  Executed in the ark repo; coordinated from here. Not gating the nous arc
  closeout.

## Sovereign-ark native resolver (ark v2 path — M2; coordinated from ark)

nous's share of the **ark v2 sovereignty path** (de-apt the install layer so
`agnova install` works on a no-apt box). Orchestration spine:
[`agnosticos/docs/development/planning/ark-v2-sovereignty-path.md`](https://github.com/MacCracken/agnosticos/blob/main/docs/development/planning/ark-v2-sovereignty-path.md).
**Host-side Linux work — NOT AGNOS-gated.** Gated only on ark's M0 `system_backend`
seam (so a native strategy is selectable). Today nous resolves marketplace → flutter
→ sys; `strategy_default()` is `MARKETPLACE_FIRST` (`STRAT_SYSTEM_FIRST` is injected
by *ark*, not nous), `src/types.cyr` has `SOURCE_SYSTEM=0`/`SOURCE_COMMUNITY=3` but
**no `SOURCE_NATIVE`**, and there is no native index / local store / lockfile.

- [ ] **`SOURCE_NATIVE`** source + a native resolver backend that reads a **signed
      native index** + a **local content-addressed artifact store** (resolve a name →
      the `.ark` by root-hash, no apt, no mela network).
- [ ] **Lockfile generation + consumption** (promoted from *Future* below — it's the
      reproducibility half of M2: deterministic dependency selection, not just bytes).
- [ ] Native-first strategy selector threaded through `src/sysdb.cyr` +
      `src/resolver.cyr` so the apt (`SOURCE_SYSTEM`) leg is **mode-gated behind ark's
      `system_backend`**, not implicit; apt-fallback behind a capability flag.
- [ ] Replace the **mela registry stub** (`registry.cyr` scans a local dir;
      `registry_install_package` returns `Ok(0)`) — resolve M2's native index vs the
      mela path (subsume on agnos, or coexist — open question in the spine doc).
- [ ] Constraint solving on **recipe-level deps** (today only marketplace-manifest deps
      feed the solver; recipe deps carry no version constraints).

Acceptance (with ark): apt absent from PATH → nous resolves a name to `SOURCE_NATIVE`
from the signed index and ark installs from the local store with a committed lock.

## Future

- Parallel resolution across sources
- Plugin system for additional package sources
- Resolver constraint language specification
