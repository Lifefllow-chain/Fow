//! Typed events for the matching contract (#53).
//!
//! Every event's first topic is the standard envelope
//! `(domain: Symbol, event: Symbol, schema_version: u32)`, identical in
//! shape across the matching/temperature/reputation/analytics contracts —
//! see `EVENTS.md` for the full catalog.

use crate::types::{InitializedEvent, MatchComputedEvent, MatchedUnit, MigratedEvent, PauseChangedEvent, UpgradedEvent};
use soroban_sdk::{symbol_short, Address, BytesN, Env, Vec};

/// Domain for every event this contract publishes.
const DOMAIN: soroban_sdk::Symbol = symbol_short!("match");
/// Version of the event payload shapes below. Bump when a payload's fields
/// change; unrelated to `TARGET_SCHEMA_VERSION` (storage schema).
const EVENT_SCHEMA_VERSION: u32 = 1;

pub fn emit_initialized(
    env: &Env,
    admin: &Address,
    inventory_contract: &Address,
    requests_contract: &Address,
    initialized_at: u64,
) {
    env.events().publish(
        (DOMAIN, symbol_short!("init"), EVENT_SCHEMA_VERSION),
        InitializedEvent {
            admin: admin.clone(),
            inventory_contract: inventory_contract.clone(),
            requests_contract: requests_contract.clone(),
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

pub fn emit_match_computed(
    env: &Env,
    request_id: u64,
    matched_units: &Vec<MatchedUnit>,
    total_matched_ml: u32,
    remaining_ml: u32,
    partial_fulfillment: bool,
) {
    let mut matched_unit_ids: Vec<u64> = Vec::new(env);
    for i in 0..matched_units.len() {
        matched_unit_ids.push_back(matched_units.get(i).unwrap().unit_id);
    }

    env.events().publish(
        (DOMAIN, symbol_short!("matched"), EVENT_SCHEMA_VERSION),
        MatchComputedEvent {
            request_id,
            matched_unit_ids,
            total_matched_ml,
            remaining_ml,
            partial_fulfillment,
            matched_at: env.ledger().timestamp(),
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
