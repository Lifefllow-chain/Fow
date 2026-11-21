use soroban_sdk::{contracttype, Address, String, Vec};

// Cross-contract types — imported from the single authoritative definition.
pub use lifebank_interfaces::{
    BloodComponent, BloodRequest, BloodType, RequestStatus, Urgency,
};

// ── Requests-internal types ───────────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum DataKey {
    Admin,
    InventoryContract,
    RequestCounter,
    Initialized,
    Metadata,
    AuthorizedHospital(Address),
    Request(u64),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ContractMetadata {
    pub name: String,
    pub version: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct RequestCreatedEvent {
    pub request_id: u64,
    pub hospital: Address,
    pub blood_type: BloodType,
    pub quantity_ml: u32,
    pub urgency: u32,
    pub timestamp: u64,
}
