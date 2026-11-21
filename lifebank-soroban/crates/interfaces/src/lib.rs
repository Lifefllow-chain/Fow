#![no_std]
//! Shared cross-contract types and client trait definitions for the Lifebank Soroban workspace.
//!
//! # Why this crate exists
//!
//! Without a single source of truth, every caller that inspects a cross-contract
//! return value must re-declare the type locally. If a domain contract adds an enum
//! variant or reorders fields, callers silently mis-decode XDR — a correctness bug
//! that type-checks fine and only appears at runtime (#32).
//!
//! # What belongs here
//!
//! - Every `contracttype` struct/enum that crosses a contract boundary
//! - `contractclient` trait definitions for each domain contract's public API
//! - Pure helper impls (no contract-specific error types)
//!
//! # Enum discipline (IMPORTANT)
//!
//! All `contracttype` enums in this crate are **append-only**. Removing or
//! reordering variants shifts XDR discriminants and silently corrupts existing
//! on-chain state. To remove a variant, bump the schema version and document
//! a migration path. The CI spec-snapshot drift check (`spec-drift.yml`) will
//! catch any breaking change at review time.

pub mod clients;
pub mod types;

pub use types::*;
