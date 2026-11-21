use soroban_sdk::{contracttype, Address, String};

pub use interfaces::types::{BloodRequest, BloodType, RequestStatus};

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum BloodComponent {
    WholeBlood,
    RedCells,
    Plasma,
    Platelets,
    Cryoprecipitate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[contracttype]
pub enum Urgency {
    Critical,
    Urgent,
    Routine,
    Scheduled,
}

impl Urgency {
    pub fn priority(&self) -> u32 {
        match self {
            Self::Critical => 4,
            Self::Urgent => 3,
            Self::Routine => 2,
            Self::Scheduled => 1,
        }
    }
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
