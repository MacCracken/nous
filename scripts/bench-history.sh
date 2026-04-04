#!/usr/bin/env bash
# bench-history.sh — Run criterion benchmarks and archive results.
#
# Usage:
#   ./scripts/bench-history.sh          # run all benchmarks
#   ./scripts/bench-history.sh --save   # run and save baseline with git hash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
HISTORY_DIR="$PROJECT_ROOT/target/bench-history"

cd "$PROJECT_ROOT"

echo "=== Running benchmarks ==="
cargo bench 2>&1 | tee /dev/stderr | grep -E '(time:|bench)' || true

if [[ "${1:-}" == "--save" ]]; then
    mkdir -p "$HISTORY_DIR"
    HASH=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
    DATE=$(date +%Y%m%d-%H%M%S)
    BASELINE="$HISTORY_DIR/${DATE}_${HASH}.txt"
    cargo bench -- --output-format=bencher 2>/dev/null | tee "$BASELINE"
    echo ""
    echo "Baseline saved to: $BASELINE"
fi
