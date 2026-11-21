use crate::types::{BloodStatus, BloodUnit, DataKey, StatusChangeHistory};
use interfaces::ttl::{bump, bump_instance, StorageClass};
use soroban_sdk::{Address, Env, String, Vec};

pub const SECONDS_PER_DAY: u64 = 86400;
pub const BLOOD_SHELF_LIFE_DAYS: u64 = 35;

// ── Storage classes & TTL policy ───────────────────────────────────────────────
//
// Every persistent key in this contract has a documented storage class and TTL
// policy. TTL is bumped on every access (read or write) via `interfaces::ttl`.
//
// | Key                                   | Class    | TTL policy                          |
// |---------------------------------------|----------|-------------------------------------|
// | BloodUnit(id)                         | BloodUnit| Extended past product expiry        |
// |                                       |          | (35-day shelf life) on write/read   |
// | BloodTypeIndex(type)                  | Index    | Bumped on access                    |
// | BankIndex(bank)                       | Index    | Bumped on access                    |
// | StatusIndex(status)                   | Index    | Bumped on access                    |
// | DonorIndex(donor)                     | Index    | Bumped on access                    |
// | StatusHistory(unit_id)                | Index    | Bumped on access                    |
// | StatusHistoryPage(unit_id, page)      | Index    | Bumped on access                    |
// | BloodUnitStatusChangeCount(unit_id)   | Counter  | Bumped on access                    |
// | Reservation(id) (temporary)           | Counter  | Bumped on access                    |
//
// Instance storage (Admin, BloodUnitCounter, StatusHistoryCounter,
// ReservationCounter) is extended at entrypoint level via `bump_instance`.
// ───────────────────────────────────────────────────────────────────────────────

/// Maximum history entries per storage page. Keeps each page small so
/// a single read never loads the entire history of a high-traffic unit.
const HISTORY_PAGE_SIZE: u32 = 50;

// ── Admin ──────────────────────────────────────────────────────────────────────

pub fn get_admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).expect("Admin not initialized")
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

// ── Authorization ──────────────────────────────────────────────────────────────

pub fn is_authorized_bank(env: &Env, bank: &Address) -> bool {
    let admin = get_admin(env);
    bank == &admin
}

// ── Blood unit counter ─────────────────────────────────────────────────────────

pub fn get_blood_unit_counter(env: &Env) -> u64 {
    env.storage().instance().get(&DataKey::BloodUnitCounter).unwrap_or(0)
}

pub fn increment_blood_unit_id(env: &Env) -> u64 {
    let next_id = get_blood_unit_counter(env) + 1;
    env.storage().instance().set(&DataKey::BloodUnitCounter, &next_id);
    next_id
}

// ── Blood unit CRUD ────────────────────────────────────────────────────────────

pub fn set_blood_unit(env: &Env, blood_unit: &BloodUnit) {
    let key = DataKey::BloodUnit(blood_unit.id);
    env.storage().persistent().set(&key, blood_unit);
    // Extend the unit's TTL past its product expiry (shelf life).
    bump(env, &key, StorageClass::BloodUnit);
}

pub fn get_blood_unit(env: &Env, id: u64) -> Option<BloodUnit> {
    let key = DataKey::BloodUnit(id);
    let unit = env.storage().persistent().get(&key);
    if unit.is_some() {
        bump(env, &key, StorageClass::BloodUnit);
    }
    unit
}

pub fn blood_unit_exists(env: &Env, id: u64) -> bool {
    let key = DataKey::BloodUnit(id);
    let exists = env.storage().persistent().has(&key);
    if exists {
        bump(env, &key, StorageClass::BloodUnit);
    }
    exists
}

// ── Indexes ────────────────────────────────────────────────────────────────────

pub fn add_to_blood_type_index(env: &Env, blood_unit: &BloodUnit) {
    let key = DataKey::BloodTypeIndex(blood_unit.blood_type);
    let mut units: Vec<u64> = env.storage().persistent().get(&key).unwrap_or(Vec::new(env));
    units.push_back(blood_unit.id);
    env.storage().persistent().set(&key, &units);
    bump(env, &key, StorageClass::Index);
}

pub fn add_to_bank_index(env: &Env, blood_unit: &BloodUnit) {
    let key = DataKey::BankIndex(blood_unit.bank_id.clone());
    let mut units: Vec<u64> = env.storage().persistent().get(&key).unwrap_or(Vec::new(env));
    units.push_back(blood_unit.id);
    env.storage().persistent().set(&key, &units);
    bump(env, &key, StorageClass::Index);
}

pub fn add_to_status_index(env: &Env, blood_unit: &BloodUnit) {
    let key = DataKey::StatusIndex(blood_unit.status);
    let mut units: Vec<u64> = env.storage().persistent().get(&key).unwrap_or(Vec::new(env));
    units.push_back(blood_unit.id);
    env.storage().persistent().set(&key, &units);
    bump(env, &key, StorageClass::Index);
}

/// Remove a single ID from a status index bucket.
/// Uses a single-pass rebuild — O(n) but only called on transitions, not reads.
pub fn remove_from_status_index(env: &Env, blood_unit_id: u64, old_status: BloodStatus) {
    let key = DataKey::StatusIndex(old_status);
    let units: Vec<u64> = env.storage().persistent().get(&key).unwrap_or(Vec::new(env));
    let mut updated: Vec<u64> = Vec::new(env);
    for i in 0..units.len() {
        let id = units.get(i).unwrap();
        if id != blood_unit_id {
            updated.push_back(id);
        }
    }
    env.storage().persistent().set(&key, &updated);
    bump(env, &key, StorageClass::Index);
}

pub fn add_to_donor_index(env: &Env, blood_unit: &BloodUnit) {
    if let Some(donor) = &blood_unit.donor_id {
        let key = DataKey::DonorIndex(donor.clone());
        let mut units: Vec<u64> = env.storage().persistent().get(&key).unwrap_or(Vec::new(env));
        units.push_back(blood_unit.id);
        env.storage().persistent().set(&key, &units);
        bump(env, &key, StorageClass::Index);
    }
}

// ── Paginated status history ───────────────────────────────────────────────────
//
// History is stored as a sequence of fixed-size pages:
//   DataKey::StatusHistory(unit_id)        → current page number (u32)
//   DataKey::StatusHistoryPage(unit_id, p) → Vec<StatusChangeHistory> for page p
//
// Each page holds at most HISTORY_PAGE_SIZE entries. When a page is full a new
// page is started. Reads for the full history iterate pages; reads for the
// latest N entries only load the last page(s).

/// Append one history record. Starts a new page when the current one is full.
pub fn record_status_change(
    env: &Env,
    blood_unit_id: u64,
    from_status: BloodStatus,
    to_status: BloodStatus,
    authorized_by: &Address,
    reason: Option<String>,
) {
    let changed_at = env.ledger().timestamp();
    let history_id = increment_status_history_counter(env);

    let entry = StatusChangeHistory {
        id: history_id,
        blood_unit_id,
        from_status,
        to_status,
        authorized_by: authorized_by.clone(),
        changed_at,
        reason,
    };

    // Determine which page to append to
    let page_key = DataKey::StatusHistory(blood_unit_id); // stores current page number
    let current_page: u32 = env.storage().persistent().get(&page_key).unwrap_or(0);
    bump(env, &page_key, StorageClass::Index);

    let page_data_key = DataKey::StatusHistoryPage(blood_unit_id, current_page);
    let mut page: Vec<StatusChangeHistory> = env
        .storage()
        .persistent()
        .get(&page_data_key)
        .unwrap_or(Vec::new(env));

    if page.len() >= HISTORY_PAGE_SIZE {
        // Current page is full — start a new one
        let next_page = current_page + 1;
        env.storage().persistent().set(&page_key, &next_page);
        bump(env, &page_key, StorageClass::Index);
        let new_page_key = DataKey::StatusHistoryPage(blood_unit_id, next_page);
        let mut new_page: Vec<StatusChangeHistory> = Vec::new(env);
        new_page.push_back(entry);
        env.storage().persistent().set(&new_page_key, &new_page);
        bump(env, &new_page_key, StorageClass::Index);
    } else {
        page.push_back(entry);
        env.storage().persistent().set(&page_data_key, &page);
        bump(env, &page_data_key, StorageClass::Index);
    }

    // Increment change count
    let count_key = DataKey::BloodUnitStatusChangeCount(blood_unit_id);
    let count = get_blood_unit_status_change_count(env, blood_unit_id);
    env.storage().persistent().set(&count_key, &(count + 1));
    bump(env, &count_key, StorageClass::Counter);
}

/// Return all history entries for a unit by iterating pages.
/// Callers that only need recent entries should use `get_status_history_page`.
pub fn get_status_history(env: &Env, blood_unit_id: u64) -> Vec<StatusChangeHistory> {
    let page_key = DataKey::StatusHistory(blood_unit_id);
    let last_page: u32 = env.storage().persistent().get(&page_key).unwrap_or(0);
    bump(env, &page_key, StorageClass::Index);

    let mut all: Vec<StatusChangeHistory> = Vec::new(env);
    for p in 0..=last_page {
        let page_data_key = DataKey::StatusHistoryPage(blood_unit_id, p);
        let page: Vec<StatusChangeHistory> = env
            .storage()
            .persistent()
            .get(&page_data_key)
            .unwrap_or(Vec::new(env));
        bump(env, &page_data_key, StorageClass::Index);
        for i in 0..page.len() {
            all.push_back(page.get(i).unwrap());
        }
    }
    all
}

/// Return a single page of history. O(1) storage reads.
pub fn get_status_history_page(
    env: &Env,
    blood_unit_id: u64,
    page: u32,
) -> Vec<StatusChangeHistory> {
    let page_data_key = DataKey::StatusHistoryPage(blood_unit_id, page);
    let result = env
        .storage()
        .persistent()
        .get(&page_data_key)
        .unwrap_or(Vec::new(env));
    bump(env, &page_data_key, StorageClass::Index);
    result
}

/// Return the current (last) page number for a unit's history.
pub fn get_history_page_count(env: &Env, blood_unit_id: u64) -> u32 {
    let page_key = DataKey::StatusHistory(blood_unit_id);
    let count = env.storage().persistent().get(&page_key).unwrap_or(0);
    bump(env, &page_key, StorageClass::Index);
    count
}

pub fn get_blood_unit_status_change_count(env: &Env, blood_unit_id: u64) -> u64 {
    let key = DataKey::BloodUnitStatusChangeCount(blood_unit_id);
    let count = env.storage().persistent().get(&key).unwrap_or(0);
    bump(env, &key, StorageClass::Counter);
    count
}

fn increment_status_history_counter(env: &Env) -> u64 {
    let key = DataKey::StatusHistoryCounter;
    let current: u64 = env.storage().instance().get(&key).unwrap_or(0u64);
    let next_id = current + 1;
    env.storage().instance().set(&key, &next_id);
    next_id
}

// ── Reservation ────────────────────────────────────────────────────────────────

pub fn increment_reservation_id(env: &Env) -> u64 {
    let key = DataKey::ReservationCounter;
    let current: u64 = env.storage().instance().get(&key).unwrap_or(0);
    let next_id = current + 1;
    env.storage().instance().set(&key, &next_id);
    next_id
}

pub fn set_reservation(env: &Env, id: u64, reservation: &crate::types::Reservation) {
    let key = DataKey::Reservation(id);
    env.storage().temporary().set(&key, reservation);
    bump(env, &key, StorageClass::Counter);
}

pub fn get_reservation(env: &Env, id: u64) -> Option<crate::types::Reservation> {
    let key = DataKey::Reservation(id);
    let reservation = env.storage().temporary().get(&key);
    if reservation.is_some() {
        bump(env, &key, StorageClass::Counter);
    }
    reservation
}

pub fn remove_reservation(env: &Env, id: u64) {
    env.storage().temporary().remove(&DataKey::Reservation(id));
}
