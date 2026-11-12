# 0006 — SemVer pre-release and build semantics

**Status:** Accepted (1.7.0)

## Context

nous ordered SemVer versions by major, minor and patch only. Everything after
the patch number was dropped, so `1.0.0-rc.1` compared equal to `1.0.0`, and
a pre-release tag in a constraint was refused with an error. That mattered
because mela accepts `-prerelease` versions for marketplace packages
(`is_valid_semver`, mela `src/manifest.cyr`):

- a marketplace package at `1.0.0-rc.1` was never offered `1.0.0` as an
  update;
- `>=1.0.0` matched `1.1.0-rc.1`, a pre-release nobody asked for;
- a Flutter app's build-number bump (`1.0.0+1` to `1.0.0+2`) was not an
  update.

Ordering is fixed by the spec: SemVer 2.0.0 §11, with build metadata ignored
(§10). Matching a pre-release against a constraint is not fixed by the spec,
and the ecosystems differ:

- **Pure precedence.** Every range takes every version in it, pre-releases
  included. `>=1.0.0` matches `1.1.0-rc.1`, and `<2.0.0` matches
  `2.0.0-alpha`.
- **The npm / Cargo opt-in rule.** A pre-release version satisfies a
  constraint only when the constraint names a pre-release of the same
  `major.minor.patch`. node-semver README, "Prerelease Tags"; Rust `semver`,
  `VersionReq`: "the VersionReq must contain at least one Comparator … with an
  explicit major, minor, and patch version identical to the pre-release being
  matched … and that has a nonempty pre-release component". Under this rule
  `*` matches no pre-release.
- **pub_semver**, which orders build metadata after precedence (`Version.compareTo`:
  "Builds always come after no build string") and has its own range rules for
  pre-releases.

## Decision

- **Ordering.** `semver_cmp` is SemVer 2.0.0 §11 precedence. Build metadata
  takes no part in it, so `semver_eq("1.0.0+a", "1.0.0+b")` is 1.
- **Matching.** Constraints follow the npm / Cargo opt-in rule. nous already
  follows npm for range syntax (docs/sources.md), and `^`, `~` and the
  partials already desugar as npm / Cargo do. So `>=1.0.0` and `*` do not
  match `1.1.0-rc.1`, while `>=1.1.0-rc.1` does.
  - `constraints_compatible` is exact under the rule. When the ranges overlap
    only on pre-releases of one `M.m.p`, both constraints must opt in to that
    `M.m.p`.
- **Constraint syntax.** A full `M.m.p` may carry `-pre` and `+build`, written
  strictly per §9 / §10. Build metadata in a constraint is accepted and
  ignored, as npm and Cargo do. A pre-release on a partial version (`1.2-rc`)
  is an error.
- **Flutter updates use pub order.** `nous_update_is_newer_src(.., SOURCE_FLUTTER_APP)`
  compares with `nous_semver_cmp_pub`, which is precedence first and then build
  (no build first, parts compared like pre-release identifiers). Flutter keeps
  the build number there. Marketplace updates ignore build, per §10.
- **Debian-ordered sources are unchanged** (apt, community). A SemVer
  pre-release bound in a constraint on such a package is written the Debian
  way, `~pre`, and the opt-in rule does not apply.
- **Parsing versions stays lenient.** It is strict only for constraints.
  `semver_parse` keeps the run of `[0-9A-Za-z.-]` after `-` / `+`, drops an
  empty one, and still ignores other trailing text.

## Consequences

- **A behaviour change.** A marketplace or Flutter dependency whose only
  candidate is a pre-release now fails any constraint that does not opt in.
  It is `NOUS_ERR_VERSION_CONFLICT`, and the message says the version is a
  pre-release and shows how to opt in (for example `>=1.1.0-rc.1`). npm and
  Cargo refuse the same graph.
- **More conflicts are found.** Two constraints whose overlap holds only
  pre-releases are a conflict: `>1.0.0` with `<1.0.1`, and `=1.0.0-rc.1`
  with `*`. Before, range intersection alone called them compatible.
- **Updates.** `1.0.0` is offered over `1.0.0-rc.1`, `rc.2` over `rc.1`,
  and, for Flutter, `+2` over `+1`.
- **Flutter constraints ignore build metadata,** as npm and Cargo do. This
  differs from pub_semver, which compares build inside ranges too.
- **SemVer grows from 24 to 40 bytes** (`nous_sv_pre`, `nous_sv_build`).
  `semver_new` keeps its signature, and `nous_semver_new_full` sets all five
  fields. The `constraint` and `constraint_range` benches cost about 4% more
  (CHANGELOG 1.7.0).
- **Compound ranges stay refused** (roadmap 1.7.1). When they land, the rule
  applies per comparator set, as npm does.
