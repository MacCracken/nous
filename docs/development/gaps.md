# Nous — Known Gaps & Hardening Targets

> **Status**: Active | **Last Updated**: 2026-11-12 (1.5.2, cyrius 6.7.18)
>
> This page tracks the gaps between what nous does and what a production
> resolver needs. The prioritized, versioned plan lives in
> [`roadmap.md`](roadmap.md); this page says *why* each gap matters.

---

## Current State

nous has 15 `src/` modules behind a barrel (`src/nous.cyr`) and ships a
single-file `dist/nous.cyr` bundle (`cyrius distlib`). It depends on bayan's
`dist/bayan-toml.cyr` profile
([ADR 0004](../adr/0004-bayan-toml-profile-dependency.md)). Struct layouts are
manual, and every function carries an explicit `: i64` return type.

**Works today:**
- Transitive resolution across 4 sources (System, Marketplace, FlutterApp,
  Community), with 4 strategies.
- Version constraints: 8 operators. Kahn topological sort, DFS 3-colour cycle
  detection, and conflict reporting.
- Zugot recipe parsing, a recipe DB, `build-order.txt`, and recipe fallback in
  resolution.
- Cross-source search with marketplace-priority dedup, and installed-package
  listing in deterministic order.
- Update detection: the apt leg plus a pluggable lookup for the other sources.
- Complete child-output capture. Errors surface as `Result` and are never a
  partial result.
- JSON for every public type. Error display for all 7 kinds, typo suggestions,
  and a resolution trace.

**Closed since this page began (1.x):** all of the old P1 (graph resolution), P2
recipe parsing and build order, and P5 error quality (conflict explanation,
suggestions, trace).

---

## Open gaps

| Gap | Why it matters | Roadmap |
|---|---|---|
| **No lockfile** | The same inputs always give the same graph, but nothing pins the result across time or machines | 1.7.x |
| **Recipe deps have no version constraints** | Only marketplace-manifest deps feed the solver; recipe-only graphs cannot detect conflicts | 1.7.x |
| **`github_release` URLs are placeholders** | `recipe_parse_file` builds `https://github.com/<owner>/<repo>/releases`, not an asset URL | 1.7.x |
| **No native source** | There is no `SOURCE_NATIVE`, signed index or local store, so the apt leg is required on a no-apt box | 1.8.x (gated on ark M0) |
| **Registry is a stub** | It scans a local directory, and `registry_install_package` returns `Ok(0)` | 1.9.x |
| **No trust check during resolution** | Package signatures are not verified (sigil) | 1.9.x |
| **No caching / incremental resolution** | Every resolve re-runs every source query | 1.10.x |
| **Bare public names** | The namespace is flat, so `which_exists` / `cmd_run` collide with kavach / thoth. Deprecated in 1.5.2, removed in 2.0.0, and a full audit is planned in 1.5.3 | 1.5.3, 2.0.0 |

Not nous's job: **verifying downloaded artifacts against a recipe's SHA256.**
nous carries the checksum (`rc_sha256`), but nous never downloads anything. The
installer (ark / takumi) does that check.

---

## Cyrius-specific notes (verified on 6.7.18)

- **Layouts are manual** (`alloc` + `store64`, read through `load64` accessors),
  with no `struct` declarations. That came from early cyrius, where struct
  literals worked only in `main()`. cyrius 6.x has full structs, so whether to
  adopt them is part of the 1.5.3 idiom review.
- **`is_dir(Str)` from `lib/fs.cyr` works** (checked in 1.5.2: 1 for a
  directory, 0 for a file or a missing path). nous still uses its own
  stat-based `our_is_dir(cstr)` from when the stdlib one was broken. That copy
  is also where the 1.3.1 stack smash happened, so retiring it is a 1.5.3
  candidate.
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
