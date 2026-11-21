//! Canonical cross-contract type definitions for the LifeBank Soroban contracts.
//!
//! Every struct/enum that crosses a contract boundary is defined here exactly
//! once. Domain contracts and the coordinator import these types instead of
//! re-declaring them, so the compiler enforces a single source of truth.
//!
//! # Enum discipline (append-only)
//!
//! `#[contracttype]` enums are encoded by their variant discriminant. Adding a
//! variant at the end is safe; removing or reordering variants shifts the
//! discriminants of every later variant and silently corrupts cross-contract
//! decoding. Cross-contract enums in this crate are therefore **append-only**:
//! never remove or reorder variants. Breaking changes require a schema-version
//! bump (see #31) and an explicit spec-snapshot update in the PR.

use soroban_sdk::{contracttype, Address, BytesN, String, Symbol, Vec};

// ---------------------------------------------------------------------------
// Requests
// ---------------------------------------------------------------------------

/// Lifecycle status of a blood request.
///
/// Append-only: do not remove or reorder variants.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequestStatus {
    Pending,
    Approved,
    Fulfilled,
    Cancelled,
    Expired,
}

/// A blood request submitted by a hospital.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BloodRequest {
    pub id: u64,
    pub requester: Address,
    pub blood_type: BloodType,
    pub quantity: u32,
    pub status: RequestStatus,
    pub created_at: u64,
    pub updated_at: u64,
    pub deadline: u64,
    pub fulfilled_by: Option<Address>,
}

// ---------------------------------------------------------------------------
// Blood inventory
// ---------------------------------------------------------------------------

/// ABO/Rh blood type.
///
/// Append-only: do not remove or reorder variants.
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

/// Lifecycle status of an individual blood unit.
///
/// Append-only: do not remove or reorder variants.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BloodStatus {
    Available,
    Reserved,
    InTransit,
    Delivered,
    Expired,
    Discarded,
}

/// A single tracked unit of blood.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BloodUnit {
    pub id: u64,
    pub blood_type: BloodType,
    pub donor: Address,
    pub status: BloodStatus,
    pub collected_at: u64,
    pub expires_at: u64,
    pub reserved_by: Option<Address>,
}

// ---------------------------------------------------------------------------
// Payments
// ---------------------------------------------------------------------------

/// Lifecycle status of a payment.
///
/// Append-only: do not remove or reorder variants.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Escrowed,
    Released,
    Refunded,
    Disputed,
    Cancelled,
}

/// A payment record tied to a request.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Payment {
    pub id: u64,
    pub request_id: u64,
    pub payer: Address,
    pub payee: Address,
    pub amount: i128,
    pub token: Address,
    pub status: PaymentStatus,
    pub created_at: u64,
    pub updated_at: u64,
}

// ---------------------------------------------------------------------------
// Disputes
// ---------------------------------------------------------------------------

/// Reason a payment or request has been disputed.
///
/// Append-only: do not remove or reorder variants.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DisputeReason {
    NonDelivery,
    IncorrectQuantity,
    QualityIssue,
    UnauthorizedCharge,
    Other,
}

// ---------------------------------------------------------------------------
// Reservations
// ---------------------------------------------------------------------------

/// A reservation of blood units against a request.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Reservation {
    pub id: u64,
    pub request_id: u64,
    pub unit_ids: Vec<u64>,
    pub reserved_by: Address,
    pub created_at: u64,
    pub expires_at: u64,
}

// ---------------------------------------------------------------------------
// Workflow
// ---------------------------------------------------------------------------

/// Status of a cross-contract workflow.
///
/// Append-only: do not remove or reorder variants.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowStatus {
    Created,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// A record of a cross-contract workflow.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowRecord {
    pub id: u64,
    pub request_id: u64,
    pub status: WorkflowStatus,
    pub current_step: Symbol,
    pub created_at: u64,
    pub updated_at: u64,
}

// ---------------------------------------------------------------------------
// Cold-chain excursions
// ---------------------------------------------------------------------------

/// Summary of a temperature excursion affecting a shipment.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExcursionSummary {
    pub unit_id: u64,
    pub min_temp: i32,
    pub max_temp: i32,
    pub duration_secs: u64,
    pub detected_at: u64,
    pub resolved: bool,
}

// ---------------------------------------------------------------------------
// Credentials
// ---------------------------------------------------------------------------

/// Type of a verifiable credential.
///
/// Append-only: do not remove or reorder variants.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialType {
    DonorIdentity,
    HospitalLicense,
    BloodBankCertification,
    StaffAuthorization,
    Other,
}

/// A verifiable credential issued to an address.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Credential {
    pub id: u64,
    pub subject: Address,
    pub issuer: Address,
    pub credential_type: CredentialType,
    pub data_hash: BytesN<32>,
    pub issued_at: u64,
    pub expires_at: Option<u64>,
    pub revoked: bool,
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// A generic key/value metadata entry shared across contracts.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataEntry {
    pub key: Symbol,
    pub value: String,
}