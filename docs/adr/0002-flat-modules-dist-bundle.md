# 0002 — Flat `src/` modules + `dist/nous.cyr` bundle

**Status:** Accepted

## Context

nous is a library consumed by **ark** (the AGNOS package manager), and also
builds a small smoke binary (`src/main.cyr`). Two competing layouts exist in the
Cyrius ecosystem:

- **Monolith** (e.g. vidya): one large `src/main.cyr`, no `[lib]`, no bundle.
- **Modular + bundle** (e.g. patra, sigil): many `src/*.cyr` files listed under
  `[lib]`, concatenated into a single `dist/<name>.cyr` by `cyrius distlib`,
  which downstream consumers include as one file.

Originally ark consumed nous by listing all 14 `src/*.cyr` modules individually
in its `cyrius.cyml`. That couples ark to nous's internal file layout: every
module add/rename/reorder is a breaking change to the consumer manifest.

## Decision

Keep nous's **flat 14-module `src/` layout** (one module per concern: types,
util, error, strategy, source, command, sort, registry, sysdb, resolver, json,
version, graph, recipe) behind a barrel `src/nous.cyr`, **and** publish a
single-file **`dist/nous.cyr`** bundle via `cyrius distlib` (`[lib] modules` in
`cyrius.cyml`, order matching the barrel). The bundle is committed and shipped as
a release asset; CI fails if it drifts from the sources.

The smoke binary stays under `[build]` (entry `src/main.cyr`); `modules` lives
under `[lib]`, never `[build]` (the latter auto-prepends and inflates the
binary). `lib/` (stdlib) is gitignored, resolved from the version-pinned snapshot.

## Consequences

- Internal modularity is preserved (clear boundaries, focused files, easier
  review) **without** leaking that structure to consumers.
- ark can migrate to `modules = ["dist/nous.cyr"]` — one file, decoupled from
  nous's internal layout (tracked as consumer coordination in the roadmap).
- The bundle is a build artifact that must be regenerated on any `src/` change;
  a CI `distlib`-sync gate enforces this, and the release regenerates it.
- The bundle is a pure concatenation — stdlib is **not** bundled; the consumer
  supplies its own `[deps] stdlib` surface (same shape as patra/sigil).
