# Roadmap

What's next for nous. Shipped work is recorded in [CHANGELOG.md](../../CHANGELOG.md),
not here.

**Current state (1.7.1, cyrius 6.7.18).**
- **Resolution:** transitive resolution across system, marketplace, Flutter and
  community sources, with 4 strategies.
- **Determinism:** byte-ordered everywhere; results never depend on the hash seed.
- **Sources:** apt dependencies resolve from apt, and SearchAll errors on a
  cross-source name collision.
- **Graph:** iterative Tarjan for cycles (apt-only cycles accepted, ADR 0005) and
  for install order.
- **Versions:** constraints intersected as ranges, plus a check on the version
  chosen. Debian ordering for apt and community. SemVer 2.0.0 for the rest,
  with pre-releases (npm / Cargo matching rule) and pub build ordering for
  Flutter updates (1.7.0, ADR 0006). Compound ranges, npm and Cargo forms
  (1.7.1, ADR 0007).
- **Inputs:** the zugot recipe DB (all categories, duplicates reported), and
  manifests and JSON via bayan's parser.
- **Plumbing:** complete child-output capture with a default idle deadline, and
  apt queries batched per breadth-first wave.
- **Docs:** a one-line doc comment on every public fn (232), checked per module
  in CI; every Result-returning fn is declared `: Result`.
- **Size:** 15 modules, 651 assertions, 24 benchmarks, 5 fuzz harnesses
  (`--poison=ab`), and a ~45 KB release smoke binary.

---

## 1.7.x — Reproducible resolution

One bite per release (CLAUDE.md task sizing). They are ordered by what each
one builds on.

- [ ] **1.7.2 — Lockfile generation + consumption.** Deterministic
      dependency selection, the reproducibility half of the ark v2 M2 path
      below. It is not gated on ark. Its format gets an ADR.
- [ ] **1.7.3 — Lockfile integrity.** Every node, transitive ones included,
      records name, source, exact version and artifact hash. Locked mode
      refuses any unlocked node (pip / Cargo practice).
- [ ] **Constraint solving on recipe-level deps** (gated on zugot). zugot's
      `[depends]` lists carry names only (0 constrained entries in the tree), so
      nous has nothing to solve until zugot gains a versioned syntax. That is a
      zugot change first.

## 1.8.x — Sovereign native resolver (ark v2 M2)

nous's share of the **ark v2 sovereignty path**: take apt out of the install
layer, so `agnova install` works on a box with no apt. The orchestration spine
is [`agnosticos/docs/development/planning/ark-v2-sovereignty-path.md`](https://github.com/MacCracken/agnosticos/blob/main/docs/development/planning/ark-v2-sovereignty-path.md).

This is host-side Linux work, **not AGNOS-gated**. It is gated only on ark's M0
`system_backend` seam, which makes a native strategy selectable.

Today:
- nous resolves marketplace → flutter → sys.
- `strategy_default()` is `MARKETPLACE_FIRST`; ark injects `STRAT_SYSTEM_FIRST`.
- There is **no `SOURCE_NATIVE`**, no native index and no local store.

- [ ] **`SOURCE_NATIVE`**: a source plus a resolver backend. It reads a **signed
      native index** and a **local content-addressed artifact store**, and
      resolves a name to its `.ark` by root hash, with no apt and no mela network.
- [ ] **Native-first strategy selector**, threaded through `src/sysdb.cyr` and
      `src/resolver.cyr`. The apt (`SOURCE_SYSTEM`) leg becomes **mode-gated behind
      ark's `system_backend`** instead of implicit, with apt fallback behind a
      capability flag.

Acceptance (with ark): with apt absent from PATH, nous resolves a name to
`SOURCE_NATIVE` from the signed index, and ark installs it from the local store
with a committed lock (1.7.x).

- [ ] **Rollback and freeze protection.** Track the highest version seen per
      (name, source), and use signed, expiring index metadata (TUF §1.5.2).

## 1.9.x — Marketplace (mela) integration

- [ ] **Replace the registry stub.** Today `registry.cyr` scans a local directory
      and `registry_install_package` returns `Ok(0)`. The replacement is either
      mela's API or the native index. Subsume on agnos or coexist: that is an open
      question in the spine doc.
- [ ] **Package metadata sync** from mela.
- [ ] **Trust.** Verify package signatures through sigil during resolution.

## Version selection (gated on a multi-version source)

- [ ] **Version selection with backtracking** (PubGrub-style; Dart pub, uv,
      Cargo). Prefer locked versions, allow several major versions side by
      side, and explain failures as causal chains. Today each name has one
      candidate, checked rather than chosen, so a set solvable by an older
      version is rejected (1.5.3 audit, research #18).
  - **Moved out of 1.7.x (1.7.0):** no source offers more than one candidate
    per name today. The registry holds installed versions, apt is queried with
    `--no-all-versions`, and a recipe has one version. Selection needs the
    native index (1.8.x) or mela's metadata (1.9.x) first.

## 1.10.x — Caching & performance

- [ ] **Persistent resolution cache:** skip re-resolution when the inputs are
      unchanged.
- [ ] **Index caching** for marketplace and system packages.
- [ ] **Incremental resolution:** re-resolve only the affected subgraph.
- [ ] **Parallel resolution** across sources.
- [ ] **The system leg through apt's own solver** (`apt-get -s install <pkg>`
      `Inst` lines) instead of re-deriving `Depends`. That would handle
      alternatives and virtual packages, which are listed in
      `nous_pl_unresolved` today.
- [ ] **Faster capture copy-out.** The stdlib `memcpy` is byte-wise (~2 ns/byte);
      suggest a word-wise version upstream (decision D11: accepted for now).

## 2.0.0 backlog (breaking)

- [ ] Remove the nine deprecated bare names: `which_exists`, `cmd_run` (1.5.2);
      `to_cstr`, `join_path`, `registry_get`, `registry_list`, `search_result_new`,
      `recipe_parse_file` (1.5.3).
- [ ] Decide on prefixing the rest of the public API (`nous_*`), as mela 0.9.3
      did. The 1.5.3 audit found no other live or latent collision.
- [ ] Portability: the `struct stat` offsets in `src/util.cyr` are x86_64-only.
- [ ] A plugin system for additional package sources (likely reshapes the
      resolver API).
- [ ] A resolver constraint language specification, if it changes the
      constraint API.

## Ecosystem coordination (other repos, not nous versions)

- **ark → nous 1.5.1+:** rename `ERR_INVALID_PACKAGE_NAME` to
  `NOUS_ERR_INVALID_PACKAGE_NAME` at `ark/src/engine.cyr:41` and `:514`. No bayan
  change is needed: ark already lists stdlib `bayan`, which defines both the
  `bayan_toml_*` and `bayan_json_*` functions.
- **ark → nous 1.5.3:** behaviour changes to absorb (CHANGELOG 1.5.3):
  - A missing dependency or root is now `NOUS_ERR_NOT_FOUND`, no longer
    silently dropped.
  - Unparseable constraints are `NOUS_ERR_INVALID_VERSION_CONSTRAINT`.
  - SearchAll can return `NOUS_ERR_AMBIGUOUS_SOURCE`.
  - Marketplace packages report `trusted = 0`.
  - `pl_order` is now install order. ark's "deps-first" print at
    `src/engine.cyr:524` becomes correct.
  - Seven more names are deprecated, but none is called by ark.
- **ark → nous 1.6.0:** nothing to absorb. The compiled code is byte-identical
  to 1.5.3; only comments and return-type annotations changed.
- **ark → nous 1.6.1:** nothing breaks. `ark search` and `ark list` now show a
  Flutter app as flutter-app rather than marketplace.
- **ark → nous 1.7.0:** a marketplace or Flutter dependency whose only
  candidate is a pre-release now fails a constraint that does not name a
  pre-release of that version (npm / Cargo rule; ADR 0006). ark shows nous's
  message, which says how to opt in. Updates from an rc to its release are now
  offered.
- **ark → nous 1.7.1:** nothing breaks. Manifests with compound constraints
  (`^1 || ^2`, `>=1.2, <1.5`, `1.2 - 2.0`) now resolve instead of failing with
  `NOUS_ERR_INVALID_VERSION_CONSTRAINT`.
- **zugot:** the nous recipe (`marketplace/nous.cyml`) is still at 1.2.6. Bump it
  after each release tag, since it needs the release asset and its checksum.
