# 0004 — bayan's `toml` profile instead of the full stdlib `bayan` fold

**Status:** Accepted (1.5.1)

## Context

nous builds its own CYML parser (`nous_cyml_parse`, `src/recipe.cyr`). From
bayan it needs only the six section/pair record helpers that parser produces
and reads: `bayan_toml_section_new` / `_name` / `_pairs` and
`bayan_toml_pair_new` / `_key` / `_value`, each a 16-byte record
(`{name, pairs}` / `{key, value}`). Since 1.2.6 those came from the stdlib
`bayan` fold, through its unprefixed `_compat` aliases (`toml_section_new`, …),
declared as `"bayan"` in `[deps] stdlib`.

cyrius 6.7.18 folds bayan **1.11.1**. That fold carries every format bayan
supports (json, yaml, toml, toml-tree, cyml, csv, base64, u128, bigint,
markdown, pdf). Its `# Requires` block pulls `lib/math.cyr`, `lib/slice.cyr`,
`lib/fmt.cyr` and seven `lib/unicode/*` files, about 330 KB of Unicode tables
that are global data, which DCE cannot drop. Measured on nous's release build
(`CYRIUS_DCE=1`):

| Build | Release binary |
|---|---|
| 1.5.0, cyrius 6.7.9, stdlib bayan 1.5.13 | 58,504 B |
| 1.5.1 candidate, cyrius 6.7.18, stdlib bayan 1.11.1 | **406,296 B** (≈7×) |
| 1.5.1, cyrius 6.7.18, `dist/bayan-toml.cyr` profile | **32,528 B** |

bayan publishes per-format profiles from its own repo (`[lib.toml]` →
`dist/bayan-toml.cyr`, 85 KB). The toml profile needs only alloc, str, vec,
hashseed, string, io, result and syscalls (`dist/bayan-toml.deps`). The stdlib
snapshot ships only the full fold, so a profile has to come in as a named git
dependency. samay, shravan and mihi already pull `dist/bayan-json.cyr` that way.

Alternatives considered:

- **Keep the full fold.** No code change, but 7× the binary for six accessors.
- **Hand-roll the six helpers in nous.** Removes the dependency entirely, but
  duplicates bayan's record layout. Under unprefixed names it would also
  collide with any consumer that includes bayan (the 1.4.0 `agent_info_new`
  class). The user chose the profile.
- **The profile as a named dep** (chosen).

## Decision

Drop `"bayan"` from `[deps] stdlib`. Add:

```toml
[deps.bayan]
git = "https://github.com/MacCracken/bayan.git"
tag = "1.11.1"
modules = ["dist/bayan-toml.cyr"]
```

Call bayan's canonical `bayan_toml_*` names directly. The unprefixed `toml_*`
compat aliases live only in the full fold's `_compat` module, not in the
profile.

## Consequences

- Release binary 58,504 → 32,528 B (−44% against 1.5.0). The lib/ closure stays
  at 38 files (no math, slice or unicode).
- `cyrius deps` now fetches one git dependency over https. `cyrius.lock`
  records its commit (`965298d`, the same one samay pins), and CI's
  `cyrius deps` step does the fetch.
- **Consumer-facing.** `dist/nous.cyr` calls `bayan_toml_*`, and the sidecar
  (`dist/nous.deps`, now 4 leaves) does not list named deps by design. A
  consumer must supply those six functions, either by pulling the same
  `dist/bayan-toml.cyr` profile or by keeping the full stdlib `bayan` fold,
  which defines them too. It must include only one of the two: both declare
  the same names.
- Bumping bayan is now a tag edit in `cyrius.cyml`, independent of the cyrius
  pin. A future cyrius bump no longer drags bayan along.
- Supersedes the 1.2.6 choice to source the records from the stdlib fold's
  `_compat` surface (roadmap v1.2.6).
