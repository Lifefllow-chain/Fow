//! Cross-contract client trait definitions for every domain contract.
//!
//! Each trait below mirrors the public API of a domain contract and is
//! annotated with `#[contractclient]`, which generates a typed client that
//! callers can use to invoke the contract. Because both the implementing
//! contract and every caller depend on this crate, the interface is defined
//! exactly once and any drift is caught by the compiler.
//!
//! Where a contract only *calls* another and the built WASM is available,
//! prefer `soroban_sdk::contractimport!` (see `wasm_imports` module) so the
//! client is generated from the actual compiled interface. The traits here
//! remain the canonical, always-available definition used by the
//! implementing contracts themselves.

use soroban_sdk::{contractclient, Address, BytesN, Env, Vec};

use crate::types::{
    BloodRequest, BloodStatus, BloodType, Coordinates, DonationRecord, DonorProfile,
    InventoryItem, Match, MatchStatus, PaymentRecord, PaymentStatus, RequestStatus,
    TemperatureReading, TemperatureStatus,
};

/// Client for the `requests` contract.
#[contractclient(name = "RequestsClient")]
pub trait RequestsInterface {
    /// Create a new blood request. Returns the request id.
    fn create_request(
        env: Env,
        requester: Address,
        blood_type: BloodType,
        units: u32,
        urgency: u32,
        location: Coordinates,
    ) -> u64;

    /// Fetch a blood request by id.
    fn get_request(env: Env, request_id: u64) -> BloodRequest;

    /// Update the status of a blood request.
    fn update_status(env: Env, request_id: u64, status: RequestStatus);

    /// List all open requests.
    fn list_open_requests(env: Env) -> Vec<BloodRequest>;
}

/// Client for the `inventory` contract.
#[contractclient(name = "InventoryClient")]
pub trait InventoryInterface {
    /// Add units of a given blood type to inventory.
    fn add_units(env: Env, blood_type: BloodType, units: u32);

    /// Remove units of a given blood type from inventory.
    fn remove_units(env: Env, blood_type: BloodType, units: u32);

    /// Get the inventory item for a blood type.
    fn get_item(env: Env, blood_type: BloodType) -> InventoryItem;

    /// Get the current blood status snapshot.
    fn get_status(env: Env) -> BloodStatus;
}

/// Client for the `payments` contract.
#[contractclient(name = "PaymentsClient")]
pub trait PaymentsInterface {
    /// Settle a payment for a fulfilled request. Returns the payment id.
    fn settle(env: Env, request_id: u64, recipient: Address, amount: i128) -> u64;

    /// Fetch a payment record by id.
    fn get_payment(env: Env, payment_id: u64) -> PaymentRecord;

    /// Update the status of a payment.
    fn update_status(env: Env, payment_id: u64, status: PaymentStatus);
}

/// Client for the `identity` contract.
#[contractclient(name = "IdentityClient")]
pub trait IdentityInterface {
    /// Register a donor profile. Returns the donor address.
    fn register_donor(env: Env, donor: Address, blood_type: BloodType) -> Address;

    /// Fetch a donor profile.
    fn get_donor(env: Env, donor: Address) -> DonorProfile;

    /// Record a donation for a donor.
    fn record_donation(env: Env, donor: Address, record: DonationRecord);

    /// Verify whether an address is a registered donor.
    fn is_verified(env: Env, donor: Address) -> bool;
}

/// Client for the `matching` contract.
#[contractclient(name = "MatchingClient")]
pub trait MatchingInterface {
    /// Find and create a match for a request. Returns the match id.
    fn create_match(env: Env, request_id: u64) -> u64;

    /// Fetch a match by id.
    fn get_match(env: Env, match_id: u64) -> Match;

    /// Update the status of a match.
    fn update_status(env: Env, match_id: u64, status: MatchStatus);
}

/// Client for the `temperature` contract.
#[contractclient(name = "TemperatureClient")]
pub trait TemperatureInterface {
    /// Record a temperature reading for a shipment.
    fn record_reading(env: Env, shipment_id: u64, celsius: i32) -> u64;

    /// Fetch a temperature reading by id.
    fn get_reading(env: Env, reading_id: u64) -> TemperatureReading;

    /// Get the temperature status for a shipment.
    fn get_status(env: Env, shipment_id: u64) -> TemperatureStatus;
}

/// Client for the `coordinator` contract.
#[contractclient(name = "CoordinatorClient")]
pub trait CoordinatorInterface {
    /// Orchestrate the full lifecycle for a request. Returns the request id.
    fn orchestrate(env: Env, request_id: u64) -> u64;

    /// Get the current status of a coordinated request.
    fn get_status(env: Env, request_id: u64) -> RequestStatus;

    /// Hash identifying the coordinator's current schema version.
    fn schema_version(env: Env) -> BytesN<32>;
}

/// WASM-generated clients for contracts that only *call* other contracts.
///
/// These are gated behind the `wasm-imports` feature so that building the
/// shared crate itself does not require the domain WASMs to be present. The
/// build script (`scripts/build-all.sh`) builds contracts in dependency order
/// and enables this feature for downstream callers.
#[cfg(feature = "wasm-imports")]
pub mod wasm_imports {
    //! Clients generated directly from compiled contract WASMs.
    //!
    //! Each `contractimport!` reads the WASM produced by the corresponding
    //! domain contract and generates a client from its actual interface,
    //! guaranteeing the caller matches the deployed contract exactly.

    pub mod requests {
        soroban_sdk::contractimport!(
            file = "../../target/wasm32-unknown-unknown/release/lifebank_requests.wasm"
        );
    }

    pub mod inventory {
        soroban_sdk::contractimport!(
            file = "../../target/wasm32-unknown-unknown/release/lifebank_inventory.wasm"
        );
    }

    pub mod payments {
        soroban_sdk::contractimport!(
            file = "../../target/wasm32-unknown-unknown/release/lifebank_payments.wasm"
        );
    }

    pub mod identity {
        soroban_sdk::contractimport!(
            file = "../../target/wasm32-unknown-unknown/release/lifebank_identity.wasm"
        );
    }

    pub mod matching {
        soroban_sdk::contractimport!(
            file = "../../target/wasm32-unknown-unknown/release/lifebank_matching.wasm"
        );
    }

    pub mod temperature {
        soroban_sdk::contractimport!(
            file = "../../target/wasm32-unknown-unknown/release/lifebank_temperature.wasm"
        );
    }
}