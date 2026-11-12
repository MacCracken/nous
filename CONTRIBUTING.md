# Contributing to Nous

Thank you for your interest in contributing to Nous.

## Getting Started

```bash
git clone https://github.com/MacCracken/nous.git
cd nous
cyrius deps                      # vendors lib/ and bayan; needs network once
cyrius test tests/nous.tcyr      # expect "0 failed"
cyrius fuzz --poison=ab tests/   # 6 fuzz harnesses
./scripts/bench-history.sh       # benchmarks
```

### Requirements

- **Cyrius**, at the version pinned in `cyrius.cyml` (the single source of
  truth). To bump it, change the pin, then run `cyrius deps` on its own.
- `lib/` is vendored by `cyrius deps` and is not committed; `cyrius.lock` is.

[docs/guides/getting-started.md](docs/guides/getting-started.md) has the details.

## Development Process

Follow the work loop defined in [CLAUDE.md](CLAUDE.md):

1. Make changes
2. Run the cleanliness checks. This is the CI order, so a clean local run
   should green-light CI:
   ```bash
   for f in src/*.cyr tests/*.tcyr tests/*.bcyr tests/*.fcyr; do cyrius fmt "$f" --check; done  # exit 0 = formatted
   for f in src/*.cyr tests/*.tcyr tests/*.bcyr tests/*.fcyr; do cyrius lint "$f"; done          # no "warn " lines
   cyrius vet src/main.cyr
   cyrius deny src/main.cyr
   for f in src/*.cyr; do cyrius doc --check "$f"; done   # exit = undocumented public fns; must be 0
   cyrius distlib                                         # then commit dist/nous.cyr if it changed
   ```
3. Add tests for new code (`tests/nous.tcyr`; a fuzz harness for a new parser)
4. Run benchmarks: `./scripts/bench-history.sh`. Performance claims need numbers.
5. Update CHANGELOG.md

## Code Standards

- **A doc comment on every public function.** A public function is one whose
  name does not start with `_`. Its doc comment is the `#` line directly above
  it, or above its `#deprecated(...)` attribute. `cyrius doc` shows only that
  one line, so make it a self-contained summary: what it returns, in which
  type (`Str`, cstr, `vec of X`), and the 0 / `Err` cases. Put any history or
  rationale above it, separated by a `#` line.
- **Explicit return types:** `: Result` on any function that returns
  `Ok(..)` / `Err(..)`, directly or by returning another Result function's
  call; `: i64` on everything else. `cyrius header src/<module>.cyr` reports
  any function that returns a Result without declaring it.
- **Errors are Results.** Bind both halves (`var t, v = f();`) and re-emit both
  (`return Err(v);`). There are no panic or abort paths in library code.
- **JSON both ways** for every public type, in `src/json.cyr`, with a roundtrip
  test.
- **Deterministic resolution.** Never let hash order (`map_keys`) reach output;
  use `_nous_sorted_keys`.
- **The namespace is flat.** Give new public names a `nous_` prefix, and check
  sibling AGNOS repos for the name first.
- **Cite sources.** An algorithm or format rule cites its source in the code and
  in [docs/sources.md](docs/sources.md).

## Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/) style. Keep messages concise and focused on *why*.

## License

By contributing, you agree that your contributions will be licensed under GPL-3.0-only.
