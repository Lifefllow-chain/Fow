use soroban_sdk::contracttype;

// Cross-contract types live in the shared `interfaces` crate — one definition,
// compiler-enforced. Do not re-declare them here.
pub use interfaces::types::{ExcursionSummary, WorkflowRecord, WorkflowStatus};

/// Coordinator-local storage keys only.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    /// Guardian can pause instantly; only Admin can unpause.
    Guardian,
    RequestContract,
    InventoryContract,
    PaymentContract,
    IdentityContract,
    MatchingContract,
    Workflow(u64),
    /// Granular pause flags — stored as Option<u64> (paused_at timestamp).
    /// "Paused" is the legacy single bool kept for compat; new code uses PauseFlag.
    Paused,
    /// Per-operation pause flag. Value = ledger timestamp when paused.
    PauseFlag(soroban_sdk::Symbol),
}
