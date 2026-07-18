use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    Unauthorized = 600,
    UnitNotFound = 601,
    ThresholdNotFound = 602,
    InvalidThreshold = 603,
    AlreadyInitialized = 604,
    ContractPaused = 605,
    /// Coordinator contract address not configured
    CoordinatorNotSet = 606,
    /// Cross-contract call to coordinator failed
    CoordinatorCallFailed = 607,
    /// Storage is already at the schema version this binary targets.
    MigrationAlreadyApplied = 608,

    // ── Oracle authentication (#41) ─────────────────────────────────────────
    /// No device is registered under this device id.
    DeviceNotRegistered = 609,
    /// A device is already registered under this device id.
    DeviceAlreadyRegistered = 610,
    /// Caller is not the registered submitter for this device.
    SubmitterMismatch = 611,
    /// Device has been quarantined after repeated implausible readings.
    DeviceQuarantined = 612,
    /// `seq_start` is not strictly greater than the device's last accepted sequence.
    InvalidSequence = 613,
    /// `seq_end` is before `seq_start`, or the range length doesn't match the batch.
    InvalidSequenceRange = 614,
    /// A batch must contain at least one reading.
    EmptyBatch = 615,
    /// Declared readings hash doesn't match the recomputed hash of the batch payload.
    ReadingsHashMismatch = 616,
    /// No threshold configuration exists for this product id.
    ProductThresholdNotFound = 617,
    /// The shipment has not been started (no pinned threshold version).
    ShipmentNotStarted = 618,
    /// The shipment has already been started.
    ShipmentAlreadyStarted = 619,
    /// No breach verdict has been recorded yet for this shipment.
    ShipmentVerdictNotFound = 620,
    /// No evidence chain has been recorded yet for this shipment.
    ShipmentEvidenceNotFound = 621,
}
