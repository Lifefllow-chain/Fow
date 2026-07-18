//! Typed events for the temperature contract (#53).
//!
//! Every event's first topic is the standard envelope
//! `(domain: Symbol, event: Symbol, schema_version: u32)`, identical in
//! shape across the matching/temperature/reputation/analytics contracts —
//! see `EVENTS.md` for the full catalog.

use crate::types::{
    CompromisedStatusResetEvent, CoordinatorSetEvent, ExcursionReportedEvent, InitializedEvent,
    MigratedEvent, OracleAddedEvent, PauseChangedEvent, ReadingLoggedEvent, ThresholdSetEvent,
    UpgradedEvent,
};
use soroban_sdk::{symbol_short, Address, BytesN, Env};

/// Domain for every event this contract publishes.
const DOMAIN: soroban_sdk::Symbol = symbol_short!("temp");
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

pub fn emit_pause_changed(env: &Env, admin: &Address, paused: bool, changed_at: u64) {
    env.events().publish(
        (DOMAIN, symbol_short!("pause"), EVENT_SCHEMA_VERSION),
        PauseChangedEvent {
            admin: admin.clone(),
            paused,
            changed_at,
        },
    );
}

pub fn emit_threshold_set(
    env: &Env,
    unit_id: u64,
    min_celsius_x100: i32,
    max_celsius_x100: i32,
    set_at: u64,
) {
    env.events().publish(
        (DOMAIN, symbol_short!("thresh"), EVENT_SCHEMA_VERSION),
        ThresholdSetEvent {
            unit_id,
            min_celsius_x100,
            max_celsius_x100,
            set_at,
        },
    );
}

pub fn emit_reading_logged(
    env: &Env,
    unit_id: u64,
    temperature_celsius_x100: i32,
    timestamp: u64,
    is_violation: bool,
    consecutive_streak: u32,
    compromised: bool,
) {
    env.events().publish(
        (DOMAIN, symbol_short!("reading"), EVENT_SCHEMA_VERSION),
        ReadingLoggedEvent {
            unit_id,
            temperature_celsius_x100,
            timestamp,
            is_violation,
            consecutive_streak,
            compromised,
        },
    );
}

pub fn emit_compromised_status_reset(env: &Env, unit_id: u64, reset_at: u64) {
    env.events().publish(
        (DOMAIN, symbol_short!("cmp_rst"), EVENT_SCHEMA_VERSION),
        CompromisedStatusResetEvent { unit_id, reset_at },
    );
}

pub fn emit_coordinator_set(env: &Env, coordinator: &Address, set_at: u64) {
    env.events().publish(
        (DOMAIN, symbol_short!("coord"), EVENT_SCHEMA_VERSION),
        CoordinatorSetEvent {
            coordinator: coordinator.clone(),
            set_at,
        },
    );
}

pub fn emit_oracle_added(env: &Env, oracle: &Address, added_at: u64) {
    env.events().publish(
        (DOMAIN, symbol_short!("oracle"), EVENT_SCHEMA_VERSION),
        OracleAddedEvent {
            oracle: oracle.clone(),
            added_at,
        },
    );
}

pub fn emit_excursion_reported(
    env: &Env,
    unit_id: u64,
    payment_id: u64,
    violation_count: u32,
    peak_celsius_x100: i32,
    detected_at: u64,
    reported_by: &Address,
) {
    env.events().publish(
        (DOMAIN, symbol_short!("excursion"), EVENT_SCHEMA_VERSION),
        ExcursionReportedEvent {
            unit_id,
            payment_id,
            violation_count,
            peak_celsius_x100,
            detected_at,
            reported_by: reported_by.clone(),
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
