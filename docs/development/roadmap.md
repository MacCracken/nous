# Roadmap

What's next for nous. Shipped work is recorded in [CHANGELOG.md](../../CHANGELOG.md),
not here.

**Current state (1.7.3, cyrius 6.7.18).**
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
- **Lockfile:** CYML in the Cargo.lock shape; locked resolution refuses any
  drift (1.7.2, ADR 0008). It also pins artifact checksums where the source
  publishes one, apt's `SHA256` and a recipe's `sha256` (1.7.3, ADR 0009).
- **Inputs:** the zugot recipe DB (all categories, duplicates reported), and
  manifests and JSON via bayan's parser.
- **Plumbing:** complete child-output capture with a default idle deadline, and
  apt queries batched per breadth-first wave.
- **Docs:** a one-line doc comment on every public fn (268), checked per module
  in CI; every Result-returning fn is declared `: Result`.
- **Size:** 16 modules, 786 assertions, 28 benchmarks, 6 fuzz harnesses
  (`--poison=ab`), and a ~45 KB release smoke binary.

---

## 1.7.x — Reproducible resolution

One bite per release (CLAUDE.md task sizing). They are ordered by what each
one builds on.

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
- [ ] **Checksums for marketplace and Flutter nodes** (ADR 0009). Since 1.7.3
      they lock without one: nous's `installed/<pkg>/manifest.json` has no hash,
      and mela's `package_hash` lives in its own `index.json`, under another
      layout. The native index's entries carry a `.ark` root hash, which is the
      checksum to lock (`sha256:<hex>`). That also settles which view of the
      marketplace nous reads (ark v2 open question 4).
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

- [ ] Remove the deprecated bare names: `which_exists`, `cmd_run` (1.5.2);
      `to_cstr`, `join_path`, `registry_get`, `registry_list`, `search_result_new`,
      `recipe_parse_file` (1.5.3); `manifest_from_json`, `update_from_json`
      (1.7.4, and with them the `NOUS_NO_BARE_JSON` switch).
- [ ] Decide on prefixing the rest of the public API (`nous_*`), as mela 0.9.3
      did. The 1.5.3 audit reported no other collision, but it missed the two
      JSON decoders that 1.5.3 itself added, which collide with mela (fixed in
      1.7.4). A sweep has to check the new names a release adds, as well as the
      old ones.
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
- **ark → nous 1.7.2:** the lockfile is ready to wire in, and nothing changes
  until ark does. ark owns the file: its path, an atomic write, and the commit.
  It calls `nous_lock_from_plan` + `nous_lock_to_str` to write a lock, and
  `nous_lock_parse` + `nous_resolve_locked` for a locked install. Two new
  error kinds, 9 and 10, need handling if ark matches kinds exhaustively.
- **ark → nous 1.7.4:** define `NOUS_NO_BARE_JSON` in both `[build] defines`
  and `[test] defines`. ark compiles mela beside nous, and without the define
  cyrius 6.7.18 refuses the build: `manifest_from_json` / `update_from_json`
  are defined by both with different parameter types. ark calls neither nous
  name. Done in ark 1.5.1. 1.7.4 also compiles for x86_64-agnos again (the
  idle deadline and the no-follow size walk, both 1.5.3), which ark's agnos
  cross-build needs.
- **ark → nous 1.7.3:** nothing breaks. 1.7.2 locks keep passing.
  - **Locks gain checksums.** A regenerated lock has a `checksum` line for each
    apt package and each recipe with a source hash, and locked mode then
    refuses other bytes under the same version.
  - **A new diff kind.** `NOUS_LOCK_CHECKSUM_CHANGED` (7) needs a case if ark
    matches diff kinds exhaustively. The error kind is still
    `NOUS_ERR_LOCK_MISMATCH`.
  - **On Debian, the host needs apt's package lists** (`apt-get update`).
    Without them, apt shows no `SHA256`, and a lock made with them reads as
    drift.
- **zugot:** 43 of 563 recipes have an empty `sha256`, so they lock no
  checksum (1.7.3). Filling one in pins that recipe's bytes from the next
  regenerated lock on.
- **zugot:** the nous recipe (`marketplace/nous.cyml`) is still at 1.2.6. Bump it
  after each release tag, since it needs the release asset and its checksum.
