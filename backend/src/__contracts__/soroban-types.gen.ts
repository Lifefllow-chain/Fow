// AUTO-GENERATED — do not edit by hand.
// Source: lifebank-soroban/crates/interfaces + stellar contract inspect
// Regenerate: npm run generate:soroban-types
// Shared cross-contract types for the Lifebank Soroban workspace.

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
