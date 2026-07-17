use crate::error::ContractError;
use soroban_sdk::{contracttype, Address, String, Vec};

// Cross-contract types — single source of truth from the interfaces crate.
// BloodType, BloodStatus, BloodUnit (with is_expired / shelf_life_remaining),
// ALLOWED_BLOOD_STATUS_TRANSITIONS and is_valid_blood_transition are all
// defined there; re-exported by lib.rs for internal use.
pub use lifebank_interfaces::{
    BloodStatus, BloodType, BloodUnit, ALLOWED_BLOOD_STATUS_TRANSITIONS,
    is_valid_blood_transition,
};

/// Validate that a blood unit's fields are internally consistent.
/// Kept as a free function (not an inherent impl) because BloodUnit is
/// defined in the interfaces crate and Rust's orphan rules prevent adding
/// inherent methods to foreign types.
pub fn validate_blood_unit(unit: &BloodUnit, current_time: u64) -> Result<(), ContractError> {
    if unit.quantity_ml < 100 || unit.quantity_ml > 600 {
        return Err(ContractError::InvalidQuantity);
    }
    if unit.expiration_timestamp <= unit.donation_timestamp {
        return Err(ContractError::InvalidTimestamp);
    }
    if unit.donation_timestamp > current_time + 3600 {
        return Err(ContractError::InvalidTimestamp);
    }
    Ok(())
}

// ── Inventory-internal types ──────────────────────────────────────────────────

/// Storage key types for efficient querying.
#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    BloodUnit(u64),
    BloodUnitCounter,
    BloodTypeIndex(BloodType),
    BankIndex(Address),
    StatusIndex(BloodStatus),
    DonorIndex(Address),
    Admin,
    StatusHistory(u64),
    StatusHistoryPage(u64, u32),
    StatusHistoryCounter,
    BloodUnitStatusChangeCount(u64),
    Reservation(u64),
    ReservationCounter,
    Paused,
}

/// Reservation record for blood units locked for a specific requester.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Reservation {
    pub unit_ids: Vec<u64>,
    pub requester: Address,
    pub created_timestamp: u64,
    pub expiration_timestamp: u64,
    pub request_id: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct BloodRegisteredEvent {
    pub blood_unit_id: u64,
    pub bank_id: Address,
    pub blood_type: BloodType,
    pub quantity_ml: u32,
    pub expiration_timestamp: u64,
    pub registered_at: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct StatusChangeEvent {
    pub blood_unit_id: u64,
    pub from_status: BloodStatus,
    pub to_status: BloodStatus,
    pub authorized_by: Address,
    pub changed_at: u64,
    pub reason: Option<String>,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct AuditEvent {
    pub unit_id: u64,
    pub previous_status: BloodStatus,
    pub new_status: BloodStatus,
    pub actor: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct StatusChangeHistory {
    pub id: u64,
    pub blood_unit_id: u64,
    pub from_status: BloodStatus,
    pub to_status: BloodStatus,
    pub authorized_by: Address,
    pub changed_at: u64,
    pub reason: Option<String>,
}
