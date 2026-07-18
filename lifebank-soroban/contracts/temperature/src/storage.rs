use soroban_sdk::{Env, Vec};
use crate::types::{
    DataKey, DeviceInfo, ProductThreshold, ShipmentBreachState, ShipmentEvidence,
    TemperatureReading, TemperatureThreshold,
};

pub fn get_admin(env: &Env) -> soroban_sdk::Address {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .unwrap()
}

pub fn set_admin(env: &Env, admin: &soroban_sdk::Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_threshold(env: &Env, unit_id: u64) -> Option<TemperatureThreshold> {
    env.storage()
        .persistent()
        .get(&DataKey::Threshold(unit_id))
}

pub fn set_threshold(env: &Env, unit_id: u64, threshold: &TemperatureThreshold) {
    env.storage()
        .persistent()
        .set(&DataKey::Threshold(unit_id), threshold);
}

pub fn get_temp_page(
    env: &Env,
    unit_id: u64,
    page: u32,
) -> Vec<TemperatureReading> {
    env.storage()
        .persistent()
        .get(&DataKey::TempPage(unit_id, page))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn set_temp_page(
    env: &Env,
    unit_id: u64,
    page: u32,
    readings: &Vec<TemperatureReading>,
) {
    env.storage()
        .persistent()
        .set(&DataKey::TempPage(unit_id, page), readings);
}

pub fn get_temp_page_len(env: &Env, unit_id: u64, page: u32) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::TempPageLen(unit_id, page))
        .unwrap_or(0)
}

pub fn set_temp_page_len(env: &Env, unit_id: u64, page: u32, len: u32) {
    env.storage()
        .persistent()
        .set(&DataKey::TempPageLen(unit_id, page), &len);
}

// ── Oracle authentication (#41) ─────────────────────────────────────────────

pub fn get_device(env: &Env, device_id: u64) -> Option<DeviceInfo> {
    env.storage().persistent().get(&DataKey::Device(device_id))
}

pub fn set_device(env: &Env, device_id: u64, device: &DeviceInfo) {
    env.storage()
        .persistent()
        .set(&DataKey::Device(device_id), device);
}

pub fn get_product_threshold_version(env: &Env, product_id: u64) -> Option<u32> {
    env.storage()
        .persistent()
        .get(&DataKey::ProductThresholdVersion(product_id))
}

pub fn set_product_threshold_version(env: &Env, product_id: u64, version: u32) {
    env.storage()
        .persistent()
        .set(&DataKey::ProductThresholdVersion(product_id), &version);
}

pub fn get_product_threshold(
    env: &Env,
    product_id: u64,
    version: u32,
) -> Option<ProductThreshold> {
    env.storage()
        .persistent()
        .get(&DataKey::ProductThreshold(product_id, version))
}

pub fn set_product_threshold(
    env: &Env,
    product_id: u64,
    version: u32,
    threshold: &ProductThreshold,
) {
    env.storage()
        .persistent()
        .set(&DataKey::ProductThreshold(product_id, version), threshold);
}

pub fn get_shipment_threshold_version(env: &Env, unit_id: u64) -> Option<u32> {
    env.storage()
        .persistent()
        .get(&DataKey::ShipmentThresholdVersion(unit_id))
}

pub fn set_shipment_threshold_version(env: &Env, unit_id: u64, version: u32) {
    env.storage()
        .persistent()
        .set(&DataKey::ShipmentThresholdVersion(unit_id), &version);
}

pub fn get_shipment_product(env: &Env, unit_id: u64) -> Option<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::ShipmentProduct(unit_id))
}

pub fn set_shipment_product(env: &Env, unit_id: u64, product_id: u64) {
    env.storage()
        .persistent()
        .set(&DataKey::ShipmentProduct(unit_id), &product_id);
}

pub fn get_shipment_started_at(env: &Env, unit_id: u64) -> Option<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::ShipmentStartedAt(unit_id))
}

pub fn set_shipment_started_at(env: &Env, unit_id: u64, started_at: u64) {
    env.storage()
        .persistent()
        .set(&DataKey::ShipmentStartedAt(unit_id), &started_at);
}

pub fn get_shipment_evidence(env: &Env, unit_id: u64) -> Option<ShipmentEvidence> {
    env.storage()
        .persistent()
        .get(&DataKey::ShipmentEvidence(unit_id))
}

pub fn set_shipment_evidence(env: &Env, unit_id: u64, evidence: &ShipmentEvidence) {
    env.storage()
        .persistent()
        .set(&DataKey::ShipmentEvidence(unit_id), evidence);
}

pub fn get_shipment_breach_state(env: &Env, unit_id: u64) -> Option<ShipmentBreachState> {
    env.storage()
        .persistent()
        .get(&DataKey::ShipmentBreachState(unit_id))
}

pub fn set_shipment_breach_state(env: &Env, unit_id: u64, state: &ShipmentBreachState) {
    env.storage()
        .persistent()
        .set(&DataKey::ShipmentBreachState(unit_id), state);
}
