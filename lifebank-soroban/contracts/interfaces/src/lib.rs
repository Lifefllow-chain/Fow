#![no_std]
//! Shared event envelope, versioned payload structs, and cross-contract domain
//! types/clients for all lifebank-soroban contracts.
//!
//! See `lifebank-soroban/EVENTS.md` for the canonical event catalog this crate
//! implements, and `lifebank-soroban/docs/interface-compatibility.md` for the
//! append-only enum policy that governs `domain`.
//!
//! Envelope convention (every event published by every contract):
//! topics = `(contract_domain: Symbol, event_name: Symbol, schema_version: u32)`
//! data   = a `#[contracttype]` struct defined in `events::<contract_domain>`.
//!
//! Payload structs are append-only: add fields, never remove/rename/retype one
//! in place. A breaking change to an existing event gets a new struct and the
//! emitter bumps `schema_version` for that event name.

pub mod clients;
pub mod domain;
pub mod envelope;
pub mod events;
