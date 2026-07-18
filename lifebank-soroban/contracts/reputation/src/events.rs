//! Typed events for the reputation contract (#53).
//!
//! Every event's first topic is the standard envelope
//! `(domain: Symbol, event: Symbol, schema_version: u32)`, identical in
//! shape across the matching/temperature/reputation/analytics contracts —
//! see `EVENTS.md` for the full catalog.

use crate::{
    AssignmentRecordedEvent, FraudFlaggedEvent, InitializedEvent, MigratedEvent,
    PauseChangedEvent, PenaltyAppealedEvent, PenaltyAppliedEvent, PenaltyResolvedEvent,
    RatingSubmittedEvent, ScoreUpdatedEvent, UpgradedEvent, ViolationType,
};
use soroban_sdk::{symbol_short, Address, BytesN, Env};

/// Domain for every event this contract publishes.
const DOMAIN: soroban_sdk::Symbol = symbol_short!("rep");
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

pub fn emit_rating_submitted(env: &Env, entity_id: u64, score: i64, timestamp: u64) {
    env.events().publish(
        (DOMAIN, symbol_short!("rating"), EVENT_SCHEMA_VERSION),
        RatingSubmittedEvent {
            entity_id,
            score,
            timestamp,
        },
    );
}

pub fn emit_assignment_recorded(
    env: &Env,
    entity_id: u64,
    completed: bool,
    response_secs: u64,
    timestamp: u64,
) {
    env.events().publish(
        (DOMAIN, symbol_short!("assign"), EVENT_SCHEMA_VERSION),
        AssignmentRecordedEvent {
            entity_id,
            completed,
            response_secs,
            timestamp,
        },
    );
}

pub fn emit_fraud_flagged(env: &Env, entity_id: u64, timestamp: u64) {
    env.events().publish(
        (DOMAIN, symbol_short!("fraud"), EVENT_SCHEMA_VERSION),
        FraudFlaggedEvent {
            entity_id,
            timestamp,
        },
    );
}

pub fn emit_penalty_applied(
    env: &Env,
    entity_id: u64,
    penalty_id: u32,
    violation_type: ViolationType,
    timestamp: u64,
) {
    env.events().publish(
        (DOMAIN, symbol_short!("pen_new"), EVENT_SCHEMA_VERSION),
        PenaltyAppliedEvent {
            entity_id,
            penalty_id,
            violation_type,
            timestamp,
        },
    );
}

pub fn emit_penalty_appealed(env: &Env, entity_id: u64, penalty_id: u32) {
    env.events().publish(
        (DOMAIN, symbol_short!("pen_apl"), EVENT_SCHEMA_VERSION),
        PenaltyAppealedEvent {
            entity_id,
            penalty_id,
        },
    );
}

pub fn emit_penalty_resolved(env: &Env, entity_id: u64, penalty_id: u32, removed: bool) {
    env.events().publish(
        (DOMAIN, symbol_short!("pen_res"), EVENT_SCHEMA_VERSION),
        PenaltyResolvedEvent {
            entity_id,
            penalty_id,
            removed,
        },
    );
}

pub fn emit_score_updated(env: &Env, entity_id: u64, score: i64) {
    env.events().publish(
        (DOMAIN, symbol_short!("updated"), EVENT_SCHEMA_VERSION),
        ScoreUpdatedEvent { entity_id, score },
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
