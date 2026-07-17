#!/usr/bin/env ts-node
/**
 * generate-soroban-types.ts
 *
 * Reads the Soroban contract spec JSON produced by
 *   stellar contract inspect --wasm <wasm> --output json
 * and emits TypeScript type definitions that match the on-chain XDR layout.
 *
 * This is the single source of truth for shared types across the three layers:
 *   Rust contracts (crates/interfaces) → this script → backend TypeScript
 *
 * Usage:
 *   ts-node backend/scripts/generate-soroban-types.ts \
 *     --spec-dir lifebank-soroban/spec-snapshots \
 *     --out backend/src/__contracts__/soroban-types.gen.ts
 *
 * Add to package.json scripts:
 *   "generate:types": "ts-node scripts/generate-soroban-types.ts ..."
 */

import * as fs from 'fs';
import * as path from 'path';

// ── CLI args ──────────────────────────────────────────────────────────────────

const args = process.argv.slice(2);
const specDirIdx = args.indexOf('--spec-dir');
const outIdx = args.indexOf('--out');

const specDir = specDirIdx >= 0 ? args[specDirIdx + 1] : 'lifebank-soroban/spec-snapshots';
const outFile = outIdx >= 0 ? args[outIdx + 1] : 'backend/src/__contracts__/soroban-types.gen.ts';

// ── Spec type shapes (subset of stellar contract inspect JSON output) ─────────

interface SpecEntry {
  type: 'UDTEnumV0' | 'UDTStructV0' | 'FunctionV0' | string;
  name?: string;
  cases?: Array<{ name: string; value: number }>;
  fields?: Array<{ name: string; type: SpecType }>;
}

type SpecType =
  | { type: 'U64' }
  | { type: 'I128' }
  | { type: 'U32' }
  | { type: 'Bool' }
  | { type: 'String' }
  | { type: 'Address' }
  | { type: 'Option'; valueType: SpecType }
  | { type: 'Vec'; elementType: SpecType }
  | { type: 'Map'; keyType: SpecType; valueType: SpecType }
  | { type: 'Custom'; name: string }
  | { type: string };

// ── Code generation ───────────────────────────────────────────────────────────

function specTypeToTs(t: SpecType): string {
  switch (t.type) {
    case 'U64':
    case 'U32':
      return 'bigint';
    case 'I128':
      return 'bigint';
    case 'Bool':
      return 'boolean';
    case 'String':
      return 'string';
    case 'Address':
      return 'string';
    case 'Option':
      return `${specTypeToTs((t as any).valueType)} | null`;
    case 'Vec':
      return `Array<${specTypeToTs((t as any).elementType)}>`;
    case 'Map':
      return `Map<${specTypeToTs((t as any).keyType)}, ${specTypeToTs((t as any).valueType)}>`;
    case 'Custom':
      return (t as any).name;
    default:
      return 'unknown';
  }
}

function generateEnum(entry: SpecEntry): string {
  const cases = (entry.cases ?? [])
    .map((c) => `  ${c.name} = ${c.value},`)
    .join('\n');
  return `export enum ${entry.name} {\n${cases}\n}`;
}

function generateStruct(entry: SpecEntry): string {
  const fields = (entry.fields ?? [])
    .map((f) => `  ${f.name}: ${specTypeToTs(f.type)};`)
    .join('\n');
  return `export interface ${entry.name} {\n${fields}\n}`;
}

// ── Main ──────────────────────────────────────────────────────────────────────

function main() {
  if (!fs.existsSync(specDir)) {
    console.error(`Spec directory not found: ${specDir}`);
    console.error('Run  lifebank-soroban/scripts/update-snapshots.sh  first.');
    process.exit(1);
  }

  const files = fs.readdirSync(specDir).filter((f) => f.endsWith('.json'));
  if (files.length === 0) {
    console.error(`No .json spec files found in ${specDir}`);
    process.exit(1);
  }

  // Collect all unique type entries across all contracts (deduplicate by name).
  const seen = new Set<string>();
  const lines: string[] = [
    '// AUTO-GENERATED — do not edit by hand.',
    '// Source: lifebank-soroban/crates/interfaces + stellar contract inspect',
    '// Regenerate: npm run generate:types',
    '// Shared cross-contract types for the Lifebank Soroban workspace.',
    '',
  ];

  for (const file of files.sort()) {
    const raw = fs.readFileSync(path.join(specDir, file), 'utf8');
    let spec: { entries?: SpecEntry[] };
    try {
      spec = JSON.parse(raw);
    } catch {
      console.warn(`Skipping ${file}: invalid JSON`);
      continue;
    }

    for (const entry of spec.entries ?? []) {
      if (!entry.name || seen.has(entry.name)) continue;
      seen.add(entry.name);

      if (entry.type === 'UDTEnumV0') {
        lines.push(generateEnum(entry), '');
      } else if (entry.type === 'UDTStructV0') {
        lines.push(generateStruct(entry), '');
      }
    }
  }

  if (lines.length <= 5) {
    // Stellar CLI was not available; emit the hand-written fallback types
    // that mirror crates/interfaces/src/types.rs exactly.
    lines.push(...FALLBACK_TYPES);
  }

  fs.mkdirSync(path.dirname(outFile), { recursive: true });
  fs.writeFileSync(outFile, lines.join('\n'));
  console.log(`✅ Generated ${outFile}  (${seen.size} types from ${files.length} contracts)`);
}

// ── Fallback: hand-written types mirroring crates/interfaces/src/types.rs ────
// Used when stellar-cli is not installed (e.g. local dev without the toolchain).
// MUST be kept in sync with the Rust source — the spec-drift CI job catches drift.

const FALLBACK_TYPES = `
// ── Blood classification ──────────────────────────────────────────────────────

/** APPEND-ONLY: never remove or reorder variants. */
export enum BloodType {
  APositive = 0,
  ANegative = 1,
  BPositive = 2,
  BNegative = 3,
  ABPositive = 4,
  ABNegative = 5,
  OPositive = 6,
  ONegative = 7,
}

/** APPEND-ONLY: never remove or reorder variants. */
export enum BloodComponent {
  WholeBlood = 0,
  RedCells = 1,
  Plasma = 2,
  Platelets = 3,
  Cryoprecipitate = 4,
}

/** APPEND-ONLY: never remove or reorder variants. */
export enum BloodStatus {
  Available = 0,
  Reserved = 1,
  InTransit = 2,
  Delivered = 3,
  Expired = 4,
  Compromised = 5,
  Disposed = 6,
}

export interface BloodUnit {
  id: bigint;
  blood_type: BloodType;
  quantity_ml: number;
  bank_id: string;
  donor_id: string | null;
  donation_timestamp: bigint;
  expiration_timestamp: bigint;
  status: BloodStatus;
  metadata: Map<string, string>;
}

// ── Request types ─────────────────────────────────────────────────────────────

/** APPEND-ONLY: never remove or reorder variants. */
export enum Urgency {
  Critical = 0,
  Urgent = 1,
  Routine = 2,
  Scheduled = 3,
}

/** APPEND-ONLY: never remove or reorder variants. */
export enum RequestStatus {
  Pending = 0,
  Approved = 1,
  Fulfilled = 2,
  Cancelled = 3,
}

export interface BloodRequest {
  id: bigint;
  hospital_id: string;
  blood_type: BloodType;
  component: BloodComponent;
  quantity_ml: number;
  urgency: Urgency;
  created_timestamp: bigint;
  required_by_timestamp: bigint;
  status: RequestStatus;
  assigned_units: Array<bigint>;
  fulfilled_quantity_ml: number;
  reservation_id: bigint | null;
}

// ── Payment types ─────────────────────────────────────────────────────────────

/** APPEND-ONLY: never remove or reorder variants. */
export enum PaymentStatus {
  Pending = 0,
  Locked = 1,
  Released = 2,
  Refunded = 3,
  Disputed = 4,
  Cancelled = 5,
}

/** APPEND-ONLY: never remove or reorder variants. */
export enum DisputeReason {
  FailedDelivery = 0,
  TemperatureExcursion = 1,
  PaymentContested = 2,
  WrongItem = 3,
  DamagedGoods = 4,
  LateDelivery = 5,
  Other = 6,
}

export interface Payment {
  id: bigint;
  request_id: bigint;
  payer: string;
  payee: string;
  amount: bigint;
  status: PaymentStatus;
  created_at: bigint;
  updated_at: bigint;
  dispute_reason_code: number | null;
  dispute_case_id: string | null;
  dispute_resolved: boolean;
  token: string | null;
}
`.trimStart().split('\n');

main();
