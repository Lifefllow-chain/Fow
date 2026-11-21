use soroban_sdk::{contractclient, Address, Env, String};

use crate::types::{BloodRequest, BloodStatus, BloodType, BloodUnit, DisputeReason, Payment, PaymentStatus};

/// Cross-contract interface for the requests contract.
/// Import `RequestContractClient` from this module for cross-contract calls.
#[contractclient(name = "RequestContractClient")]
pub trait RequestContractInterface {
    fn get_request(env: Env, request_id: u64) -> BloodRequest;
}

/// Cross-contract interface for the inventory contract.
/// Import `InventoryContractClient` from this module for cross-contract calls.
#[contractclient(name = "InventoryContractClient")]
pub trait InventoryContractInterface {
    fn get_blood_unit(env: Env, blood_unit_id: u64) -> BloodUnit;
    fn get_units_by_blood_type(env: Env, blood_type: BloodType) -> Vec<u64>;
    fn update_status(
        env: Env,
        unit_id: u64,
        new_status: BloodStatus,
        authorized_by: Address,
        reason: Option<String>,
    ) -> BloodUnit;
    fn mark_delivered(
        env: Env,
        unit_id: u64,
        authorized_by: Address,
        delivery_location: String,
    ) -> BloodUnit;
    fn get_admin(env: Env) -> Address;
}

/// Cross-contract interface for the payments contract.
/// Import `PaymentContractClient` from this module for cross-contract calls.
#[contractclient(name = "PaymentContractClient")]
pub trait PaymentContractInterface {
    fn get_payment(env: Env, payment_id: u64) -> Payment;
    fn update_status(env: Env, payment_id: u64, status: PaymentStatus);
    fn record_dispute(env: Env, payment_id: u64, reason: DisputeReason, case_id: String);
}

// Vec needs to be in scope for the generated client code
use soroban_sdk::Vec;
