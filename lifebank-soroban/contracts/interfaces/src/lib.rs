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
//! # Cross-contract types
//!
//! This crate is the single source of truth for every type that crosses a
//! contract boundary. Domain contracts and their callers (e.g. the coordinator)
//! both depend on the definitions in [`types`] and the client traits in
//! [`clients`]; no contract hand-mirrors another contract's types.
//!
//! # Append-only enum policy
//!
//! `#[contracttype]` enums used across contract boundaries are **append-only**:
//! new variants may be added at the end, but existing variants must never be
//! removed or reordered. Soroban encodes enum variants by discriminant, so
//! reordering silently shifts every downstream decode — a correctness bug that
//! type-checks fine and only surfaces at runtime. Removing or reordering a
//! variant requires a schema-version bump (see #31) and a corresponding update
//! to the committed spec snapshots checked by CI.

pub mod clients;
pub mod envelope;
pub mod events;
pub mod ttl;
pub mod types;

pub use clients::*;
pub use types::*;