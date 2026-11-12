# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.5.1] - 2026-11-12

**cyrius 6.7.9 → 6.7.18, bayan moved from the stdlib fold to its slim `toml` profile, and the
two issues the 1.5.0 audit found are fixed.** Fixing the first exposed a third defect, also fixed:
capture results aliased one shared buffer. 310 assertions (was 287), 0 failures; fuzz ok. fmt /
lint / vet / deny / doc clean, and lint now prints no `note` lines either. Build warning-free.

### ⚠ BREAKING — `ErrKind`'s bare `ERR_*` members are now `NOUS_ERR_*`

| Before | After | Value |
|---|---|---|
| `ERR_COMMAND_EXEC` | `NOUS_ERR_COMMAND_EXEC` | 0 |
| `ERR_REGISTRY_IO` | `NOUS_ERR_REGISTRY_IO` | 1 |
| `ERR_INVALID_MANIFEST` | `NOUS_ERR_INVALID_MANIFEST` | 2 |
| `ERR_INVALID_PACKAGE_NAME` | `NOUS_ERR_INVALID_PACKAGE_NAME` | 3 |
| `ERR_INVALID_VERSION_CONSTRAINT` | `NOUS_ERR_INVALID_VERSION_CONSTRAINT` | 4 |
| `ERR_DEPENDENCY_CYCLE` | `NOUS_ERR_DEPENDENCY_CYCLE` | 5 |
| `ERR_VERSION_CONFLICT` | `NOUS_ERR_VERSION_CONFLICT` | 6 |

Enum members are flat global constants, and bare `ERR_*` names are reserved for sakshi's base
logger. cyrius lint has said so in a `note` since 6.4.51 (a leaf library must use `<LIB>_ERR_*`).
This is the same cross-library collision class as 1.4.0's `agent_info_new`. No collision existed
yet (none of the seven names is defined in the 6.7.18 stdlib or any sibling repo's `src/`), so the
rename removes a latent hazard rather than a live one.

- **Values are unchanged**, so the JSON wire format (`{"kind":N}`) is unchanged. A new test,
  `err_kind_values`, pins all seven.
- ⚖️ **Measured blast radius: one consumer, two sites.** `ark/src/engine.cyr:41` and `:514`
  compare against `ERR_INVALID_PACKAGE_NAME`. ark pins nous 1.4.0, so nothing breaks until it
  moves to 1.5.1.
- ⚖️ **SemVer:** this is an incompatible public-API change shipped as a patch. CLAUDE.md asks for
  a major bump for a breaking change, and 1.4.0 used a minor for its rename. **Kept at 1.5.1 by
  the maintainer's decision** (2026-11-12), with the one known consumer migration above.

**Migration** (idempotent; `\b` does not match inside `NOUS_ERR_`):

```bash
sed -i -E 's/\bERR_(COMMAND_EXEC|REGISTRY_IO|INVALID_MANIFEST|INVALID_PACKAGE_NAME|INVALID_VERSION_CONSTRAINT|DEPENDENCY_CYCLE|VERSION_CONFLICT)\b/NOUS_ERR_\1/g' src/*.cyr
```

### Changed — bayan comes from its `toml` profile, not the stdlib fold ([ADR 0004](docs/adr/0004-bayan-toml-profile-dependency.md))

cyrius 6.7.18 folds bayan 1.11.1. Its `# Requires` pulls `math`, `slice`, `fmt` and seven
`unicode/*` files, about 330 KB of tables that DCE cannot drop, and that took nous's release
binary from **58,504 to 406,296 B**. nous uses six 16-byte record helpers from bayan.

- `"bayan"` is dropped from `[deps] stdlib`, replaced by `[deps.bayan]` (git, https, tag `1.11.1`,
  `modules = ["dist/bayan-toml.cyr"]`, commit `965298d` in `cyrius.lock`). This is the same shape as
  samay, shravan and mihi.
- `src/recipe.cyr` calls bayan's canonical `bayan_toml_section_new` / `_name` / `_pairs` and
  `bayan_toml_pair_new` / `_key` / `_value`. The unprefixed `toml_*` aliases exist only in the
  full fold's `_compat` module.
- **Release binary: 32,528 B** with this change alone (32,688 B with the capture fix); −44% against
  1.5.0. lib/ stays at 38 files.
- ⚠ **Consumer-facing.** `dist/nous.cyr` now needs those six `bayan_toml_*` functions.
  `dist/nous.deps` went 12 → 4 leaves (`hashmap`, `fs`, `process`, `syscalls`): since 6.7.12/6.7.13
  distlib credits leaves to the named dep that brings them, and a sidecar never lists named deps.
  A consumer supplies the functions either with the same profile or with the full stdlib `bayan`
  fold, which defines them too, but not both. **ark needs no change here**: it already lists
  stdlib `bayan`.
- CI's `cyrius deps` step now fetches that one git dep; its comment says so.

### Fixed — ⛔ child-output capture silently truncated at 64 KiB

Every capture read into one fixed 65,536-byte buffer. `exec_capture` stops at its buffer length,
and a full buffer reads the same as EOF, so longer output was cut with no error. The last line
could also be cut mid-record. Reproduced before the fix: a 200,000-byte output came back as
65,536 bytes.

- **Worst site: `sysdb_list`**, which runs `dpkg-query -W` over *every* installed package. At
  ~45 B a line it passes 64 KiB at ~1,500 packages, so on most real Debian/Ubuntu systems
  `resolver_list` silently dropped installed system packages.
- **Also affected:** `sysdb_search` (a broad `apt-cache search`) and `sysdb_updates` (a long
  `apt list --upgradable`).
- **Now:** `nous_capture_n(args, init, max)` reads up to `cap + 1` bytes. If the output did not
  fit, it re-runs with double the room, up to `NOUS_CAPTURE_MAX` (16 MiB). Past that, or when the
  stdlib's idle deadline cut the run (`proc_set_timeout_ms`), the result is
  `Err(NOUS_ERR_COMMAND_EXEC)` with the command line and the limit. It is never a partial `Ok`. The
  re-run is safe because nous captures only read-only queries.
- **New API, all `nous_`-prefixed:** `nous_capture_n`, `nous_capture(args)` (default limits) and
  `nous_cmd_capture(cmd)` (via `/bin/sh -c`). ai-hwaccel already defines a 4-arg `cmd_capture`.
- **The system-leg error now surfaces.** `resolver_search`, `resolver_list` and
  `resolver_check_updates` pass a `sysdb` Err up instead of skipping it. Skipping returned `Ok`
  with the system results missing. `resolve_sys` and `resolver_updates` already propagated.
  ⚖️ These can now return Err in the two cases above, where they used to return a partial Ok.
- **`cmd_run` is kept for compatibility** (returns a Str or 0). It no longer truncates and returns
  an owned copy, but it still cannot tell failure from no output. nous no longer calls it.
- **The bare `cmd_buf` global is removed.** No user was found in any repo.

### Fixed — ⛔ capture results aliased one shared buffer (found while fixing the above)

`str_new` / `str_sub` / `str_trim` / `str_split` make **views**, not copies. Every Str a capture
returned (package names, versions, descriptions, deps) pointed into the one global `cmd_buf`, and
the next capture overwrote it.

- **Reproduced before the fix:** after `a = cmd_run("echo hello"); b = cmd_run("echo WORLD");`,
  `a` read `WORLD`.
- **In real use:** `installed = resolver_list(r)` followed by
  `resolver_check_updates(r, installed, f)` had the `apt list` capture rewrite the system entries'
  names in `installed`.
- **Now:** a complete capture is copied out (`str_clone`), and only the scratch buffer is reused.

**Tests.** `capture_complete` (14 assertions) covers aliasing, the 200,000-byte capture, growth
1024 → 4096, output exactly at max, and overflow as an Err naming the command and the limit, plus
no-output `Ok(0)`. `capture_deadline` (2) covers a deadline cut as an Err. Mutation-checked:
dropping the copy-out fails "first capture survives the second", and returning a partial capture
instead of growing fails five assertions.

### Changed — toolchain + libs

- **Pin `6.7.9` → `6.7.18`**, re-vendored with `cyrius deps` alone. The 38 stdlib files are
  byte-identical to the 6.7.18 snapshot, and `bayan-toml.cyr` is byte-identical to bayan 1.11.1's
  `dist/`.

### Audited — the 6.7.10–6.7.18 changes that alter already-compiling code do not reach nous

Method: three parallel reads of the 45,800-line changelog range, then a comment- and
string-stripped scan of nous for every flagged shape.

| Change | What nous has |
|---|---|
| Wrapped pair calls `(f(x))` refused, transparent to pair checks (6.7.13 MV-8, 6.7.14 RC-21) | None |
| A zone constant global no longer reset after an earlier call initialiser (6.7.14 FE-15) | Nothing after `var exit_code = main();` |
| Zone global redeclared / conflicting value (6.7.15 FE-13, 6.7.12 FX-6) | No duplicate top-level `var` |
| `dir_list` opens with `O_DIRECTORY` (6.7.11) | All three sites check `our_is_dir` first |
| Capture with a closed std fd now works (6.7.15 `_proc_redirect`) | Only if nous runs with fd 0/1/2 closed, and then it gets real output instead of 0 bytes |
| `lib/process.cyr` now includes `lib/io.cyr` (6.7.16) | No nous fn collides with an io name |
| DCE reachability is a worklist (6.7.18) | Output changes only where it was wrong: call chains over 512 deep |
| Enum separators, include tails, value builtins as statements (6.7.16/17) | All are errors now, and the build passes |
| `args_init` reads cmdline to EOF (6.7.12) | nous ignores its return |
| `fncallN(&f, ..)` of a pair fn returns the pair (6.7.17) | nous's one `fncall2` callee is a variable |
| bayan 1.11.x `#deprecated` lookups | nous calls none |
| Plaintext dependency URLs refused (6.7.18) | `[deps.bayan]` is https |

### Performance

Same machine, 1.5.0 on 6.7.9 vs 1.5.1 on 6.7.18, median of 3 alternating runs:

| Bench | 1.5.0 / 6.7.9 | 1.5.1 / 6.7.18 | Δ |
|---|---|---|---|
| `detect_source` | 640 ns | 576 ns | −10.0% |
| `validate_name` | 518 ns | 523 ns | +1.0% |
| `resolve_mkt_hit` | 2.29 µs | 2.23 µs | −2.7% |
| `resolve_mkt_miss` | 4.67 µs | 4.51 µs | −3.4% |
| `search_100` | 88.50 µs | 84.72 µs | −4.3% |
| `list_installed_100` | 13.10 µs | 13.34 µs | +1.9% |
| `strat_mkt_first` | 2.24 µs | 2.22 µs | −1.0% |
| `strat_sys_first` | 2.27 µs | 2.27 µs | −0.2% |
| `strat_only_mkt` | 2.25 µs | 2.23 µs | −1.0% |
| `strat_search_all` | 2.23 µs | 2.21 µs | −0.8% |
| `graph_resolve` | 9.18 µs | 8.98 µs | −2.1% |
| `recipe_parse` | 18.94 µs | 17.98 µs | −5.1% |
| `levenshtein` | 2.28 µs | 2.26 µs | −0.7% |
| `constraint` | 713 ns | 702 ns | −1.5% |
| `cycle_detect_20` | 24.42 µs | 22.70 µs | −7.0% |
| `topo_sort_20` | 36.50 µs | 34.57 µs | −5.3% |
| `recipe_db_load` | 7.147 ms | 6.987 ms | −2.2% |
| `serde_roundtrip` | 2.89 µs | 2.76 µs | −4.7% |
| `capture_200k` (new) | — (truncated) | 1.92 ms | — |

- **The 1.5.0 `detect_source` regression is gone**: 1.5.0 measured 576 → 615 ns, and 6.7.18 is
  back to 574–583 ns.
- **`recipe_parse` −5%** (18.9–19.3 vs 17.3–18.5 µs, no overlap) comes from calling `bayan_toml_*`
  directly, without the compat-wrapper hop. `serde_roundtrip` also separates (2.86–2.92 vs
  2.68–2.77 µs). The `cycle_detect` / `topo_sort` ranges overlap, so they are noise. One 1.5.0
  run was uniformly elevated, and the medians absorb it.
- ⚖️ **The copy-out has a cost.** In an A/B at 60,000 bytes, which fits the old buffer, a raw
  `exec_capture` took 620–630 µs and `nous_capture` 728–755 µs: about +120 µs. The stdlib's
  `memcpy` (behind `str_clone`) is a byte loop, about 2 ns a byte. That is the price of fixing
  the aliasing, and against real apt/dpkg runs (tens to hundreds of ms) it is noise.
- **Binary size:** release build 58,504 → 32,688 B (−44%); dev build 631,944 → 286,640 B.

### Corrections to earlier entries

- **1.5.0 said `cyrius lib sync --dry-run` prints addresses "on 6.6.20, 6.7.9 and 6.7.18".** The
  6.7.18 observation was wrong. That run was in a project pinned to 6.6.20, so the 6.7.18 CLI
  re-executed into 6.6.20. It was fixed upstream in 6.7.12 (TL-14), and 6.7.18 prints paths.
- **1.5.0's lock trap (`lib sync` before `deps`) is fixed upstream in 6.7.13** (TL-11 / D42).
  Reproduced from the 1.5.0 tree: on 6.7.18 a scoped `lib sync` across a pin move refuses, names
  `--full`, and writes nothing. Running `cyrius deps` alone remains the procedure.

### Known — found by this release's audit (tracked in the roadmap)

- **Bare-name function collisions, latent.** `which_exists(name)` is also defined by kavach with
  the same arity, and `cmd_run(...)` by thoth (same arity) and by cyrius's build tool. No consumer
  vendors nous next to kavach or thoth today. Renaming either is breaking, so this is left for a
  planned API break.

## [1.5.0] - 2026-11-12

**cyrius 6.6.20 → 6.7.9, stdlib re-vendored to the 6.7.9 bundled versions.** 287 assertions,
0 failures; fuzz ok. No nous source change and no public API change. fmt / lint / vet / deny / doc
clean; build warning-free.

### Changed — toolchain + bundled libs

- **Pin `6.6.20` → `6.7.9`.**
- **`lib/` re-vendored from the 6.7.9 snapshot**: 38 files, each byte-identical to the snapshot.
  bayan **1.5.12 → 1.5.13**. `hashmap_fast` is no longer in the resolved closure (39 → 38 files),
  because nothing nous pulls in includes it under 6.7.9. `[deps] stdlib` is unchanged.
- **`cyrius.lock` regenerated** (38 entries, pin line `6.7.9`). It was generated from an empty
  `lib/` with `cyrius deps` alone (see *Toolchain findings* below).
- **`dist/`**: only the bundle's version header changes. `dist/nous.deps` is unchanged (12 leaves).

### Audited — the 6.7.x changes that alter already-compiling code do not reach nous

These changes change what code that already compiles computes, so a green suite proves nothing
about them. Method: a comment- and string-stripped scan of `src/` and `tests/` (3,637 code lines)
for each shape, plus reading the affected stdlib path.

| Change | What it changed | What nous has |
|---|---|---|
| A lone `!` (6.7.3) | Silently dropped before 6.7.3, so `if (!ok)` ran as `if (ok)` | None. nous spells it `== 0` |
| Unary `-` / `~` on a constant before a literal operator (bug 6.6.10–6.7.8, fixed 6.7.9) | `-K + 10` was 15, `~5 * 2` was 10 | No unary `-` or `~` anywhere (nous writes `(0 - 1)`), so 1.4.1 on 6.6.20, inside the bug's range, was not affected either |
| Typed `*T` pointer step leaking through operators (6.6.17–6.7.8, fixed 6.7.9) | `(x == q) + 100` stepped by `sizeof(T)` | No typed pointers |
| Method-call and generic results typed by their last argument (fixed 6.7.9) | `p.m(1.5) * 2` multiplied bit patterns | No methods, generics or floats |
| By-value struct argument evaluated after later arguments (fixed 6.7.7) | `rd(b, bump(&b))` saw the bump | No `struct` declarations; layouts are manual `alloc` + `store64` |
| `fncall8` arguments 7 and 8 swapped (fixed 6.7.6) | — | Only `fncall2` (the `resolver_check_updates` callback) |
| **The exec family resets child signal state** (`lib/process.cyr`, 6.7.9) | Children start with SIGPIPE / SIGXFSZ at default and (Linux) an empty mask, not the parent's | 5 `exec_capture` sites (`apt-cache`, `dpkg-query`, `/bin/sh -c` for `apt list`). The child's stderr already goes to `/dev/null` and nous discards the exit status, so the captured bytes are the same |

⚖️ The one observable difference is under a parent that ignores SIGPIPE, which a consumer may.
There, a child whose output overruns nous's 64 KiB capture now dies of SIGPIPE instead of getting
EPIPE. nous captures the same bytes either way. That cap is a separate, pre-existing limit (see
*Known*).

### Performance — flat, one small codegen cost

Same machine, 1.4.1 on 6.6.20 vs 1.5.0 on 6.7.9, median of 3 alternating runs:

| Bench | 1.4.1 / 6.6.20 | 1.5.0 / 6.7.9 | Δ |
|---|---|---|---|
| `detect_source` | 576 ns | 615 ns | **+6.8%** |
| `validate_name` | 497 ns | 492 ns | −1.0% |
| `resolve_mkt_hit` | 2.14 µs | 2.14 µs | +0.2% |
| `resolve_mkt_miss` | 4.49 µs | 4.43 µs | −1.2% |
| `search_100` | 83.39 µs | 83.95 µs | +0.7% |
| `list_installed_100` | 12.43 µs | 12.43 µs | 0.0% |
| `strat_mkt_first` | 2.16 µs | 2.15 µs | −0.6% |
| `strat_sys_first` | 2.19 µs | 2.19 µs | −0.1% |
| `strat_only_mkt` | 2.17 µs | 2.16 µs | −0.3% |
| `strat_search_all` | 2.17 µs | 2.16 µs | −0.2% |
| `graph_resolve` | 8.81 µs | 8.89 µs | +0.9% |
| `recipe_parse` | 17.97 µs | 18.35 µs | +2.1% |
| `levenshtein` | 2.20 µs | 2.19 µs | −0.3% |
| `constraint` | 692 ns | 688 ns | −0.6% |
| `cycle_detect_20` | 24.04 µs | 24.84 µs | +3.3% |
| `topo_sort_20` | 36.51 µs | 38.24 µs | +4.8% |
| `recipe_db_load` | 6.831 ms | 6.885 ms | +0.8% |
| `serde_roundtrip` | 2.77 µs | 2.76 µs | −0.3% |

- ⚖️ **`detect_source` +6.8% is real**: the runs spanned 573–577 ns vs 614–621 ns, with no
  overlap. Every stdlib function on its path (`str_index_of`, `str_ends_with`, `str_from`,
  `str_sub`, `str_eq`, `str_len`, `str_data`, plus `alloc` and `vec`) is unchanged apart from
  comments, so the ~40 ns comes from 6.7.9's codegen or code layout. Not chased: it is a name
  classifier, not a hot path.
- `cycle_detect_20` and `topo_sort_20` overlap run to run (22.2–26.7 and 33.4–42.5 µs on 6.7.9),
  so they count as noise.
- **Closes the 1.4.1 `search_100` watch.** Re-measured: 1.4.1 83–88 µs, 1.5.0 81–89 µs.
- **Binary size:** release build (`CYRIUS_DCE=1`) 58,520 → 58,504 B.

### Toolchain findings (not nous defects)

- **Running `cyrius lib sync` before `cyrius deps` after a pin bump breaks the lock.** `lib sync`
  re-hashes only the 33 declared `[deps] stdlib` leaves but rewrites the lock's pin line to the new
  pin. The transitive leaves (`atomic`, `boxed`) keep the old pin's hashes. The next `cyrius deps`
  then refuses both ("cyrius.lock and the pinned stdlib snapshot DISAGREE under an unchanged pin
  6.7.9"). Reproduced on two scratch copies from the same 6.6.20 lock: `deps` alone re-locks
  cleanly, while `lib sync` then `deps` fails with 2 errors. **Bump procedure: change the pin, then
  run `cyrius deps` alone.** 1.4.1 avoided the trap only because no lock existed yet.
- `cyrius lib sync --dry-run` prints addresses instead of file names (`would sync:
  140577547063856`). Seen on 6.6.20, 6.7.9 and 6.7.18. Display only.

### Known — pre-existing, surfaced by this audit (tracked in the roadmap)

- **The 64 KiB capture cap silently truncates.** `cmd_run` and every `sysdb` query read child
  output into one 65,536-byte buffer and drop the rest, with no error. A broad
  `apt-cache search --names-only` or a long `apt list --upgradable` can exceed that, losing results
  and leaving a partial last line.
- **Bare `ERR_*` enum members.** `cyrius lint` notes (a `note`, not a `warn`, so the gate is green;
  6.6.20 printed the same seven) that `ErrKind`'s `ERR_*` names are reserved for sakshi's base
  logger. A leaf library should use `NOUS_ERR_*`. No collision exists today: none of the seven
  names is defined in the 6.7.9 stdlib or in any sibling repo's `src/`. Renaming them is a
  breaking change.

## [1.4.1] - 2026-11-12

**cyrius 6.6.2 → 6.6.20, stdlib re-vendored to the 6.6.20 bundled versions.** 287 assertions,
0 failures; fuzz ok; no public API change. fmt / lint / vet / deny / doc clean, build
warning-free.

### Changed — toolchain + bundled libs

- **Pin `6.6.2` → `6.6.20`.**
- **`lib/` re-vendored from the 6.6.20 snapshot** (`cyrius lib sync`, then `cyrius deps`). All 39
  files are byte-identical to the snapshot. The one fold nous uses moved **bayan 1.5.5 → 1.5.12**
  (re-vendored from its tag by cyrius 6.6.18). The transitive closure gained four leaves that the
  6.6.20 stdlib now includes (`alloc_cx`, `chrono`, `hashmap_fast`, `poison`): 35 → 39 files.
  `[deps] stdlib` itself is unchanged.
- **`cyrius.lock` added.** `cyrius deps` now records the sha256 of every vendored leaf (39
  entries). ark, patra, sigil, daimon, agnostik, bayan and yukti all track theirs.

### Changed — `dist/` bundle shape (consumer-facing; toolchain-driven, no nous source change)

- **`dist/nous.cyr` opens with a compile-verified `# Requires` block**: one
  `include "lib/<leaf>.cyr"` per sidecar leaf (cyrius 6.6.18), so the bundle compiles when
  included on its own. Stdlib content is still not bundled. A `cyrius deps` consumer is
  unaffected: include-once is keyed on the literal path, so ark's own includes of the same leaves
  dedupe.
- **`dist/nous.deps` 16 → 12 leaves.** The sidecar is now what distlib's compile-verify proves the
  bundle needs, instead of a copy of `[deps] stdlib` (cyrius 6.6.18). Dropped: `fmt`, `args`,
  `assert`, `tagged`, `bench`. Those serve the tests and the smoke binary, and the bundle compiles
  without them. Added: `result`.
- ⚖️ A consumer that relied on nous's sidecar to bring in one of the five dropped leaves must now
  declare it itself. ark declares its own `[deps] stdlib`, and since cyrius 6.6.18 a failed build
  names the leaf to add.

### Fixed — the three new `assigning non-pointer to typed pointer` warnings

6.6.20 flags three sites. Each local was initialised from `str_from("")`, which types it `Str`,
and was then assigned an `i64` that holds a `Str` at runtime. None of them hid a bug.

| Site | Local | Assigned from |
|---|---|---|
| `src/sysdb.cyr` `sysdb_updates` | `av` | `vec_get(rest, 1)` |
| `src/recipe.cyr` `recipe_parse_file` | `url` | `cyml_get(src, "url")` |
| `src/recipe.cyr` `recipe_parse_file` | `sha256` | `cyml_get(src, "sha256")` |

The locals now start as `i64` and fall back to `""`, the same idiom `recipe_parse_file` already
uses for `name` / `ver` / `desc`. Every path produces the same value as before.

### Performance — flat

Same machine, 1.4.0 on 6.6.2 vs 1.4.1 on 6.6.20, median of 3 alternating runs (`cyrius bench
tests/nous.bcyr`, each build on its own pinned stdlib):

| Bench | 1.4.0 / 6.6.2 | 1.4.1 / 6.6.20 | Δ |
|---|---|---|---|
| `detect_source` | 570 ns | 581 ns | +1.9% |
| `validate_name` | 509 ns | 499 ns | −2.0% |
| `resolve_mkt_hit` | 2.19 µs | 2.17 µs | −1.2% |
| `resolve_mkt_miss` | 4.63 µs | 4.46 µs | −3.8% |
| `search_100` | 77.92 µs | 87.53 µs | +12.3% |
| `list_installed_100` | 12.75 µs | 12.65 µs | −0.8% |
| `strat_mkt_first` | 2.18 µs | 2.16 µs | −0.8% |
| `strat_sys_first` | 2.21 µs | 2.17 µs | −1.7% |
| `strat_only_mkt` | 2.19 µs | 2.16 µs | −1.6% |
| `strat_search_all` | 2.17 µs | 2.18 µs | +0.3% |
| `graph_resolve` | 8.67 µs | 8.87 µs | +2.3% |
| `recipe_parse` | 18.31 µs | 19.03 µs | +4.0% |
| `levenshtein` | 2.21 µs | 2.23 µs | +1.2% |
| `constraint` | 691 ns | 709 ns | +2.6% |
| `cycle_detect_20` | 23.20 µs | 22.44 µs | −3.3% |
| `topo_sort_20` | 35.84 µs | 33.71 µs | −6.0% |
| `recipe_db_load` | 6.982 ms | 7.007 ms | +0.4% |
| `serde_roundtrip` | 2.68 µs | 2.77 µs | +3.2% |

- ⚖️ **`search_100` (`resolver_search`) is the one row outside noise, and only just.** Across
  every clean run, 1.4.0 spanned 76–93 µs (4 runs) and 1.4.1 spanned 84–92 µs (5 runs). It is
  flagged here, not chased in a patch release.
- ⚖️ The first 6.6.20 run showed 2–2.6× slowdowns on several rows (`graph_resolve` 22.5 µs,
  `strat_*` ~5.4 µs). Five later runs did not reproduce it, so it was discarded.
  `bench-history.csv` carries a clean `1.4.1-local` run.
- **Binary size:** release build (`CYRIUS_DCE=1`) 56,264 → 58,520 B (+4.0%). The non-DCE dev
  build grew 555,976 → 631,960 B, which is stdlib growth in unreachable code (1,452 → 1,660
  unreachable fns).

## [1.4.0] - 2026-09-10

### ⚠ BREAKING — `agent_info_new` renamed to `nous_agent_info_new`

It **collided with agnostik's** `agent_info_new(id, name: Str, atype, status)` — a different
function, 4 args over a 32-byte {id, name, type, status} struct, where nous's is 3 args over a
24-byte {name, version, desc} one. Any project vendoring both dists got **"last definition wins",
so calls to the other arity silently mis-bound their arguments**. ark vendors both and is where it
surfaced: cyrius 6.6.2 makes an arity-disagreeing duplicate a hard error rather than a warning,
which is the only reason anyone found out.

⚖️ **nous renamed rather than agnostik**, for a reason and not a coin flip: agnostik is the
SHARED-types library for AGNOS and agent identity/type/status is squarely its domain, so its names
are meant to be ecosystem-wide. nous's is package metadata sitting beside `manifest_new`; the
generic name was the intruder.

⚖️ **Measured blast radius: zero external callers.** Scanned every `src/` under `~/Repos` — the
only callers of either symbol are the two repos that define them. Three sites changed inside nous
(`src/types.cyr`, `src/registry.cyr`, `tests/nous.tcyr`). A minor, not a patch, because the public
surface changed incompatibly.

## [1.3.2] - 2026-09-10

**cyrius 6.3.35 → 6.6.2 value-form migration**, plus a CI step that would have corrupted the
shared stdlib. 287 assertions, 0 failures.

### Fixed — ⛔ the error-propagation trap: errors that read as SUCCESS

cyrius 6.6.0 made `Result` a `(tag, payload)` register pair. The pre-flip propagation idiom

    var rr = registry_new(mkt_dir);
    if (is_err_result(rr) == 1) { return rr; }

migrated naively returns the **payload alone**, so the caller receives the error VALUE as its
TAG — a propagated `Err(77)` arrives as `tag=77, is_err=0`. Neither spelling is a type error, so
it survives a green build AND a green test suite. This is the shape that produced **19 silent
defects in yukti** during its own 6.6.0 migration.

cycc named four functions — `resolver_new`, `resolver_resolve`, `resolver_resolve_all`,
`resolver_resolve_all_with_recipes`. A full audit of every `Result`-returning function found
**more than the compiler reported**, including the entire pass-through body of
`resolve_sys` (`src/resolver.cyr`), whose single `return result;` covered BOTH the Err and Ok
paths — every system-layer error would have read as success. All now `return Err(x);` / `Ok(x)`.

⚖️ **Honest scope:** of the propagation sites repaired, only the `validate_package_name` one was a
LIVE defect today — the other callees currently return `Ok` on every path, so those were LATENT
traps. All are fixed regardless: each callee is `Result`-typed and may grow an `Err` path.

**Zero mixed-return warnings** now, and the detector was confirmed live by reverting one fix and
watching cycc name the file and line.

### Changed — value form throughout

71 sites: 8 in `src/` (4 files), 63 in `tests/`. `payload(r)` → the second bound variable,
predicates take the tag, `var r = f();` → `var r_t, r = f();`. nous has no hand-rolled
`tagged_new` boxes, so no `boxed_*` is used anywhere.

### Fixed — ⛔ CI symlinked `lib/` into the SHARED stdlib snapshot

Two jobs ran `ln -sf "$HOME/.cyrius/lib" lib`. That is a directory-level `lib` symlink into the
version-pinned snapshot — the pattern CLAUDE.md forbids, because a write through it lands in the
shared stdlib and corrupts **every project on the machine** (the 2026-09-01 incident: 27 files
replaced, `lib/alloc.cyr` losing the v6.4.1 information-leak fix, shabda ingesting the corruption
12 seconds later).

cyrius has REFUSED to vendor into a symlinked `lib/` since 6.5.37, so the following
`cyrius deps` step would have failed CI outright. Both steps removed — and they were never
needed: `cyrius deps`, `cyrius build` and `cyrius test` each populate `lib/` from the pinned
snapshot on their own (verified from an empty tree: 35 declared modules, byte-identical build,
287/287 tests).

### Changed — stdlib re-synced, formatting

Every stdlib leaf byte-identical to the 6.6.2 snapshot; zero orphans; zero retired-name calls in
`lib/`. Five files reformatted — pre-existing drift, since cyrfmt only began tracking paren depth
at v6.5.28 and nous was pinned 6.3.35.

## [1.3.1] - 2026-07-02

### Changed

- **Toolchain: cyrius pin `6.2.12` → `6.3.35`** (current toolchain). Test suite green
  (287 passed) on 6.3.35; `dist/nous.cyr` re-cut. Consumers must re-resolve the bundle.

### Fixed

- **Stack smash in the `stat` helpers (`src/util.cyr`).** `our_is_dir` / `file_mtime` /
  `file_stat_size` sized their `struct stat` buffer as `var statbuf[18]` — 18 u64 slots
  (144 B) under the pre-6.3.13 heap-local model, but an 18-**byte** stack buffer since
  cyrius 6.3.13 moved function-local `var X[N]` onto the stack. `sys_stat`'s 144-byte
  write overran it and smashed the return address (SIGSEGV). Latent until a 6.3.x
  consumer built nous — surfaced when ark exercised the resolver's `registry_new →
  our_is_dir` path. Sized to `var statbuf[144]` (matches the stdlib `struct stat`).

## [1.3.0] - 2026-06-29

### Added

- **Multi-source update detection** (`src/updates.cyr`): `resolver_check_updates(r,
  installed, avail_fn)`. The apt/system leg comes from `sysdb` (which knows both
  installed and available via `apt list --upgradable`); for every other source the
  caller supplies the available version through a **pluggable lookup callback**
  (`fncall2(avail_fn, name, source)`), and nous owns the semver comparison
  (`update_is_newer`, via `semver_parse`/`semver_cmp`) and normalization into
  `avail_update` records. This keeps nous free of any remote/transport concern —
  the marketplace consumer (e.g. ark, via mela's `/latest` endpoint) and a future
  native index both plug into the one seam. `avail_fn == 0` ⇒ apt-only (equivalent
  to the legacy `resolver_updates`).
- Why a callback rather than nous querying directly: nous tracks **installed**
  packages (apt via dpkg; marketplace via the local registry scan) but has no
  source of **available**/latest versions for non-apt sources — that data is
  remote (mela) or a future native index. The seam lets the owner of each source
  feed availability without nous taking a transport dependency.
- 1 test group (`check_updates_pluggable`, 287 total): `update_is_newer` semver
  cases; a marketplace package shows an update when the callback returns a newer
  version, none when equal, and `avail_fn == 0` stays apt-only.
- **Hardened** `update_is_newer` against the untrusted callback input
  (adversarial review): `semver_parse` returns 0 on an oversized (>i64) version
  component, so both parse results are guarded — a malformed `/latest` response
  is treated as "not newer" rather than dereferencing a null SemVer.

### Notes

- `resolver_updates(r)` (apt-only) is unchanged and retained for back-compat.
- Bundled module order: `updates.cyr` comes after `version.cyr` so it can use
  `semver_*`. `dist/nous.cyr` regenerated via `cyrius distlib`.

## [1.2.7] - 2026-06-16

### Fixed

- **SIGSEGV when the marketplace directory is absent.** `registry_new` hard-errored
  if the marketplace base dir did not exist. On a fresh system (no
  `/var/lib/agnos/marketplace`), this forced consumers to fall back to a sentinel
  (null) resolver and then dereference a non-error payload, crashing the process.
  `registry_new` now treats a missing dir as an **empty registry**, so resolution
  still falls through to the system layer and the recipe DB. Resolving any package
  against an unprovisioned system now returns cleanly instead of crashing.
  Regression covered by `test_registry_new` (missing dir → ok/empty) and
  `test_resolver_missing_marketplace` (resolve + resolve_all survive empty everything).

### Changed

- **Toolchain pin 6.2.11 → 6.2.12** (`cyrius.cyml`); matches the installed toolchain
  and ark, clears the pin-drift warning. 276-test suite green; fmt/lint clean.

## [1.2.6] - 2026-06-15

Cyrius **6.0.3 → 6.2.11** + adoption of the consolidated `bayan` stdlib bundle.

### Changed

- **Toolchain pin 6.0.3 → 6.2.11** (`cyrius.cyml`). Smoke build is clean,
  pin-drift warning gone; fmt/lint/vet/deny/doc, the 271-test suite, and all 18
  benchmarks pass unchanged on 6.2.11. No behaviour change for consumers: public
  API unchanged, resolution remains deterministic, cycle detection unchanged.
- **Swapped `json` + `toml` for `bayan` in `[deps] stdlib`.** The 6.2.x stdlib
  slim-down removed the standalone `json.cyr`, `toml.cyr` (plus `base64`,
  `bigint`, `csv`, `cyml`, `linalg`, `matrix`, `u128`) and folded them into the
  consolidated `bayan` bundle, which re-exports the old un-namespaced symbols
  via its `_compat` surface. nous now depends on `bayan` and sources the six
  `toml_*` section/pair struct constructors/accessors
  (`toml_section_new`/`toml_section_name`/`toml_section_pairs`,
  `toml_pair_new`/`toml_pair_key`/`toml_pair_value`) from it rather than
  hand-rolling them. nous still ships its own manual `src/json.cyr` (domain
  serializers — no name overlap with bayan) and its own CYML parser.

### Breaking

- **`dist/nous.cyr` consumers must supply `bayan` in their `[deps] stdlib`.**
  The bundle references the `toml_*` structs but (per the consumer-supplied-stdlib
  model) does not define them; they now come from `bayan` instead of the removed
  `toml`. ark must add `"bayan"` (and may drop `"json"`/`"toml"`).

### Fixed

- **Renamed nous's CYML parser `cyml_parse` → `nous_cyml_parse`**
  (`src/recipe.cyr`, callers in `recipe.cyr`/`tests`). bayan's `_compat` surface
  exports a `cyml_parse(data, len)` of a different (doc/entry) model, which
  collided with nous's single-bracket `[section]` parser `cyml_parse(src)`.
  Renaming keeps nous's parser (it handles `[section]` headers bayan's does not)
  while letting `bayan` satisfy the `toml_*` structs. A full symbol-collision
  audit (212 nous fns × 344 bayan fns) confirms these were the only clashes.
  `recipe_parse` (28µs) and `recipe_db_load` (10.7ms) benchmarks unchanged.

### Notes

- `dist/nous.cyr` regenerated (v1.2.6). `vet`: 14 deps, 0 untrusted, 0 missing;
  `deny`: 0 violations. DCE note grew (542 → 880 unreachable fns pre-DCE) from
  bayan's bundle bulk; `CYRIUS_DCE=1` strips it in release builds.

## [1.2.5] - 2026-05-27

Cyrius 6.0.3 + **retraction of the phantom issue-0001 "codegen bug."**

### Changed

- **Toolchain pin 6.0.1 → 6.0.3** (`cyrius.cyml`). The smoke build is clean and
  the pin-drift warning is gone; fmt/lint/vet/deny/doc, the 271-test suite, and
  all 18 benchmarks pass unchanged on 6.0.3.

### Fixed

- **Reverted the unnecessary issue-0001 `vec_get` de-nesting workaround.**
  Investigation showed there was **never a Cyrius codegen bug**: the typed
  `vec_get` nested-call / re-eval form compiles correctly on **both cycc 6.0.1
  and 6.0.3**, and the original reproducer was self-defective — its `dfs`
  omitted the `store64(result_holder, …)` that the real
  `detect_cycle_dfs` performs, so it returned `0` unconditionally on every
  compiler (its bisection table is not reproducible). Re-nesting every blamed
  site (`dep_graph_detect_cycle`, `dep_graph_topo_sort`, `resolver_resolve_all`,
  `mpkg_to_resolved`, `recipe_db_load`, sysdb parsers) and running the suite
  gives **271/0 on cycc 6.0.1 _and_ 6.0.3**. The de-nesting in `src/graph.cyr`,
  `src/resolver.cyr`, `src/recipe.cyr`, and `src/sysdb.cyr` is restored to its
  original nested form and every "Cyrius 6.0.1 miscompile" comment removed.
  [Issue 0001](docs/development/issues/0001-cyrius-6.0.1-vec-get-recompute.md) is
  **closed as not-a-defect**; ADR
  [0003](docs/adr/0003-cyrius-6.0.1-vec-get-workaround.md) is **superseded**.
- The real CI failure the 0001 narrative had obscured was entirely
  [issue 0002](docs/development/issues/0002-sysdb-exec-bare-name-path.md) — the
  sysdb bare-`argv[0]` PATH bug, a genuine nous-side defect, fixed in 1.2.0 via
  `which_path`/`tool_path`. That fix **stands**.

### Notes

- No behaviour change for consumers: public API unchanged, resolution remains
  deterministic, cycle detection unchanged. `dist/nous.cyr` regenerated (net
  −42 lines from removing the workaround temps and comments).

## [1.2.4] - 2026-05-26

Docs modernization + P(-1) arc closeout — **closes the 1.2.x arc.**

### Changed

- **De-Rust-ified `CLAUDE.md`** — dropped "flat library crate", `MSRV: 1.89`,
  `#[non_exhaustive]`/`#[must_use]` Rust attributes, and serde framing; replaced
  with the Cyrius idiom (6.0.1 pin, `#must_use`, exhaustive `match`, manual JSON
  via `src/json.cyr`, explicit `: i64` returns).
- **`src/main.cyr` banner fallback is now version-less** (`"nous — package
  resolver for AGNOS"`) — the runtime read of `VERSION` is unchanged; the
  fallback no longer needs a per-release bump (cyrius has no compile-time
  string-from-file templating).
- **Refreshed `docs/architecture/overview.md`** (bundle/build model, corrected
  stale test counts, pruned the "future modules" table — P1 modules shipped) and
  **`docs/development/gaps.md`** header/current-state (was pre-1.0).

### Added

- **`docs/adr/`** — architectural decision records (required by the project's own
  Documentation Standards): 0001 `Result<T,E>` error handling, 0002 flat modules
  + `dist/nous.cyr` bundle, 0003 the Cyrius 6.0.1 `vec_get` de-nest workaround.
- **`docs/guides/getting-started.md`** — build/test/cleanliness/bundle/consume.
- **`docs/audit/2026-05-26-1.2.x-closeout.md`** — the P(-1) Scaffold Hardening
  closeout: cleanliness + 271/0 + stable benches confirmed; internal review
  (zero TODO/unwrap/panic; both filed toolchain issues worked around); `@unsafe`
  decision (not adopted — deny/lint don't require it); external resolver-
  completeness assessment (backtracking/SAT is a documented future, not a gap).

## [1.2.3] - 2026-05-26

Return-type annotation sweep — aligns nous with the modern Cyrius idiom.

### Changed

- **Annotated all 206 `src/*.cyr` functions with explicit `: i64` return types**
  (was 0% typed). This matches the ecosystem convention: the cyrius stdlib is
  100% typed and sibling projects patra/sigil annotate every function `: i64`
  uniformly (`Str` and `Result` are fat-pointer/tagged values — all i64 under
  the hood; the precise `: Str` is a stdlib-only ergonomic). In 6.0.1 return
  types are documentation (optional; no lint/deny rule enforces them), so this
  is a mechanical, behaviour-preserving sweep — **271/0 unchanged**, codegen
  verified (build/smoke/test/fuzz), fmt/lint/vet/deny/doc all clean.
- This is idiom alignment only; it does **not** replace the issue-0001
  de-nesting (shipped in 1.2.0/1.2.1).

## [1.2.2] - 2026-05-26

Policy gates — completes the CLAUDE.md cleanliness order in CI.

### CI

- **Added `cyrius deny src/main.cyr` to CI** — the built-in project-policy
  check (delegates to `cybs`; no config file). Passes clean today (14 deps,
  0 violations).
- **Added `cyrius doc --check src/nous.cyr` to CI** — `doc --check` exits with
  the undocumented-function count; the barrel has no functions of its own, so
  it passes. CI now mirrors the full local cleanliness order: fmt → lint → vet
  → deny → doc. Per-function doc comments (and promoting the doc gate to
  per-module) are tracked for 1.2.4.

### Notes

- Re-confirmed the 1.2.x arc against sibling **vidya** and the **cyrius** stdlib/
  docs. Surfaced one headline gap — nous annotates 0% of its functions with
  return types vs the cyrius stdlib's 100% / vidya's ~99% — now scheduled as the
  1.2.3 return-type sweep. See `docs/development/roadmap.md`.

## [1.2.1] - 2026-05-26

Hygiene + codegen-workaround sweep — first 1.2.x arc follow-on after 1.2.0.
271/0 throughout; behaviour unchanged.

### Changed

- **Completed the Cyrius 6.0.1 `vec_get`-nesting de-nest sweep**
  ([issue 0001](docs/development/issues/0001-cyrius-6.0.1-vec-get-recompute.md)).
  Bound the remaining nested / re-evaluated typed accessors to locals in
  `src/source.cyr` (`collect_known_names`), `src/recipe.cyr` (`read_build_order`,
  `finalize_graph`), `src/json.cyr` (the str-vec / search / manifest-deps
  serializers), and `src/sort.cyr` — all three insertion sorts re-read
  `vec_get(v, j)` twice per inner-loop iteration, the proven-dangerous re-eval
  shape. No `outer(vec_get(…))` sites remain in `src/`.
- **`cyrius fmt` applied across `src/` + `tests/`** — the whole tree is now in
  the 6.0.1 canonical format (idempotent).
- **Cleared per-module lint warnings** — removed the consecutive-blank-line
  warnings in `error.cyr`, `recipe.cyr`, `resolver.cyr`, `source.cyr`. Every
  module + test entry lints clean.

### CI

- **Format check promoted from advisory to fail-on-drift** — and the check
  itself fixed: `cyrius fmt <f> --check` is exit-code based (0 = formatted,
  non-zero = drift; it prints nothing), but the old step diffed its empty stdout
  against the file and so falsely reported drift on every clean file. Now asserts
  exit 0 per file.
- **Lint promoted from barrel-only to per-module fail-on-warn**, auto-discovering
  every `src/*.cyr` + test entry.

## [1.2.0] - 2026-05-26

Opens the **1.2.x modernization arc** — bring nous up to the Cyrius 6.0.1
library conventions already adopted by its sibling AGNOS libraries
(patra 1.9.5, sigil 3.4.3). See `docs/development/roadmap.md` for the
1.2.1–1.2.4 follow-ons.

### Changed

- **Cyrius toolchain bump**: 5.7.29 → 6.0.1 (`cyrius.cyml` pin). Source
  required no edits; the smoke build is clean — both the pin-drift
  warning (`pins 5.7.29 but cycc is 6.0.1`) and the stdlib-shadow note
  are gone. 6.0.1 is the 6.0.0 cycle-open hotfix (two-binary rename
  `cyrc→cybs`, `cc5→cycc`; fixes the `_init_cyrius_lib` skip-prefix
  off-by-one that broke the version-pinned `lib/` fallback).
- **`lib/` de-vendored**. Removed the 24 committed `lib/*.cyr` stdlib
  files (synced from the 5.7.29 stdlib, stale at 6.0.1). nous is
  stdlib-only, so `cyrius build` now resolves `include "lib/X.cyr"` from
  the version-pinned `~/.cyrius/versions/6.0.1/lib/` snapshot — the same
  convention patra/sigil use (their `lib/` is gitignored, not committed).
  All 17 declared stdlib modules are present in the 6.0.1 snapshot.
- **`.gitignore` de-Rust-ified**. Dropped the Rust-era entries
  (`Cargo.lock`, `*.rs.bk`, `/target/criterion/`, tarpaulin, `*.profraw`,
  `lcov.info`) and adopted the Cyrius shape: ignore `/build/` and `/lib/`;
  keep `dist/` tracked. Matches patra/sigil.

### Added

- **`[lib]` section + `dist/nous.cyr` bundle**. `cyrius.cyml` now declares
  `[lib] modules = [...]` (the 14 `src/*.cyr` in barrel order), and
  `cyrius distlib` concatenates them into `dist/nous.cyr` — nous's
  consumer-facing single-file artifact (~82 KB, 2424 lines). Stdlib is
  not bundled; the consumer supplies its own `[deps] stdlib` surface, the
  same shape as `dist/patra.cyr` / `dist/sigil.cyr`. This gives ark a
  one-file consumption path in place of the 14 individually-listed
  `src/*.cyr` modules it pins today (consumer migration tracked as 1.2.3).
- **`modules` kept under `[lib]`, never `[build]`** — documented inline.
  5.7.x+ treats `[build].modules` as an auto-prepend list that
  re-injects every src file ahead of the entry and inflates the binary
  (~38%); nous removed `[build] modules` at 1.1.2 for the same reason.

### Fixed

- **Worked around a Cyrius 6.0.1 codegen bug** exposed by the stdlib bump: a
  typed `vec_get(): i64` nested as a call argument (`str_from(vec_get(…))`,
  `map_get(m, vec_get(…))`) or re-evaluated for the same index within a scope
  returns a **wrong value**. It produced silently-incorrect resolution — a
  cyclic graph read as acyclic (→ SIGSEGV via `vec_len(0)`), topological sort
  emitting a 1-element order, and `resolver_resolve_all` resolving only the seed
  package (dependency list lost in `mpkg_to_resolved`). Fixed by binding the
  accessor to a local first in `src/graph.cyr` (`dep_graph_detect_cycle`,
  `dep_graph_topo_sort`, `resolver_resolve_all`), `src/recipe.cyr`
  (`recipe_db_load`), `src/resolver.cyr` (`mpkg_to_resolved`), and
  `src/sysdb.cyr` (`sysdb_search`, `sysdb_get_installed`, `sysdb_list`,
  `sysdb_info`). The sysdb sites are apt/dpkg-gated and so manifested **only in
  CI** (Ubuntu): `integration_apt` skips on the dev box (Arch, no dpkg), so a
  green local 271/0 still failed CI until those were de-nested. The CI test step
  now always prints per-test output (it previously died at `set -e` before
  echoing, masking the failure as a bare "exit code 2"). The suite is
  **271 passed, 0 failed** on 6.0.1. Full reproduction, bisection proof
  (only `lib/vec.cyr`'s added `: i64` annotations trigger it; bodies identical),
  and the remaining same-pattern sites scheduled for the 1.2.1 hardening sweep
  are documented in
  `docs/development/issues/0001-cyrius-6.0.1-vec-get-recompute.md`. The Cyrius
  compiler itself is **not** modified — the bug is reported upstream and treated
  as an external dependency.

- **sysdb now resolves absolute tool paths before exec.** `exec_capture`'s child
  `execve`s `argv[0]` directly with an empty envp, so it never searched `PATH` —
  a bare `"apt-cache"`/`"dpkg-query"` silently failed to launch, so
  `sysdb_search` returned nothing and `sysdb_is_installed` returned 0. This was
  **pre-existing** (the `exec_capture` envp/execve behaviour is identical on
  5.7.29 and 6.0.1) and only surfaced in CI: `integration_apt` skips on the dev
  box (Arch, no dpkg) but runs on CI's Ubuntu, where it failed "search found
  results" + "coreutils installed". Fixed with `which_path`/`tool_path` in
  `src/command.cyr` (resolve via `getenv("PATH")` in the parent) — the array
  exec (no shell, no injection) is preserved, only `argv[0]` becomes absolute.
  See `docs/development/issues/0002-sysdb-exec-bare-name-path.md`.

### CI / Release

- **Hardened toolchain install** (`ci.yml` build + integration jobs,
  `release.yml`) to the patra pattern: pre-flight the release asset URL
  (HTTP status check) and verify the download is a real gzip tarball
  before extraction — surfaces a misconfigured `cyrius =` pin as a clear
  error instead of tar's misleading "not in gzip format". Toolchain now
  installs into a versioned `$HOME/.cyrius/versions/<v>/{bin,lib}` layout
  with `bin`/`lib` symlinks + a `current` marker (multi-version safe).
- **Stdlib via symlink in CI** — new `ln -sf "$HOME/.cyrius/lib" lib`
  step replaces the deleted committed `lib/`.
- **distlib sync gate** (`ci.yml`) — regenerates the bundle and
  `git diff --exit-code dist/nous.cyr` fails the build if the committed
  bundle is stale.
- **Release ships the bundle** — `nous-<tag>.cyr` (the `dist/` bundle) is
  now a release asset alongside the x86_64 binary, source tarball, and
  `SHA256SUMS`. Mirrors patra shipping `patra-<tag>.cyr`.
- **Dropped the `strip` of the release binary.** A cyrius raw ELF is not a
  standard toolchain artifact — `strip` left the byte size unchanged but
  corrupted it so it SIGSEGV'd at runtime (the release smoke step caught it;
  it would otherwise have shipped a segfaulting `nous-<tag>-x86_64-linux`).
  DCE is the only size reduction now, matching the patra/sigil release flow.

## [1.1.2] - 2026-04-28

### Changed

- **Cyrius toolchain bump**: 5.1.7 → 5.7.29. Source required no edits;
  271 tests still pass, 18 benchmarks still green. Verified locally
  against cyrius 5.7.28 (one patch behind 5.7.29 release).
- **Vendored `lib/*.cyr` re-synced** from cyrius 5.7.29 stdlib for the
  17 modules nous declares in `[deps] stdlib`: `alloc`, `args`,
  `assert`, `bench`, `fmt`, `fnptr`, `fs`, `hashmap`, `io`, `json`,
  `process`, `str`, `string`, `syscalls`, `tagged`, `toml`, `vec`.
- **`cyrius.cyml` modernized** to match the argonaut/daimon pattern
  picked up across AGNOS:
  - `version = "${file:VERSION}"` template instead of a hardcoded
    string. Eliminates the 1.1.0 / 1.1.1 drift that existed between
    the manifest and the `VERSION` file.
  - `repository = "https://github.com/MacCracken/nous"` field added.
  - `[build] modules = [...]` removed — nous already barrel-includes
    every module through `src/nous.cyr`; carrying the same list under
    `[build]` re-injects them ahead of `main.cyr` and inflates the
    binary by ~38% (yukti's deliberation, captured in their cyml
    comment).
  - New `[release]` section (`bins = ["nous"]`) — declarative binary
    inventory for future `cyrius package` / ark tooling.
- **CI workflow rewrite** (`.github/workflows/ci.yml`) following
  argonaut's modernization template:
  - Toolchain version is grepped from `cyrius.cyml` (the now-canonical
    pin) rather than the deprecated `.cyrius-toolchain` file.
  - `CYRIUS_HOME` exported to `$GITHUB_ENV` so subsequent steps see it.
  - New steps: `cyrius deps` (no-op today, future-proof), `cyrius fmt
    --check` drift detection, `cyrius vet` include audit, ELF-magic
    sanity check after build.
  - Lint and fmt steps auto-discover `src/*.cyr` and `tests/*.{tcyr,
    bcyr,fcyr}` so adding a module doesn't require editing CI.
  - Test step asserts on the `0 failed` line rather than a hardcoded
    assertion count (was `271 assertions`, now self-reports). Uses
    `grep -a` so the leading non-printable byte in `cyrius test`'s
    summary doesn't trip GNU grep's binary-file heuristic and silently
    drop the matching line on the runner (caught in the first CI run
    of this branch — green tests, red step).
  - Bench is non-fatal; output uploaded as artifact.
  - Security scan added (raw `system()` / `exec_str` patterns rejected).
- **Release workflow rewrite** (`.github/workflows/release.yml`):
  - Accepts both `vX.Y.Z` and `X.Y.Z` tag styles.
  - Source tarball produced with `git archive` (was missing).
  - `SHA256SUMS` covers source + binary in a single file.
  - Changelog body extracted from this file for the GitHub release
    notes (was `generate_release_notes: true` boilerplate).
  - Pre-release flag auto-set when version starts with `0.`.
- **`scripts/bench-history.sh`** rewritten — was still calling
  `cargo bench` from the pre-port Rust era. Now invokes
  `cyrius bench tests/nous.bcyr`, appends one CSV row per bench
  per labelled run, supports a `--save` snapshot mode.
- **`src/main.cyr` banner reads VERSION at runtime** instead of
  hardcoding `"nous 1.1.0"`. Mirrors daimon's idiom
  (`file_read_all("VERSION", ver_buf, 15)` from `lib/io.cyr`, which is
  already in nous's `[deps] stdlib`); strips a trailing newline,
  null-terminates, and falls back to a literal only when the VERSION
  file isn't accessible from cwd. Verified end-to-end: cwd-with-VERSION
  reads correctly, cwd-without falls back, a `VERSION` bump is picked
  up at next run with no rebuild required.

### Removed

- **`.cyrius-toolchain` file** — deprecated by cyrius v5.5.41 in
  favour of the `cyrius` field in `cyrius.cyml`. CI/release workflows
  read from the manifest now.

### Notes

- `CLAUDE.md` retains some Rust-era wording ("flat library crate").
  Updated the cleanliness-check command list to its Cyrius equivalents
  in this pass; broader rewrite deferred.

## [1.1.1] - 2026-04-16

### Removed

- Dead code cleanup — 7 unused functions removed:
  - `recipe_new` — replaced by `recipe_alloc` + `recipe_set_pkg/meta/deps` pattern
  - `resolver_make_with_recipes` — superseded by `resolver_make` (already allocates rdb slot)
  - `trace_add` — `trace_msg` used instead
  - `resolver_detect_source` — thin wrapper; callers use `detect_source` directly
  - `resolver_is_sys` — unused by any consumer path
  - `resolver_strategy` — accessor; callers use `rs_strat` directly
  - `dg_versions` — graph versions field defined but never read

### Changed

- Zero dead functions in library — every defined function is referenced

## [1.1.0] - 2026-04-16

### Changed

- **Module split** — `src/nous.cyr` (2,444 lines) split into 14 focused modules matching ark's architecture pattern:
  - `types.cyr` (233) — enums, constructors, accessors
  - `util.cyr` (132) — filesystem, string, path helpers
  - `error.cyr` (110) — error constructors, display, name validation
  - `strategy.cyr` (16) — resolution strategy constructors
  - `source.cyr` (139) — source display, detection, typo suggestions
  - `command.cyr` (33) — shell execution, PATH scanning
  - `sort.cyr` (56) — insertion sort by name
  - `registry.cyr` (158) — marketplace registry
  - `sysdb.cyr` (184) — apt/dpkg wrapper
  - `resolver.cyr` (157) — main resolver engine
  - `json.cyr` (253) — JSON serialization/deserialization
  - `version.cyr` (176) — SemVer, constraint matching
  - `graph.cyr` (264) — dependency graph, cycle detection, topo sort
  - `recipe.cyr` (545) — CYML parser, recipe DB, recipe-based resolution
- `nous.cyr` is now a barrel file that includes all modules in order
- `cyrius.cyml` version bumped to 1.1.0, modules list added for consumer dependency declarations
- No API changes — all function names and signatures are identical

## [1.0.2] - 2026-04-16

### Fixed

- `cyml_parse` infinite loop guard — force `pos` advance if key loop produces zero-length key, guaranteeing termination on any input (flagged by external audit)

## [1.0.1] - 2026-04-16

### Removed

- `rust-old/` directory (934MB, mostly target/ artifacts). Original Rust source is preserved in git history. Benchmark comparison captured in `benchmarks-rust-v-cyrius.md`.

### Changed

- Cleaned `.gitignore` — removed `/rust-old/target/` entry
- Updated architecture overview and CHANGELOG references

## [1.0.0] - 2026-04-16

### Added

- **P5: Error Quality** — complete
  - Conflict explanation engine: detailed error messages identify which packages impose conflicting constraints ("A requires foo>=2.0 conflicts with B requires foo<1.0")
  - Typo suggestion engine: Levenshtein distance matching against known packages ("Package 'ngins' not found. Did you mean 'nginx'?")
  - Resolution trace: `resolver_with_trace(r)` enables step-by-step logging of resolution decisions ("marketplace: foo ... hit", "system: bar ... miss"). Trace accessible via `resolver_get_trace(r)`.

- **P(-1) Scaffold Hardening**
  - `cyrius lint` clean on all source files (0 warnings)
  - Shared `finalize_graph()` function — deduplicated cycle/conflict/topo logic between `resolver_resolve_all` and `resolver_resolve_all_with_recipes`

- **Security Audit** — internal + external research
  - Fixed P0: command injection in sysdb_* functions (replaced shell strings with array-based exec)
  - Fixed P0: NULL dereference in accessor chains (added NULL checks at every level)
  - Fixed P1: path traversal in scan_installed (reject ".." and "/" in entries)
  - Fixed P1: integer overflow in semver_parse (MAX_SAFE bound check)
  - Fixed P2: TOCTOU in scan_installed (removed file_exists before read)
  - External research: 8 attack categories analyzed (dependency confusion, supply chain, etc.)
  - Full audit report: `docs/audit/2026-04-16.md`

- **CI/CD** — complete GitHub Actions pipeline
  - `build-and-test` job: lint all source, build, smoke check, 271 tests, fuzz
  - `bench` job: run 18 benchmarks, check regression thresholds (1ms micro / 30ms db load)
  - `integration` job: test suite on ubuntu-latest with real apt/dpkg
  - `docs` job: verify all required documentation exists
  - Release workflow: CI gate, version verification, DCE build, tar+sha256 packaging
  - `docs/api.md` — complete API reference with code examples for all consumer functions

### Changed

- All sysdb_* functions now use array-based `exec_capture()` instead of shell string concatenation
- `finalize_graph()` extracts shared logic from both resolve_all variants
- 271 tests (75 test groups), 18 benchmarks, 3 fuzz harnesses
  - New tests: registry_install_stub, constraints_compatible, error_to_kind, semver_eq, caret_zero, tilde_zero, empty_graph, single_node_graph, semver_overflow, json_str_vec, integration_apt, cyml_parse_sections
  - New benchmarks: levenshtein, constraint, cycle_detect_20, topo_sort_20, recipe_db_load
  - `docs/api.md` — complete API reference with Cyrius code examples for all consumer functions

## [0.3.0] - 2026-04-16

### Added

- **P2: Zugot Recipe Integration** — parse and resolve from .cyml recipes
  - CYML parser (`cyml_parse`, `cyml_parse_file`) — handles `[section]` headers (stdlib toml.cyr only handles `[[arrays]]`)
  - Recipe struct with 12 fields: name, version, description, license, arch, groups, release, url, sha256, patches, runtime_deps, build_deps
  - Recipe parser (`recipe_parse_file`) — reads .cyml files, extracts all sections
  - Recipe database (`recipe_db_load`, `recipe_db_get`, `recipe_db_count`) — scans zugot directory tree (11 categories), indexes by name
  - Build-order reader (`read_build_order`) — parses build-order.txt for tier-sorted install order
  - `resolver_with_recipes(r, rdb)` — attach recipe DB to resolver
  - `resolver_resolve_all_with_recipes(r, names)` — transitive resolution with recipe fallback
  - `recipe_to_resolved(recipe)` — convert Recipe to ResolvedPkg for graph integration
  - `parse_toml_array(s)` — parse TOML inline arrays `["a", "b"]` to vec
  - Tests against real zugot recipes (curl, ark), synthetic dep chains
  - `recipe_parse` benchmark (19us avg per recipe)
  - 218 tests total, 13 benchmarks

- **P1: Dependency Resolution** — full transitive dependency graph resolution
  - SemVer parsing (`semver_parse`, `semver_cmp`, `semver_to_str`) — supports `major.minor.patch`
  - Version constraint parsing (`constraint_parse`) — supports `>=`, `>`, `<=`, `<`, `=`, `^` (caret), `~` (tilde), `*` (any)
  - Constraint matching (`constraint_matches`) — evaluates whether a version satisfies a constraint
  - Dependency graph (`dep_graph_new`, `dep_graph_add`) — tracks packages, edges, and version constraints
  - Cycle detection (`dep_graph_detect_cycle`) — DFS with 3-color marking, returns cycle path
  - Topological sort (`dep_graph_topo_sort`) — Kahn's algorithm for install order
  - Conflict detection — checks constraint compatibility for diamond dependencies
  - `resolver_resolve_all(r, names)` — resolves a list of packages transitively, returns `ResPlan` with ordered package list
  - 12 new test groups (52 new assertions): semver_parse, semver_cmp, semver_to_str, constraint_parse, constraint_matches, dep_graph_basic, topo_sort, cycle_detection, diamond_deps, resolve_all, resolve_all_cycle
  - `graph_resolve` benchmark (7us avg for single-package transitive resolution)
  - 192 tests total, 12 benchmarks

- **Cyrius port** — full port from Rust to Cyrius 5.1.7
  - 2143 lines Rust → 2174 lines Cyrius (src/nous.cyr + src/main.cyr + tests + benchmarks + fuzz)
  - Manual struct layout: 12 struct types with alloc/store64 constructors, load64 accessors
  - JSON serialization/deserialization for all public types (resolved_to_json, installed_to_json, update_to_json, strategy_to_json, manifest_to_json, error_kind_to_json, agent_info_to_json, search_result_to_json + from_json variants)
  - Manifest dependency parsing (json_extract_deps parses "dependencies":{} objects)
  - registry_reload() — public cache refresh for marketplace registry
  - registry_install_package() — stub for mela interface parity
  - nous_error_to_kind() — serializable error companion
  - Complete error display for all 7 NousError variants (was 3, now 7)
  - source_from_json() — deserialize PackageSource from JSON
  - our_is_dir() — workaround for broken fs.cyr is_dir (direct stat syscall)
  - 140 tests across 40 test groups (up from 87/26)
  - 11 benchmarks including 4 strategy comparisons and serde roundtrip (up from 6)
  - 3 fuzz harnesses (validate_name, detect_source, json_extract)
  - 115KB standalone x86_64 ELF binary

### Changed

- **Breaking**: Language changed from Rust to Cyrius 5.1.7. All APIs renamed to snake_case function style (e.g., `NousResolver::resolve()` → `resolver_resolve(r, name)`).
- **Breaking**: Serde derives replaced with manual JSON serialization functions.
- **Breaking**: Struct field access via accessor functions (e.g., `rp_name(pkg)` instead of `pkg.name`).
- Rust source removed (preserved in git history and `benchmarks-rust-v-cyrius.md`).

### Removed

- Rust dependencies (thiserror, serde, serde_json, tracing, chrono)
- Cargo.toml, Cargo.lock, rust-toolchain.toml, deny.toml, codecov.yml
- Criterion benchmarks (replaced with Cyrius bench harness)

### Previous (Rust era)

- P(-1) scaffold hardening pass (round 3)
  - `LocalRegistry` cached inside `NousResolver` at construction time — eliminates per-call filesystem scans. Benchmarks: resolve_marketplace_hit 687µs→163ns (~4200x), search_100 1.41ms→25µs (~56x), list_installed_100 1.40ms→11µs (~127x)
  - `NousResolver::new()` now returns `Result` — propagates registry errors instead of silently returning empty results
  - `Serialize`/`Deserialize` on `NousResolver` and `SystemPackageDb` — serializes configuration, reconstructs live state on deserialize
  - `Debug` derive on `NousResolver`, `SystemPackageDb`, `LocalRegistry`
  - `LocalRegistry::base_dir()` accessor, `LocalRegistry::reload()` for cache refresh
  - `LocalRegistry::search()` and `list_installed()` now return borrowed data (no cloning)
  - 3 additional tests (56 total): registry error propagation (1), serde roundtrips for NousResolver and SystemPackageDb (2)
- P(-1) scaffold hardening pass (round 2)
  - Package name validation (`InvalidPackageName` error) — rejects empty, whitespace, and special characters
  - Deterministic ordering for `list_installed()` and `search()` results (sorted by name)
  - Replaced `which` shell-out with native PATH scanning in `which_exists()`
  - `dir_size()` → `dir_size_recursive()` — now counts nested directory contents
  - `installed_at` now uses manifest file mtime instead of `Utc::now()`
  - Fixed `bench-history.sh` — `--save` now works with current criterion version
  - 7 additional tests (53 total): name validation (4), deterministic ordering (1), InvalidPackageName error display + serde (2)
  - README: fixed stale `anyhow` → `thiserror` in dependency table
  - Updated architecture/overview.md, roadmap.md, gaps.md to reflect current state
- P(-1) scaffold hardening pass (round 1)
  - `#[non_exhaustive]` on all public enums (`PackageSource`, `ResolutionStrategy`)
  - `#[must_use]` on all pure functions and constructors
  - `PartialEq` + `Eq` derives on all public data types (`ResolvedPackage`, `InstalledPackage`, `AvailableUpdate`, `UnifiedSearchResult`, registry stub types)
  - Criterion benchmark suite (`benches/resolver.rs`) covering detect_source, resolve, search, list_installed, strategy variants, and serde roundtrip
  - `scripts/bench-history.sh` for benchmark archival
  - 9 additional tests (40 total, up from 31): serde roundtrip for `PackageSource`, `ResolutionStrategy`, `UnifiedSearchResult`, `MarketplaceManifest`; community variant coverage; dedup verification; flutter runtime guard; multi-package listing
  - CHANGELOG.md, CONTRIBUTING.md, SECURITY.md, CODE_OF_CONDUCT.md
  - `docs/architecture/overview.md`
  - `docs/development/roadmap.md`
- Replaced `anyhow` with `thiserror` for typed error handling (P5 partial)
  - `NousError` enum with variants: `CommandExec`, `RegistryIo`, `InvalidManifest`, `InvalidVersionConstraint`, `DependencyCycle`, `VersionConflict`
  - `NousErrorKind` — serializable error snapshot for logging and wire transport
  - `error::Result<T>` type alias replaces `anyhow::Result<T>` in all public API
  - 6 error-specific tests (46 total): Display formatting, serde roundtrip on `NousErrorKind`, error-to-kind conversion

### Changed

- **Breaking**: `NousResolver::new()` now returns `Result<Self>` instead of `Self`. The marketplace directory must exist.
- **Breaking**: `LocalRegistry::search()` returns `Vec<&InstalledMarketplacePackage>` (borrowed) instead of `Vec<InstalledMarketplacePackage>` (owned).
- **Breaking**: `LocalRegistry::list_installed()` returns `&[InstalledMarketplacePackage]` (borrowed slice) instead of `Vec<InstalledMarketplacePackage>` (owned).
- **Breaking**: All public `Result` types are now `Result<T, NousError>` instead of `anyhow::Result<T>`. Consumers can now match on specific error variants.

### Removed

- `anyhow` dependency — no longer needed

### Fixed

- `cargo fmt` formatting violations in test assertions
- 3 clippy `collapsible_if` warnings (registry_stub.rs, lib.rs)
- Dead code warning on `LocalRegistry::install_package` stub

## [0.1.0] - 2026-04-04

### Added

- Initial release
- `NousResolver` — multi-source package resolver with configurable strategies
- `SystemPackageDb` — apt/dpkg wrapper for system package queries
- `LocalRegistry` — filesystem-based marketplace package registry (stub)
- Resolution strategies: `MarketplaceFirst`, `SystemFirst`, `OnlySource`, `SearchAll`
- Heuristic source detection from package name format
- Cross-source unified search with marketplace-priority dedup
- Installed package listing across all sources
- System update checking via `apt list --upgradable`
- Full serde support on all public types
- Structured logging via `tracing`
- 31 unit tests with serde roundtrip coverage

[Unreleased]: https://github.com/MacCracken/nous/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/MacCracken/nous/releases/tag/v0.1.0
