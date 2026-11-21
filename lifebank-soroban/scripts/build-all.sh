#!/bin/bash

set -e

echo "🔨 Building all Lifebank contracts..."
echo ""

# Build in dependency order so that `contractimport!` consumers always see
# freshly built WASMs. The `interfaces` crate holds all shared cross-contract
# types and clients and must be built first; the coordinator depends on every
# domain contract and is therefore built last.

TARGET_DIR="target/wasm32-unknown-unknown/release"

# 1. Shared interface crate (no WASM artifact, but must compile first)
echo "📚 Building shared interfaces crate..."
cargo build --release --target wasm32-unknown-unknown -p interfaces

# 2. Domain contracts (in dependency order)
DOMAIN_CRATES=(
    "requests"
    "inventory"
    "payments"
    "identity"
    "matching"
    "temperature"
    "delivery"
    "reputation"
    "analytics"
)

for crate in "${DOMAIN_CRATES[@]}"; do
    echo "🔨 Building ${crate} contract..."
    cargo build --release --target wasm32-unknown-unknown -p "${crate}"
done

# 3. Coordinator last — it imports the domain contract WASMs
echo "🔨 Building coordinator contract..."
cargo build --release --target wasm32-unknown-unknown -p coordinator

echo ""
echo "✅ Build complete!"
echo ""
echo "📦 Contract artifacts:"
echo "  - Requests:    ${TARGET_DIR}/requests_contract.wasm"
echo "  - Inventory:   ${TARGET_DIR}/inventory_contract.wasm"
echo "  - Payments:    ${TARGET_DIR}/payments_contract.wasm"
echo "  - Identity:    ${TARGET_DIR}/identity_contract.wasm"
echo "  - Matching:    ${TARGET_DIR}/matching_contract.wasm"
echo "  - Temperature: ${TARGET_DIR}/temperature_contract.wasm"
echo "  - Delivery:    ${TARGET_DIR}/delivery_contract.wasm"
echo "  - Reputation:  ${TARGET_DIR}/reputation_contract.wasm"
echo "  - Analytics:   ${TARGET_DIR}/analytics_contract.wasm"
echo "  - Coordinator: ${TARGET_DIR}/coordinator_contract.wasm"
echo ""

# Optional: Optimize WASM files
if command -v soroban &> /dev/null; then
    echo "🔧 Optimizing WASM files..."

    ALL_CONTRACTS=(
        "requests"
        "inventory"
        "payments"
        "identity"
        "matching"
        "temperature"
        "delivery"
        "reputation"
        "analytics"
        "coordinator"
    )

    for contract in "${ALL_CONTRACTS[@]}"; do
        soroban contract optimize \
            --wasm "${TARGET_DIR}/${contract}_contract.wasm"
    done

    echo "✅ Optimization complete!"
fi