//! # Shared storage TTL management for the LifeBank Soroban workspace
//!
//! This module is the single source of truth for storage-class TTL policy used
//! by both the `payments` and `inventory` contracts. Every persistent key in
//! those contracts must be assigned a [`StorageClass`] and bumped on access so
//! that live funds and blood units never silently archive.
//!
//! ## Storage class policy
//!
//! | Class                | Threshold (ledgers) | Extend-to (ledgers) | Rationale |
//! |----------------------|---------------------|---------------------|-----------|
//! | `Payment`            | 30 days             | 120 days            | Escrow `Payment` records must outlive the dispute-resolution horizon (30-day dispute window plus a generous safety margin) so a payment can always be read, disputed, and settled. |
//! | `BloodUnit`          | 30 days             | 180 days            | Blood-unit records must outlive product expiry (whole-blood/component shelf life up to ~42 days, plus margin) so inventory reads never fail on a unit that is still physically valid. |
//! | `Config`             | 30 days             | 365 days            | Long-lived contract configuration; bumped on every entrypoint so it effectively never archives. |
//! | `Instance`           | 30 days             | 365 days            | Contract instance storage (admin, counters, metadata); extended at entrypoint level. |
//!
//! All durations are expressed in ledgers assuming the Stellar mainnet target
//! of ~5 seconds per ledger ([`LEDGERS_PER_DAY`] = 17_280).
//!
//! ## Usage
//!
//! Persistent keys are bumped on every read/write via [`bump`]; instance
//! storage is extended once per entrypoint via [`bump_instance`]. Domain
//! deadlines (e.g. the dispute-resolution horizon at `lock`, or product expiry
//! for blood units) can be honoured with [`bump_to`], which extends an entry to
//! at least a caller-supplied number of ledgers.

use soroban_sdk::{Env, IntoVal, Val};

/// Number of ledgers in one day, assuming ~5 second ledger close time.
pub const LEDGERS_PER_DAY: u32 = 17_280;

/// Default threshold (ledgers below which an entry is bumped) for all classes.
pub const DEFAULT_THRESHOLD: u32 = 30 * LEDGERS_PER_DAY;

/// Extend-to target for `Payment` entries: 120 days.
///
/// Must exceed the dispute-resolution horizon so a locked payment can always
/// be read and disputed.
pub const PAYMENT_EXTEND_TO: u32 = 120 * LEDGERS_PER_DAY;

/// Extend-to target for `BloodUnit` entries: 180 days.
///
/// Must exceed product expiry so a still-valid unit is never archived.
pub const BLOOD_UNIT_EXTEND_TO: u32 = 180 * LEDGERS_PER_DAY;

/// Extend-to target for `Config` entries: 365 days.
pub const CONFIG_EXTEND_TO: u32 = 365 * LEDGERS_PER_DAY;

/// Extend-to target for instance storage: 365 days.
pub const INSTANCE_EXTEND_TO: u32 = 365 * LEDGERS_PER_DAY;

/// The dispute-resolution horizon, in ledgers (30 days).
///
/// At `lock`, a payment entry's TTL is extended to at least this horizon so
/// the dispute flow can always be exercised.
pub const DISPUTE_HORIZON: u32 = 30 * LEDGERS_PER_DAY;

/// Storage class of a persistent key, determining its TTL policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageClass {
    /// Escrow `Payment` records.
    Payment,
    /// Blood-unit records.
    BloodUnit,
    /// Long-lived contract configuration.
    Config,
}

impl StorageClass {
    /// The threshold (in ledgers) below which an entry of this class is bumped.
    pub const fn threshold(self) -> u32 {
        match self {
            StorageClass::Payment => DEFAULT_THRESHOLD,
            StorageClass::BloodUnit => DEFAULT_THRESHOLD,
            StorageClass::Config => DEFAULT_THRESHOLD,
        }
    }

    /// The extend-to target (in ledgers) for an entry of this class.
    pub const fn extend_to(self) -> u32 {
        match self {
            StorageClass::Payment => PAYMENT_EXTEND_TO,
            StorageClass::BloodUnit => BLOOD_UNIT_EXTEND_TO,
            StorageClass::Config => CONFIG_EXTEND_TO,
        }
    }
}

/// Bump a persistent entry's TTL according to its [`StorageClass`].
///
/// Call this on every persistent read/write path so that live entries never
/// archive. If the entry does not exist this is a no-op (the host ignores
/// `extend_ttl` for missing keys).
pub fn bump<K>(env: &Env, key: &K, class: StorageClass)
where
    K: IntoVal<Env, Val>,
{
    env.storage()
        .persistent()
        .extend_ttl(key, class.threshold(), class.extend_to());
}

/// Extend a persistent entry's TTL to at least `min_ledgers` from now,
/// honouring a domain deadline (e.g. the dispute horizon or product expiry).
///
/// The entry is extended to the larger of the class's `extend_to` target and
/// `min_ledgers`, so a domain deadline can only lengthen, never shorten, the
/// policy horizon.
pub fn bump_to<K>(env: &Env, key: &K, class: StorageClass, min_ledgers: u32)
where
    K: IntoVal<Env, Val>,
{
    let extend_to = class.extend_to().max(min_ledgers);
    env.storage()
        .persistent()
        .extend_ttl(key, class.threshold(), extend_to);
}

/// Extend the contract's instance storage TTL.
///
/// Call this once at the top of every entrypoint so instance data (admin,
/// counters, metadata) never archives.
pub fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(DEFAULT_THRESHOLD, INSTANCE_EXTEND_TO);
}