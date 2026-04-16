# Benchmarks: Rust vs Cyrius

> **Date**: 2026-04-16
> **Machine**: Same host, sequential runs
> **Rust**: 1.93.0, Criterion 0.5, `cargo bench`
> **Cyrius**: 5.1.7, bench harness, `cyrius bench`

---

## Performance

Rust uses Criterion (statistical, reports median). Cyrius uses a timer loop (reports average and min).

| Benchmark | Rust (median) | Cyrius (min) | Ratio |
|-----------|--------------|-------------|-------|
| **detect_source** (6 names) | 55 ns | 907 ns | 16x |
| **resolve_mkt_hit** (50 pkgs) | 111 ns | 2 us | 18x |
| **resolve_mkt_miss** | 52 ns | 3 us | 58x |
| **search_100** | 20 us | 74 us | 4x |
| **list_installed_100** | 6.6 us | 13 us | 2x |
| **strat_mkt_first** | 70 ns | 2 us | 29x |
| **strat_sys_first** | 71 ns | 2 us | 28x |
| **strat_only_mkt** | 63 ns | 2 us | 32x |
| **strat_search_all** | 70 ns | 2 us | 29x |
| **serde_roundtrip** | 491 ns | 1 us | 2x |

The gap on micro-benchmarks (resolve, strategy) is allocation overhead — Cyrius bump-allocates every struct and string, Rust uses stack and borrows. The gap closes on bulk operations (search: 4x, list: 2x, serde: 2x) where real work dominates.

---

## Binary Size

| | Rust (release) | Cyrius |
|---|---|---|
| Binary | ~800 KB | 412 KB |
| With debug | ~2.8 MB | — |
| External crate deps | 5 | 0 |
| Compile time | ~8s | <1s |

---

## Lines of Code

| | Rust v0.1.0 | Cyrius v1.0.0 |
|---|---|---|
| Library | 1,362 | 2,444 |
| Tests | 781 | 1,317 |
| Benchmarks | 147 | 278 |
| **Total source** | 2,290 | 4,039 |

The Cyrius version is ~1.8x larger in lines. The expansion comes from manual struct layout (constructors + accessors that Rust derives), the CYML parser (stdlib toml.cyr doesn't handle `[sections]`), and P1/P2 features that didn't exist in Rust.
