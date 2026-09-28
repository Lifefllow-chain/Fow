//! Types for the temperature oracle contract.
//!
//! These types back the oracle-authentication model described in issue #41:
//! registered device identities, integrity-checked readings, signed batches,
//! versioned per-product thresholds, and an evidence chain that lets an
//! arbiter reproduce a breach verdict off-chain from committed hashes alone.

use soroban_sdk::{contracttype, Address, BytesN, String, Vec};

/// Product classes that carry their own threshold configuration.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductClass {
    WholeBlood,
    Platelets,
    Plasma,
}

/// A registered sensor/gateway identity.
///
/// Readings are only accepted from the `submitter` registered for the device,
/// and only for the shipment the device is scoped to. `device_pubkey` is the
/// ed25519 key used to verify signed batches; it is optional so that a device
/// may be registered before its key is provisioned.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviceRecord {
    pub device_id: BytesN<32>,
    pub submitter: Address,
    pub device_pubkey: Option<BytesN<32>>,
    pub shipment_id: BytesN<32>,
    pub registered_at: u64,
    pub quarantined: bool,
    pub implausible_count: u32,
}

/// Per-device integrity state: monotonic sequence + last accepted timestamp.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviceState {
    pub last_seq: u64,
    pub last_timestamp: u64,
}

/// A single reading as submitted by a device.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Reading {
    pub seq: u64,
    pub timestamp: u64,
    /// Temperature in milli-degrees Celsius (signed).
    pub temperature_milli_c: i64,
}

/// A signed batch of readings buffered offline by a device.
///
/// The device signs `(device_id, seq_range, readings_hash)`; the contract
/// verifies the signature with `env.crypto().ed25519_verify` so a compromised
/// gateway cannot forge readings on behalf of another device.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedBatch {
    pub device_id: BytesN<32>,
    pub seq_start: u64,
    pub seq_end: u64,
    pub readings_hash: BytesN<32>,
    pub signature: BytesN<64>,
    pub readings: Vec<Reading>,
}

/// Versioned threshold configuration for a product class.
///
/// A shipment pins the config version active at its start, so threshold
/// changes mid-shipment cannot retroactively alter that shipment's verdict.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThresholdConfig {
    pub version: u32,
    pub product: ProductClass,
    pub min_milli_c: i64,
    pub max_milli_c: i64,
    /// Maximum tolerated excursion duration in seconds.
    pub max_excursion_secs: u64,
    pub effective_at: u64,
}

/// On-chain excursion state machine for a shipment.
///
/// Only the summary is persisted; raw readings live in temporary storage or
/// events. The settlement decision needs the summary, not every sample.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExcursionState {
    InRange,
    Excursion,
    Resolved,
}

/// Evidence chain committed on-chain so an arbiter can independently verify
/// the off-chain raw data against these commitments.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceChain {
    pub config_version: u32,
    /// Ordered readings-hash commitments from accepted batches.
    pub readings_hashes: Vec<BytesN<32>>,
    pub first_seq: u64,
    pub last_seq: u64,
}

/// Summary consumed by the coordinator/payments for dispute resolution.
///
/// Carries the pinned config version and the readings-hash chain so a breach
/// verdict is reproducible off-chain from events + committed hashes alone.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExcursionSummary {
    pub shipment_id: BytesN<32>,
    pub product: ProductClass,
    pub config_version: u32,
    pub state: ExcursionState,
    pub min_milli_c: i64,
    pub max_milli_c: i64,
    pub excursion_secs: u64,
    pub evidence: EvidenceChain,
}

/// Per-shipment oracle state: pinned thresholds + excursion summary.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipmentState {
    pub shipment_id: BytesN<32>,
    pub product: ProductClass,
    /// Threshold config version pinned at shipment start.
    pub config_version: u32,
    pub started_at: u64,
    pub state: ExcursionState,
    pub min_milli_c: i64,
    pub max_milli_c: i64,
    pub excursion_secs: u64,
    pub evidence: EvidenceChain,
}

/// Event payload emitted once per accepted signed batch.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchAcceptedEvent {
    pub device_id: BytesN<32>,
    pub shipment_id: BytesN<32>,
    pub seq_start: u64,
    pub seq_end: u64,
    pub readings_hash: BytesN<32>,
}

/// Distinct error surface for oracle authentication and integrity failures.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OracleError {
    /// Submitter is not the registered submitter for the device/shipment.
    UnauthorizedSubmitter,
    /// Device id is not registered.
    UnknownDevice,
    /// Device has been quarantined after too many implausible readings.
    DeviceQuarantined,
    /// Sequence number is not strictly greater than the last accepted one.
    NonMonotonicSequence,
    /// Reading timestamp is outside the sanity window vs. ledger time.
    TimestampOutOfWindow,
    /// Reading value is physically impossible.
    ImplausibleReading,
    /// Batch signature failed ed25519 verification.
    InvalidSignature,
    /// Batch seq range does not match the contained readings.
    SeqRangeMismatch,
    /// Threshold config version is not known.
    UnknownThresholdConfig,
    /// Caller lacks the required admin/OracleUpdater role.
    NotAuthorized,
}

/// Human-readable label for a product class (used in events/audit output).
impl ProductClass {
    pub fn label(&self) -> String {
        match self {
            ProductClass::WholeBlood => String::from_str("whole_blood"),
            ProductClass::Platelets => String::from_str("platelets"),
            ProductClass::Plasma => String::from_str("plasma"),
        }
    }
}
