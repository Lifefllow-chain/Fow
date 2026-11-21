#!/bin/bash
#
# build-all.sh — build every Lifebank contract in dependency order.
#
# Build order:
#   1. crates/interfaces   — shared types & client traits (no WASM output)
#   2. contracts/*         — domain contracts that depend on interfaces
#
# The interfaces crate is a plain lib so cargo resolves it automatically;
# we list it first to make the dependency order explicit and to validate it
# compiles cleanly before any contract begins.

set -euo pipefail
cd "$(dirname "$0")/.."

echo "🔨 Building lifebank-interfaces (shared types & client traits)..."
cargo build -p lifebank-interfaces

echo ""
echo "🔨 Building all contract WASMs (release, wasm32v1-none)..."
cargo build --release --target wasm32v1-none \
  -p analytics-contract \
  -p coordinator-contract \
  -p delivery-contract \
  -p identity-contract \
  -p inventory-contract \
  -p matching-contract \
  -p payment-contract \
  -p reputation-contract \
  -p request-contract \
  -p temperature-contract

echo ""
echo "✅ Build complete."
echo ""
echo "📦 Contract artifacts (target/wasm32v1-none/release/):"
for name in \
  analytics_contract coordinator_contract delivery_contract identity_contract \
  inventory_contract matching_contract payment_contract reputation_contract \
  request_contract temperature_contract; do
  wasm="target/wasm32v1-none/release/${name}.wasm"
  [ -f "$wasm" ] && echo "  ${name}.wasm  ($(wc -c < "$wasm") bytes)"
done

echo ""
if command -v stellar &>/dev/null; then
  echo "🔧 Optimising WASMs with stellar contract optimize..."
  for name in \
    analytics_contract coordinator_contract delivery_contract identity_contract \
    inventory_contract matching_contract payment_contract reputation_contract \
    request_contract temperature_contract; do
    wasm="target/wasm32v1-none/release/${name}.wasm"
    [ -f "$wasm" ] && stellar contract optimize --wasm "$wasm"
  done
  echo "✅ Optimisation complete."
fi
