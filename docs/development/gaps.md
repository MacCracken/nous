# Nous — Known Gaps & Hardening Targets

> **Status**: Active | **Last Updated**: 2026-11-12 (1.7.3, cyrius 6.7.18)
>
> This page tracks the gaps between what nous does and what a production
> resolver needs. The prioritized, versioned plan lives in
> [`roadmap.md`](roadmap.md); this page says *why* each gap matters.

---

## Current State

nous has 16 `src/` modules behind a barrel (`src/nous.cyr`) and ships a
single-file `dist/nous.cyr` bundle (`cyrius distlib`). It depends on bayan's
`dist/bayan-toml.cyr` and `dist/bayan-json.cyr` profiles
([ADR 0004](../adr/0004-bayan-toml-profile-dependency.md)). Struct layouts are
manual. Every function carries an explicit return type (`: Result` or `: i64`),
and every public function has a doc comment that CI checks (1.6.0).

**Works today:**
- Transitive resolution across 4 sources (System, Marketplace, FlutterApp,
  Community), with 4 strategies.
- Version constraints: 8 operators plus compound ranges (1.7.1), intersected
  as range sets. Tarjan SCC for cycles and install order (apt-only cycles
  allowed), and conflict reporting.
- Zugot recipe parsing, a recipe DB, `build-order.txt`, and recipe fallback in
  resolution.
- Cross-source search with marketplace-priority dedup, and installed-package
  listing in deterministic order.
- Update detection: the apt leg plus a pluggable lookup for the other sources.
- Complete child-output capture. Errors surface as `Result` and are never a
  partial result.
- JSON for every public type, both ways. Error display for all 9 kinds, typo
  suggestions, and a resolution trace.

**Closed since this page began (1.x):** all of the old P1 (graph resolution), P2
recipe parsing and build order, and P5 error quality (conflict explanation,
suggestions, trace).

**Closed by the 1.5.3 P(-1) sweep** ([audit](../audit/2026-11-12-1.5.3-p-1.md)):
- Determinism: order no longer follows the hash seed.
- Sound conflicts: every constraint counts, and the chosen version is checked.
- Missing dependencies are errors.
- Debian versions and Debian cycles are handled.
- Source confusion is closed: apt deps resolve from apt.
- apt option injection, silent truncation and the symlink DoS are fixed.
- JSON is escaped and every type roundtrips.
- Manifests are parsed by structure.
- The recipe DB loads all of zugot.
- Trust is no longer hard-coded.
- The parsers have real fuzz targets.

---

## Open gaps

| Gap | Why it matters | Roadmap |
|---|---|---|
| **Marketplace and Flutter nodes lock no checksum** | Since 1.7.3 a lock pins the bytes of apt packages and recipes (ADR 0009), but nous's marketplace view has no hash to pin, so "same version, other bytes" goes unseen there | 1.8.x (the native index) |
| **Recipe deps have no version constraints** | Only marketplace-manifest deps feed the solver; recipe-only graphs cannot detect conflicts | gated on a zugot syntax |
| **No native source** | There is no `SOURCE_NATIVE`, signed index or local store, so the apt leg is required on a no-apt box | 1.8.x (gated on ark M0) |
| **Registry is a stub** | It scans a local directory, and `registry_install_package` returns `Ok(0)` | 1.9.x |
| **No trust check during resolution** | Package signatures are not verified (sigil) | 1.9.x |
| **No caching / incremental resolution** | Every resolve re-runs every source query | 1.10.x |
| **No version selection / backtracking** | One candidate per name, checked rather than chosen; a set solvable by an older version is rejected | after 1.8.x / 1.9.x (PubGrub-style; needs a multi-version source) |
| **System leg re-derived from `Depends`** | apt's own solver (`apt-get -s install`) would handle alternatives and virtual packages; nous records them in `nous_pl_unresolved` | roadmap |
| **Bare public names** | The namespace is flat. Nine names are deprecated in favour of `nous_*` / stdlib (1.5.2, 1.5.3); the rest of the bare API is a 2.0.0 decision | 2.0.0 |

Not nous's job:
- **Verifying downloaded artifacts against a recipe's SHA256.** nous carries
  the checksum (`rc_sha256`, and since 1.7.3 `nous_rp_checksum` and the lock's
  `checksum`), but nous never downloads anything. The installer (ark /
  takumi) does that check, and apt checks its own `.deb`s.
- **Resolving a `github_release` asset URL.** That needs the GitHub API, and
  nous does no network access. takumi resolves it at fetch time
  (`takumi/src/fetch.cyr`). Since 1.7.0 nous exposes the recipe's
  `nous_rc_github_release` and `nous_rc_release_asset`.

---

## Cyrius-specific notes (verified on 6.7.18)

- **Layouts are manual** (`alloc` + `store64`, read through `load64` accessors),
  with no `struct` declarations. That came from early cyrius, where struct
  literals worked only in `main()`. cyrius 6.x has full structs; adopting them
  would change no behaviour and is not scheduled.
- **`is_dir(Str)` from `lib/fs.cyr` works** (checked in 1.5.2). nous keeps its
  one-`stat` `our_is_dir(cstr)` on purpose: the stdlib version does open +
  getdents + close (3 syscalls), and it is called once per directory entry.
- **`map_keys` is hash-slot order, and the hash is seeded per process** (since
  cyrius 6.5.39). Never let it reach output: use `_nous_sorted_keys`. The 1.5.3
  P0 came from exactly this.
- **`stack` is a reserved word.**
- **Argument kinds differ.** `dir_list` and `is_dir` take a `Str`.
  `file_read_all`, `file_exists` and `our_is_dir` take a cstr (`to_cstr(s)`).
- **`str_new` / `str_sub` / `str_trim` / `str_split` return views**, not copies.
  Copy (`str_clone`) anything that must outlive the buffer it points into; the
  1.5.1 capture-aliasing bug came from this.
- **Results are value-form pairs** (6.6.0+). Bind both halves
  (`var t, v = f();`) and re-emit both when propagating (`return Err(v);`).
- **A child process gets an empty environment.** Pass absolute tool paths
  (`which_path` / `tool_path`; see issue 0002).
- **The namespace is flat.** Give new public names a `nous_` prefix, and grep
  sibling repos' `src/` for a name before adding it.

---

*This document should be consumed by any agent hardening nous or ark.*
