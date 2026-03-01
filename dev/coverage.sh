#!/usr/bin/env bash
set -euo pipefail

# ZeroClaw Coverage Measurement Script
# Usage: ./dev/coverage.sh [module]
# Examples:
#   ./dev/coverage.sh          # Full coverage report
#   ./dev/coverage.sh security # Coverage for security module only

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

# Ensure cargo-tarpaulin is installed
if ! cargo tarpaulin --version >/dev/null 2>&1; then
  echo "Installing cargo-tarpaulin..."
  cargo install cargo-tarpaulin
fi

MODULE="${1:-}"

if [ -n "$MODULE" ]; then
  echo "=== Coverage for module: $MODULE ==="
  cargo tarpaulin \
    --config tarpaulin.toml \
    --workspace \
    --files "src/${MODULE}/**" \
    --skip-clean \
    2>&1
else
  echo "=== Full workspace coverage ==="
  cargo tarpaulin \
    --config tarpaulin.toml \
    --workspace \
    --skip-clean \
    2>&1

  echo ""
  echo "=== Per-module summary ==="
  # Extract per-module coverage from JSON report if available
  REPORT="target/tarpaulin/tarpaulin-report.json"
  if [ -f "$REPORT" ] && command -v python3 >/dev/null 2>&1; then
    python3 -c "
import json, collections, os

with open('$REPORT') as f:
    data = json.load(f)

modules = collections.defaultdict(lambda: {'covered': 0, 'total': 0})

for path, info in data.get('files', {}).items():
    rel = os.path.relpath(path, '$REPO_ROOT')
    if not rel.startswith('src/'):
        continue
    parts = rel.split('/')
    module = parts[1] if len(parts) > 2 else 'root'
    covered = sum(1 for t in info.get('traces', []) if t.get('stats', {}).get('Line', 0) > 0)
    total = len(info.get('traces', []))
    modules[module]['covered'] += covered
    modules[module]['total'] += total

print(f'{'Module':<25} {'Coverage':>10} {'Lines':>10}')
print('-' * 47)
for mod in sorted(modules.keys()):
    m = modules[mod]
    pct = (m['covered'] / m['total'] * 100) if m['total'] > 0 else 0
    print(f'{mod:<25} {pct:>9.1f}% {m[\"covered\"]:>4}/{m[\"total\"]}')

total_c = sum(m['covered'] for m in modules.values())
total_t = sum(m['total'] for m in modules.values())
pct = (total_c / total_t * 100) if total_t > 0 else 0
print('-' * 47)
print(f'{'TOTAL':<25} {pct:>9.1f}% {total_c:>4}/{total_t}')
" 2>/dev/null || echo "(Install python3 for per-module breakdown)"
  fi
fi

echo ""
echo "HTML report: target/tarpaulin/tarpaulin-report.html"
