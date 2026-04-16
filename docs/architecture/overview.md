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
  nous.tcyr           Test suite (140 assertions, 40 groups)
  nous.bcyr           Benchmarks (11 benches)
  nous.fcyr           Fuzz harnesses (3 harnesses)

```

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

See [roadmap.md](../development/roadmap.md) for prioritized backlog.

| Module | Purpose |
|---|---|
| Version constraints | SemVer parsing, constraint matching (>=, ^, ~, =) |
| Dependency graph | Graph construction, transitive resolution |
| Topological sort | Install-order computation via Kahn's algorithm |
| Cycle/conflict detection | DFS coloring for cycles, constraint intersection for conflicts |
| Zugot integration | Recipe parsing, build-order awareness |
| Resolution cache | Persistent cache for resolved dependency graphs |
| Mela client | Real marketplace API (replaces registry stub) |
