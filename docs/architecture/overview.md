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
  command.cyr         Shell command execution, PATH scanning
  sort.cyr            Insertion sort by name
  registry.cyr        Marketplace package registry
  sysdb.cyr           apt/dpkg system package database
  resolver.cyr        Main resolver engine, trace
  json.cyr            JSON serialization and deserialization
  version.cyr         SemVer parsing, constraint matching
  graph.cyr           Dependency graph, cycle detection, topological sort
  recipe.cyr          Zugot CYML parsing, recipe DB, recipe-based resolution

tests/
  nous.tcyr           Test suite (271 assertions)
  nous.bcyr           Benchmarks (18 benches)
  nous.fcyr           Fuzz harnesses (3 harnesses)

dist/
  nous.cyr            Consumer-facing single-file bundle (cyrius distlib; ADR-0002)
```

All `src/*.cyr` functions carry explicit `: i64` return types (Cyrius 6.0.1
idiom). `lib/` (stdlib) is gitignored — resolved from the version-pinned
`~/.cyrius/versions/<v>/lib/` snapshot, not committed.

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
   - **System**: shells out to `apt-cache` / `dpkg-query` via `exec_capture`
   - **Marketplace**: reads `installed/<pkg>/manifest.json` from the registry directory
   - **Flutter**: checks marketplace packages with `runtime: "flutter"`
4. First match (or all matches for `SearchAll`) is returned as a ResolvedPkg

## Struct Layout

All structs use manual `alloc` + `store64` construction with `load64` accessors.
Fields are at `field_index * 8` byte offsets. See `src/nous.cyr` header comments for
the complete layout of all 12 struct types.

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
- **Manual struct layout**: Cyrius struct constructors only work in `main()`, so all types use `alloc`+`store64` constructors with `load64` accessor functions.
- **Custom `our_is_dir()`**: The stdlib `is_dir()` from fs.cyr is broken; nous uses a direct stat syscall.
- **`dir_list()` takes Str**: fs.cyr's `dir_list` expects Str type arguments, not C strings.

## Future Modules (planned)

Version constraints, the dependency graph, topological sort, cycle/conflict
detection, and zugot integration are **shipped** (see the module map above). The
remaining planned work — see [roadmap.md](../development/roadmap.md):

| Module | Purpose |
|---|---|
| Resolution cache | Persistent cache for resolved dependency graphs |
| Mela client | Real marketplace API (replaces the registry stub) |
