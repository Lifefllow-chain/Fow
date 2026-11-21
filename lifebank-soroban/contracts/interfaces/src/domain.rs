//! Canonical cross-contract domain types.
//!
//! Every type in this module is used across a contract boundary (returned from
//! or passed into a cross-contract call). The implementing contract and every
//! caller import the *same* definition from here — there is exactly one source
//! of truth, so a field reorder or enum-variant change is a compile error for
//! every consumer instead of a silent runtime mis-decode.
//!
//! # Append-only enum policy
//!
//! `#[contracttype]` enums are encoded by *variant index*. Reordering or
//! removing a variant silently shifts the discriminant of every later variant,
//! corrupting already-stored data and cross-contract return values. Therefore:
//!
//! * **Append new variants at the end only.**
//! * Never remove or reorder an existing variant.
//! * A breaking change requires a schema-version bump (see issue #31) and a
//!   coordinated migration.
//!
//! This policy is enforced by `scripts/check-spec-drift.sh` (spec snapshots)
//! and documented in `lifebank-soroban/docs/interface-compatibility.md`.

use soroban_sdk::{contracttype, Address, Map, String, Symbol, Vec};

// ── Requests domain ───────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BloodType {
    APositive,
    ANegative,
    BPositive,
    BNegative,
    ABPositive,
    ABNegative,
    OPositive,
    ONegative,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BloodComponent {
    WholeBlood,
    RedCells,
    Plasma,
    Platelets,
    Cryoprecipitate,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Urgency {
    Critical,
    Urgent,
    Routine,
    Scheduled,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequestStatus {
    Pending,
    Approved,
    Fulfilled,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BloodRequest {
    pub id: u64,
    pub hospital_id: Address,
    pub blood_type: BloodType,
    pub component: BloodComponent,
    pub quantity_ml: u32,
    pub urgency: Urgency,
    pub created_timestamp: u64,
    pub required_by_timestamp: u64,
    pub status: RequestStatus,
    pub assigned_units: Vec<u64>,
    pub fulfilled_quantity_ml: u32,
    pub reservation_id: Option<u64>,
}

// ── Inventory domain ──────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BloodStatus {
    Available,
    Reserved,
    InTransit,
    Delivered,
    Expired,
    Compromised,
    Disposed,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct BloodUnit {
    pub id: u64,
    pub blood_type: BloodType,
    pub quantity_ml: u32,
    pub bank_id: Address,
    pub donor_id: Option<Address>,
    pub donation_timestamp: u64,
    pub expiration_timestamp: u64,
    pub status: BloodStatus,
    pub metadata: Map<Symbol, String>,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Reservation {
    pub unit_ids: Vec<u64>,
    pub requester: Address,
    pub created_timestamp: u64,
    pub expiration_timestamp: u64,
    pub request_id: u64,
}

// ── Payments domain ───────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Locked,
    Released,
    Refunded,
    Disputed,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DisputeReason {
    FailedDelivery,
    TemperatureExcursion,
    PaymentContested,
    WrongItem,
    DamagedGoods,
    LateDelivery,
    Other,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Payment {
    pub id: u64,
    pub request_id: u64,
    pub payer: Address,
    pub payee: Address,
    pub amount: i128,
    pub status: PaymentStatus,
    pub created_at: u64,
    pub updated_at: u64,
    pub dispute_reason_code: Option<u32>,
    pub dispute_case_id: Option<String>,
    pub dispute_resolved: bool,
    pub token: Option<Address>,
}

// ── Identity domain ───────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialType {
    MedicalFacilityLicense,
    RiderCertification,
    BloodBankAccreditation,
    DonorEligibility,
    LabAccreditation,
}
