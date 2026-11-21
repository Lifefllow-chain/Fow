//! Cross-contract client traits for each domain contract's public API.
//!
//! A caller (e.g. the coordinator) depends on these traits instead of
//! hand-writing a `contractclient` block, so the client signature is defined
//! once and shared with the implementing contract.

use crate::domain::{
    BloodRequest, BloodStatus, BloodUnit, CredentialType, DisputeReason, Payment, Reservation,
};
use soroban_sdk::{contractclient, Address, Env, String, Vec};

#[contractclient(name = "RequestContractClient")]
pub trait RequestContractInterface {
    fn get_request(env: Env, request_id: u64) -> BloodRequest;
}

#[contractclient(name = "InventoryContractClient")]
pub trait InventoryContractInterface {
    fn get_blood_unit(env: Env, blood_unit_id: u64) -> BloodUnit;
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
    fn reserve_blood(
        env: Env,
        requester: Address,
        unit_ids: Vec<u64>,
        request_id: u64,
        duration_seconds: u64,
    ) -> u64;
    fn release_reservation(env: Env, reservation_id: u64);
    fn get_reservation(env: Env, reservation_id: u64) -> Reservation;
    fn extend_reservation(
        env: Env,
        reservation_id: u64,
        additional_seconds: u64,
        authorized_by: Address,
    ) -> Reservation;
}

#[contractclient(name = "PaymentContractClient")]
pub trait PaymentContractInterface {
    fn get_payment(env: Env, payment_id: u64) -> Payment;
    fn get_coordinator(env: Env) -> Option<Address>;
    fn release_escrow(env: Env, caller: Address, payment_id: u64);
    fn refund_escrow(env: Env, caller: Address, payment_id: u64);
    fn record_dispute(
        env: Env,
        caller: Address,
        payment_id: u64,
        reason: DisputeReason,
        case_id: String,
    );
}

#[contractclient(name = "IdentityContractClient")]
pub trait IdentityContractInterface {
    fn is_valid(env: Env, subject: Address, cred: CredentialType, allow_grace: bool) -> bool;
}
