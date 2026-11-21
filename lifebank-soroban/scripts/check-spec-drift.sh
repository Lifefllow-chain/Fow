#!/usr/bin/env bash
#
# check-spec-drift.sh — CI spec-snapshot drift check
#
# Builds every contract WASM, extracts each contract's spec (XDR) via
# `stellar contract inspect`, and diffs it against the committed snapshot in
# `lifebank-soroban/specs/<contract>.xdr`.
#
# Any difference fails CI. This makes cross-contract interface changes
# (added/removed/reordered enum variants, field reordering, signature changes)
# visible at review time instead of surfacing as runtime mis-decoding.
#
# Usage:
#   scripts/check-spec-drift.sh            # verify snapshots (CI mode)
#   scripts/check-spec-drift.sh --update   # rewrite committed snapshots
#
# Environment:
#   SKIP_BUILD=1   skip running build-all.sh (assume WASMs already built)
#   STELLAR_BIN    override the stellar CLI binary (default: stellar)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
SPECS_DIR="${ROOT_DIR}/specs"
WASM_DIR="${ROOT_DIR}/target/wasm32-unknown-unknown/release"

UPDATE=0
for arg in "$@"; do
  case "${arg}" in
    --update) UPDATE=1 ;;
    -h|--help)
      sed -n '2,20p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *)
      echo "error: unknown argument '${arg}'" >&2
      exit 2
      ;;
  esac
done

# Contracts whose specs are snapshotted. Keep in sync with build-all.sh.
CONTRACTS=(
  "requests"
  "inventory"
  "payments"
  "registry"
  "coordinator"
)

# ---------------------------------------------------------------------------
# Locate the stellar CLI (fall back to soroban CLI for older toolchains).
# ---------------------------------------------------------------------------
resolve_cli() {
  local bin="${STELLAR_BIN:-stellar}"
  if command -v "${bin}" >/dev/null 2>&1; then
    echo "${bin}"
    return 0
  fi
  if command -v soroban >/dev/null 2>&1; then
    echo "soroban"
    return 0
  fi
  echo "error: neither 'stellar' nor 'soroban' CLI found on PATH" >&2
  echo "       install with: cargo install --locked stellar-cli" >&2
  exit 1
}

CLI="$(resolve_cli)"

# ---------------------------------------------------------------------------
# Build all WASMs in dependency order.
# ---------------------------------------------------------------------------
if [[ "${SKIP_BUILD:-0}" != "1" ]]; then
  echo "==> Building all contract WASMs (dependency order)..."
  "${SCRIPT_DIR}/build-all.sh"
else
  echo "==> SKIP_BUILD=1: assuming WASMs are already built"
fi

# ---------------------------------------------------------------------------
# Extract the spec XDR for a single WASM.
# ---------------------------------------------------------------------------
extract_spec() {
  local wasm="$1"
  # `stellar contract inspect` emits the contract spec as base64-encoded XDR.
  # Older `soroban` CLI uses the same subcommand shape.
  "${CLI}" contract inspect --wasm "${wasm}" --output xdr-base64
}

# ---------------------------------------------------------------------------
# Verify (or update) snapshots.
# ---------------------------------------------------------------------------
mkdir -p "${SPECS_DIR}"

DRIFT=0
MISSING=0
declare -a DRIFTED=()
declare -a MISSING_FILES=()

for contract in "${CONTRACTS[@]}"; do
  wasm="${WASM_DIR}/${contract}.wasm"
  snapshot="${SPECS_DIR}/${contract}.xdr"

  if [[ ! -f "${wasm}" ]]; then
    echo "error: expected WASM not found: ${wasm}" >&2
    echo "       did build-all.sh produce this artifact?" >&2
    exit 1
  fi

  fresh="$(extract_spec "${wasm}")"

  if [[ "${UPDATE}" -eq 1 ]]; then
    printf '%s\n' "${fresh}" > "${snapshot}"
    echo "==> updated snapshot: specs/${contract}.xdr"
    continue
  fi

  if [[ ! -f "${snapshot}" ]]; then
    MISSING=$((MISSING + 1))
    MISSING_FILES+=("${contract}")
    continue
  fi

  if ! diff -u "${snapshot}" <(printf '%s\n' "${fresh}") >/dev/null 2>&1; then
    DRIFT=$((DRIFT + 1))
    DRIFTED+=("${contract}")
  fi
done

if [[ "${UPDATE}" -eq 1 ]]; then
  echo "==> Snapshots updated. Commit the changes in specs/ explicitly."
  exit 0
fi

# ---------------------------------------------------------------------------
# Report.
# ---------------------------------------------------------------------------
if [[ "${MISSING}" -gt 0 ]]; then
  echo "" >&2
  echo "error: missing spec snapshot(s) for: ${MISSING_FILES[*]}" >&2
  echo "       generate them with: scripts/check-spec-drift.sh --update" >&2
fi

if [[ "${DRIFT}" -gt 0 ]]; then
  echo "" >&2
  echo "error: contract spec drift detected for: ${DRIFTED[*]}" >&2
  echo "" >&2
  for contract in "${DRIFTED[@]}"; do
    echo "--- diff for ${contract} ---" >&2
    diff -u "${SPECS_DIR}/${contract}.xdr" \
      <(extract_spec "${WASM_DIR}/${contract}.wasm") >&2 || true
    echo "" >&2
  done
  echo "A cross-contract interface changed. This is a breaking change for any" >&2
  echo "caller (including the coordinator and the backend contract-types)." >&2
  echo "" >&2
  echo "If the change is intentional:" >&2
  echo "  1. Confirm contracttype enums used across boundaries are append-only" >&2
  echo "     (removing/reordering variants requires a schema-version bump)." >&2
  echo "  2. Regenerate snapshots: scripts/check-spec-drift.sh --update" >&2
  echo "  3. Regenerate backend TS types from the new spec XDR." >&2
  echo "  4. Commit the updated specs/ snapshots explicitly in this PR." >&2
fi

if [[ "${DRIFT}" -gt 0 || "${MISSING}" -gt 0 ]]; then
  exit 1
fi

echo "==> No spec drift detected (${#CONTRACTS[@]} contracts checked)."