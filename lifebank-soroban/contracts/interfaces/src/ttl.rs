//! Workspace-wide storage TTL policy shared by the `payments` and `inventory`
//! contracts.
//!
//! # Why this exists
//!
//! Soroban persistent entries are archived once their TTL lapses. Every
//! persistent entry in this workspace (escrow `Payment` records, blood-unit
//! records and their indexes) must therefore be extended on access, otherwise
//! the entrypoints that read live funds/units start failing.
//!
//! # Ledger close time assumption
//!
//! All horizons below are derived from [`LEDGERS_PER_DAY`], which assumes a
//! **5 second** ledger close time. If Stellar changes the close time, only
//! `LEDGERS_PER_DAY` needs to change and every horizon shifts consistently.
//!
//! # Storage class / TTL policy table
//!
//! | Class              | Key shape                                   | Threshold | Extend-to | Rationale |
//! |--------------------|---------------------------------------------|-----------|-----------|-----------|
//! | `Payment`          | `DataKey::Payment(id)`                      | 30 days   | 120 days  | Escrow funds must outlive the dispute-resolution horizon. |
//! | `Pledge`           | `DataKey::Pledge(id)`                       | 30 days   | 120 days  | Pledges back payments; same horizon as payments. |
//! | `Vesting`          | `DataKey::Vesting(id)`                      | 30 days   | 120 days  | Vesting schedules are long-lived obligations. |
//! | `PaymentIndex`     | `DataKey::PayerIndex(..)`, `PayeeIndex(..)`, `StatusIndex(..)`, `RequestIndex(..)` | 30 days | 120 days | Indexes must stay live as long as the records they point at. |
//! | `BloodUnit`        | `DataKey::BloodUnit(id)`                    | 30 days   | 120 days  | Units must outlive product expiry plus a margin. |
//! | `BloodUnitIndex`   | `DataKey::BloodTypeIndex(..)`, `BankIndex(..)`, `DonorIndex(..)`, `StatusIndex(..)` | 30 days | 120 days | Indexes must stay live as long as the units they point at. |
//! | `StatusHistory`    | `DataKey::StatusHistory(..)`, `StatusHistoryPage(..)`, `BloodUnitStatusChangeCount(..)` | 30 days | 120 days | Audit trail must survive as long as the unit record. |
//! | `Instance`         | contract instance storage                   | 30 days   | 120 days  | Instance storage holds config/admin; extended at every entrypoint. |
//!
//! Temporary-storage `Reservation` entries in `inventory` are intentionally
//! excluded: they are short-lived by design and are not part of this policy.

use soroban_sdk::{Env, IntoVal, Val};

/// Number of ledgers in one day, assuming a 5 second ledger close time.
pub const LEDGERS_PER_DAY: u32 = 17_280;

/// Number of ledgers in one hour (5 second close time).
pub const LEDGERS_PER_HOUR: u32 = 720;

/// Convert a duration in seconds to a ledger count (rounded up).
pub fn secs_to_ledgers(secs: u64) -> u32 {
    let ledgers = (secs + 4) / 5;
    if ledgers > u32::MAX as u64 {
        u32::MAX
    } else {
        ledgers as u32
    }
}

/// Convert a duration in days to a ledger count.
pub const fn days_to_ledgers(days: u32) -> u32 {
    days * LEDGERS_PER_DAY
}

/// The storage class of a persistent entry. One variant per documented class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageClass {
    Payment,
    Pledge,
    Vesting,
    PaymentIndex,
    BloodUnit,
    BloodUnitIndex,
    StatusHistory,
}

// ── Per-class thresholds / extend-to targets (in ledgers) ──────────────────

/// Threshold for `Payment` entries: extend when fewer than 30 days remain.
pub const PAYMENT_THRESHOLD_LEDGERS: u32 = days_to_ledgers(30);
/// Extend-to target for `Payment` entries: 120 days.
pub const PAYMENT_EXTEND_TO_LEDGERS: u32 = days_to_ledgers(120);

/// Threshold for `Pledge` entries.
pub const PLEDGE_THRESHOLD_LEDGERS: u32 = days_to_ledgers(30);
/// Extend-to target for `Pledge` entries.
pub const PLEDGE_EXTEND_TO_LEDGERS: u32 = days_to_ledgers(120);

/// Threshold for `Vesting` entries.
pub const VESTING_THRESHOLD_LEDGERS: u32 = days_to_ledgers(30);
/// Extend-to target for `Vesting` entries.
pub const VESTING_EXTEND_TO_LEDGERS: u32 = days_to_ledgers(120);

/// Threshold for payment index entries.
pub const INDEX_THRESHOLD_LEDGERS: u32 = days_to_ledgers(30);
/// Extend-to target for payment index entries.
pub const INDEX_EXTEND_TO_LEDGERS: u32 = days_to_ledgers(120);

/// Threshold for `BloodUnit` entries.
pub const BLOOD_UNIT_THRESHOLD_LEDGERS: u32 = days_to_ledgers(30);
/// Extend-to target for `BloodUnit` entries.
pub const BLOOD_UNIT_EXTEND_TO_LEDGERS: u32 = days_to_ledgers(120);

/// Threshold for blood-unit index entries.
pub const BLOOD_UNIT_INDEX_THRESHOLD_LEDGERS: u32 = days_to_ledgers(30);
/// Extend-to target for blood-unit index entries.
pub const BLOOD_UNIT_INDEX_EXTEND_TO_LEDGERS: u32 = days_to_ledgers(120);

/// Threshold for status-history entries.
pub const HISTORY_THRESHOLD_LEDGERS: u32 = days_to_ledgers(30);
/// Extend-to target for status-history entries.
pub const HISTORY_EXTEND_TO_LEDGERS: u32 = days_to_ledgers(120);

/// Threshold for contract instance storage.
pub const INSTANCE_THRESHOLD_LEDGERS: u32 = days_to_ledgers(30);
/// Extend-to target for contract instance storage.
pub const INSTANCE_EXTEND_TO_LEDGERS: u32 = days_to_ledgers(120);

/// Minimum lifetime of a payment entry after `lock`: the dispute-resolution
/// horizon (30 days) plus a safety margin.
pub const DISPUTE_RESOLUTION_HORIZON_SECS: u64 = 30 * 24 * 60 * 60;

/// How long a blood-unit entry must outlive its product expiry.
pub const BLOOD_UNIT_POST_EXPIRY_MARGIN_SECS: u64 = 30 * 24 * 60 * 60;

/// Return the `(threshold, extend_to)` ledger pair for a storage class.
pub const fn class_ttl(class: StorageClass) -> (u32, u32) {
    match class {
        StorageClass::Payment => (PAYMENT_THRESHOLD_LEDGERS, PAYMENT_EXTEND_TO_LEDGERS),
        StorageClass::Pledge => (PLEDGE_THRESHOLD_LEDGERS, PLEDGE_EXTEND_TO_LEDGERS),
        StorageClass::Vesting => (VESTING_THRESHOLD_LEDGERS, VESTING_EXTEND_TO_LEDGERS),
        StorageClass::PaymentIndex => (INDEX_THRESHOLD_LEDGERS, INDEX_EXTEND_TO_LEDGERS),
        StorageClass::BloodUnit => (BLOOD_UNIT_THRESHOLD_LEDGERS, BLOOD_UNIT_EXTEND_TO_LEDGERS),
        StorageClass::BloodUnitIndex => (
            BLOOD_UNIT_INDEX_THRESHOLD_LEDGERS,
            BLOOD_UNIT_INDEX_EXTEND_TO_LEDGERS,
        ),
        StorageClass::StatusHistory => (HISTORY_THRESHOLD_LEDGERS, HISTORY_EXTEND_TO_LEDGERS),
    }
}

/// Extend the TTL of a persistent entry according to its storage class.
///
/// This is the bump-on-access helper: call it after every persistent read or
/// write so the entry never drifts towards archival.
pub fn bump<K>(env: &Env, key: &K, class: StorageClass)
where
    K: IntoVal<Env, Val>,
{
    let (threshold, extend_to) = class_ttl(class);
    env.storage().persistent().extend_ttl(key, threshold, extend_to);
}

/// Extend the contract instance storage TTL. Call at the top of every
/// entrypoint.
pub fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_THRESHOLD_LEDGERS, INSTANCE_EXTEND_TO_LEDGERS);
}

/// Extend a persistent entry so it lives at least until `deadline_secs`
/// (a Unix timestamp), but never less than the class default.
///
/// Used for the domain-deadline rules: escrow payments must outlive the
/// dispute-resolution horizon, blood units must outlive product expiry.
pub fn bump_to_deadline<K>(env: &Env, key: &K, class: StorageClass, deadline_secs: u64)
where
    K: IntoVal<Env, Val>,
{
    let (threshold, class_extend_to) = class_ttl(class);
    let now = env.ledger().timestamp();
    let required = if deadline_secs > now {
        secs_to_ledgers(deadline_secs - now)
    } else {
        0
    };
    let extend_to = if required > class_extend_to {
        required
    } else {
        class_extend_to
    };
    env.storage().persistent().extend_ttl(key, threshold, extend_to);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secs_to_ledgers_rounds_up() {
        assert_eq!(secs_to_ledgers(0), 0);
        assert_eq!(secs_to_ledgers(1), 1);
        assert_eq!(secs_to_ledgers(5), 1);
        assert_eq!(secs_to_ledgers(6), 2);
        assert_eq!(secs_to_ledgers(86_400), LEDGERS_PER_DAY);
    }

    #[test]
    fn class_ttl_targets_exceed_thresholds() {
        for class in [
            StorageClass::Payment,
            StorageClass::Pledge,
            StorageClass::Vesting,
            StorageClass::PaymentIndex,
            StorageClass::BloodUnit,
            StorageClass::BloodUnitIndex,
            StorageClass::StatusHistory,
        ] {
            let (threshold, extend_to) = class_ttl(class);
            assert!(extend_to > threshold, "extend_to must exceed threshold");
        }
    }

    #[test]
    fn dispute_horizon_is_thirty_days() {
        assert_eq!(DISPUTE_RESOLUTION_HORIZON_SECS, 30 * 24 * 60 * 60);
        assert_eq!(
            secs_to_ledgers(DISPUTE_RESOLUTION_HORIZON_SECS),
            days_to_ledgers(30)
        );
    }
}
