//! # Payments Contract – Storage Lifecycle & TTL Policy
//!
//! ## Storage Class Table
//!
//! | Key                          | Storage class | TTL policy                                      |
//! |------------------------------|---------------|-------------------------------------------------|
//! | `ADMIN_KEY`                  | Instance      | Lives with contract instance; no per-key rent   |
//! | `PAUSED_KEY`                 | Instance      | Lives with contract instance; no per-key rent   |
//! | `PAYMENT_COUNTER`            | Instance      | Lives with contract instance; no per-key rent   |
//! | `PLEDGE_COUNTER`             | Instance      | Lives with contract instance; no per-key rent   |
//! | `REWARD_TOKEN_KEY`           | Instance      | Lives with contract instance; no per-key rent   |
//! | `REQ_IDX`                    | Instance      | Lives with contract instance; no per-key rent   |
//! | `STATS_KEY`                  | Instance      | Lives with contract instance; no per-key rent   |
//! | `REQ_CONTRACT`               | Instance      | Lives with contract instance; no per-key rent   |
//! | `DISPUTE_TIMEOUT`            | Instance      | Lives with contract instance; no per-key rent   |
//! | `payment_key(id)`            | Persistent    | **PAYMENT_TTL** — extended to at least the      |
//! |                              |               | dispute-resolution horizon on lock; then        |
//! |                              |               | PAYMENT_EXTEND_TO on every read/write           |
//! | `pledge_key(id)`             | Persistent    | **PLEDGE_TTL** — bumped on every read/write     |
//! | `payer_index_key(addr)`      | Persistent    | **INDEX_TTL** — bumped on every write           |
//! | `payee_index_key(addr)`      | Persistent    | **INDEX_TTL** — bumped on every write           |
//! | `status_index_key(status)`   | Persistent    | **INDEX_TTL** — bumped on every write           |
//! | `vesting_key(donor)`         | Persistent    | **VESTING_TTL** — bumped on every read/write    |
//!
//! ## Instance-storage extension
//!
//! `bump_instance` is called at the top of every public entrypoint so the
//! contract's instance storage (admin, counters, indexes) never lapses.
//!
//! ## Domain-deadline rule for Locked payments
//!
//! When a payment is locked via `create_escrow`, its TTL is extended to at
//! least `DISPUTE_HORIZON_LEDGERS` (≈ 60 days) regardless of the default
//! threshold.  This ensures the on-chain record outlives the longest
//! possible dispute-resolution window.

use soroban_sdk::{Env, IntoVal, Val};

// ── Constants ─────────────────────────────────────────────────────────────────

/// Approximate number of Stellar ledgers per day (5-second ledger close time).
pub const LEDGERS_PER_DAY: u32 = 17_280;

/// Threshold below which a persistent key is considered at-risk and must be
/// bumped.  Set to 30 days so bumps happen well before expiry.
pub const PAYMENT_TTL_THRESHOLD: u32 = LEDGERS_PER_DAY * 30;

/// Target TTL after a bump for a regular payment entry (≈ 90 days).
pub const PAYMENT_EXTEND_TO: u32 = LEDGERS_PER_DAY * 90;

/// Minimum TTL for a *locked* (escrowed) payment entry.
/// Must outlive the longest dispute-resolution window.
/// Set to 60 days so there is always a 30-day margin beyond the default
/// 7-day dispute timeout (configurable up to the admin's choosing).
pub const DISPUTE_HORIZON_LEDGERS: u32 = LEDGERS_PER_DAY * 60;

/// Threshold / extend-to for pledge records (≈ 90 / 180 days).
pub const PLEDGE_TTL_THRESHOLD: u32 = LEDGERS_PER_DAY * 90;
pub const PLEDGE_EXTEND_TO: u32 = LEDGERS_PER_DAY * 180;

/// Threshold / extend-to for address-indexed Vecs (payer, payee, status).
pub const INDEX_TTL_THRESHOLD: u32 = LEDGERS_PER_DAY * 30;
pub const INDEX_EXTEND_TO: u32 = LEDGERS_PER_DAY * 90;

/// Threshold / extend-to for vesting schedules (≈ 1 / 3 years).
pub const VESTING_TTL_THRESHOLD: u32 = LEDGERS_PER_DAY * 365;
pub const VESTING_EXTEND_TO: u32 = LEDGERS_PER_DAY * 365 * 3;

/// Threshold / extend-to for the contract instance itself.
pub const INSTANCE_TTL_THRESHOLD: u32 = LEDGERS_PER_DAY * 30;
pub const INSTANCE_EXTEND_TO: u32 = LEDGERS_PER_DAY * 90;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Extend the TTL of a persistent key if its remaining TTL is below
/// `threshold`.  If it is already above the threshold this is a no-op.
///
/// `extend_to` is the target TTL after the bump (must be ≥ `threshold`).
pub fn bump_persistent<K>(env: &Env, key: &K, threshold: u32, extend_to: u32)
where
    K: IntoVal<Env, Val>,
{
    env.storage()
        .persistent()
        .extend_ttl(key, threshold, extend_to);
}

/// Bump a payment entry with the regular payment policy.
pub fn bump_payment<K>(env: &Env, key: &K)
where
    K: IntoVal<Env, Val>,
{
    bump_persistent(env, key, PAYMENT_TTL_THRESHOLD, PAYMENT_EXTEND_TO);
}

/// Bump a *locked* (escrowed) payment entry to at least the dispute horizon.
///
/// This is called once when `create_escrow` transitions a payment to Locked.
/// Subsequent reads/writes use `bump_payment` (which may be a no-op if the
/// entry already has > DISPUTE_HORIZON_LEDGERS remaining).
pub fn bump_locked_payment<K>(env: &Env, key: &K)
where
    K: IntoVal<Env, Val>,
{
    // Use the dispute horizon as *both* the threshold and the extend-to so
    // we unconditionally hold at least that many ledgers.
    bump_persistent(env, key, DISPUTE_HORIZON_LEDGERS, DISPUTE_HORIZON_LEDGERS);
}

/// Bump a pledge entry with the pledge policy.
pub fn bump_pledge<K>(env: &Env, key: &K)
where
    K: IntoVal<Env, Val>,
{
    bump_persistent(env, key, PLEDGE_TTL_THRESHOLD, PLEDGE_EXTEND_TO);
}

/// Bump an address or status index Vec with the index policy.
pub fn bump_index<K>(env: &Env, key: &K)
where
    K: IntoVal<Env, Val>,
{
    bump_persistent(env, key, INDEX_TTL_THRESHOLD, INDEX_EXTEND_TO);
}

/// Bump a vesting schedule entry with the vesting policy.
pub fn bump_vesting<K>(env: &Env, key: &K)
where
    K: IntoVal<Env, Val>,
{
    bump_persistent(env, key, VESTING_TTL_THRESHOLD, VESTING_EXTEND_TO);
}

/// Extend the contract instance TTL.  Call at the top of every entrypoint.
pub fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_EXTEND_TO);
}
