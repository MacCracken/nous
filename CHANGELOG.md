# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.7.3] - 2026-11-12

**Lockfile integrity.** A lock now pins the bytes as well as the version.
When a source publishes a hash, the lock records it and locked resolution
checks it, so "same version, other bytes" is drift. The maintainer chose
where marketplace hashes come from; the rest are defaults. All of it is in
[ADR 0009](docs/adr/0009-lockfile-checksums.md).

| Source | Checksum |
|---|---|
| system (apt) | the candidate stanza's `SHA256`, the `.deb`'s digest |
| community (recipe) | the recipe's `sha256`, its source archive's digest; none when empty (43 of 563 zugot recipes) |
| marketplace, flutter-app | none until the 1.8.x native index (maintainer decision) |

### Added

- **`ResolvedPkg` gets an artifact checksum:**
  - `nous_rp_checksum(p)` reads it: a `"sha256:<hex>"` Str, or 0 for none;
  - `nous_rp_set_checksum(p, c)` sets it;
  - the struct grows from 56 to 64 bytes. `resolved_pkg_new` keeps its
    seven arguments and starts the field at 0.
  - **Filled in by:** `sysdb_info` / `sysdb_info_many` from apt's `SHA256`
    field, and `recipe_to_resolved` from the recipe's `sha256`.
  - Its JSON gains `"checksum"` (a string, or `null`).
- **A `checksum` key in the lockfile**, written between `source` and
  `dependencies` and only when there is one, as Cargo.lock does. The format
  version stays 1, so a 1.7.2 reader ignores the line.
  - `nous_lp_checksum(p)` reads it.
  - `nous_lock_pkg_new_full(name, version, source, deps, checksum)` builds a
    package with one. `nous_lock_pkg_new` keeps its four arguments and locks
    none.
  - The lock JSON carries it too.
- **`NOUS_LOCK_CHECKSUM_CHANGED` (7)**, a new `LockDiffKind`. Its text is
  `curl: checksum sha256:… -> sha256:…`, or `-> none` when the hash is gone.
  A consumer that matches `LockDiffKind` exhaustively must handle it.
- **ADR 0009**, "Artifact checksums in the lockfile".

### Checksum semantics (ADR 0009)

- **The form is `sha256:` and 64 lowercase hex digits**, the OCI image spec's
  digest grammar.
  - A published digest is trimmed and lowercased.
  - Anything that is not 64 hex digits gives no checksum. That includes an
    empty recipe `sha256`, and an apt stanza that only dpkg's status file
    knows.
- **Parsing:** `checksum` is optional. When present it must be in exactly
  that form, or `NOUS_ERR_INVALID_LOCKFILE` names the package. Locking a plan
  refuses a malformed one too.
- **Drift** is reported when the old lock has a checksum, the version and
  source are unchanged, and the new checksum differs or is missing.
  - A new version or source is reported on its own, without a second entry
    for the checksum.
  - **A checksum the old lock lacks is not drift**, so every 1.7.2 lock keeps
    passing `nous_resolve_locked`. Regenerate it to pin the bytes. Cargo
    refuses this case, but here that would break 1.7.2 locks in a minor
    release.
  - So `nous_lock_diff(was, now)` is one-sided. Compare both ways for
    equality.
  - **apt checksums need apt's package lists.** Without them, `apt-cache
    show` has no `SHA256`, and a locked apt package reads `-> none`, which
    is drift.
- **nous still hashes nothing.** apt, takumi and ark check the downloaded
  bytes; nous pins the hash the source published.

### Performance

Measured as medians of 5 alternating rounds against the 1.7.2 build, in
two separate runs. In the same runs, benches whose code did not change moved
by up to ±10%, so read small numbers as noise.

| Bench | 1.7.2 | 1.7.3 | Change | Why |
|---|---|---|---|---|
| `apt_parse_info` (new; a 17-field curl stanza) | 8.04 µs | 8.61 µs | +7% | reading, checking and lowercasing the digest (7-run A/B) |
| `serde_roundtrip` | 6.69 µs | 7.27 µs | +9% to +11% | the new `"checksum"` JSON field |
| `lock_write_parse_100` | 406 µs | 413 µs | +2% to +4% | one more key lookup per package |
| `lock_diff_100` | 13.3 µs | 13.5 µs | +1% | the checksum compare |
| `resolve_mkt_*`, `strat_*` | | | +1% to +3% | the 64-byte ResolvedPkg |
| `lock_checksum_rw_100` (new) | | 549 µs | | a lock with a checksum on all 100 packages |

- **Two first drafts were too slow, and were fixed before release:**
  - **The digest:** built through a string builder, it cost more than the
    rest of the stanza's parse (+27%). It is now filled into a 71-byte
    buffer in place.
  - **The diff:** calling two accessors for every package cost +5.6%. It
    now reads the field in place, and only when the old lock has a
    checksum.
- **In context:** apt spends about 430 ms per `apt-cache` call, so +0.6 µs
  per apt package does not show in a real resolution.
- **Release binary:** 46,504 → 46,696 B.

### Verification

- **Tests:** 786 passed, 0 failed (+68 assertions). They cover:
  - the digest helper: case, trimming, length and non-hex input;
  - apt stanzas:
    - SHA256 taken, not MD5sum;
    - only the first stanza's checksum is used;
    - a stanza dpkg's status file alone knows, or a malformed digest, gives
      none;
    - batch parsing;
  - the ResolvedPkg JSON roundtrip, with `null` for none;
  - the exact lockfile text, with a `checksum` line only where there is one;
  - roundtrips through text and through JSON;
  - 7 parse cases;
  - refusing an unlockable checksum;
  - every diff rule: a changed, gone or newly gained checksum, a changed
    version, a changed source;
  - with the recipe DB:
    - a recipe whose `sha256` changes or is removed under the same version
      is drift;
    - a lock without checksums (1.7.2) still passes.
- **Mutation checks:** 27 targeted breaks. Each one fails the suite on a
  named assertion: none of them gets through as a crash. Five also trip
  `fuzz_lock` (invariants 54, 55, 57 and 58), and one trips `fuzz_apt`
  (invariant 46).
- **Fuzzing:**
  - `fuzz_lock` gives its random locks random checksums, and its seeds
    include a checksum line;
  - its equality invariants now check both directions;
  - new invariant 58: every parsed checksum is in the lockfile form;
  - `fuzz_apt` seeds an uppercase SHA256, with new invariant 46: a checksum
    taken from apt output is none or in the lockfile form;
  - all 6 harnesses pass under `--poison=ab`.
- **Gates:** fmt, lint, vet, deny, the doc gate and the Result-annotation
  check are clean, and `dist/nous.cyr` is regenerated (v1.7.3).
- **Against real apt output:** stanzas captured from debian:12 were run
  through nous's parser.
  - After `apt-get update`, curl, libc6, bash and zlib1g each got exactly
    their stanza's `SHA256`.
  - With no apt lists, bash and coreutils (known only to dpkg's status
    file) got none.
  - So a lock made with apt lists fails locked mode on a host without them:
    run `apt-get update` first (ADR 0009, Consequences).

### Changed

- **Doc fix in `src/types.cyr`:** the Constraint layout said 40 bytes, but it
  has been 56 since 1.7.1.
- **`SECURITY.md`:**
  - only 1.7.x is supported (it listed 0.1.x);
  - the security practices described the Rust port (`cargo audit`,
    `std::process::Command`), and now describe what nous does today.

## [1.7.2] - 2026-11-12

**The lockfile.** A resolution can now be pinned and checked later: the same
graph, or a clear refusal that names what moved. This is nous's half of
milestone M2 on the ark v2 path. The four design points were the
maintainer's decisions, recorded in
[ADR 0008](docs/adr/0008-lockfile.md):

- **CYML format** in the Cargo.lock shape;
- **nous does text in and text out**, and ark owns the file;
- **drift is a strict error**;
- **apt packages are locked and enforced.**

### Added

- **`src/lock.cyr`**, the 16th module (in `[lib] modules` and the barrel):

| Function | Does |
|---|---|
| `nous_lock_from_plan(plan, roots)` | `Ok(NousLock)`: every node (name, version, source, edges to other nodes) and the roots |
| `nous_lock_to_str(lock)` | the lockfile text: byte-stable, byte-ordered, no timestamp |
| `nous_lock_parse(text)` | `Ok(NousLock)`, or `Err(NOUS_ERR_INVALID_LOCKFILE)` saying what is wrong |
| `nous_resolve_locked(r, lock)` | resolves the lock's roots; `Ok(ResPlan)` only when the graph is exactly the lock's, else `Err(NOUS_ERR_LOCK_MISMATCH)` |
| `nous_lock_diff(was, now)` | the differences as `NousLockDiff`s, without failing |
| `nous_lock_diff_text(d)` | `"curl: version 8.19.0 -> 8.20.0"` |
| `nous_lock_get(lock, name)` | a locked package, by binary search |
| `nous_lock_to_json` / `_from_json`, `nous_lock_diff_to_json` / `_from_json` | the JSON encoding (CLAUDE.md: every public type both ways) |

- **New types and accessors:**
  - `NousLock` (`nous_lock_version` / `_roots` / `_pkgs`);
  - `NousLockPkg` (`nous_lp_name` / `_version` / `_source` / `_deps`);
  - `NousLockDiff` (`nous_ld_kind` / `_name` / `_old` / `_new`), with the
    `LockDiffKind` enum: `NOUS_LOCK_ADDED`, `_REMOVED`,
    `_VERSION_CHANGED`, `_SOURCE_CHANGED`, `_DEPS_CHANGED`, `_ROOT_ADDED`
    and `_ROOT_REMOVED`.
- **Error kinds `NOUS_ERR_INVALID_LOCKFILE` (9) and `NOUS_ERR_LOCK_MISMATCH`
  (10).** `nous_error_msg` writes each one. A mismatch lists up to 8
  differences, then a count, and ends with "Regenerate the lockfile to accept
  the change."

### Lock semantics (ADR 0008)

- **It pins and verifies; it does not choose.** nous has one candidate per
  name, so a newer version is drift, not a fallback.
- **A source change is drift**, which closes dependency confusion across time.
- **Edges cover locked packages only.** A name apt could not show (a virtual
  package) is not locked.
- **The parser is strict about what it needs:**
  - format version 1;
  - each package's name (unique), version, source and dependency list;
  - every dependency and root a locked package.
- **It is lenient about the rest.** Unknown keys are ignored, so a 1.7.2
  reader takes a 1.7.3 lock with checksums. Any order is accepted and put in
  byte order.
- **A name or version holding `"`, `\` or a byte outside printable ASCII is
  refused** when locked or read. nous's CYML reader decodes no escapes, and
  real names and versions never contain such bytes.

### Performance

- **Existing benches:** within ±3.1% of 1.7.1, from 7 alternating rounds.
  No existing code path changed.
- **New benches:** `lock_write_parse_100` (write and parse a 100-package
  lock) about 420 µs; `lock_diff_100` about 14 µs.
- **Scaling is linear.** A scratch run at 100 and 1000 packages gave:
  - write: 0.11 → 1.07 ms;
  - parse: 0.34 → 3.2 ms, of which the CYML reader is 0.11 → 1.0 ms.
- **Release binary:** 45,392 → 46,504 B.

### Verification

- **Tests:** 718 passed, 0 failed (+67 assertions). They cover:
  - the exact lockfile text, and that it is byte-stable across resolutions;
  - a byte-for-byte text roundtrip, and a JSON roundtrip;
  - locked resolution passing;
  - version drift, with the message and the diff;
  - an added package and a changed edge list, and the reverse removal;
  - source drift from a hand-made lock;
  - a vanished root, which comes through as `NOUS_ERR_NOT_FOUND`;
  - 16 parse cases, strict and lenient, and normalization of an unordered
    lock;
  - an apt-only plan with Debian versions and a virtual dependency;
  - refusing an unlockable value, and a root not in the plan;
  - the recipe DB path (community packages).
- **Mutation checks:** 13 targeted reversions each fail the suite:
  - edges not filtered;
  - packages unsorted;
  - one-line arrays;
  - no check that dependencies are locked;
  - duplicates allowed;
  - an unknown source accepted;
  - any format version accepted;
  - the diff ignoring source;
  - the diff ignoring dependencies;
  - locked mode never failing;
  - locked mode ignoring recipes;
  - quotes allowed;
  - no check that roots are locked.
- **Fuzzing:** a new harness, `tests/fuzz_lock.fcyr`, mutates lockfile text
  and builds random valid locks in memory. Its invariants:
  - reparse, with `to_str` a fixed point;
  - no self-diff, before or after a JSON roundtrip;
  - every dependency and root is locked;
  - a random lock survives `to_str` + `parse`.

  A mis-mapped source and a missing dependency check each fail it (exits 57
  and 56). 6/6 harnesses pass under `--poison=ab`.
- **Gates:** clean (fmt, lint, vet, deny, the per-module doc gate,
  `cyrius header`). `cyrius distlib` is in sync.

## [1.7.1] - 2026-11-12

**Compound version ranges, npm and Cargo forms.** Since 1.5.3 nous refused
them with an error, so a manifest written as `^1 || ^2` or `>=1.2, <1.5`
failed to resolve. The design is in
[ADR 0007](docs/adr/0007-compound-ranges.md).

### Added

- **The forms accepted** (npm "Range Grammar"; Cargo "Multiple version
  requirements"):

| Form | Example | Meaning |
|---|---|---|
| AND, npm | `>=1.2.7 <1.3.0` | every comparator holds (one comparator set) |
| AND, Cargo | `>= 1.2, < 1.5` | the same; an operator may stand apart from its version |
| OR | `1.2.7 \|\| >=1.2.9 <2.0.0` | any comparator set holds; an empty side is `*` |
| Hyphen | `1.2.3 - 2.3` | `>=1.2.3 <=2.3`, which is `[1.2.3, 2.4.0)`; a partial low end fills zeros |

- **`OP_SET` (8):** a new `ConstraintOp` variant for a compound constraint.
  `nous_ct_sets(c)` reads its comparator sets: a vec (OR) of vecs (AND) of
  simple constraints. A text that is one comparator, `>= 1.2` included, is
  the same simple constraint as before.
- **The pre-release rule applies per comparator set**, as npm does.
  `^1.0.0 || >=2.0.0-beta.1 <3.0.0` takes `2.0.0-beta.2`, but not `1.5.0-rc`.
- **Errors for malformed compounds:**
  - a single `|`;
  - a stray comma (`>=1,,<2`, `>=1,`, `, >=1`);
  - an operator with no version;
  - a hyphen range that is not exactly `A - B` with plain versions.

  A refused comparator's own error comes through: `^1 || !=2` reports `'!='`.

### Changed

- **Range sets:**
  - `finalize_graph` intersects each dependency's constraints as range sets
    (unions of ranges);
  - it checks the chosen version against every comparator set;
  - the pre-release hint works for compound constraints too.
- **`constraints_compatible` stays exact.** A release in common settles it.
  Otherwise the pre-releases each side admits, per set and per `M.m.p`,
  must meet.
- **Constraint grows from 40 to 56 bytes** (`ct_sets`, `ct_rset`).

Nothing that parsed before changes meaning. The single-comparator parser runs
first, and only a text it refuses that holds a separator (`|`, `,`, inner
whitespace) goes to the compound parser.

### Performance

Medians of 11 alternating runs of the 1.7.0 and 1.7.1 bench binaries:

| Bench | 1.7.0 | 1.7.1 | Change |
|---|---|---|---|
| `constraint` (simple) | 1085 ns | 1128 ns | +4.0% |
| `constraint_range` (simple) | 556 ns | 583 ns | +4.9% |
| `graph_resolve` | 7606 ns | 7752 ns | +1.9% |
| `constraint_compound` (new: parse 2 sets, 2 matches, compatibility) | — | ~4.4 µs | — |

The other 20 benches are within ±2.5% (9 rounds; `capture_200k` +3.8% and
`deb_version_cmp` −5.1%, code neither touches).

Where the simple-constraint cost comes from:

- **A pre-scan pass, now gone.** The first version scanned every constraint
  for separators before parsing it, and cost +7.4% and +11%. Now the
  one-comparator parse runs first, so a simple constraint is read in one
  pass.
- **The record size: 0.4–2% of what remains.** A 3-way run built 1.7.0 with
  only the allocation changed to 56 bytes. The rest is the routing between
  the simple and compound paths. Overloading fields to stay at 40 bytes was
  not worth it.

The release smoke binary grows from 45,264 to 45,392 B.

### Verification

- **Tests:** 651 passed, 0 failed (+73 assertions), covering:
  - the npm README examples, both AND forms, and OR with an empty side;
  - every hyphen-partial example in the README;
  - representation and range-set merging;
  - per-set pre-releases;
  - all the new errors;
  - compatibility, including pre-release-only overlaps;
  - resolution: a pass, a refused chosen version, and disjoint range sets
    naming both requirers, plus the pre-release hint for a compound.
- **Mutation checks:** 12 targeted reversions each fail the suite:
  - no per-set pre-release rule;
  - only the first set matched;
  - no pre-release band in compatibility;
  - no release fast path;
  - no merging;
  - touching ranges not merged;
  - range-set intersection ignoring one side;
  - an exclusive hyphen top;
  - stray commas allowed;
  - no operator join;
  - no opt-in check in the band;
  - `finalize_graph` skipping the intersection.
- **Fuzzing:** `fuzz_version` now generates compound constraints (sets of 1–2
  comparators, either AND style, `||`, hyphen ranges). New invariants:
  - `A || B` matches exactly when A or B does, and errs when either does;
  - `A B` matches a release exactly when both do;
  - `A - B` matches exactly what `>=A <=B` does.

  The first-set-only, no-pre-rule, merge-drop and exclusive-hyphen mutations
  each fail it (exits 43, 43, 36, 45). 5/5 harnesses pass under
  `--poison=ab`.
- **Gates:** clean (fmt, lint, vet, deny, the per-module doc gate,
  `cyrius header`). `cyrius distlib` is in sync.

## [1.7.0] - 2026-11-12

**SemVer pre-releases and build metadata, done to spec.** nous dropped
everything after the patch number, so `1.0.0-rc.1` was `1.0.0`. mela accepts
pre-release versions for marketplace packages, so this was live. The
semantics are recorded in
[ADR 0006](docs/adr/0006-semver-prerelease-and-build.md).

### ⚠ Behaviour changes — read before moving ark to 1.7.0

| Change | Before | Now |
|---|---|---|
| Ordering | `1.0.0-rc.1` == `1.0.0`; `rc.1` == `rc.2` | SemVer 2.0.0 §11 precedence: `1.0.0-alpha < 1.0.0-alpha.1 < … < 1.0.0-rc.1 < 1.0.0`. Build metadata is still ignored (§10) |
| Matching a pre-release | `>=1.0.0` and `*` matched `1.1.0-rc.1` (read as `1.1.0`) | npm / Cargo rule: a pre-release matches only a constraint that names a pre-release of the same M.m.p. `>=1.1.0-rc.1` matches `1.1.0-rc.2`; `>=1.0.0` and `*` do not |
| Resolution | a marketplace / Flutter dependency at a pre-release passed `>=1.0.0` | `NOUS_ERR_VERSION_CONFLICT`, and the message says it is a pre-release and how to opt in (for example `>=1.1.0-rc.1`) |
| Pre-release constraints | `1.0.0-beta` refused with an error | accepted on a full M.m.p, strictly per §9. Build metadata is accepted and ignored |
| `constraints_compatible` | range intersection only | exact: an overlap holding only pre-releases needs both sides to opt in, so `>1.0.0` with `<1.0.1` is a conflict |
| Updates | `1.0.0` was not offered over `1.0.0-rc.1` | offered. Flutter also orders build numbers (`1.0.0+2` over `1.0.0+1`, pub_semver); marketplace ignores build |

apt and community versions are unaffected: they keep Debian ordering.

### Added

- **SemVer record:** grows from 24 to 40 bytes, with `nous_sv_pre(v)` and
  `nous_sv_build(v)` (a Str each, or 0). There is a new constructor,
  `nous_semver_new_full(major, minor, patch, pre, build)`. `semver_new` is
  unchanged.
- **`nous_semver_cmp_pub(a, b)`:** pub_semver order, which is precedence, then
  build (no build first, parts compared like pre-release identifiers).
- **`semver_to_str`:** writes `-pre` and `+build`.
- **Recipe fields:** `nous_rc_github_release(r)` and `nous_rc_release_asset(r)`
  (a Str each, `""` when absent), with `nous_recipe_set_github`. Recipe grows
  from 96 to 112 bytes. This was the roadmap item "`github_release` URL
  resolution". Turning those fields into an asset URL needs the GitHub API,
  and nous does no network access, so takumi resolves it at fetch time
  (`takumi/src/fetch.cyr`). Before, the only trace of these fields was
  `rc_url`'s `…/releases` page, which is unchanged.

### Changed

- **`semver_parse`:** keeps the run of `[0-9A-Za-z.-]` after `-` and after
  `+`, also after a partial (`2.0-beta`). It is still lenient: an empty part is
  dropped, and other trailing text is still ignored. Constraint parsing is
  strict, and new errors name what is wrong: a pre-release on a partial, an
  empty identifier, a leading zero, a character outside `[0-9A-Za-z-]`.
- **Constraints on apt and community packages:** a SemVer pre-release bound is
  compared the Debian way, as `~pre`.
- **Roadmap 1.7.x** is now one bite per release:
  - 1.7.1: compound ranges;
  - 1.7.2: the lockfile;
  - 1.7.3: lockfile integrity.
- **Two items move** to where their prerequisite lives:
  - recipe-level constraint solving is gated on a zugot syntax, because zugot
    deps carry names only (0 constrained entries);
  - PubGrub-style version selection is gated on a source that offers more than
    one version per name (the native index or mela). No source does today.

### Performance

Medians of 11 alternating runs of the 1.6.1 and 1.7.0 bench binaries:

| Bench | 1.6.1 | 1.7.0 | Change |
|---|---|---|---|
| `constraint` (2 parses, 4 versions, 4 matches) | 1162 ns | 1213 ns | +4.4% |
| `constraint_range` (2 parses + compatibility) | 596 ns | 622 ns | +4.4% |
| `graph_resolve` | 8446 ns | 8343 ns | −1.2% (noise) |
| `manifest_parse` | 5710 ns | 5687 ns | −0.4% (noise) |

The rest of the 23 benches are within ±2% (7 rounds). The first working
version cost +15% on `constraint` and +10% on `constraint_range`. Three
changes brought both down to +4.4%:

- the version parser's core is unrolled again, and the record is allocated once;
- the hot paths read fields in place, because Cyrius does not inline calls;
- the compatibility check no longer allocates.

What remains is the larger record and the pre-release checks.

The release smoke binary grows from 40,936 to 45,264 B: its `update_is_newer`
checks now reach the pre-release code.

### Verification

- **Tests:** 578 passed, 0 failed, with 75 new assertions:
  - the §11 example chain both ways, plus numeric, alphanumeric and
    any-length identifiers;
  - pub build order;
  - the npm README pre-release examples;
  - §9 / §10 syntax errors;
  - exact compatibility;
  - a pre-release refused in resolution with the opt-in hint, and accepted
    when opted in;
  - rc and Flutter update cases;
  - the recipe fields.
- **Mutation checks:** 11 targeted reversions each fail the suite:
  - precedence ignores pre-releases;
  - no opt-in rule;
  - compatibility by range only;
  - numeric identifiers sorted after alphanumeric ones;
  - the longer identifier list sorted first;
  - numeric identifiers compared as text;
  - pub ignores build;
  - no Flutter branch;
  - `finalize_graph` checks by range only;
  - no leading-zero check;
  - the recipe fields unset.
- **Fuzzing:** `fuzz_version` gains invariants:
  - total order for both comparisons (reflexive, antisymmetric, transitive);
  - pub refines precedence;
  - `semver_to_str` reparses to an equal version;
  - a pre-release never matches a constraint that does not opt in;
  - soundness: a version both constraints match implies `constraints_compatible`.

  Its generator builds constraints around the case's own version, so these
  invariants catch real bugs. The opt-in mutation and a compatibility
  mutation each fail it (exits 35 and 36). 5/5 harnesses pass under
  `--poison=ab`.
- **Gates:** clean (fmt, lint, vet, deny, the per-module doc gate,
  `cyrius header`). `cyrius distlib` is in sync.

## [1.6.1] - 2026-11-12

**One source label for a Flutter app, whichever call reports it.**

### Fixed

- **Flutter apps in search and list.** `resolver_resolve` labels a
  Flutter-runtime registry package `SOURCE_FLUTTER_APP` (1.5.3).
  `resolver_search` and `resolver_list` still labelled it
  `SOURCE_MARKETPLACE`, so the same package reported two sources depending on
  the call, and ark showed a Flutter app as a marketplace package in
  `ark search` and `ark list`. The rule now lives in one private helper,
  `_nous_mkt_src`, which all four registry sites use (`resolve_mkt`,
  `resolve_flutter`, `resolver_search`, `resolver_list`). Before, they used
  three different forms of the test.

### Changed (output)

| Call | Before | Now |
|---|---|---|
| `resolver_search`: `rp_src` of a Flutter-runtime registry package | `SOURCE_MARKETPLACE` | `SOURCE_FLUTTER_APP` |
| `resolver_list`: `ip_src` of a Flutter-runtime registry package | `SOURCE_MARKETPLACE` | `SOURCE_FLUTTER_APP` |
| `resolver_search`: `sr_sources` | `[marketplace, system]` | `[marketplace, flutter-app, system]`, so every result's source is in the list |

Native (non-Flutter) packages, the system leg and every error are unchanged.
ark reads only `sr_results` from a search, and builds its update set from its
own PackageDb rather than `resolver_list`. So nothing in ark breaks; its
search and list output gains the right label.

### Performance

The label lookup costs about 12 ns per registry package. Medians of 7
alternating runs of the 1.6.0 and 1.6.1 bench binaries:

| Bench | 1.6.0 | 1.6.1 | Change |
|---|---|---|---|
| `list_installed_100` | 12.62 µs | 13.80 µs | +9.3% |
| `search_100` | 96.18 µs | 94.29 µs | −2.0% (noise) |

A first version, which used `str_eq_cstr`, cost +23.5% on
`list_installed_100`, because it ran `strlen` on the literal for every
package. Checking the length first rejects the common `"native"` with one
compare. The release binary is unchanged at 40,936 B, because the smoke
binary never reaches search or list.

### Verification

- **Tests:** 503 passed, 0 failed (+10 assertions in `flutter_labels`).
- **Mutation checks:** each of the four fixes was reverted alone, and every
  revert fails the suite:
  - the search label (2 assertions fail);
  - the list label (2 fail);
  - the `sr_sources` entry (2 fail);
  - the helper's Flutter rule (`test_resolve_flutter` fails, then segfaults
    on the 0 it gets).
- **Fuzzing:** 5/5 harnesses pass under `--poison=ab`.
- **Gates:** clean, including the per-module doc gate and `cyrius header`.
  `cyrius distlib` is in sync.

## [1.6.0] - 2026-11-12

**Documentation completeness. No behaviour change.** The smoke binary, its
`CYRIUS_DCE=1` release build (40,936 B) and the test binary are all
byte-identical to 1.5.3 (`cmp`, built from `git archive` of the 1.5.3 commit
with the same toolchain). Only comments and return-type annotations changed.
ark has nothing to absorb.

### Added

- **A doc comment on every public function.** 167 of the 232 public fns had
  none. Each now has a one-line summary, which `cyrius doc src/<module>.cyr`
  prints: what it returns, in which type (`Str`, cstr, `vec of X`), and its 0
  and `Err` cases.
- **Fragment summaries replaced.** cyrdoc shows only the comment line directly
  above a function. For 50 functions that was the tail of a longer note, for
  example "strategy, as documented." for `strategy_from_json` and "time." for
  `nous_capture`. Each now ends with a real summary, and the history stays
  above it.
- **A per-module doc gate in CI.** `cyrius doc --check` now runs on every
  `src/*.cyr`, and each must report 0 undocumented. Before, it ran only on the
  barrel, which has no functions, so it always passed. The step is safe under
  the runner's `bash -e`. Mutation-tested: deleting one doc line fails it.
- **`: Result` on the 31 functions that return `Ok(..)` / `Err(..)`**
  (decision D13). That is 26 public and 5 private, found by a cross-module
  fixpoint over `return` sites. `cyrius header`, which refused 24 of them
  before, now writes a prototype for every module.
  - **Bytes are unchanged** (above).
  - **Consumer diagnostics are unchanged.** The compiler already inferred the
    pair from the body, so a one-variable bind (`var r = resolver_new(..);`)
    is the same compile error against the 1.5.3 and the 1.6.0 `dist/nous.cyr`.
    This was checked by compiling a consumer against both.

### Changed

- **CONTRIBUTING.md** is rewritten for Cyrius. It still described the Rust
  crate (`cargo test`, clippy, MSRV). It now gives the gate commands, the
  doc-comment rule and the `: Result` rule.
- **CLAUDE.md** cleanliness step:
  - the doc check is per module;
  - the fmt check is described as exit-code based. It said "diff against
    in-place", but `--check` prints nothing.
- **Stale docs brought current:**
  - `docs/architecture/overview.md`: the module map gains `updates.cyr`; sort,
    graph, json, version and command are described as they are since 1.5.3;
    the test counts go from 271 assertions / 18 benches / 3 harnesses to
    493 / 23 / 5; the "stdlib `is_dir` is broken" note is corrected.
  - `docs/guides/getting-started.md`: the assertion count, the fuzz command,
    and both bayan profiles.
  - `docs/development/gaps.md`: 9 error kinds, Tarjan rather than DFS.
  - `docs/api.md`: which functions return a Result, and `cyrius doc`.
- **Stale source comments:**
  - the `src/util.cyr` section header (`is_dir` works; nous keeps a one-stat
    version);
  - the `src/graph.cyr` section header (Tarjan, not a DFS colour walk);
  - the Constraint layout in `src/types.cyr` (40 bytes, not 16).
- `dist/nous.cyr` is regenerated (the comments and annotations).

### Found, not fixed

- `resolver_search` and `resolver_list` label a Flutter-runtime registry
  package `SOURCE_MARKETPLACE`. `resolve_mkt` labels it `SOURCE_FLUTTER_APP`
  (1.5.3). Fixing this changes output, so it is left out of this no-behaviour
  release; see roadmap 1.6.x.

### Verification

- **Tests and fuzzing:** 493 passed, 0 failed. 5/5 fuzz harnesses pass under
  `--poison=ab`.
- **Gates:** fmt, lint (no `warn ` lines), vet, deny, the per-module doc gate,
  and `cyrius header` (0 errors) are all clean. `cyrius distlib` is in sync.
- **Benchmarks:** recorded as `1.6.0-local` in `bench-history.csv`. No
  performance claim: the code is identical, and the -13.5% to +15.6% spread
  per bench against `1.5.3-local` is run-to-run noise.

## [1.5.3] - 2026-11-12

**The P(-1) sweep: audit, hardening, security, correctness and optimization, with
repairs.**

- **Method:** two independent code reviews (security and input hardening;
  correctness and determinism), external spec research, and my own review.
  Findings were proven by running probes wherever possible. apt behaviour was
  checked on Debian 12 in docker.
- **Result:** 2 P0s and 12 P1 / high findings repaired, plus P2/P3 and performance work.
  Assertions went from 314 to **493**, and 5 fuzz harnesses now run under
  `--poison=ab`; before, there was one fixed-input smoke file.
- **Gates:** clean (fmt / lint, with no `note` lines / vet / deny / doc), and the
  build is warning-free. No toolchain change.

The full report, with every finding, its proof and its fix, is
[docs/audit/2026-11-12-1.5.3-p-1.md](docs/audit/2026-11-12-1.5.3-p-1.md). The
policy questions were the maintainer's decisions D1–D13, recorded there.

### ⚠ Behaviour changes — read before moving ark to 1.5.3

No API is removed and every error value is unchanged; two are added. The scope
decision put everything in this patch release, so these changes ship here.

| Change | Before | Now |
|---|---|---|
| Missing dependency / root (D4) | silently dropped: `a -> ghost` gave Ok [a], a missing root Ok [] | `Err(NOUS_ERR_NOT_FOUND)` (new kind **7**), naming who needed it, with a typo suggestion. An apt dependency apt cannot show (virtual, alternative) is listed in `nous_pl_unresolved(plan)` instead |
| Version constraints (D1) | only the first constraint per dependency counted, and the chosen version was never checked | all are intersected as ranges and the chosen version must satisfy them, so more (correct) `NOUS_ERR_VERSION_CONFLICT` |
| Unsupported constraint syntax (D1) | `!=1.0`, `>=1.0, <2.0`, `^1 \|\| ^2`, `latest` parsed silently into something else | `Err(NOUS_ERR_INVALID_VERSION_CONSTRAINT)` (now actually raised) |
| SearchAll (D5) | identical to MarketplaceFirst | asks every source; a name in two is `Err(NOUS_ERR_AMBIGUOUS_SOURCE)` (new kind **8**) |
| apt dependencies (D5) | looked up per strategy, so a marketplace manifest named `libssl3` was spliced into curl's closure | an apt package's deps resolve from apt only |
| Dependency cycles (D3) | every cycle rejected, so any apt closure reaching libc6 ↔ libgcc-s1 failed | cycles of apt packages only are accepted (Debian Policy §7.2; [ADR 0005](docs/adr/0005-system-only-dependency-cycles.md)); any cycle touching a non-apt package is still an error |
| `trusted` (D9) | hard-coded 1 for marketplace packages | 0 until sigil verifies signatures (roadmap 1.9.x); apt stays 1 |
| `pl_order` | reverse install order (docs said install order; ark prints it as "deps-first") | install order, matching `pl_packages` |
| Install order | Kahn's order reversed, and hash-seeded, so different on every run | Tarjan emission order, deterministic. It is a valid order but can differ from 1.5.2 |
| Flutter apps | labelled `SOURCE_MARKETPLACE` under the default, SystemFirst and SearchAll strategies | `SOURCE_FLUTTER_APP` |
| `sysdb_updates` | `apt list --upgradable` | `apt-get -s dist-upgrade` `Inst` lines. Held-back packages are not listed |
| Recipe DB | 342 of zugot's 563 files loaded | all 535 unique recipes; duplicates use a fixed precedence and are reported (D6) |
| JSON output | strings raw (invalid JSON for `"`, `\`, newlines) | escaped; SearchResult adds `"sources"` |
| Bundle needs | `bayan_toml_*` | `bayan_toml_*` and `bayan_json_*`. ark's full stdlib `bayan` already provides both |

### Fixed — P0

- **Resolution was not deterministic** (both reviews; proven). `map_keys` walks
  hash-slot order, and the stdlib seeds the hash from getrandom per process
  (`lib/hashseed.cyr`, since cyrius 6.5.39). That made several things change
  from run to run on identical inputs: install order, dependency order, the
  reported cycle, whether a conflict was caught, and suggestion ties. **Six runs
  gave six install orders; now they give one.** All six `map_keys` sites go
  through `_nous_sorted_keys` (byte order). Pinned by `determinism_hash_order`,
  which asserts the exact order; the old code matched it about 1 run in 120.
- **Conflict detection was unsound** (both reviews and research; proven).
  - Only the first constraint per dependency was kept, so `*`, `=1.0.0` and
    `=2.0.0` passed 15 times in 40.
  - The version chosen was never checked: `>=2.0.0` resolved to 1.0.0.
  - `constraints_compatible` sampled about five versions. So `>1.2.0` with
    `<1.3.0` "conflicted", and swapping the arguments changed the answer.

  Now there are real version ranges, every constraint is intersected, and the
  chosen version is checked (see Fixed — versions).

### Fixed — security

- **Option injection into apt** (P1; proven on real apt). `validate_package_name`
  accepted a leading `-`, so `apt-cache show -c/tmp/evil.conf` loaded an
  attacker's file as apt config: `apt-config` showed the forged
  `Dir::State::lists`. An installed marketplace package could ship such a file
  and name it as a dependency.
  - A leading `-` is now rejected.
  - Only Debian Policy §5.6.1 names reach an apt argv at all.
- **Symlink fan-out denial of service** (P1; proven). `dir_size_recursive`
  followed links with no limit. A package directory with `a -> .` and `b -> .`
  meant 2^40 paths, at 500 MB and climbing after 10 s. Now links aren't followed
  (`lstat`) and depth is capped at 64. The same input takes **2 ms**.
- **SIGSEGV from a manifest constraint** (P1; proven). A 20-digit version
  component made `constraint_parse` store a null version, and `semver_cmp` then
  dereferenced it. Now it's `NOUS_ERR_INVALID_VERSION_CONSTRAINT`, and the matcher
  refuses a null version.
- **Files over a fixed buffer were half-parsed** (P1; proven). Manifests were
  read into 8 KiB, recipes and `build-order.txt` into 64 KiB. A long
  description before the dependencies silently lost every one; the 2026-04-16
  audit's "truncation means a parse failure" was wrong.
  - Files are now read whole, sized by `stat`, capped at `NOUS_FILE_MAX`
    (1 MiB), and refused rather than half-read.
  - This also saves memory: 563 recipes used to take 64 KiB each.
- **JSON injection** (P2). Strings went out raw, so a description
  `x","trusted":true,"z":"` injected keys. Now the stdlib escaper
  (`str_builder_add_json_str`) handles every string.
- **Manifest structure confusion** (P2, decision D7). The field scanners took the
  first `"name"` anywhere: `{"author":{"name":"mallory"},"name":"real"}` read as
  mallory, and a string value `"dependencies"` hijacked the deps. Manifests are
  now parsed with bayan's JSON parser (`dist/bayan-json.cyr`).
- **Dependency confusion inside transitive resolution** (decision D5): see
  Behaviour changes.
- **Hard-coded trust** (decision D9): see Behaviour changes.
- **P3s:**
  - `which_path` accepted relative PATH entries and directories; now it takes
    absolute entries and executable regular files only.
  - `apt-cache search` treated the query as a regex: `g++` matched 22,782
    packages, and now the escaped query matches 114.
  - `Installed-Size × 1024` could overflow; it now saturates.
  - `semver_parse`'s overflow guard was off by one: `9223372036854775808` wrapped
    negative.
  - `json_extract_str` read one byte past its input.

### Fixed — versions (decisions D1 + D2)

- **Constraints are version ranges**, desugared per npm/Cargo. The full table is
  in [docs/sources.md](docs/sources.md).
  - Partials used to give the wrong range. `^0` excluded 0.5.0, and now means
    `<1.0.0`. `~1` excluded 1.5.0, and now means `<2.0.0`. `>1.2` admitted
    1.2.1, and now means `>=1.3.0`.
  - A leading `v` is accepted, so `v2.0.0` vs `1.0.0` no longer misses the
    update or offers a downgrade.
  - Unsupported syntax is an error that matches nothing. Pre-release and
    compound ranges are roadmap 1.7.x.
- **Debian version comparison** (`nous_deb_version_cmp`): deb-version(7) /
  dpkg `verrevcmp`, used for apt and community versions.
  - Before, nous missed revision-only security updates (`7.81.0-1ubuntu1.16`
    vs `.15`), `~` pre-releases, and letter suffixes (`0.9.8zh`).
  - It also offered an epoch downgrade (`2.0-1` over `1:1.0-1`).
  - `resolver_check_updates` compares per source with `nous_update_is_newer_src`,
    because ark sends zugot versions through it.

### Fixed — correctness

- **Debian dependency cycles** (D3): accepted when the cycle is apt-only. curl's
  32-package closure now resolves on Debian 12 (verified). Cycle detection is an
  **iterative Tarjan** pass. The recursive DFS crashed on 50k+ chains, and a
  100k chain now takes 0.5 s. The self-loop path is correct: `aa -> aa`, not
  `xx -> aa -> aa`.
- **apt / dpkg parsing** (verified on Debian 12). The parsers are pure
  `_sysdb_parse_*` functions now, unit-tested and fuzzed without apt.
  - `apt-cache show` printed every version (2 for libc6), and nous merged them,
    keeping the oldest. Now it uses `--no-all-versions` and the first stanza.
  - `Pre-Depends` is parsed.
  - `python3:any` failed validation and aborted the resolve; the qualifier is
    stripped.
  - A held package read as not installed, and a removed-with-config one as
    installed. Now dpkg's own `${db:Status-Status}` decides.
  - `Description-<lang>` is read.
- **The recipe DB lost 221 of 563 zugot recipes.**
  - It scanned `desktop`, but the directory is `desktops` (196 recipes), and it
    scanned a `bazaar` that does not exist.
  - A multi-line `runtime = [` gave 0 deps: firefox now has 30.
  - A `# "x"` comment added a dep.

  Every category directory is now scanned in byte order. Duplicate names
  (zugot has 28) use a fixed precedence (base first, then sorted) and are
  listed through `nous_recipe_db_duplicates` (D6).
- **JSON roundtrip** (the CLAUDE.md rule). `from_json` dropped size, deps, trust
  and dates, and never decoded escapes. AvailUpdate, SearchResult, AgentInfo and
  Manifest had no `from_json` at all. Every type now roundtrips every field,
  including hostile strings.
- **Flutter labelling and `pl_order`:** see Behaviour changes.
- **The test suite leaked temp directories.** `rm_rf` passed a cstr to
  `is_dir(Str)`, so about 30 `/tmp/nous-test-*` were left per run. Now it uses
  `lstat`, never follows links, and leaves 0.
- **Found by the new fuzzer:** a dependency item starting with `:` kept the
  colon in its name (`tests/fuzz_apt.fcyr`, case 2096).

### Changed — performance

Measured on the same machine, median of 3 alternating runs against 1.5.2:

| Bench | 1.5.2 | 1.5.3 | Δ |
|---|---|---|---|
| `detect_source` | 592 ns | 223 ns | **−62.3%** |
| `validate_name` | 532 ns | 547 ns | +2.8% |
| `resolve_mkt_hit` | 2.23 µs | 2.27 µs | +1.7% |
| `resolve_mkt_miss` | 4.54 µs | 2.44 µs | **−46.3%** |
| `search_100` | 85.79 µs | 91.35 µs | +6.5% |
| `list_installed_100` | 12.62 µs | 11.77 µs | −6.8% |
| `strat_mkt_first` | 2.24 µs | 2.28 µs | +1.8% |
| `strat_sys_first` | 2.28 µs | 2.34 µs | +2.5% |
| `strat_only_mkt` | 2.30 µs | 2.27 µs | −1.3% |
| `strat_search_all` | 2.48 µs | 2.30 µs | −7.2% |
| `graph_resolve` | 8.98 µs | 7.87 µs | −12.4% |
| `recipe_parse` | 18.12 µs | 18.38 µs | +1.4% |
| `levenshtein` | 2.27 µs | 2.28 µs | +0.4% |
| `constraint` | 713 ns | 1.06 µs | +49.2% |
| `cycle_detect_20` | 23.32 µs | 24.94 µs | +6.9% |
| `topo_sort_20` | 34.10 µs | 42.34 µs | +24.2% |
| `recipe_db_load` | 7.091 ms | 11.245 ms | +58.6% |
| `serde_roundtrip` | 2.78 µs | 6.92 µs | +148.6% |
| `capture_200k` | 1.986 ms | 2.831 ms | +42.5% |
| `sort_installed_2000` (new) | — | 808 µs | — |
| `deb_version_cmp` (new) | — | 665 ns | — |
| `constraint_range` (new) | — | 547 ns | — |
| `manifest_parse` (new) | — | 5.35 µs | — |

- **Sorting is a stable natural merge sort**, replacing three O(n²) insertion
  sorts (TAOCP §5.2.4). The order is element-for-element identical, ties
  included. On 2,000 packages:

  | Input shape | 1.5.2 | 1.5.3 |
  |---|---|---|
  | Random order | 32 ms | 0.74 ms |
  | Already sorted (dpkg's output) | 146 µs | 127 µs |
  | Two sorted runs (`resolver_list`) | 445 µs | 191 µs |

  A plain merge sort was tried first and **lost** on the sorted shapes
  (149 → 532 µs), which is why it is the natural variant.
- **apt lookups are batched** per breadth-first wave. apt spends ~430 ms per
  call loading its cache, and 8 names cost the same as 1. curl's closure went
  from **14.7 s to 2.9 s**, with an identical 32-package install order (diffed).
- **Smaller wins:**
  - The queues use a head index instead of `vec_remove(…, 0)`: a 2,000-leaf
    star graph goes from 8.7 ms to 4.7 ms.
  - `detect_source` no longer allocates.
  - Recipe arrays are a zero-copy view when they hold no comment.
  - Constraint ranges are cached.
- **⚖️ The price of correctness,** each one measured and attributed:
  - `serde_roundtrip`, about +4 µs: real escaping plus a real parser.
  - `constraint`, about +0.35 µs: strict parsing.
  - `capture_200k`, about +0.85 ms per capture: the 60 s idle deadline (D12)
    makes the stdlib wait in 1 ms polling slices. That's against roughly 430 ms
    of apt startup per call.
  - `topo_sort_20`, about +8 µs: sorted keys for determinism.
  - `recipe_db_load`: 535 recipes instead of 342, about 10% faster per recipe.
- **Release binary:** 36,848 → 40,936 B (`CYRIUS_DCE=1`).

### Added

- **API:**
  - `nous_pl_unresolved` / `nous_unres_name` / `nous_unres_requirer`
  - `nous_recipe_db_duplicates` / `nous_rdup_name` / `_kept` / `_ignored`
  - `nous_constraint_error`, `ct_prec`
  - `nous_deb_version_cmp`, `nous_update_is_newer_src`
  - `sysdb_info_many`
  - `update_from_json`, `search_result_from_json`, `agent_info_from_json`,
    `manifest_from_json`
  - `nous_registry_get` / `nous_registry_list`, `nous_search_result_new`,
    `nous_recipe_parse_file`
- **Error kinds:** `NOUS_ERR_NOT_FOUND` (7) and `NOUS_ERR_AMBIGUOUS_SOURCE` (8).
- **Tunables:** `NOUS_CAPTURE_IDLE_MS` (60 s), `NOUS_FILE_MAX` (1 MiB),
  `NOUS_DIR_DEPTH_MAX` (64), `NOUS_SYSDB_BATCH` (256).
- **Fuzzing (D8):** sandhi's `fuzz/fz_driver.cyr` is vendored verbatim as
  `tests/fz_driver.cyr`. There are 4 seeded-mutation targets, each with
  invariants:
  - `fuzz_cyml`: the recipe parser.
  - `fuzz_json`: every `*_from_json`, plus a property that random hostile
    strings roundtrip exactly.
  - `fuzz_version`: `constraints_compatible` is symmetric, and Debian
    comparison is antisymmetric with cmp(a, a) = 0.
  - `fuzz_apt`: every extracted dependency name is clean.

  CI runs `cyrius fuzz --poison=ab tests/`, and all of them pass.
- **Dependencies:** bayan's `dist/bayan-json.cyr` profile (D7, same tag), and the
  stdlib `mmap` leaf (for the fuzz driver; DCE drops it from the binary).
- **Tests:** 314 → 493 assertions. The apt-only ones were run on Debian 12 in
  docker, and their regression tests were mutation-checked.

### Deprecated (removed in 2.0.0)

All of these are latent collisions; none is live in ark. Every one warns at the
call site (`#deprecated`), and nous no longer calls any of them.

| Deprecated | Use | Collides with |
|---|---|---|
| `to_cstr` | stdlib `str_cstr` | hadara, hoosh, vidya |
| `join_path` | stdlib `path_join` | commandress |
| `registry_get` / `registry_list` | `nous_registry_get` / `nous_registry_list` | bote, hoosh, szal, thoth |
| `search_result_new` | `nous_search_result_new` | daimon, with a different arity (a hard error if both are bundled) |
| `recipe_parse_file` | `nous_recipe_parse_file` | takumi |

### Docs

- New: the [audit report](docs/audit/2026-11-12-1.5.3-p-1.md) and
  [ADR 0005](docs/adr/0005-system-only-dependency-cycles.md).
- `CLAUDE.md`: the cycle rule now carries the apt-only exception (D3).
- [`docs/sources.md`](docs/sources.md): citations for Tarjan, Knuth's merge, dpkg
  `verrevcmp`, npm/Cargo desugaring and Debian Policy, plus known deviations.
- Updated: `docs/api.md`, `gaps.md`, `roadmap.md` and the architecture overview.

### Not repaired here (on the roadmap)

- Version selection with backtracking (PubGrub-style).
- SemVer pre-release ordering and compound ranges.
- Lockfile integrity.
- The system leg via apt's own solver (`apt-get -s install`).
- Rollback and freeze protection.
- An upstream word-wise `memcpy` (D11).
- `: Result` annotations (D13, with the 1.6.0 docs).
- `struct stat` portability.

## [1.5.2] - 2026-11-12

**Quick fixes batched from the 1.5.0 / 1.5.1 audits and the doc backlog.** No toolchain change
(cyrius 6.7.18) and no breaking change. 314 assertions (was 310), 0 failures; fuzz ok. fmt / lint
/ vet / deny / doc clean; build warning-free.

### Deprecated — two bare names that collide in cyrius's flat namespace

| Deprecated | Use instead | Collides with |
|---|---|---|
| `which_exists(name)` | `nous_which_exists(name)` (new) | kavach's `which_exists(name)`: same arity, but it checks four fixed directories rather than PATH |
| `cmd_run(cmd)`, which returns Str or 0 | `nous_cmd_capture(cmd)`, which returns a Result (1.5.1) | thoth's `cmd_run(line)` |

- Both still work. `#deprecated` warns at every call site with the replacement's name; for
  example, `'which_exists' is deprecated: use nous_which_exists; …`. Both go in 2.0.0.
- nous itself no longer calls either. `sysdb_new` uses `nous_which_exists`, and the 1.5.1
  aliasing test uses `nous_cmd_capture`.
- The collisions are latent: no consumer vendors nous next to kavach or thoth today.

### Fixed — `updates.cyr` was outside the vet / deny gates

`src/main.cyr` included 14 of the 15 modules, so `cyrius vet src/main.cyr` and `cyrius deny
src/main.cyr` never saw `src/updates.cyr` (1.3.0's update detection). It is now included, and both
gates report 15 deps. Two smoke checks of `update_is_newer` were added, and the barrel's header
comment now lists the module.

- The smoke binary grows from 32,688 to 36,848 B (DCE), because the semver code it now exercises
  is live. The library bundle is unaffected.

### Added — tests

- **`integration_apt`** now pins 1.5.1's Err propagation. With `NOUS_CAPTURE_MAX` forced to
  16 bytes, `resolver_list` and `resolver_search` must return `Err(NOUS_ERR_COMMAND_EXEC)`.
  - It runs only where apt exists, i.e. CI's apt job.
  - **Verified on Debian 12 in docker**: it passes there, and a mutant restoring the old
    skip-the-Err behaviour fails exactly these four assertions.
- **`which_exists`** (2): `nous_which_exists` finds `sh` and rejects a missing tool.
- **`capture_complete`** now runs through `nous_cmd_capture`, with 2 new tag assertions.

### Docs

- **`docs/api.md` rewritten to the current API:**
  - value-form Results throughout (was `payload(..)`), plus the propagation rule
  - `resolver_check_updates` (undocumented since 1.3.0)
  - the error-kind table, capture, and the deprecated names
  - the consumer note: the bundle, plus how to supply `bayan_toml_*`

  Every function name was checked against `src/` and the stdlib. `resolver_is_sys` (it never
  existed) is removed, and `resolver_strategy` is now `rs_strat`, the real name.
- **`docs/development/gaps.md` refreshed.** It had been stale since 1.2.x.
  - The closed P1 / P2 / P5 work is collapsed into one line.
  - The open gaps are a table that points at roadmap versions.
  - The Cyrius notes were re-verified on 6.7.18. The stdlib `is_dir` now works (1 / 0 / 0 for a
    directory / file / missing path), so nous's hand-rolled `our_is_dir` is a 1.5.3 retirement
    candidate.
- **`docs/development/roadmap.md` restructured.**
  - Completed sections are removed; CHANGELOG holds the history.
  - Order: 1.5.2 → 1.5.3 P(-1) sweep → 1.6.0 docs → 1.7.x reproducible resolution → 1.8.x native
    resolver → 1.9.x mela → 1.10.x caching → 2.0.0 backlog, plus ecosystem coordination (ark,
    zugot).
  - Dropped the "migrate ark to `dist/nous.cyr`" item: ark already pins
    `modules = ["dist/nous.cyr"]`.
- **`CLAUDE.md`**: the toolchain line no longer restates a version (it said 6.0.1), and the
  obsolete "smoke string is hardcoded" note is gone (the banner has read `VERSION` since 1.2.4).

### Performance — flat

Same toolchain, 1.5.1 vs 1.5.2, median of 3 alternating runs:

| Bench | 1.5.1 | 1.5.2 | Δ |
|---|---|---|---|
| `detect_source` | 556 ns | 591 ns | +6.3% |
| `validate_name` | 513 ns | 521 ns | +1.6% |
| `resolve_mkt_hit` | 2.18 µs | 2.17 µs | −0.5% |
| `resolve_mkt_miss` | 4.53 µs | 4.45 µs | −1.9% |
| `search_100` | 83.77 µs | 83.57 µs | −0.2% |
| `list_installed_100` | 13.12 µs | 12.48 µs | −4.9% |
| `strat_mkt_first` | 2.16 µs | 2.18 µs | +0.7% |
| `strat_sys_first` | 2.19 µs | 2.20 µs | +0.4% |
| `strat_only_mkt` | 2.17 µs | 2.19 µs | +0.7% |
| `strat_search_all` | 2.16 µs | 2.18 µs | +1.0% |
| `graph_resolve` | 8.81 µs | 8.75 µs | −0.7% |
| `recipe_parse` | 17.63 µs | 17.51 µs | −0.7% |
| `levenshtein` | 2.21 µs | 2.21 µs | +0.4% |
| `constraint` | 696 ns | 693 ns | −0.4% |
| `cycle_detect_20` | 21.23 µs | 23.44 µs | +10.4% |
| `topo_sort_20` | 32.14 µs | 36.30 µs | +13.0% |
| `recipe_db_load` | 6.781 ms | 7.051 ms | +4.0% |
| `serde_roundtrip` | 2.69 µs | 2.75 µs | +2.0% |
| `capture_200k` | 1.999 ms | 1.835 ms | −8.2% |

- ⚖️ **`detect_source` is about 25 ns (~4%) slower**, and that held over six alternating rounds
  (545–568 vs 567–602 ns). Nothing on its path changed: `source.cyr` and the stdlib `str`
  functions are byte-identical. That fits a code-layout effect from the two functions added ahead
  of the bench harness. Not chased.
- `cycle_detect_20`, `topo_sort_20` and `recipe_db_load` overlap run to run (20.7–25.7 vs
  20.7–27.7 µs; 31.9–38.9 vs 31.5–42.7 µs), so they are noise. `graph.cyr` is unchanged.

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
