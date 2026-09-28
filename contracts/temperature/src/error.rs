use soroban_sdk::contracterror;

/// Errors returned by the temperature oracle contract.
///
/// The variants below cover the oracle-authentication surface introduced for
/// issue #41: device registry authorization, reading integrity (sequence,
/// timestamp, plausibility), signed-batch verification, quarantine, and
/// versioned threshold governance.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// Contract has not been initialized yet.
    NotInitialized = 1,
    /// Contract has already been initialized.
    AlreadyInitialized = 2,
    /// Caller is not authorized to perform this action.
    Unauthorized = 3,
    /// Shipment was not found.
    ShipmentNotFound = 4,
    /// Shipment is not in a state that allows this operation.
    InvalidShipmentState = 5,
    /// A reading was submitted for a shipment that is no longer active.
    ShipmentNotActive = 6,

    // --- Device registry (issue #41) ---
    /// The device id has not been registered by an admin/OracleUpdater.
    DeviceNotRegistered = 7,
    /// The device id is already registered.
    DeviceAlreadyRegistered = 8,
    /// The submitter is not the registered submitter for this device/shipment.
    DeviceSubmitterMismatch = 9,
    /// The device is not assigned to the shipment the reading targets.
    DeviceNotAssignedToShipment = 10,
    /// The device has been quarantined after too many implausible readings.
    DeviceQuarantined = 11,

    // --- Reading integrity (issue #41) ---
    /// The reading sequence number is not strictly greater than the last seen
    /// sequence for this device (replay or reordering attempt).
    InvalidSequence = 12,
    /// The reading timestamp is outside the accepted sanity window relative to
    /// ledger time (too far in the future or older than the shipment window).
    InvalidTimestamp = 13,
    /// The reading value is physically impossible for the configured product.
    ImplausibleReading = 14,
    /// The batch sequence range is malformed (end before start, or empty).
    InvalidSequenceRange = 15,

    // --- Signed batches (issue #41) ---
    /// The device signature over (device_id, seq_range, readings_hash) failed
    /// ed25519 verification.
    InvalidDeviceSignature = 16,
    /// The device has no registered pubkey, so signed batches cannot be verified.
    DevicePubkeyMissing = 17,
    /// The readings hash committed in the batch does not match the payload.
    ReadingsHashMismatch = 18,

    // --- Threshold governance (issue #41) ---
    /// No threshold config exists for the requested product.
    ThresholdConfigNotFound = 19,
    /// The threshold config version is invalid or has been superseded.
    InvalidThresholdVersion = 20,
    /// The threshold config is already pinned for this shipment and cannot be
    /// changed retroactively.
    ThresholdAlreadyPinned = 21,

    // --- Evidence chain (issue #41) ---
    /// The excursion summary is missing its evidence chain commitment.
    MissingEvidenceChain = 22,
    /// The evidence chain commitment does not match the recorded readings hash.
    EvidenceChainMismatch = 23,
}
