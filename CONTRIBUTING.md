# Contributing to Nous

Thank you for your interest in contributing to Nous.

## Getting Started

```bash
git clone https://github.com/MacCracken/nous.git
cd nous
cargo test
cargo bench
```

### Requirements

- Rust stable (MSRV 1.89)
- `cargo-audit` and `cargo-deny` for security checks

## Development Process

Follow the work loop defined in [CLAUDE.md](CLAUDE.md):

1. Make changes
2. Run cleanliness checks:
   ```bash
   cargo fmt --check
   cargo clippy --all-features --all-targets -- -D warnings
   cargo audit
   cargo deny check
   RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps
   ```
3. Add tests for new code
4. Run benchmarks: `./scripts/bench-history.sh`
5. Update CHANGELOG.md

## Code Standards

- `#[non_exhaustive]` on all public enums
- `#[must_use]` on all pure functions
- `Serialize` + `Deserialize` on all public types
- Serde roundtrip tests for every type
- Zero `unwrap`/`panic` in library code
- Deterministic resolution — same inputs, same outputs

## Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/) style. Keep messages concise and focused on *why*.

## License

By contributing, you agree that your contributions will be licensed under GPL-3.0-only.
