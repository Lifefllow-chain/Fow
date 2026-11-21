use soroban_sdk::{contracttype, Address, Map, String, Symbol, Vec};

// ── Blood classification ───────────────────────────────────────────────────────

/// All eight ABO/Rh blood groups.
///
/// APPEND-ONLY: never remove or reorder variants — doing so shifts XDR
/// discriminants and silently corrupts stored values across all contracts.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BloodType {
    APositive,
    ANegative,
    BPositive,
    BNegative,
    ABPositive,
    ABNegative,
    OPositive,
    ONegative,
}

impl BloodType {
    /// Whether a donor of this type can give to a recipient of `recipient` type.
    pub fn can_donate_to(self, recipient: BloodType) -> bool {
        use BloodType::*;
        match (self, recipient) {
            (ONegative, _) => true,
            (OPositive, APositive | BPositive | ABPositive | OPositive) => true,
            (ANegative, APositive | ANegative | ABPositive | ABNegative) => true,
            (APositive, APositive | ABPositive) => true,
            (BNegative, BPositive | BNegative | ABPositive | ABNegative) => true,
            (BPositive, BPositive | ABPositive) => true,
            (ABNegative, ABPositive | ABNegative) => true,
            (ABPositive, ABPositive) => true,
            _ => false,
        }
    }
}

/// Blood product component type.
///
/// APPEND-ONLY: see enum discipline note in crate root.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BloodComponent {
    WholeBlood,
    RedCells,
    Plasma,
    Platelets,
    Cryoprecipitate,
}

/// Current state of a blood unit in the supply chain.
///
/// APPEND-ONLY: see enum discipline note in crate root.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BloodStatus {
    Available,
    Reserved,
    InTransit,
    Delivered,
    Expired,
    Compromised,
    Disposed,
}

impl BloodStatus {
    /// Terminal states cannot transition to any other status.
    /// `Compromised` is not terminal — units must move to `Disposed`.
    pub fn is_terminal(self) -> bool {
        matches!(self, BloodStatus::Delivered | BloodStatus::Disposed)
    }

    /// All variants in declaration order — used for exhaustive matrix tests.
    pub const ALL: [BloodStatus; 7] = [
        BloodStatus::Available,
        BloodStatus::Reserved,
        BloodStatus::InTransit,
        BloodStatus::Delivered,
        BloodStatus::Expired,
        BloodStatus::Compromised,
        BloodStatus::Disposed,
    ];
}

/// Canonical set of legal `(from, to)` blood-status transitions.
/// Every contract-level check must derive from this list.
pub const ALLOWED_BLOOD_STATUS_TRANSITIONS: &[(BloodStatus, BloodStatus)] = {
    use BloodStatus::*;
    &[
        (Available, Reserved),
        (Available, Expired),
        (Available, Compromised),
        (Reserved, InTransit),
        (Reserved, Available),
        (Reserved, Expired),
        (Reserved, Compromised),
        (InTransit, Delivered),
        (InTransit, Expired),
        (InTransit, Compromised),
        (Expired, Disposed),
        (Compromised, Disposed),
    ]
};

/// Returns true when transitioning `from` → `to` is legal.
pub fn is_valid_blood_transition(from: BloodStatus, to: BloodStatus) -> bool {
    ALLOWED_BLOOD_STATUS_TRANSITIONS
        .iter()
        .any(|&(a, b)| a == from && b == to)
}

// ── Request types ──────────────────────────────────────────────────────────────

/// Urgency levels for a blood request.
///
/// APPEND-ONLY: see enum discipline note in crate root.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Urgency {
    Critical,
    Urgent,
    Routine,
    Scheduled,
}

impl Urgency {
    /// Numeric priority — higher = more urgent.
    pub fn priority(self) -> u32 {
        match self {
            Self::Critical => 4,
            Self::Urgent => 3,
            Self::Routine => 2,
            Self::Scheduled => 1,
        }
    }
}

/// Lifecycle status of a blood request.
///
/// APPEND-ONLY: see enum discipline note in crate root.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequestStatus {
    Pending,
    Approved,
    Fulfilled,
    Cancelled,
}

/// A blood request as stored and returned by the requests contract.
/// This is the authoritative definition — callers must not redeclare it.
#[contracttype]
#[derive(Clone, Debug)]
pub struct BloodRequest {
    pub id: u64,
    pub hospital_id: Address,
    pub blood_type: BloodType,
    pub component: BloodComponent,
    pub quantity_ml: u32,
    pub urgency: Urgency,
    pub created_timestamp: u64,
    pub required_by_timestamp: u64,
    pub status: RequestStatus,
    pub assigned_units: Vec<u64>,
    pub fulfilled_quantity_ml: u32,
    /// Reservation ID on the inventory contract, set when units are reserved.
    pub reservation_id: Option<u64>,
}

// ── Blood unit ─────────────────────────────────────────────────────────────────

/// A blood unit as stored and returned by the inventory contract.
/// This is the authoritative definition — callers must not redeclare it.
#[contracttype]
#[derive(Clone, Debug)]
pub struct BloodUnit {
    pub id: u64,
    pub blood_type: BloodType,
    pub quantity_ml: u32,
    pub bank_id: Address,
    pub donor_id: Option<Address>,
    pub donation_timestamp: u64,
    pub expiration_timestamp: u64,
    pub status: BloodStatus,
    pub metadata: Map<Symbol, String>,
}

impl BloodUnit {
    pub fn is_expired(&self, current_time: u64) -> bool {
        current_time >= self.expiration_timestamp
    }

    pub fn shelf_life_remaining(&self, current_time: u64) -> i64 {
        (self.expiration_timestamp as i64) - (current_time as i64)
    }
}

// ── Payment types ──────────────────────────────────────────────────────────────

/// Lifecycle status of a payment / escrow record.
///
/// APPEND-ONLY: see enum discipline note in crate root.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Locked,
    Released,
    Refunded,
    Disputed,
    Cancelled,
}

/// Reason codes for raising a payment dispute.
///
/// APPEND-ONLY: see enum discipline note in crate root.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DisputeReason {
    FailedDelivery,
    TemperatureExcursion,
    PaymentContested,
    WrongItem,
    DamagedGoods,
    LateDelivery,
    Other,
}

/// A payment / escrow record as stored and returned by the payments contract.
/// This is the authoritative definition — callers must not redeclare it.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Payment {
    pub id: u64,
    pub request_id: u64,
    pub payer: Address,
    pub payee: Address,
    pub amount: i128,
    pub status: PaymentStatus,
    pub created_at: u64,
    pub updated_at: u64,
    pub dispute_reason_code: Option<u32>,
    pub dispute_case_id: Option<String>,
    pub dispute_resolved: bool,
    /// Token contract address — set only for escrow-backed payments.
    pub token: Option<Address>,
}
