# Nous

**Nous** (Greek: νοῦς — intellect, the faculty that apprehends first principles directly) — Package resolver for AGNOS.

The mind that figures out where packages come from. Given a package name, nous determines the source, resolves dependencies, and returns a resolution plan. It does **not** execute installs — that is [ark](https://github.com/MacCracken/ark)'s job.

## What Nous Does

Nous is the intelligence layer between ark (the CLI) and the package sources. When a user runs `ark install foo`, ark asks nous: "where does `foo` come from, what does it depend on, and is it trusted?"

```
ark install foo
    └── nous.resolve("foo")
            ├── check marketplace (mela) → found? return it
            ├── check system packages    → found? return it
            ├── check community (bazaar) → found? return it
            └── not found → PackageSource::Unknown
```

## Resolution Strategy

Nous supports configurable resolution order:

| Strategy | Order | Use Case |
|----------|-------|----------|
| **MarketplaceFirst** (default) | Marketplace → System → Community | Prefer AGNOS-native packages |
| **SystemFirst** | System → Marketplace → Community | Prefer OS-level packages |
| **MarketplaceOnly** | Marketplace only | Strict AGNOS ecosystem |
| **SystemOnly** | System only | Traditional package management |

## Core Types

| Type | Description |
|------|-------------|
| `NousResolver` | Main resolver engine — wraps system DB + marketplace registry |
| `ResolvedPackage` | Resolution result (name, version, source, deps, trust status, size) |
| `PackageSource` | Where a package comes from (System, Marketplace, FlutterApp, Community, Unknown) |
| `ResolutionStrategy` | Resolution order preference |
| `UnifiedSearchResult` | Cross-source search results |
| `InstalledPackage` | Currently installed package metadata |
| `AvailableUpdate` | Package with a newer version available |
| `SystemPackageDb` | Interface to system-level package database |

## API

```rust
use nous::{NousResolver, ResolutionStrategy};

let resolver = NousResolver::new(&marketplace_dir, &cache_dir)
    .with_strategy(ResolutionStrategy::MarketplaceFirst);

// Resolve a single package
let package = resolver.resolve("hoosh")?;

// Search across all sources
let results = resolver.search("audio")?;

// List installed packages
let installed = resolver.list_installed()?;

// Check for updates
let updates = resolver.check_updates()?;

// Detect source heuristically
let source = NousResolver::detect_source("firefox-esr"); // → System
let source = NousResolver::detect_source("hoosh");        // → Marketplace
```

## Package Source Detection

Nous uses heuristics to detect package source when no explicit source is provided:

- Names matching known AGNOS crates/apps → **Marketplace**
- Names matching common system packages (lib*, *-dev, *-bin) → **System**
- Names matching Flutter/app patterns → **FlutterApp**
- Community/bazaar prefix → **Community**

## Dependencies

| Crate | Purpose |
|-------|---------|
| anyhow | Error handling |
| serde / serde_json | Serialization |
| tracing | Structured logging |
| chrono | Timestamps |

## Related

- [ark](https://github.com/MacCracken/ark) — Package manager CLI (consumes nous for resolution)
- [takumi](https://github.com/MacCracken/takumi) — Build system (builds packages from recipes)
- [zugot](https://github.com/MacCracken/zugot) — Recipe repository (package definitions)
- [mela](https://github.com/MacCracken/mela) — Marketplace (package discovery and distribution)
- [sigil](https://github.com/MacCracken/sigil) — Trust verification (package signing)
- [AGNOS Philosophy](https://github.com/MacCracken/agnosticos/blob/main/docs/philosophy.md)

## License

GPL-3.0-only
