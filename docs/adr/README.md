# Architectural Decision Records

Significant design decisions for nous, in the format required by `CLAUDE.md`
(Context · Decision · Consequences · Status).

| ADR | Title | Status |
|-----|-------|--------|
| [0001](0001-result-error-handling.md) | `Result<T, E>` error handling | Accepted |
| [0002](0002-flat-modules-dist-bundle.md) | Flat `src/` modules + `dist/nous.cyr` bundle | Accepted |
| [0003](0003-cyrius-6.0.1-vec-get-workaround.md) | Cyrius 6.0.1 `vec_get` de-nesting workaround | Superseded (1.2.5 — misdiagnosis, reverted) |
| [0004](0004-bayan-toml-profile-dependency.md) | bayan's `toml` profile instead of the full stdlib `bayan` fold | Accepted (1.5.1) |
| [0005](0005-system-only-dependency-cycles.md) | Dependency cycles allowed among system (apt) packages only | Accepted (1.5.3) |

New ADRs: copy the four-heading shape from any record above, number sequentially
(`NNNN-short-title.md`), and add a row here. Create one when choosing between
competing approaches, adopting/rejecting a dependency, making a breaking API
change, or picking a performance trade-off.
