#![no_std]
//! Shared event envelope and versioned payload structs for all lifebank-soroban
//! contracts. See `lifebank-soroban/EVENTS.md` for the canonical event catalog
//! this crate implements.
//!
//! Envelope convention (every event published by every contract):
//! topics = `(contract_domain: Symbol, event_name: Symbol, schema_version: u32)`
//! data   = a `#[contracttype]` struct defined in `events::<contract_domain>`.
//!
//! Payload structs are append-only: add fields, never remove/rename/retype one
//! in place. A breaking change to an existing event gets a new struct and the
//! emitter bumps `schema_version` for that event name.
//!
//! The `ttl` module holds the workspace-wide storage TTL policy shared by the
//! `payments` and `inventory` contracts: per-storage-class threshold/extend-to
//! targets and the `bump` / `bump_to_deadline` helpers.

pub mod envelope;
pub mod events;
pub mod ttl;
