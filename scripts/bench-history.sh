#!/usr/bin/env bash
# bench-history.sh — run cyrius benchmarks and archive results.
#
# Usage:
#   ./scripts/bench-history.sh                  # run all benchmarks
#   ./scripts/bench-history.sh <label>          # run + append to bench-history.csv
#   ./scripts/bench-history.sh --save           # save baseline tagged with git hash
#
# CI invokes the labelled form with `ci-<short-sha>`.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
HISTORY_DIR="$PROJECT_ROOT/build/bench-history"
HISTORY_CSV="$PROJECT_ROOT/bench-history.csv"
BENCH_FILE="tests/nous.bcyr"

cd "$PROJECT_ROOT"

if ! command -v cyrius >/dev/null 2>&1; then
    echo "error: cyrius not on PATH (expected from \$CYRIUS_HOME/bin)" >&2
    exit 1
fi

ARG="${1:-}"

echo "=== Running cyrius bench $BENCH_FILE ==="
OUTPUT=$(cyrius bench "$BENCH_FILE" 2>&1)
echo "$OUTPUT"

# Extract numeric "avg" lines for archival (one per benchmark).
SUMMARY=$(echo "$OUTPUT" | grep -E 'avg' || true)

if [[ "$ARG" == "--save" ]]; then
    mkdir -p "$HISTORY_DIR"
    HASH=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
    DATE=$(date +%Y%m%d-%H%M%S)
    BASELINE="$HISTORY_DIR/${DATE}_${HASH}.txt"
    echo "$SUMMARY" | tee "$BASELINE"
    echo
    echo "Baseline saved to: $BASELINE"
elif [[ -n "$ARG" ]]; then
    # Labelled CI run: append a single row to bench-history.csv.
    # Columns: timestamp,label,bench_line
    [ -f "$HISTORY_CSV" ] || echo "timestamp,label,bench" > "$HISTORY_CSV"
    TS=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    while IFS= read -r line; do
        [ -n "$line" ] || continue
        # Escape commas in the bench line for CSV safety.
        printf '%s,%s,"%s"\n' "$TS" "$ARG" "${line//\"/\"\"}" >> "$HISTORY_CSV"
    done <<< "$SUMMARY"
    echo
    echo "Appended $(echo "$SUMMARY" | grep -c '^') row(s) to $HISTORY_CSV"
fi
