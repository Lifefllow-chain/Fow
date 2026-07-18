use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq, Copy)]
pub struct TemperatureReading {
    pub temperature_celsius_x100: i32,
    pub timestamp: u64,
    pub is_violation: bool,
}

impl Default for TemperatureReading {
    fn default() -> Self {
        TemperatureReading {
            temperature_celsius_x100: 0,
            timestamp: 0,
            is_violation: false,
        }
    }
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq, Copy)]
pub struct TemperatureThreshold {
    pub min_celsius_x100: i32,
    pub max_celsius_x100: i32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemperatureSummary {
    pub count: u32,
    pub avg_celsius_x100: i32,
    pub min_celsius_x100: i32,
    pub max_celsius_x100: i32,
    pub violation_count: u32,
}

/// Summary of a sustained temperature excursion, passed to the coordinator
/// when automatically raising a payment dispute.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExcursionSummary {
    /// Blood unit affected
    pub unit_id: u64,
    /// Number of consecutive violations that triggered this excursion
    pub violation_count: u32,
    /// Peak temperature recorded during the excursion (×100 scale)
    pub peak_celsius_x100: i32,
    /// Ledger timestamp when the excursion was first detected
    pub detected_at: u64,
}

#[contracttype]
#[derive(Clone, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Threshold(u64),
    TempPage(u64, u32),
    TempPageLen(u64, u32),
    /// Tracks consecutive violation streak for a blood unit
    ConsecutiveViolationStreak(u64),
    /// Tracks if unit has been compromised (3+ consecutive violations)
    IsCompromised(u64),
    Paused,
    /// Address of the coordinator contract for cross-contract dispute escalation
    CoordinatorContract,
    /// Whitelisted IoT oracle addresses allowed to report excursions
    OracleWhitelist(soroban_sdk::Address),
}

// ── Events (#53) ─────────────────────────────────────────────────────────────

/// Emitted once, when the contract is initialized.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InitializedEvent {
    pub admin: Address,
    pub initialized_at: u64,
}

/// Emitted by `pause`/`unpause`.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PauseChangedEvent {
    pub admin: Address,
    pub paused: bool,
    pub changed_at: u64,
}

/// Emitted by `set_threshold`.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThresholdSetEvent {
    pub unit_id: u64,
    pub min_celsius_x100: i32,
    pub max_celsius_x100: i32,
    pub set_at: u64,
}

/// Emitted by `log_reading`.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadingLoggedEvent {
    pub unit_id: u64,
    pub temperature_celsius_x100: i32,
    pub timestamp: u64,
    pub is_violation: bool,
    pub consecutive_streak: u32,
    pub compromised: bool,
}

/// Emitted by `reset_compromised_status`.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompromisedStatusResetEvent {
    pub unit_id: u64,
    pub reset_at: u64,
}

/// Emitted by `set_coordinator`.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoordinatorSetEvent {
    pub coordinator: Address,
    pub set_at: u64,
}

/// Emitted by `add_oracle`.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OracleAddedEvent {
    pub oracle: Address,
    pub added_at: u64,
}

/// Emitted by `report_excursion_to_coordinator`.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExcursionReportedEvent {
    pub unit_id: u64,
    pub payment_id: u64,
    pub violation_count: u32,
    pub peak_celsius_x100: i32,
    pub detected_at: u64,
    pub reported_by: Address,
}

/// Emitted by `upgrade`.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpgradedEvent {
    pub new_wasm_hash: soroban_sdk::BytesN<32>,
}

/// Emitted by `migrate`.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MigratedEvent {
    pub new_schema_version: u32,
}
