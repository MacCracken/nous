# Sources

Every algorithm, format and rule nous implements, the source it follows, and
where the code is. CLAUDE.md requires this so a reviewer can trace each
algorithm back to its origin and check the implementation against it.
Deviations from a source are stated, not hidden.

## Algorithms

| What | Source | Code | Notes |
|---|---|---|---|
| Strongly connected components (Tarjan), cycles and install order | R. E. Tarjan, "Depth-first search and linear graph algorithms", *SIAM J. Comput.* 1(2):146–160, 1972. [doi:10.1137/0201010](https://doi.org/10.1137/0201010) | `src/graph.cyr` `_nous_tarjan`, `dep_graph_detect_cycle`; `src/recipe.cyr` `finalize_graph` | Iterative, with explicit frames (1.5.3; the recursive DFS overflowed the stack on 50k+ chains). Components are emitted dependencies first, which is the install order. Nodes are visited in byte order and edges in dependency-list order. A cyclic component is accepted only when every member is an apt package (ADR 0005). |
| Cycle path report | Breadth-first search within one component, back to its byte-first member | `src/graph.cyr` `_nous_comp_cycle_path` | The shortest cycle through that member. |
| Topological sort (Kahn) | A. B. Kahn, "Topological sorting of large networks", *CACM* 5(11):558–562, 1962. [doi:10.1145/368996.369025](https://doi.org/10.1145/368996.369025) | `src/graph.cyr` `dep_graph_topo_sort` (public) | Kept for its public contract. Order is dependents first; on a cyclic graph the result is partial. `finalize_graph` uses Tarjan instead. |
| Stable natural merge sort | D. E. Knuth, *The Art of Computer Programming*, Vol. 3, 2nd ed., 1998, §5.2.4 "Sorting by Merging" (Algorithm N, natural two-way merge) | `src/sort.cyr` `_nous_sort_by_key` | Runs are maximal non-decreasing stretches, merged pairwise. O(n) on sorted input, O(n log n) worst case. Stable. |
| Edit distance (Levenshtein) | V. I. Levenshtein, *Soviet Physics Doklady* 10(8):707–710, 1966. Two-row dynamic programme per R. A. Wagner, M. J. Fischer, *JACM* 21(1):168–173, 1974. [doi:10.1145/321796.321811](https://doi.org/10.1145/321796.321811) | `src/source.cyr` `levenshtein`, `suggest_package` | Unit costs, byte-wise (not Unicode-aware). Offered at distance ≤ 3. Ties break by candidate order: registry names in byte order, then recipe names in byte order. |
| Debian version comparison | deb-version(7) ([man page](https://manpages.debian.org/testing/dpkg-dev/deb-version.7.en.html)); Debian Policy §5.6.12 ([Version](https://www.debian.org/doc/debian-policy/ch-controlfields.html#version)); dpkg `lib/dpkg/version.c` `verrevcmp` | `src/version.cyr` `nous_deb_version_cmp`, `_nous_verrevcmp` | `[epoch:]upstream[-revision]`. The epoch is compared numerically, then upstream, then revision, by alternating non-digit runs (`~` < end < letters < other) and numeric runs. Used for apt and community versions (1.5.3). |
| Fuzzing | sandhi 1.13.7 `fuzz/fz_driver.cyr` (vendored as `tests/fz_driver.cyr`): seeded LCG mutation, a forked child, FNV-1a digests | `tests/fuzz_*.fcyr` | Run under `cyrius fuzz --poison=ab` in CI. |

## Formats and specifications

| What | Source | Code | Deviations |
|---|---|---|---|
| Version numbers | Semantic Versioning 2.0.0, [semver.org/spec/v2.0.0](https://semver.org/spec/v2.0.0.html): §9 pre-release, §10 build metadata, §11 precedence | `src/version.cyr` `semver_parse`, `semver_cmp`, `_nous_ids_cmp` | §11 precedence including pre-releases (1.7.0); build is ignored (§10). See [Known deviations](#known-deviations). |
| Compound ranges | npm node-semver, [Ranges](https://github.com/npm/node-semver#ranges), [Hyphen Ranges](https://github.com/npm/node-semver#hyphen-ranges-xyz---abc), [Range Grammar](https://github.com/npm/node-semver#range-grammar); Cargo, [Multiple version requirements](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#multiple-version-requirements); semver crate [`VersionReq`](https://docs.rs/semver/latest/semver/struct.VersionReq.html) | `src/version.cyr` `constraint_parse`, `_nous_ct_parse_set`, `_nous_set_matches`, `_nous_ct_rset`; `src/graph.cyr` `_nous_compat_sets` | npm's `\|\|`, whitespace AND and `A - B`, plus Cargo's comma AND, in one grammar (1.7.1, [ADR 0007](adr/0007-compound-ranges.md)). A bare version is npm's exact, not Cargo's caret. |
| Pre-release matching | npm node-semver, [Prerelease Tags](https://github.com/npm/node-semver#prerelease-tags); Rust semver, [`VersionReq`](https://docs.rs/semver/latest/semver/struct.VersionReq.html) | `src/version.cyr` `constraint_matches`, `_nous_ct_pre_on`; `src/graph.cyr` `constraints_compatible` | A pre-release matches only a constraint naming a pre-release of the same M.m.p, so `*` takes none ([ADR 0006](adr/0006-semver-prerelease-and-build.md)). |
| Flutter build ordering | Dart pub_semver, [`Version.compareTo`](https://github.com/dart-lang/tools/blob/main/pkgs/pub_semver/lib/src/version.dart) | `src/version.cyr` `nous_semver_cmp_pub`; `src/updates.cyr` `nous_update_is_newer_src` | Used for Flutter update detection only; constraints ignore build, as npm and Cargo do. |
| Ranges: `^`, `~`, comparators, partials, X-ranges | npm node-semver, [Ranges](https://github.com/npm/node-semver#ranges) / [Advanced Range Syntax](https://github.com/npm/node-semver#advanced-range-syntax); Cargo, [Specifying Dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html); semver crate [`Op`](https://docs.rs/semver/latest/semver/enum.Op.html) | `src/version.cyr` `constraint_parse`, `_nous_range_build` | The desugaring table is below. A bare version follows **npm** (exact; partials are X-ranges), not Cargo (where bare means `^`). |
| apt / dpkg interfaces | `apt-cache(8)` (`--no-all-versions`), `dpkg-query(1)` (`${db:Status-Status}`), `apt-get(8)` (`-s` simulation `Inst` lines), `apt(8)` ("not a stable CLI interface") | `src/sysdb.cyr` | Parses machine-stable output only (1.5.3). The child environment is empty, so the C locale applies. |
| Debian package names | Debian Policy §5.6.1 ([Source](https://www.debian.org/doc/debian-policy/ch-controlfields.html#source)): lowercase alphanumeric first, then `[a-z0-9+.-]`, ≥ 2 characters | `src/sysdb.cyr` `_sysdb_debian_name` | Only such names ever reach an apt argv (1.5.3). |
| Circular dependencies | Debian Policy §7.2 ([Binary Dependencies](https://www.debian.org/doc/debian-policy/ch-relationships.html)): "the breaking point is arbitrary" | `src/recipe.cyr` `finalize_graph` | Accepted for apt-only cycles; see ADR 0005. |
| Manifest / recipe syntax (CYML, TOML-like) | cyrius's `cyrius.cyml`; bayan's TOML section and pair records (`dist/bayan-toml.cyr`) | `src/recipe.cyr` `nous_cyml_parse`, `cyml_get` | A subset: `[section]` / `[[array]]` headers, `key = value`, strings (`"…"`, `"""…"""`), arrays (multi-line since 1.5.3), and `#` comments outside quotes. |
| JSON | RFC 8259, [The JSON Data Interchange Format](https://www.rfc-editor.org/rfc/rfc8259); bayan's `dist/bayan-json.cyr` parser | `src/json.cyr`, `src/registry.cyr` | Output is escaped by the stdlib's `str_builder_add_json_str`. Input is parsed by bayan's tree parser (1.5.3). |

### Range desugaring (npm / Cargo, 1.5.3)

`M.m.p` is the written version; *P* is its precision (components written).

| Constraint | P = 3 | P = 2 (`M.m`) | P = 1 (`M`) |
|---|---|---|---|
| `=V` / bare `V` | `[V, V]` | `[M.m.0, M.(m+1).0)` | `[M.0.0, (M+1).0.0)` |
| `>=V` | `[V, ∞)` | `[M.m.0, ∞)` | `[M.0.0, ∞)` |
| `>V` | `(V, ∞)` | `[M.(m+1).0, ∞)` | `[(M+1).0.0, ∞)` |
| `<V` | `[0, V)` | `[0, M.m.0)` | `[0, M.0.0)` |
| `<=V` | `[0, V]` | `[0, M.(m+1).0)` | `[0, (M+1).0.0)` |
| `^V` | M > 0: `[V, (M+1).0.0)`; M = 0, m > 0: `[V, 0.(m+1).0)`; `^0.0.p`: `[V, 0.0.(p+1))` | M > 0: `[V, (M+1).0.0)`; M = 0: `[V, 0.(m+1).0)` | `[V, (M+1).0.0)` |
| `~V` | `[V, M.(m+1).0)` | `[V, M.(m+1).0)` | `[V, (M+1).0.0)` |
| `*`, `x`, `""` | any | | |

Compound forms (1.7.1) combine those rows:

| Form | Meaning |
|---|---|
| `A B` / `A, B` | both: the intersection of A's and B's ranges (one comparator set) |
| `S1 \|\| S2` | either set; an empty side is `*` |
| `A - B` | `>=A <=B`: a partial A fills zeros, a partial B is an X-range (`1.2.3 - 2.3` = `[1.2.3, 2.4.0)`) |

A pre-release version must be opted in to by a comparator of the set that
takes it (ADR 0006, per set as npm does).

Conflict detection intersects every constraint on a dependency (as unions of
ranges, since 1.7.1) and then checks the chosen version against each one. For apt and community packages the check
uses Debian comparison against the bound written without trailing zero parts
(`2.36.0` → `2.36`, so `2.36-9` satisfies `>=2.36`).

## Known deviations

- **Lenient version parsing.** `semver_parse` reads the run of
  `[0-9A-Za-z.-]` after `-` / `+` without checking §9's rules (empty
  identifiers, leading zeros), since it parses versions nous is handed.
  Constraints are parsed strictly. Numeric identifiers with leading zeros
  compare by value.
- **Both AND styles are accepted together** (1.7.1): whitespace (npm) and
  commas (Cargo), even mixed in one set. Neither tool takes the other's form.
  A bare version is exact (npm); in Cargo it means `^`.
- **No version selection or backtracking.** Each name has one candidate (the
  installed or available version), and nous checks it against the constraints
  rather than choosing among versions (PubGrub-style). It is gated on a source
  that offers more than one version (roadmap).
- **Flutter build numbers in constraints** are ignored (npm / Cargo), while
  pub_semver compares them inside ranges too. Update detection does order
  them (1.7.0).
- **Leading zeros** and **fewer than three components** are accepted in
  versions, against SemVer §2. A leading `v` is accepted (as npm does).
- **`struct stat` offsets** in `src/util.cyr` are x86_64 Linux; aarch64's
  layout differs.
