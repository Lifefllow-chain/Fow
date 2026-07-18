//! Typed events for the analytics contract (#53).
//!
//! Every event's first topic is the standard envelope
//! `(domain: Symbol, event: Symbol, schema_version: u32)`, identical in
//! shape across the matching/temperature/reputation/analytics contracts —
//! see `EVENTS.md` for the full catalog.

use crate::types::{
    InitializedEvent, MetricKind, MetricRecordedEvent, MigratedEvent,
    PaymentReleasedRecordedEvent, PeriodType, ReportingPeriodChangedEvent, UpgradedEvent,
};
use soroban_sdk::{symbol_short, Address, BytesN, Env};

/// Domain for every event this contract publishes.
const DOMAIN: soroban_sdk::Symbol = symbol_short!("anlytcs");
/// Version of the event payload shapes below. Bump when a payload's fields
/// change; unrelated to `TARGET_SCHEMA_VERSION` (storage schema).
const EVENT_SCHEMA_VERSION: u32 = 1;

pub fn emit_initialized(env: &Env, admin: &Address, initialized_at: u64) {
    env.events().publish(
        (DOMAIN, symbol_short!("init"), EVENT_SCHEMA_VERSION),
        InitializedEvent {
            admin: admin.clone(),
            initialized_at,
        },
    );
}

pub fn emit_reporting_period_changed(
    env: &Env,
    old_period_type: PeriodType,
    new_period_type: PeriodType,
    new_duration_secs: u64,
    changed_at: u64,
) {
    env.events().publish(
        (DOMAIN, symbol_short!("per_chg"), EVENT_SCHEMA_VERSION),
        ReportingPeriodChangedEvent {
            old_period_type,
            new_period_type,
            new_duration_secs,
            changed_at,
        },
    );
}

pub fn emit_metric_recorded(
    env: &Env,
    period_index: u64,
    metric: MetricKind,
    lifetime_total: u64,
    recorded_at: u64,
) {
    env.events().publish(
        (DOMAIN, symbol_short!("metric"), EVENT_SCHEMA_VERSION),
        MetricRecordedEvent {
            period_index,
            metric,
            lifetime_total,
            recorded_at,
        },
    );
}

pub fn emit_payment_released_recorded(
    env: &Env,
    period_index: u64,
    amount: i128,
    lifetime_count: u64,
    lifetime_volume: i128,
    recorded_at: u64,
) {
    env.events().publish(
        (DOMAIN, symbol_short!("pay_rel"), EVENT_SCHEMA_VERSION),
        PaymentReleasedRecordedEvent {
            period_index,
            amount,
            lifetime_count,
            lifetime_volume,
            recorded_at,
        },
    );
}

pub fn emit_upgraded(env: &Env, new_wasm_hash: &BytesN<32>) {
    env.events().publish(
        (DOMAIN, symbol_short!("upgraded"), EVENT_SCHEMA_VERSION),
        UpgradedEvent {
            new_wasm_hash: new_wasm_hash.clone(),
        },
    );
}

pub fn emit_migrated(env: &Env, new_schema_version: u32) {
    env.events().publish(
        (DOMAIN, symbol_short!("migrated"), EVENT_SCHEMA_VERSION),
        MigratedEvent { new_schema_version },
    );
}
