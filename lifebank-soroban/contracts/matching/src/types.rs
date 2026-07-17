use soroban_sdk::{contracttype, Address, Vec};

// Shared domain types now come from the interfaces crate — no local copies.
// Previously this file held hand-mirrored BloodType, BloodStatus, BloodUnit,
// BloodComponent, Urgency, RequestStatus, and BloodRequest.  Those definitions
// are now the authoritative versions in lifebank-interfaces; callers import
// them from there (via re-exports in lib.rs) and the compiler enforces a
// single definition across all contracts (#32).

// ---------------------------------------------------------------------------
// Matching-specific types
// ---------------------------------------------------------------------------

use lifebank_interfaces::BloodType;

/// Describes how closely a unit's blood type matches the request.
///
/// APPEND-ONLY: never remove or reorder variants — doing so shifts XDR
/// discriminants and corrupts stored values.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MatchKind {
    /// Unit blood type == request blood type.
    Exact,
    /// Unit blood type is ABO/Rh compatible but not identical.
    Compatible,
}

/// A single matched blood unit with its computed score.
#[contracttype]
#[derive(Clone, Debug)]
pub struct MatchedUnit {
    pub unit_id: u64,
    pub blood_type: BloodType,
    pub quantity_ml: u32,
    pub bank_id: Address,
    pub expiration_timestamp: u64,
    pub score: u32,
    pub match_kind: MatchKind,
}

/// Full result returned by `match_request`.
#[contracttype]
#[derive(Clone, Debug)]
pub struct MatchResult {
    pub request_id: u64,
    pub matched_units: Vec<MatchedUnit>,
    pub total_matched_ml: u32,
    pub remaining_ml: u32,
    pub partial_fulfillment: bool,
}

/// Storage keys for the matching contract.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    InventoryContract,
    RequestsContract,
    Initialized,
    Paused,
}
