# Architecture Overview

## Module Map

```
src/
  nous.cyr            Barrel file — includes all modules in dependency order
  main.cyr            Entry point
  types.cyr           Enums, struct layout, constructors, accessors
  util.cyr            Filesystem, string, path helpers
  error.cyr           Error constructors, display, name validation
  strategy.cyr        Resolution strategy constructors
  source.cyr          Source display, detection, typo suggestions
  command.cyr         Child-output capture (complete, or Err), PATH lookup
  sort.cyr            Stable natural merge sort by name; sorted map keys
  registry.cyr        Marketplace package registry
  sysdb.cyr           apt/dpkg system package database
  resolver.cyr        Main resolver engine (single-name resolve, search, list)
  json.cyr            JSON serialization (escaped) and deserialization (bayan-json)
  version.cyr         SemVer, constraint ranges (npm/Cargo), Debian version order
  graph.cyr           Dependency graph, Tarjan SCC (cycles + install order), Kahn sort
  recipe.cyr          Zugot CYML parsing, recipe DB, trace, transitive resolution
  updates.cyr         Update detection: apt leg + a pluggable lookup per source

tests/
  nous.tcyr           Test suite (493 assertions)
  nous.bcyr           Benchmarks (23 benches)
  nous.fcyr           Fuzz smoke harness
  fuzz_*.fcyr         Mutation fuzz targets: cyml, json, version, apt (4)
  fz_driver.cyr       Fuzz driver, vendored from sandhi

dist/
  nous.cyr            Consumer-facing single-file bundle (cyrius distlib; ADR-0002)
```

Every `src/*.cyr` function carries an explicit return type: `: Result` for the
31 that return `Ok(..)` / `Err(..)`, `: i64` for the rest (1.6.0). Every public
function has a one-line doc comment, which `cyrius doc src/<module>.cyr` prints
and CI enforces per module (1.6.0). `lib/` (stdlib) is gitignored — resolved from
the version-pinned `~/.cyrius/versions/<v>/lib/` snapshot, not committed.

## Data Flow

```
                  Resolver
                 /    |    \
                /     |     \
          SysDb   Registry   (Flutter via registry)
            |         |
       apt-cache   filesystem
       dpkg-query  manifest.json
```

1. Consumer (ark) calls `resolver_resolve(r, name)` or `resolver_search(r, query)`
2. Resolver applies the configured Strategy to determine source order
3. Each source is queried in order:
   - **System**: runs `apt-cache` / `dpkg-query` (array-form, absolute path) via `nous_capture`
   - **Marketplace**: reads `installed/<pkg>/manifest.json` from the registry directory
   - **Flutter**: checks marketplace packages with `runtime: "flutter"`
4. First match (or all matches for `SearchAll`) is returned as a ResolvedPkg

## Struct Layout

All structs use manual `alloc` + `store64` construction with `load64` accessors.
Fields are at `field_index * 8` byte offsets. The layout of every struct type is
listed at the top of `src/types.cyr`.

## Build & Distribution

- **Library bundle** — `[lib] modules` in `cyrius.cyml` lists the 15 `src/*.cyr`
  in barrel order; `cyrius distlib` concatenates them into the committed
  `dist/nous.cyr`, behind a compile-verified `# Requires` block of
  `include "lib/<leaf>.cyr"` lines (cyrius 6.6.18+). The bundle does not embed
  stdlib content: consumers vendor the leaves listed in `dist/nous.deps`.
  CI fails if the committed bundle drifts.
- **bayan** — nous's one git dependency, `[deps.bayan]`. Two of its profiles
  are used ([ADR-0004](../adr/0004-bayan-toml-profile-dependency.md)):
  - `dist/bayan-toml.cyr`, for the CYML parser's `bayan_toml_section_*` /
    `bayan_toml_pair_*` records;
  - `dist/bayan-json.cyr` (1.5.3), for manifests and every `*_from_json`.

  A consumer of `dist/nous.cyr` supplies those functions, from the same
  profiles or the full stdlib `bayan` fold, never both.
- **Resolution (1.5.3).** A breadth-first closure where an apt package's
  dependencies resolve from apt only. Apt-side names are fetched one wave per
  `apt-cache` call. Then one iterative Tarjan pass gives the cycles (accepted
  only when apt-only; ADR-0005) and the install order. Every manifest
  constraint is intersected as a version range and checked against the chosen
  version: Debian order for apt and community, SemVer otherwise.
  Iteration order is byte order everywhere, never hash order.
- **Child processes** — every apt/dpkg query goes through `nous_capture*`
  (`src/command.cyr`), which captures the complete output as an owned Str. It
  returns `Err(NOUS_ERR_COMMAND_EXEC)` past 16 MiB or on an idle-deadline cut,
  never a partial result.
- **Smoke binary** — `[build] entry = src/main.cyr` → `build/nous`; a runtime
  self-check ("all smoke checks passed"). `modules` lives under `[lib]`, never
  `[build]` (the latter auto-prepends and bloats the binary).
- **Toolchain** — pinned to Cyrius 6.7.18 in `cyrius.cyml`. Release ships
  `nous-<tag>.cyr` (the bundle), the x86_64 binary, a source tarball, and
  `SHA256SUMS`. The binary is **not** stripped (corrupts a cyrius raw ELF).
- Rationale: see [ADR-0002](../adr/0002-flat-modules-dist-bundle.md).

## Consumers

- **ark** — the AGNOS package manager CLI. Nous is the single source of truth for resolving package dependencies.

## Key Design Decisions

- **No network access**: all resolution is local. Remote registry sync is a consumer responsibility.
- **Strategy pattern**: resolution order is configurable per-call, not hardcoded.
- **Source-agnostic types**: ResolvedPkg / InstalledPkg are the same regardless of source.
- **Stub registry**: LocalRegistry is intentionally minimal — it will be replaced by the real mela marketplace client.
- **Manual struct layout**: all types use `alloc`+`store64` constructors with `load64` accessor functions. That dates from early Cyrius, where struct literals worked only in `main()`; Cyrius 6.x has full structs, and adopting them is not scheduled.
- **Custom `our_is_dir()`**: one `stat` syscall. The stdlib `is_dir()` works (checked in 1.5.2), but it opens and reads the directory, and nous calls this once per directory entry.
- **`dir_list()` takes Str**: fs.cyr's `dir_list` expects Str type arguments, not C strings.

## Future Modules (planned)

Version constraints, the dependency graph, topological sort, cycle/conflict
detection, and zugot integration are **shipped** (see the module map above). The
remaining planned work — see [roadmap.md](../development/roadmap.md):

| Module | Purpose |
|---|---|
| Resolution cache | Persistent cache for resolved dependency graphs |
| Mela client | Real marketplace API (replaces the registry stub) |
