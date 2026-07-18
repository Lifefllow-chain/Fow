use soroban_sdk::{contracttype, Address, BytesN};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq, Copy)]
pub struct TemperatureReading {
    pub temperature_celsius_x100: i32,
    pub timestamp: u64,
    pub is_violation: bool,
}

impl Default for TemperatureReading {
    fn default() -> Self {
        TemperatureReading {
            temperature_celsius_x100: 0,
            timestamp: 0,
            is_violation: false,
        }
    }
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq, Copy)]
pub struct TemperatureThreshold {
    pub min_celsius_x100: i32,
    pub max_celsius_x100: i32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemperatureSummary {
    pub count: u32,
    pub avg_celsius_x100: i32,
    pub min_celsius_x100: i32,
    pub max_celsius_x100: i32,
    pub violation_count: u32,
}

/// Summary of a sustained temperature excursion, passed to the coordinator
/// when automatically raising a payment dispute.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExcursionSummary {
    /// Blood unit affected
    pub unit_id: u64,
    /// Number of consecutive violations that triggered this excursion
    pub violation_count: u32,
    /// Peak temperature recorded during the excursion (×100 scale)
    pub peak_celsius_x100: i32,
    /// Ledger timestamp when the excursion was first detected
    pub detected_at: u64,
}

#[contracttype]
#[derive(Clone, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Threshold(u64),
    TempPage(u64, u32),
    TempPageLen(u64, u32),
    /// Tracks consecutive violation streak for a blood unit
    ConsecutiveViolationStreak(u64),
    /// Tracks if unit has been compromised (3+ consecutive violations)
    IsCompromised(u64),
    Paused,
    /// Address of the coordinator contract for cross-contract dispute escalation
    CoordinatorContract,
    /// Whitelisted IoT oracle addresses allowed to report excursions
    OracleWhitelist(soroban_sdk::Address),

    // ── Oracle authentication (#41) ─────────────────────────────────────────

    /// Registered device identity, keyed by device id.
    Device(u64),
    /// Latest threshold config version published for a product id.
    ProductThresholdVersion(u64),
    /// Versioned threshold config, keyed by (product id, version).
    ProductThreshold(u64, u32),
    /// Threshold version pinned to a shipment (blood unit) at `start_shipment`.
    ShipmentThresholdVersion(u64),
    /// Product id assigned to a shipment (blood unit) at `start_shipment`.
    ShipmentProduct(u64),
    /// Ledger timestamp a shipment was started at.
    ShipmentStartedAt(u64),
    /// Rolling hash-chain evidence anchor for a shipment.
    ShipmentEvidence(u64),
    /// Accumulated breach state for a shipment, judged under its pinned threshold.
    ShipmentBreachState(u64),
}

/// Raw sensor reading carried inside a signed batch, prior to violation
/// evaluation (unlike [`TemperatureReading`], it carries no derived state).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq, Copy)]
pub struct RawReading {
    pub temperature_celsius_x100: i32,
    pub timestamp: u64,
}

/// Registered device identity authorized to submit readings for one shipment.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviceInfo {
    /// Address whose transaction auth must accompany batches from this device
    /// (the gateway/backend relaying the device's offline-buffered data).
    pub submitter: Address,
    /// Ed25519 public key the physical device signs batches with.
    pub pubkey: BytesN<32>,
    /// Blood unit / shipment this device is scoped to.
    pub shipment_id: u64,
    /// Last accepted sequence number, enforcing monotonic ingestion.
    pub last_seq: u64,
    /// Count of batches rejected for implausible readings.
    pub implausible_count: u32,
    /// Set once `implausible_count` reaches the quarantine threshold.
    pub quarantined: bool,
}

/// Versioned per-product temperature threshold configuration.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq, Copy)]
pub struct ProductThreshold {
    pub min_celsius_x100: i32,
    pub max_celsius_x100: i32,
    pub version: u32,
}

/// On-chain evidence anchor for a shipment: the threshold version its verdict
/// was judged under, plus a rolling hash chain over every ingested batch, so
/// an arbiter can replay off-chain raw data and reproduce this commitment.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipmentEvidence {
    pub product_id: u64,
    pub threshold_version: u32,
    pub chain_hash: BytesN<32>,
    pub batch_count: u32,
}

/// Accumulated excursion state for a shipment, derived only from batches that
/// passed timestamp and plausibility validation.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq, Copy, Default)]
pub struct ShipmentBreachState {
    pub violation_count: u32,
    pub peak_celsius_x100: i32,
    pub detected_at: u64,
}

/// Result of ingesting a signed batch.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq, Copy)]
pub enum BatchOutcome {
    /// All readings passed validation and were folded into the shipment verdict.
    Accepted,
    /// The batch was committed to the evidence chain but excluded from the
    /// verdict because one or more readings fell outside the ledger-time
    /// sanity window.
    RejectedTimestamp,
    /// The batch was committed to the evidence chain but excluded from the
    /// verdict because one or more readings fell outside physically
    /// plausible bounds; counts toward device quarantine.
    RejectedImplausible,
}
