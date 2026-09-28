use soroban_sdk::{contracttype, Address, BytesN, Env, Vec};

use crate::types::{
    DeviceRecord, ExcursionSummary, ReadingBatch, ShipmentRecord, ThresholdConfig, ThresholdVersion,
};

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    OracleUpdater,
    /// Registered device identity: device_id -> DeviceRecord
    Device(BytesN<32>),
    /// Per-device monotonic sequence watermark (last accepted seq)
    DeviceSeq(BytesN<32>),
    /// Implausible-reading counter used for quarantine
    DeviceImplausible(BytesN<32>),
    /// Shipment record: shipment_id -> ShipmentRecord
    Shipment(BytesN<32>),
    /// Threshold config pinned at shipment start: shipment_id -> ThresholdVersion
    ShipmentThreshold(BytesN<32>),
    /// Versioned per-product threshold config: version -> ThresholdConfig
    Threshold(u32),
    /// Current active threshold version
    ActiveThresholdVersion,
    /// Excursion summary (state machine + evidence chain): shipment_id -> ExcursionSummary
    Excursion(BytesN<32>),
    /// Readings-hash chain commitment per shipment (append-only evidence)
    ReadingsHash(BytesN<32>, u32),
    /// Number of committed reading batches for a shipment
    ReadingsHashCount(BytesN<32>),
}

// ---------------------------------------------------------------------------
// Admin / role storage
// ---------------------------------------------------------------------------

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

pub fn set_oracle_updater(env: &Env, updater: &Address) {
    env.storage()
        .instance()
        .set(&DataKey::OracleUpdater, updater);
}

pub fn get_oracle_updater(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::OracleUpdater)
}

// ---------------------------------------------------------------------------
// Device registry
// ---------------------------------------------------------------------------

pub fn set_device(env: &Env, device_id: &BytesN<32>, record: &DeviceRecord) {
    env.storage()
        .persistent()
        .set(&DataKey::Device(device_id.clone()), record);
}

pub fn get_device(env: &Env, device_id: &BytesN<32>) -> Option<DeviceRecord> {
    env.storage()
        .persistent()
        .get(&DataKey::Device(device_id.clone()))
}

pub fn remove_device(env: &Env, device_id: &BytesN<32>) {
    env.storage()
        .persistent()
        .remove(&DataKey::Device(device_id.clone()));
}

// ---------------------------------------------------------------------------
// Per-device sequence watermark + quarantine counter
// ---------------------------------------------------------------------------

pub fn get_device_seq(env: &Env, device_id: &BytesN<32>) -> u64 {
    env.storage()
        .persistent()
        .get(&DataKey::DeviceSeq(device_id.clone()))
        .unwrap_or(0)
}

pub fn set_device_seq(env: &Env, device_id: &BytesN<32>, seq: u64) {
    env.storage()
        .persistent()
        .set(&DataKey::DeviceSeq(device_id.clone()), &seq);
}

pub fn get_device_implausible(env: &Env, device_id: &BytesN<32>) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::DeviceImplausible(device_id.clone()))
        .unwrap_or(0)
}

pub fn set_device_implausible(env: &Env, device_id: &BytesN<32>, count: u32) {
    env.storage()
        .persistent()
        .set(&DataKey::DeviceImplausible(device_id.clone()), &count);
}

// ---------------------------------------------------------------------------
// Shipments
// ---------------------------------------------------------------------------

pub fn set_shipment(env: &Env, shipment_id: &BytesN<32>, record: &ShipmentRecord) {
    env.storage()
        .persistent()
        .set(&DataKey::Shipment(shipment_id.clone()), record);
}

pub fn get_shipment(env: &Env, shipment_id: &BytesN<32>) -> Option<ShipmentRecord> {
    env.storage()
        .persistent()
        .get(&DataKey::Shipment(shipment_id.clone()))
}

// ---------------------------------------------------------------------------
// Versioned thresholds
// ---------------------------------------------------------------------------

pub fn set_threshold(env: &Env, version: u32, config: &ThresholdConfig) {
    env.storage()
        .persistent()
        .set(&DataKey::Threshold(version), config);
}

pub fn get_threshold(env: &Env, version: u32) -> Option<ThresholdConfig> {
    env.storage().persistent().get(&DataKey::Threshold(version))
}

pub fn get_active_threshold_version(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::ActiveThresholdVersion)
        .unwrap_or(0)
}

pub fn set_active_threshold_version(env: &Env, version: u32) {
    env.storage()
        .instance()
        .set(&DataKey::ActiveThresholdVersion, &version);
}

/// Pin the threshold version active at shipment start so later governance
/// changes cannot retroactively alter this shipment's verdict.
pub fn pin_shipment_threshold(env: &Env, shipment_id: &BytesN<32>, version: u32) {
    env.storage()
        .persistent()
        .set(&DataKey::ShipmentThreshold(shipment_id.clone()), &version);
}

pub fn get_shipment_threshold(env: &Env, shipment_id: &BytesN<32>) -> Option<u32> {
    env.storage()
        .persistent()
        .get(&DataKey::ShipmentThreshold(shipment_id.clone()))
}

// ---------------------------------------------------------------------------
// Excursion summary (state machine + evidence chain)
// ---------------------------------------------------------------------------

pub fn set_excursion(env: &Env, shipment_id: &BytesN<32>, summary: &ExcursionSummary) {
    env.storage()
        .persistent()
        .set(&DataKey::Excursion(shipment_id.clone()), summary);
}

pub fn get_excursion(env: &Env, shipment_id: &BytesN<32>) -> Option<ExcursionSummary> {
    env.storage()
        .persistent()
        .get(&DataKey::Excursion(shipment_id.clone()))
}

// ---------------------------------------------------------------------------
// Readings-hash evidence chain (append-only commitments)
// ---------------------------------------------------------------------------

pub fn get_readings_hash_count(env: &Env, shipment_id: &BytesN<32>) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::ReadingsHashCount(shipment_id.clone()))
        .unwrap_or(0)
}

pub fn append_readings_hash(env: &Env, shipment_id: &BytesN<32>, hash: &BytesN<32>) -> u32 {
    let index = get_readings_hash_count(env, shipment_id);
    env.storage().persistent().set(
        &DataKey::ReadingsHash(shipment_id.clone(), index),
        hash,
    );
    env.storage().persistent().set(
        &DataKey::ReadingsHashCount(shipment_id.clone()),
        &(index + 1),
    );
    index
}

pub fn get_readings_hash(env: &Env, shipment_id: &BytesN<32>, index: u32) -> Option<BytesN<32>> {
    env.storage()
        .persistent()
        .get(&DataKey::ReadingsHash(shipment_id.clone(), index))
}

/// Collect the full readings-hash chain for a shipment so an arbiter can
/// verify off-chain raw data against on-chain commitments.
pub fn get_readings_hash_chain(env: &Env, shipment_id: &BytesN<32>) -> Vec<BytesN<32>> {
    let count = get_readings_hash_count(env, shipment_id);
    let mut chain = Vec::new(env);
    let mut i = 0u32;
    while i < count {
        if let Some(hash) = get_readings_hash(env, shipment_id, i) {
            chain.push_back(hash);
        }
        i += 1;
    }
    chain
}

// ---------------------------------------------------------------------------
// Raw readings: temporary storage only (off-chain detail, on-chain summary)
// ---------------------------------------------------------------------------

pub fn set_temp_batch(env: &Env, shipment_id: &BytesN<32>, batch: &ReadingBatch) {
    env.storage()
        .temporary()
        .set(&DataKey::ReadingsHash(shipment_id.clone(), u32::MAX), batch);
}

pub fn get_temp_batch(env: &Env, shipment_id: &BytesN<32>) -> Option<ReadingBatch> {
    env.storage()
        .temporary()
        .get(&DataKey::ReadingsHash(shipment_id.clone(), u32::MAX))
}
