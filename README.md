# Nous

**Nous** (Greek: νοῦς — intellect, the faculty that apprehends first principles directly) — Package resolver for AGNOS.

The mind that figures out where packages come from. Given a package name, nous determines the source, resolves dependencies, and returns a resolution plan. It does **not** execute installs — that is [ark](https://github.com/MacCracken/ark)'s job.

## What Nous Does

Nous is the intelligence layer between ark (the CLI) and the package sources. When a user runs `ark install foo`, ark asks nous: "where does `foo` come from, what does it depend on, and is it trusted?"

```
ark install foo
    └── resolver_resolve(r, "foo")
            ├── check marketplace (mela) → found? return it
            ├── check system packages    → found? return it
            ├── check flutter apps       → found? return it
            └── not found → SOURCE_UNKNOWN
```

## Resolution Strategy

Nous supports configurable resolution order:

| Strategy | Order | Use Case |
|----------|-------|----------|
| **MarketplaceFirst** (default) | Marketplace → Flutter → System | Prefer AGNOS-native packages |
| **SystemFirst** | System → Marketplace → Flutter | Prefer OS-level packages |
| **OnlySource** | Single source only | Strict source targeting |
| **SearchAll** | All sources, first match | Broadest resolution |

## Core Types

| Type | Description |
|------|-------------|
| Resolver | Main resolver engine — wraps system DB + marketplace registry |
| ResolvedPkg | Resolution result (name, version, source, deps, trust status, size) |
| PackageSource | Where a package comes from (System, Marketplace, FlutterApp, Community, Unknown) |
| Strategy | Resolution order preference |
| SearchResult | Cross-source search results |
| InstalledPkg | Currently installed package metadata |
| AvailUpdate | Package with a newer version available |
| SysDb | Interface to system-level package database (apt/dpkg) |

## API

```cyrius
include "src/nous.cyr"

fn main() {
    alloc_init();

    # Create resolver
    var r = payload(resolver_new(str_from("/var/lib/mela"), str_from("/var/cache/nous")));

    # Set strategy
    r = resolver_with_strategy(r, strategy_system_first());

    # Resolve a single package
    var res = resolver_resolve(r, str_from("hoosh"));
    if (is_ok(res) == 1) {
        var pkg = payload(res);
        if (pkg != 0) {
            str_println(rp_name(pkg));    # "hoosh"
            str_println(rp_ver(pkg));     # "1.0.0"
            fmt_int(rp_src(pkg));         # SOURCE_MARKETPLACE
        }
    }

    # Search across all sources
    var sr = payload(resolver_search(r, str_from("audio")));

    # List installed packages
    var list = payload(resolver_list(r));

    # Check for updates
    var upd = payload(resolver_updates(r));

    # Detect source heuristically
    detect_source(str_from("acme/scanner"));   # SOURCE_MARKETPLACE
    detect_source(str_from("com.example.app")); # SOURCE_FLUTTER_APP
    detect_source(str_from("nginx"));           # SOURCE_UNKNOWN

    return 0;
}
```

## Package Source Detection

Nous uses heuristics to detect package source when no explicit source is provided:

- Contains `/` (publisher/name format) → **Marketplace**
- Ends with `.flutter` or reverse-domain notation (com.example.app) → **FlutterApp**
- Otherwise → **Unknown** (resolved by strategy)

## Build

```bash
cyrius build src/main.cyr build/nous    # compile
cyrius test tests/nous.tcyr             # 140 tests
cyrius bench tests/nous.bcyr            # 11 benchmarks
```

Requires Cyrius 5.1.7. Binary: ~115KB x86_64 ELF.

## Related

- [ark](https://github.com/MacCracken/ark) — Package manager CLI (consumes nous for resolution)
- [takumi](https://github.com/MacCracken/takumi) — Build system (builds packages from recipes)
- [zugot](https://github.com/MacCracken/zugot) — Recipe repository (package definitions)
- [mela](https://github.com/MacCracken/mela) — Marketplace (package discovery and distribution)
- [sigil](https://github.com/MacCracken/sigil) — Trust verification (package signing)
- [AGNOS Philosophy](https://github.com/MacCracken/agnosticos/blob/main/docs/philosophy.md)

## License

GPL-3.0-only
