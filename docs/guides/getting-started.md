# Getting started

nous is a Cyrius library (consumed by **ark**) that resolves package
dependencies across system (apt/dpkg), marketplace, and Flutter sources. It also
builds a small smoke binary.

## Prerequisites

- **Cyrius 6.0.1** (pinned in `cyrius.cyml`; the toolchain is the single source
  of truth). Install from the [cyrius releases](https://github.com/MacCracken/cyrius/releases).
- `lib/` (the stdlib) is **not** vendored — `cyrius` resolves it from the
  version-pinned snapshot (`~/.cyrius/versions/6.0.1/lib/`). Don't commit `lib/`.

## Build & run the smoke binary

```sh
mkdir -p build
CYRIUS_DCE=1 cyrius build src/main.cyr build/nous
./build/nous            # prints the version banner + "all smoke checks passed"
```

## Test, fuzz, benchmark

```sh
cyrius test tests/nous.tcyr     # 271 assertions, expect "0 failed"
cyrius test tests/nous.fcyr     # fuzz harness
sh scripts/bench-history.sh local   # benchmarks (appends to bench-history.csv)
```

## Cleanliness (mirror of CI)

Run in this order — a clean local pass green-lights CI:

```sh
for f in src/*.cyr tests/*.tcyr tests/*.bcyr tests/*.fcyr; do cyrius fmt "$f" --check; done  # 0 = formatted
for f in src/*.cyr tests/*.tcyr tests/*.bcyr tests/*.fcyr; do cyrius lint "$f"; done          # no "warn " lines
cyrius vet src/main.cyr
cyrius deny src/main.cyr        # project policy (0 violations)
cyrius doc --check src/nous.cyr # barrel doc check
```

## The `dist/nous.cyr` bundle

`dist/nous.cyr` is nous's consumer-facing single-file artifact — a concatenation
of the `[lib] modules` produced by `cyrius distlib`. Regenerate it after any
`src/` change (CI fails if it drifts):

```sh
cyrius distlib                  # rewrites dist/nous.cyr
git diff --exit-code dist/nous.cyr   # must be in sync
```

## Consuming nous (for ark and other downstreams)

Pull the bundle as a single dependency module (preferred), supplying the stdlib
surface yourself (the bundle does not embed stdlib):

```toml
# in the consumer's cyrius.cyml
[deps.nous]
git = "https://github.com/MacCracken/nous.git"
tag = "1.2.4"
modules = ["dist/nous.cyr"]
```

Then `include` it, and use the public API (see [`docs/api.md`](../api.md)):
`resolver_new`, `resolver_resolve`, `resolver_resolve_all`, `resolver_search`,
`dep_graph_detect_cycle`, `dep_graph_topo_sort`, … Fallible calls return
`Result` (`Ok`/`Err`) — branch with `is_err_result` and unwrap with `payload`.
