# 0007 — Compound version ranges

**Status:** Accepted (1.7.1)

## Context

nous refused compound constraints with an error (1.5.3): npm's whitespace
AND (`>=1.2 <2`), Cargo's comma AND (`>=1.2, <1.5`), `||` OR, and hyphen
ranges (`1.2.3 - 2.3.4`). Before 1.5.3 it had silently misread them, so
`>=1.0, <2.0` matched 3.0.0. Marketplace manifests are written by hand, and
both npm and Cargo habits show up in them.

The two grammars differ:

- **npm** ("Range Grammar", node-semver README):
  - comparators are joined by whitespace into a comparator set;
  - sets are joined by `||`;
  - `A - B` is an inclusive hyphen range;
  - an empty range is `*`;
  - a bare version is exact.
- **Cargo** ("Multiple version requirements"; semver crate `VersionReq`):
  - comparators are joined by commas;
  - "Whitespace is permitted around commas and around operators";
  - there is no `||` and no hyphen range;
  - a bare version means `^`.

The representation had to change too. A simple constraint is one interval. A
set of comparators is still one interval (an intersection of intervals). An
OR of sets is a union of intervals.

## Decision

- **Grammar: a superset of both.**
  - Whitespace or commas join comparators into a set, and an operator may
    stand apart from its version (`>= 1.2`).
  - `||` joins sets; an empty side is `*`.
  - `A - B` (spaces required, plain versions only) is `>=A <=B`. With the
    existing partial rules that gives npm's results: a partial `A` fills
    zeros, and a partial `B` is an X-range (`1.2.3 - 2.3` = `>=1.2.3 <2.4.0`).
  - A bare version stays exact, as npm has it. This is unchanged since 1.5.3
    and was chosen there.
  - Refused with an error: a single `|`, a stray comma, an operator with no
    version, and a hyphen range that is not exactly `A - B`.
- **Representation.**
  - A constraint that turns out to be one comparator is the same simple
    `Constraint` as before, built by the same parser, so `ct_op` / `ct_ver` /
    `ct_prec` keep their meaning.
  - Anything more is `OP_SET`, a new `ConstraintOp` variant (8), whose
    comparator sets are read with `nous_ct_sets(c)`: a vec (OR) of vecs (AND)
    of simple constraints. Its `ct_ver` is 0.
- **Matching** is npm's rule. A version matches when every comparator of
  at least one set holds it. The pre-release opt-in rule
  ([ADR 0006](0006-semver-prerelease-and-build.md)) is checked per set.
- **Range sets.** For conflict work a constraint is its range set: the
  sorted, merged union of its sets' intervals. `finalize_graph` intersects
  range sets across every requirer, and an empty result is a
  `NOUS_ERR_VERSION_CONFLICT` naming the pair.
- **`constraints_compatible` is exact.**
  - A release version in the intersected range sets settles it.
  - Otherwise only pre-releases are in common. Then, for each `M.m.p` whose
    pre-releases the first constraint opts in to, it checks whether the
    pre-releases each side admits (per set, within `[M.m.p-0, M.m.p)`) meet.

## Consequences

- **Nothing that parsed before changes.** A single comparator takes the
  pre-1.7.1 parser first and returns from it. Only a text that parser refuses
  and that holds a separator goes to the compound path.
- **`ConstraintOp` gains `OP_SET`.** Code that switches over `ct_op` must
  handle it. No known consumer reads `ct_op`: ark does not.
- **Constraint grows from 40 to 56 bytes** (`ct_sets`, `ct_rset`). Measured
  against 1.7.0, simple constraints cost about 3–5% more:
  - the size accounts for 0.4–2%;
  - the routing between the simple and compound paths for the rest.
- **Accepting both AND styles means `>=1 <2` and `>=1, <2` are the same.**
  Mixing them inside one set (`>=1 <2, >=1.5`) is accepted as long as every
  comparator is valid. Neither npm nor Cargo would take that mix.
- **Lockfiles (1.7.2) record exact versions,** so they do not depend on this
  grammar. Version selection, when it comes (roadmap), works on these range
  sets.
