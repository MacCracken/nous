# Roadmap

What's next for nous. Shipped work is recorded in [CHANGELOG.md](../../CHANGELOG.md),
not here.

**Current state (1.6.0, cyrius 6.7.18).**
- **Resolution:** transitive resolution across system, marketplace, Flutter and
  community sources, with 4 strategies.
- **Determinism:** byte-ordered everywhere; results never depend on the hash seed.
- **Sources:** apt dependencies resolve from apt, and SearchAll errors on a
  cross-source name collision.
- **Graph:** iterative Tarjan for cycles (apt-only cycles accepted, ADR 0005) and
  for install order.
- **Versions:** constraints intersected as ranges, plus a check on the version
  chosen. Debian ordering for apt and community, SemVer for the rest.
- **Inputs:** the zugot recipe DB (all categories, duplicates reported), and
  manifests and JSON via bayan's parser.
- **Plumbing:** complete child-output capture with a default idle deadline, and
  apt queries batched per breadth-first wave.
- **Docs:** a one-line doc comment on every public fn (232), checked per module
  in CI; every Result-returning fn is declared `: Result`.
- **Size:** 15 modules, 493 assertions, 23 benchmarks, 5 fuzz harnesses
  (`--poison=ab`), and a ~41 KB release smoke binary.

---

## 1.6.x — Follow-ups

- [ ] **Flutter label in search and list.** `resolve_mkt` labels a
      Flutter-runtime registry package `SOURCE_FLUTTER_APP` (1.5.3), but
      `resolver_search` and `resolver_list` still label it
      `SOURCE_MARKETPLACE`, so the same package reports two sources depending
      on the call. Found while documenting 1.6.0.

## 1.7.x — Reproducible resolution

- [ ] **Lockfile generation + consumption.** Deterministic dependency selection,
      the reproducibility half of the ark v2 M2 path below. It is not gated on ark.
- [ ] **Constraint solving on recipe-level deps.** Today only marketplace-manifest
      deps feed the solver, because recipe deps carry no version constraints.
- [ ] **`github_release` URL resolution.** `recipe_parse_file` builds a
      placeholder (`https://github.com/<owner>/<repo>/releases`), not an asset URL.
- [ ] **Version selection with backtracking** (PubGrub-style; Dart pub, uv,
      Cargo). Prefer locked versions, allow several major versions side by side,
      and explain failures as causal chains. Today each name has one candidate,
      checked rather than chosen, so a set solvable by an older version is
      rejected (1.5.3 audit, research #18).
- [ ] **SemVer pre-release and build ordering** (§9–§11), plus pub_semver build
      ordering for Flutter (`1.0.0+2`). Today `1.0.0-rc.1` compares as `1.0.0`,
      and a pre-release tag in a constraint is an error.
- [ ] **Compound ranges**: npm AND by whitespace, Cargo AND by comma, `||`,
      hyphen ranges. They are refused with an error today.
- [ ] **Lockfile integrity.** Every node, transitive ones included, records
      name, source, exact version and artifact hash; locked mode refuses any
      unlocked node (pip / Cargo practice).

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
- **zugot:** the nous recipe (`marketplace/nous.cyml`) is still at 1.2.6. Bump it
  after each release tag, since it needs the release asset and its checksum.
