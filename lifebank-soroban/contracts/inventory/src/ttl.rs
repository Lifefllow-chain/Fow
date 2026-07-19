//! # Inventory Contract – Storage Lifecycle & TTL Policy
//!
//! ## Storage Class Table
//!
//! | Key                              | Storage class | TTL policy                                       |
//! |----------------------------------|---------------|--------------------------------------------------|
//! | `DataKey::Admin`                 | Instance      | Lives with contract instance; no per-key rent    |
//! | `DataKey::BloodUnitCounter`      | Instance      | Lives with contract instance; no per-key rent    |
//! | `DataKey::ReservationCounter`    | Instance      | Lives with contract instance; no per-key rent    |
//! | `DataKey::StatusHistoryCounter`  | Instance      | Lives with contract instance; no per-key rent    |
//! | `DataKey::Paused`                | Instance      | Lives with contract instance; no per-key rent    |
//! | `DataKey::BloodUnit(id)`         | Persistent    | **BLOOD_UNIT_TTL** — extended past product       |
//! |                                  |               | expiry on every write; BLOOD_UNIT_EXTEND_TO on   |
//! |                                  |               | every read                                       |
//! | `DataKey::BloodTypeIndex(_)`     | Persistent    | **INDEX_TTL** — bumped on every write            |
//! | `DataKey::BankIndex(_)`          | Persistent    | **INDEX_TTL** — bumped on every write            |
//! | `DataKey::StatusIndex(_)`        | Persistent    | **INDEX_TTL** — bumped on every write            |
//! | `DataKey::DonorIndex(_)`         | Persistent    | **INDEX_TTL** — bumped on every write            |
//! | `DataKey::StatusHistory(id)`     | Persistent    | **HISTORY_TTL** — bumped with BloodUnit          |
//! | `DataKey::StatusHistoryPage(id,p)` | Persistent  | **HISTORY_TTL** — bumped with BloodUnit          |
//! | `DataKey::BloodUnitStatusChangeCount(id)` | Persistent | **HISTORY_TTL** — bumped with BloodUnit |
//! | `DataKey::Reservation(id)`       | Temporary     | Auto-expiring; no rent management needed         |
//!
//! ## Instance-storage extension
//!
//! `bump_instance` is called at the top of every public entrypoint so the
//! contract's instance storage (admin, counters) never lapses.
//!
//! ## Domain-deadline rule for blood units
//!
//! When a blood unit is registered (`register_blood`), its TTL is extended to
//! at least `expiration_timestamp + POST_EXPIRY_BUFFER_LEDGERS` converted to
//! ledgers from the current ledger.  This ensures the on-chain record survives
//! past the product's physical expiry date so auditors can always inspect it.
//!
//! On every subsequent read or write the unit is bumped with
//! `BLOOD_UNIT_EXTEND_TO` (a rolling 90-day window).

use soroban_sdk::{Env, IntoVal, Val};

// ── Constants ─────────────────────────────────────────────────────────────────

/// Approximate number of Stellar ledgers per day (5-second close time).
pub const LEDGERS_PER_DAY: u32 = 17_280;

/// Threshold below which a persistent key is bumped (30 days).
pub const BLOOD_UNIT_TTL_THRESHOLD: u32 = LEDGERS_PER_DAY * 30;

/// Target TTL after a regular bump for a blood-unit entry (≈ 90 days).
pub const BLOOD_UNIT_EXTEND_TO: u32 = LEDGERS_PER_DAY * 90;

/// Additional ledger headroom kept *after* a unit's physical expiry.
/// Set to 30 days so every expired unit stays readable for post-mortem audits.
pub const POST_EXPIRY_BUFFER_DAYS: u32 = 30;

/// The maximum shelf life of any blood product (42 days for whole blood per
/// WHO guidelines) plus `POST_EXPIRY_BUFFER_DAYS`. A freshly registered unit
/// is always extended to at least this many ledgers from now.
pub const UNIT_LIFETIME_EXTEND_TO: u32 =
    LEDGERS_PER_DAY * (42 + POST_EXPIRY_BUFFER_DAYS);

/// Threshold / extend-to for status-history entries (same rolling window as
/// the unit itself).
pub const HISTORY_TTL_THRESHOLD: u32 = BLOOD_UNIT_TTL_THRESHOLD;
pub const HISTORY_EXTEND_TO: u32 = BLOOD_UNIT_EXTEND_TO;

/// Threshold / extend-to for index Vecs.
pub const INDEX_TTL_THRESHOLD: u32 = LEDGERS_PER_DAY * 30;
pub const INDEX_EXTEND_TO: u32 = LEDGERS_PER_DAY * 90;

/// Threshold / extend-to for the contract instance.
pub const INSTANCE_TTL_THRESHOLD: u32 = LEDGERS_PER_DAY * 30;
pub const INSTANCE_EXTEND_TO: u32 = LEDGERS_PER_DAY * 90;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Extend the TTL of any persistent key.
pub fn bump_persistent<K>(env: &Env, key: &K, threshold: u32, extend_to: u32)
where
    K: IntoVal<Env, Val>,
{
    env.storage()
        .persistent()
        .extend_ttl(key, threshold, extend_to);
}

/// Bump a blood-unit record with the regular rolling policy.
pub fn bump_blood_unit<K>(env: &Env, key: &K)
where
    K: IntoVal<Env, Val>,
{
    bump_persistent(env, key, BLOOD_UNIT_TTL_THRESHOLD, BLOOD_UNIT_EXTEND_TO);
}

/// Bump a newly registered (or freshly locked) blood-unit to at least the
/// product-lifetime horizon.
///
/// Use this on `register_blood` so the unit stays live through physical expiry.
pub fn bump_blood_unit_lifetime<K>(env: &Env, key: &K)
where
    K: IntoVal<Env, Val>,
{
    // Use the lifetime as both the threshold and target so the unit is
    // unconditionally held for at least UNIT_LIFETIME_EXTEND_TO ledgers.
    bump_persistent(
        env,
        key,
        UNIT_LIFETIME_EXTEND_TO,
        UNIT_LIFETIME_EXTEND_TO,
    );
}

/// Bump a status-history key (page number or page data) with the history policy.
pub fn bump_history<K>(env: &Env, key: &K)
where
    K: IntoVal<Env, Val>,
{
    bump_persistent(env, key, HISTORY_TTL_THRESHOLD, HISTORY_EXTEND_TO);
}

/// Bump an index Vec key with the index policy.
pub fn bump_index<K>(env: &Env, key: &K)
where
    K: IntoVal<Env, Val>,
{
    bump_persistent(env, key, INDEX_TTL_THRESHOLD, INDEX_EXTEND_TO);
}

/// Extend the contract instance TTL.  Call at the top of every entrypoint.
pub fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_EXTEND_TO);
}
