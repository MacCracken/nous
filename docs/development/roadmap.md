# Roadmap

What's next for nous. Shipped work is recorded in [CHANGELOG.md](../../CHANGELOG.md),
not here.

**Current state (1.5.2, cyrius 6.7.18).**
- **Resolution:** transitive resolution across system, marketplace, Flutter and
  community sources, with 4 strategies.
- **Graph:** Kahn topological sort, DFS cycle detection, and conflict reporting.
- **Inputs:** 8 constraint operators and the zugot recipe DB.
- **Plumbing:** JSON for every public type, and complete child-output capture.
- **Size:** 15 modules, 314 assertions, 19 benchmarks, and a ~37 KB release smoke binary.

---

## 1.5.2 — Quick fixes: done, ready to release

Every item shipped; [CHANGELOG 1.5.2](../../CHANGELOG.md) has the details.

## 1.5.3 — P(-1) sweep: audit, refactor, hardening, optimization, security

Starts after 1.5.2 ships. It is the full CLAUDE.md P(-1) process (steps 0–10):
test and benchmark baseline, cleanliness, internal deep review, external
research, fixes, post-review benchmarks, and a docs audit. Repairs land in this
release.

Known starting points:

- **Namespace audit** of every public name (fns, globals, enum members) against
  the pinned stdlib and every sibling repo, plus a `nous_*` prefix plan for 2.0.0.
  1.5.1 / 1.5.2 handled the nine names found so far (seven `ERR_*`, two fns).
- **Capture copy cost.** The owned copy runs through the stdlib's byte-wise
  `memcpy`: about 2 ns a byte, or +~120 µs per 60 KB. The choice is a word-wise
  copy in nous (which hand-rolls a stdlib primitive) or an upstream fix. **Needs
  a decision.**
- **Return annotations.** Result-returning public fns are declared `: i64`.
  `cyrius header` refuses that shape since 6.7.12, though the compiled bytes are
  the same. Assess moving them to `: Result`, the way 1.2.3 swept the return types.
- **Input hardening.** Review the CYML recipe parser, manifest JSON, apt/dpkg
  output parsing, `which_path` PATH resolution, and the registry directory scan.
  Each one gets a fuzz target (`tests/nous.fcyr` covers a subset today).
- **Security re-review** against the 1.1.1 attack categories, on current code.
  `validate_package_name` must run before every argv. `/bin/sh -c` may only take
  constant command strings. Check `sysdb` parsing of untrusted tool output.
- **Retire `our_is_dir`.** nous keeps a hand-rolled stat-based copy from when
  the stdlib `is_dir` was broken. The stdlib one works on 6.7.18 (checked in
  1.5.2), and the copy is where the 1.3.1 stack smash happened.
- **Idiom check** against the cyrius 6.7.x library conventions (typed `Str` /
  `bool` returns, `const`), applied only where the code calls for it.

## 1.6.0 — Documentation completeness

- [ ] A `#` doc comment on every public fn (~310).
- [ ] Then promote CI's `cyrius doc --check` from the barrel, which has no fns and
      so checks nothing today, to every module.

## 1.7.x — Reproducible resolution

- [ ] **Lockfile generation + consumption.** Deterministic dependency selection,
      the reproducibility half of the ark v2 M2 path below. It is not gated on ark.
- [ ] **Constraint solving on recipe-level deps.** Today only marketplace-manifest
      deps feed the solver, because recipe deps carry no version constraints.
- [ ] **`github_release` URL resolution.** `recipe_parse_file` builds a
      placeholder (`https://github.com/<owner>/<repo>/releases`), not an asset URL.

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

## 2.0.0 backlog (breaking)

- [ ] Remove the deprecated bare names: `which_exists`, `cmd_run`, and whatever
      the 1.5.3 namespace audit adds.
- [ ] A plugin system for additional package sources (likely reshapes the
      resolver API).
- [ ] A resolver constraint language specification, if it changes the
      constraint API.

## Ecosystem coordination (other repos, not nous versions)

- **ark → nous 1.5.1+:** rename `ERR_INVALID_PACKAGE_NAME` to
  `NOUS_ERR_INVALID_PACKAGE_NAME` at `ark/src/engine.cyr:41` and `:514`. No bayan
  change is needed: ark already lists stdlib `bayan`, which defines the
  `bayan_toml_*` functions.
- **zugot:** the nous recipe (`marketplace/nous.cyml`) is still at 1.2.6. Bump it
  after each release tag, since it needs the release asset and its checksum.
