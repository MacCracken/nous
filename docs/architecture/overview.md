# Architecture Overview

## Module Map

```
src/
  lib.rs              Main library — types, NousResolver, SystemPackageDb
  error.rs            Typed error handling (NousError, NousErrorKind)
  registry_stub.rs    Marketplace registry (local filesystem, placeholder for mela)
```

## Data Flow

```
                  NousResolver
                  /     |     \
                 /      |      \
    SystemPackageDb  LocalRegistry  (Flutter via registry)
         |              |
    apt-cache       filesystem
    dpkg-query      manifest.json
```

1. Consumer (ark) calls `NousResolver::resolve(name)` or `search(query)`
2. NousResolver applies the configured `ResolutionStrategy` to determine source order
3. Each source is queried in order:
   - **System**: shells out to `apt-cache` / `dpkg-query`
   - **Marketplace**: reads `installed/<pkg>/manifest.json` from the registry directory
   - **Flutter**: checks marketplace packages with `runtime: "flutter"`
4. First match (or all matches for `SearchAll`) is returned as `ResolvedPackage`

## Consumers

- **ark** — the AGNOS package manager CLI. Nous is the single source of truth for resolving package dependencies.

## Key Design Decisions

- **No network access**: all resolution is local. Remote registry sync is a consumer responsibility.
- **Strategy pattern**: resolution order is configurable per-call, not hardcoded.
- **Source-agnostic types**: `ResolvedPackage` / `InstalledPackage` are the same regardless of source.
- **Stub registry**: `LocalRegistry` is intentionally minimal — it will be replaced by the real mela marketplace client.

## Future Modules (planned)

See [gaps.md](../development/gaps.md) for prioritized backlog.

| Module | Purpose |
|---|---|
| `graph.rs` | Dependency graph, topological sort, cycle detection |
| `strategy.rs` | Extracted resolution strategy logic |
| `system.rs` | Extracted SystemPackageDb |
| `recipe.rs` | Zugot build recipe parsing |
| `cache.rs` | Resolution cache |
| `registry.rs` | Mela marketplace client (replaces registry_stub) |
