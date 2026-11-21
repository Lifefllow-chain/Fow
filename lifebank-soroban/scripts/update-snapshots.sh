#!/bin/bash
# update-snapshots.sh — regenerate the committed spec snapshots after an
# intentional interface change.  Run locally, review the diff, then include
# the updated files in your PR so CI passes.
set -euo pipefail
cd "$(dirname "$0")/.."

bash scripts/build-all.sh

mkdir -p spec-snapshots

for name in \
  analytics_contract coordinator_contract delivery_contract identity_contract \
  inventory_contract matching_contract payment_contract reputation_contract \
  request_contract temperature_contract; do
  wasm="target/wasm32v1-none/release/${name}.wasm"
  if [ -f "$wasm" ]; then
    if command -v stellar &>/dev/null; then
      stellar contract inspect --wasm "$wasm" --output json \
        > "spec-snapshots/${name}.json"
    else
      echo '{"error":"stellar-cli not available — install it to generate real snapshots"}' \
        > "spec-snapshots/${name}.json"
    fi
    echo "  updated spec-snapshots/${name}.json"
  fi
done

echo ""
echo "✅ Snapshots updated. Review the diff, then commit spec-snapshots/ in your PR."
