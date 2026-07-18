#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, testutils::Ledger as _, Address, Env};

fn setup() -> (Env, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(PaymentContract, ());
    (env, contract_id)
}

fn make_payment(
    env: &Env,
    client: &PaymentContractClient,
    request_id: u64,
    amount: i128,
) -> (u64, Address, Address) {
    let payer = Address::generate(env);
    let payee = Address::generate(env);
    let id = client.create_payment(&request_id, &payer, &payee, &amount);
    (id, payer, payee)
}

/// Deploy a minimal Soroban token contract and mint `amount` to `recipient`.
fn deploy_token_with_balance(env: &Env, admin: &Address, recipient: &Address, amount: i128) -> Address {
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let token_id = token.address();
    let token_admin = soroban_sdk::token::StellarAssetClient::new(env, &token_id);
    token_admin.mint(recipient, &amount);
    token_id
}

// ── create_payment ─────────────────────────────────────────────────────────────

#[test]
fn test_create_payment_increments_counter() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    let (id1, _, _) = make_payment(&env, &client, 1, 1000);
    let (id2, _, _) = make_payment(&env, &client, 2, 2000);
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(client.get_payment_count(), 2);
}

#[test]
fn test_create_payment_rejects_zero_amount() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    let payer = Address::generate(&env);
    let payee = Address::generate(&env);
    let result = client.try_create_payment(&1u64, &payer, &payee, &0i128);
    assert!(result.is_err());
}

#[test]
fn test_create_payment_rejects_negative_amount() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    let payer = Address::generate(&env);
    let payee = Address::generate(&env);
    let result = client.try_create_payment(&1u64, &payer, &payee, &-100i128);
    assert!(result.is_err());
}

#[test]
fn test_create_payment_rejects_same_payer_payee() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    let addr = Address::generate(&env);
    let result = client.try_create_payment(&1u64, &addr, &addr, &1000i128);
    assert!(result.is_err());
}

#[test]
fn test_create_payment_stores_correct_fields() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    env.ledger().with_mut(|l| l.timestamp = 5000);
    let payer = Address::generate(&env);
    let payee = Address::generate(&env);
    let id = client.create_payment(&42u64, &payer, &payee, &999i128);

    let p = client.get_payment(&id);
    assert_eq!(p.request_id, 42);
    assert_eq!(p.payer, payer);
    assert_eq!(p.payee, payee);
    assert_eq!(p.amount, 999);
    assert_eq!(p.status, PaymentStatus::Pending);
    assert_eq!(p.created_at, 5000);
}

// ── get_payment ────────────────────────────────────────────────────────────────

#[test]
fn test_get_payment_returns_not_found_for_missing_id() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    let result = client.try_get_payment(&999u64);
    assert!(result.is_err());
}

#[test]
fn test_get_payment_returns_correct_payment() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    let (id, payer, payee) = make_payment(&env, &client, 10, 500);
    let p = client.get_payment(&id);
    assert_eq!(p.id, id);
    assert_eq!(p.payer, payer);
    assert_eq!(p.payee, payee);
    assert_eq!(p.amount, 500);
}

// ── get_payment_by_request ─────────────────────────────────────────────────────

#[test]
fn test_get_payment_by_request_finds_correct_payment() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    make_payment(&env, &client, 1, 100);
    let (id2, _, _) = make_payment(&env, &client, 99, 200);
    make_payment(&env, &client, 3, 300);

    let p = client.get_payment_by_request(&99u64);
    assert_eq!(p.id, id2);
    assert_eq!(p.request_id, 99);
}

#[test]
fn test_get_payment_by_request_returns_not_found() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    make_payment(&env, &client, 1, 100);
    let result = client.try_get_payment_by_request(&999u64);
    assert!(result.is_err());
}

// ── duplicate-payment prevention (#599) ───────────────────────────────────────

#[test]
fn test_create_payment_rejects_duplicate_request_id() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    // First payment for request 42 succeeds.
    make_payment(&env, &client, 42, 500);
    // Second payment for the same request must be rejected.
    let payer = Address::generate(&env);
    let payee = Address::generate(&env);
    let result = client.try_create_payment(&42u64, &payer, &payee, &500i128);
    assert_eq!(result, Err(Ok(Error::DuplicatePayment)));
}

#[test]
fn test_create_escrow_rejects_duplicate_request_id() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &None);

    let hospital = Address::generate(&env);
    let payee = Address::generate(&env);
    let token_id = deploy_token_with_balance(&env, &admin, &hospital, 10_000);

    // First escrow for request 7 succeeds.
    client.create_escrow(&7u64, &hospital, &payee, &1_000i128, &token_id);

    // Second escrow for the same request must be rejected.
    let result = client.try_create_escrow(&7u64, &hospital, &payee, &500i128, &token_id);
    assert_eq!(result, Err(Ok(Error::DuplicatePayment)));
}

#[test]
fn test_get_payment_by_request_resolves_without_full_scan() {
    // Verify the index lookup returns the correct payment even when many
    // payments exist for other request IDs.
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    for i in 1u64..=20 {
        make_payment(&env, &client, i, 100);
    }
    let target_request_id = 13u64;
    let p = client.get_payment_by_request(&target_request_id);
    assert_eq!(p.request_id, target_request_id);
}

#[test]
fn test_terminal_payment_does_not_block_new_active_payment_for_different_request() {
    // Payments for distinct request IDs must never interfere.
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    let (id1, _, _) = make_payment(&env, &client, 100, 200);
    client.update_status(&id1, &PaymentStatus::Refunded);

    // A payment for a different request must still be accepted.
    let (id2, _, _) = make_payment(&env, &client, 101, 300);
    assert!(id2 > id1);
    let p = client.get_payment_by_request(&101u64);
    assert_eq!(p.id, id2);
}

// ── get_payments_by_payer ──────────────────────────────────────────────────────

#[test]
fn test_get_payments_by_payer_returns_only_payer_payments() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    let payer_a = Address::generate(&env);
    let payee = Address::generate(&env);

    client.create_payment(&1u64, &payer_a, &payee, &100i128);
    client.create_payment(&2u64, &payer_a, &payee, &200i128);
    make_payment(&env, &client, 3, 300);

    let page = client.get_payments_by_payer(&payer_a, &0u32, &20u32);
    assert_eq!(page.items.len(), 2);
    assert_eq!(page.total, 2);
}

#[test]
fn test_get_payments_by_payer_empty_result() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    let stranger = Address::generate(&env);
    let page = client.get_payments_by_payer(&stranger, &0u32, &20u32);
    assert_eq!(page.items.len(), 0);
    assert_eq!(page.total, 0);
}

#[test]
fn test_get_payments_by_payer_pagination() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    let payer = Address::generate(&env);
    let payee = Address::generate(&env);

    for i in 1u64..=5 {
        client.create_payment(&i, &payer, &payee, &(i as i128 * 100));
    }

    let page0 = client.get_payments_by_payer(&payer, &0u32, &2u32);
    assert_eq!(page0.items.len(), 2);
    assert_eq!(page0.total, 5);

    let page1 = client.get_payments_by_payer(&payer, &1u32, &2u32);
    assert_eq!(page1.items.len(), 2);

    let page2 = client.get_payments_by_payer(&payer, &2u32, &2u32);
    assert_eq!(page2.items.len(), 1);
}

// ── get_payments_by_payee ──────────────────────────────────────────────────────

#[test]
fn test_get_payments_by_payee_returns_only_payee_payments() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    let payer = Address::generate(&env);
    let payee_a = Address::generate(&env);

    client.create_payment(&1u64, &payer, &payee_a, &100i128);
    client.create_payment(&2u64, &payer, &payee_a, &200i128);
    make_payment(&env, &client, 3, 300);

    let page = client.get_payments_by_payee(&payee_a, &0u32, &20u32);
    assert_eq!(page.items.len(), 2);
    assert_eq!(page.total, 2);
}

#[test]
fn test_get_payments_by_payee_pagination() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    let payer = Address::generate(&env);
    let payee = Address::generate(&env);

    for i in 1u64..=6 {
        client.create_payment(&i, &payer, &payee, &(i as i128 * 50));
    }

    let page = client.get_payments_by_payee(&payee, &0u32, &4u32);
    assert_eq!(page.items.len(), 4);
    assert_eq!(page.total, 6);

    let page2 = client.get_payments_by_payee(&payee, &1u32, &4u32);
    assert_eq!(page2.items.len(), 2);
}

// ── get_payments_by_status ─────────────────────────────────────────────────────

#[test]
fn test_get_payments_by_status_filters_correctly() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    let (id1, _, _) = make_payment(&env, &client, 1, 100);
    let (id2, _, _) = make_payment(&env, &client, 2, 200);
    make_payment(&env, &client, 3, 300);

    client.update_status(&id1, &PaymentStatus::Locked);
    client.update_status(&id2, &PaymentStatus::Locked);

    let locked = client.get_payments_by_status(&PaymentStatus::Locked, &0u32, &20u32);
    assert_eq!(locked.items.len(), 2);
    assert_eq!(locked.total, 2);

    let pending = client.get_payments_by_status(&PaymentStatus::Pending, &0u32, &20u32);
    assert_eq!(pending.items.len(), 1);
}

#[test]
fn test_get_payments_by_status_empty_when_none_match() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    make_payment(&env, &client, 1, 100);

    let page = client.get_payments_by_status(&PaymentStatus::Released, &0u32, &20u32);
    assert_eq!(page.items.len(), 0);
}

#[test]
fn test_get_payments_by_status_pagination() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    for i in 1u64..=5 {
        let (id, _, _) = make_payment(&env, &client, i, 100);
        client.update_status(&id, &PaymentStatus::Refunded);
    }

    let page0 = client.get_payments_by_status(&PaymentStatus::Refunded, &0u32, &3u32);
    assert_eq!(page0.items.len(), 3);
    assert_eq!(page0.total, 5);

    let page1 = client.get_payments_by_status(&PaymentStatus::Refunded, &1u32, &3u32);
    assert_eq!(page1.items.len(), 2);
}

// ── get_payment_statistics ─────────────────────────────────────────────────────

#[test]
fn test_statistics_empty_when_no_payments() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    let stats = client.get_payment_statistics();
    assert_eq!(stats.total_locked, 0);
    assert_eq!(stats.total_released, 0);
    assert_eq!(stats.total_refunded, 0);
    assert_eq!(stats.count_locked, 0);
    assert_eq!(stats.count_released, 0);
    assert_eq!(stats.count_refunded, 0);
}

#[test]
fn test_statistics_counts_and_totals_correctly() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);

    let (id1, _, _) = make_payment(&env, &client, 1, 1000);
    let (id2, _, _) = make_payment(&env, &client, 2, 2000);
    let (id3, _, _) = make_payment(&env, &client, 3, 500);
    let (id4, _, _) = make_payment(&env, &client, 4, 750);
    make_payment(&env, &client, 5, 300); // stays Pending

    client.update_status(&id1, &PaymentStatus::Locked);
    client.update_status(&id2, &PaymentStatus::Locked);
    client.update_status(&id3, &PaymentStatus::Released);
    client.update_status(&id4, &PaymentStatus::Refunded);

    let stats = client.get_payment_statistics();
    assert_eq!(stats.count_locked, 2);
    assert_eq!(stats.total_locked, 3000);
    assert_eq!(stats.count_released, 1);
    assert_eq!(stats.total_released, 500);
    assert_eq!(stats.count_refunded, 1);
    assert_eq!(stats.total_refunded, 750);
}

#[test]
fn test_statistics_ignores_pending_cancelled_disputed() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    let (id1, _, _) = make_payment(&env, &client, 1, 100);
    let (id2, _, _) = make_payment(&env, &client, 2, 200);
    make_payment(&env, &client, 3, 300); // stays Pending

    client.update_status(&id1, &PaymentStatus::Cancelled);
    client.update_status(&id2, &PaymentStatus::Disputed);

    let stats = client.get_payment_statistics();
    assert_eq!(stats.count_locked, 0);
    assert_eq!(stats.count_released, 0);
    assert_eq!(stats.count_refunded, 0);
    assert_eq!(stats.total_locked, 0);
}

// ── get_payment_timeline ───────────────────────────────────────────────────────

#[test]
fn test_timeline_returns_payments_in_chronological_order() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);

    env.ledger().with_mut(|l| l.timestamp = 3000);
    make_payment(&env, &client, 1, 100);

    env.ledger().with_mut(|l| l.timestamp = 1000);
    make_payment(&env, &client, 2, 200);

    env.ledger().with_mut(|l| l.timestamp = 2000);
    make_payment(&env, &client, 3, 300);

    let page = client.get_payment_timeline(&0u32, &20u32);
    assert_eq!(page.items.len(), 3);
    assert_eq!(page.items.get(0).unwrap().created_at, 1000);
    assert_eq!(page.items.get(1).unwrap().created_at, 2000);
    assert_eq!(page.items.get(2).unwrap().created_at, 3000);
}

#[test]
fn test_timeline_pagination() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);

    for i in 1u64..=5 {
        env.ledger().with_mut(|l| l.timestamp = i * 1000);
        make_payment(&env, &client, i, 100);
    }

    let page0 = client.get_payment_timeline(&0u32, &2u32);
    assert_eq!(page0.items.len(), 2);
    assert_eq!(page0.total, 5);
    assert_eq!(page0.items.get(0).unwrap().created_at, 1000);

    let page1 = client.get_payment_timeline(&1u32, &2u32);
    assert_eq!(page1.items.len(), 2);
    assert_eq!(page1.items.get(0).unwrap().created_at, 3000);

    let page2 = client.get_payment_timeline(&2u32, &2u32);
    assert_eq!(page2.items.len(), 1);
    assert_eq!(page2.items.get(0).unwrap().created_at, 5000);
}

#[test]
fn test_timeline_empty_when_no_payments() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    let page = client.get_payment_timeline(&0u32, &20u32);
    assert_eq!(page.items.len(), 0);
    assert_eq!(page.total, 0);
}

#[test]
fn test_timeline_out_of_range_page_returns_empty() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    make_payment(&env, &client, 1, 100);

    let page = client.get_payment_timeline(&99u32, &20u32);
    assert_eq!(page.items.len(), 0);
    assert_eq!(page.total, 1);
}

// ── update_status ──────────────────────────────────────────────────────────────

#[test]
fn test_update_status_changes_payment_status() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    let (id, _, _) = make_payment(&env, &client, 1, 500);

    client.update_status(&id, &PaymentStatus::Locked);
    let p = client.get_payment(&id);
    assert_eq!(p.status, PaymentStatus::Locked);

    client.update_status(&id, &PaymentStatus::Released);
    let p = client.get_payment(&id);
    assert_eq!(p.status, PaymentStatus::Released);
}

#[test]
fn test_update_status_returns_not_found_for_missing_payment() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    let result = client.try_update_status(&999u64, &PaymentStatus::Locked);
    assert!(result.is_err());
}

// ── donation pledges ───────────────────────────────────────────────────────────

#[test]
fn test_create_pledge_stores_metadata() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    let donor = Address::generate(&env);
    let pool = soroban_sdk::String::from_str(&env, "hospital-pool-42");
    let cause = soroban_sdk::String::from_str(&env, "maternal_health");
    let region = soroban_sdk::String::from_str(&env, "NG-Lagos");

    let id = client.create_pledge(
        &donor,
        &500i128,
        &2_592_000u64,
        &pool,
        &cause,
        &region,
        &true,
    );

    let p = client.get_pledge(&id);
    assert_eq!(p.donor, donor);
    assert_eq!(p.amount_per_period, 500);
    assert_eq!(p.interval_secs, 2_592_000);
    assert!(p.emergency_pool);
    assert!(p.active);
}

#[test]
fn test_create_pledge_rejects_zero_interval() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    let donor = Address::generate(&env);
    let pool = soroban_sdk::String::from_str(&env, "pool");
    let cause = soroban_sdk::String::from_str(&env, "c");
    let region = soroban_sdk::String::from_str(&env, "r");
    let r = client.try_create_pledge(&donor, &100i128, &0u64, &pool, &cause, &region, &false);
    assert!(r.is_err());
}

// ── Circuit breaker tests ─────────────────────────────────────────────────────

#[test]
fn test_pause_blocks_create_payment() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &None);

    client.pause(&admin);
    assert!(client.is_paused());

    let payer = Address::generate(&env);
    let payee = Address::generate(&env);
    let result = client.try_create_payment(&1u64, &payer, &payee, &500i128);
    assert!(result.is_err());
}

#[test]
fn test_pause_allows_get_payment() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &None);

    let (id, _, _) = make_payment(&env, &client, 1, 1000);
    client.pause(&admin);

    // Read still works
    let p = client.get_payment(&id);
    assert_eq!(p.id, id);
}

#[test]
fn test_unpause_restores_payments() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &None);

    client.pause(&admin);
    client.unpause(&admin);
    assert!(!client.is_paused());

    let (id, _, _) = make_payment(&env, &client, 99, 200);
    assert!(id > 0);
}

#[test]
#[should_panic]
fn test_non_admin_cannot_pause_payments() {
    let (env, cid) = setup();
    let client = PaymentContractClient::new(&env, &cid);
    let admin = Address::generate(&env);
    client.initialize(&admin, &None);

    let attacker = Address::generate(&env);
    client.pause(&attacker);
}

// ── Vesting schedule tests ─────────────────────────────────────────────────────

fn setup_with_admin() -> (Env, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(PaymentContract, ());
    let admin = Address::generate(&env);
    let client = PaymentContractClient::new(&env, &contract_id);
    client.initialize(&admin, &None);
    (env, contract_id, admin)
}

/// Pre-cliff claim must return CliffNotReached.
#[test]
fn test_vesting_pre_cliff_claim_fails() {
    let (env, cid, admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);
    let donor = Address::generate(&env);

    // cliff = now + 1000s, duration = 2000s
    env.ledger().with_mut(|l| l.timestamp = 5000);
    client.create_vesting(&admin, &donor, &1_000_000i128, &1000u64, &2000u64);

    // Deploy reward token and mint to contract so it can transfer
    let token_id = deploy_token_with_balance(&env, &admin, &cid, 1_000_000);

    // Try to claim at t=5500 (before cliff at t=6000)
    env.ledger().with_mut(|l| l.timestamp = 5500);
    let result = client.try_claim_vested(&donor, &token_id);
    assert_eq!(
        result,
        Err(Ok(Error::CliffNotReached)),
        "Expected CliffNotReached before cliff"
    );
}

/// At 50% of vesting duration, claimable = total/2.
#[test]
fn test_vesting_partial_claim_at_50_percent() {
    let (env, cid, admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);
    let donor = Address::generate(&env);

    // cliff = now + 0 (immediate), duration = 2000s → vest_end = now + 2000
    env.ledger().with_mut(|l| l.timestamp = 10_000);
    client.create_vesting(&admin, &donor, &1_000_000i128, &0u64, &2000u64);

    let token_id = deploy_token_with_balance(&env, &admin, &cid, 1_000_000);

    // Advance to 50% of vesting duration (cliff == vest_start == 10_000, vest_end == 12_000)
    env.ledger().with_mut(|l| l.timestamp = 11_000); // 1000s elapsed of 2000s
    let claimed = client.claim_vested(&donor, &token_id);
    assert_eq!(claimed, 500_000i128, "50% vesting should yield half the total");

    let schedule = client.get_vesting(&donor);
    assert_eq!(schedule.claimed, 500_000i128);
}

/// After vesting end, donor can claim the full remaining amount.
#[test]
fn test_vesting_full_claim_after_vest_end() {
    let (env, cid, admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);
    let donor = Address::generate(&env);

    env.ledger().with_mut(|l| l.timestamp = 1_000);
    client.create_vesting(&admin, &donor, &500_000i128, &0u64, &1000u64);

    let token_id = deploy_token_with_balance(&env, &admin, &cid, 500_000);

    // Advance past vest_end
    env.ledger().with_mut(|l| l.timestamp = 3_000);
    let claimed = client.claim_vested(&donor, &token_id);
    assert_eq!(claimed, 500_000i128, "Full amount claimable after vest end");

    let schedule = client.get_vesting(&donor);
    assert_eq!(schedule.claimed, 500_000i128);
    assert_eq!(schedule.claimed, schedule.total_amount);
}

/// Donor cannot claim more than total_amount across multiple claims.
#[test]
fn test_vesting_cannot_exceed_total_amount() {
    let (env, cid, admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);
    let donor = Address::generate(&env);

    env.ledger().with_mut(|l| l.timestamp = 1_000);
    client.create_vesting(&admin, &donor, &1_000_000i128, &0u64, &1000u64);

    let token_id = deploy_token_with_balance(&env, &admin, &cid, 1_000_000);

    // Claim full amount after vest end
    env.ledger().with_mut(|l| l.timestamp = 5_000);
    let first = client.claim_vested(&donor, &token_id);
    assert_eq!(first, 1_000_000i128);

    // Second claim should fail with NothingToClaim
    let result = client.try_claim_vested(&donor, &token_id);
    assert_eq!(
        result,
        Err(Ok(Error::NothingToClaim)),
        "Second claim after full vest should fail"
    );
}

/// Non-admin cannot create a vesting schedule.
#[test]
fn test_vesting_only_admin_can_create() {
    let (env, cid, _admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);
    let attacker = Address::generate(&env);
    let donor = Address::generate(&env);

    env.ledger().with_mut(|l| l.timestamp = 1_000);
    let result = client.try_create_vesting(&attacker, &donor, &1_000i128, &100u64, &500u64);
    assert!(result.is_err(), "Non-admin must not create vesting");
}

// ── process_expired_disputes (#595) ─────────────────────────────────────────────────

#[test]
fn test_process_expired_disputes_refunds_after_timeout() {
    let (env, cid, admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);

    let hospital = Address::generate(&env);
    let payee = Address::generate(&env);
    let token_id = deploy_token_with_balance(&env, &admin, &hospital, 10_000);

    env.ledger().with_mut(|l| l.timestamp = 1_000);
    let pid = client.create_escrow(&1u64, &hospital, &payee, &1_000i128, &token_id);

    // Record dispute at t=1000; updated_at becomes 1000.
    client.record_dispute(&pid, &DisputeReason::FailedDelivery,
        &soroban_sdk::String::from_str(&env, "case-1"));

    // Set a short timeout of 500s.
    client.set_dispute_timeout(&admin, &500u64);

    // Advance time past timeout.
    env.ledger().with_mut(|l| l.timestamp = 2_000);

    let mut ids = soroban_sdk::Vec::new(&env);
    ids.push_back(pid);
    let refunded = client.process_expired_disputes(&admin, &ids);
    assert_eq!(refunded.len(), 1);
    assert_eq!(refunded.get(0).unwrap(), pid);

    let p = client.get_payment(&pid);
    assert_eq!(p.status, PaymentStatus::Refunded);
}

#[test]
fn test_process_expired_disputes_skips_non_expired() {
    let (env, cid, admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);

    let hospital = Address::generate(&env);
    let payee = Address::generate(&env);
    let token_id = deploy_token_with_balance(&env, &admin, &hospital, 10_000);

    env.ledger().with_mut(|l| l.timestamp = 1_000);
    let pid = client.create_escrow(&2u64, &hospital, &payee, &500i128, &token_id);
    client.record_dispute(&pid, &DisputeReason::Other,
        &soroban_sdk::String::from_str(&env, "case-2"));

    client.set_dispute_timeout(&admin, &5_000u64);

    // Only 100s elapsed — not expired.
    env.ledger().with_mut(|l| l.timestamp = 1_100);

    let mut ids = soroban_sdk::Vec::new(&env);
    ids.push_back(pid);
    let refunded = client.process_expired_disputes(&admin, &ids);
    assert_eq!(refunded.len(), 0);

    let p = client.get_payment(&pid);
    assert_eq!(p.status, PaymentStatus::Disputed);
}

#[test]
fn test_process_expired_disputes_skips_non_disputed_payments() {
    let (env, cid, admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);

    env.ledger().with_mut(|l| l.timestamp = 1_000);
    let (pid, _, _) = make_payment(&env, &client, 3, 200);
    // Payment is Pending, not Disputed.
    client.set_dispute_timeout(&admin, &1u64);
    env.ledger().with_mut(|l| l.timestamp = 9_000);

    let mut ids = soroban_sdk::Vec::new(&env);
    ids.push_back(pid);
    let refunded = client.process_expired_disputes(&admin, &ids);
    assert_eq!(refunded.len(), 0);
}

/// VestingCreated and VestingClaimed events are emitted.
#[test]
fn test_vesting_events_emitted() {
    let (env, cid, admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);
    let donor = Address::generate(&env);

    env.ledger().with_mut(|l| l.timestamp = 1_000);
    client.create_vesting(&admin, &donor, &200_000i128, &0u64, &1000u64);

    let token_id = deploy_token_with_balance(&env, &admin, &cid, 200_000);

    env.ledger().with_mut(|l| l.timestamp = 2_500); // past vest_end
    client.claim_vested(&donor, &token_id);

    // Events are published — verify no panic and schedule is updated
    let schedule = client.get_vesting(&donor);
    assert_eq!(schedule.claimed, 200_000i128);
}

// ── Time-locked funds machinery (#45) ───────────────────────────────────────────
//
// Decision-table coverage for the dispute × pause × deadline precedence
// rules: an open dispute freezes `release_after` but never `refund_after`'s
// permissionless exit; pauses extend every deadline by exactly however long
// the contract was down; a resolved dispute re-derives (extends)
// `release_after` by the dispute's duration.

fn make_escrow(
    env: &Env,
    client: &PaymentContractClient,
    admin: &Address,
    request_id: u64,
    amount: i128,
) -> (u64, Address, Address, Address) {
    let hospital = Address::generate(env);
    let payee = Address::generate(env);
    let token_id = deploy_token_with_balance(env, admin, &hospital, amount);
    let id = client.create_escrow(&request_id, &hospital, &payee, &amount, &token_id);
    (id, hospital, payee, token_id)
}

// ── claim_expired_refund: acceptance criterion 1 ────────────────────────────

/// Any escrow past refund_after is refundable by an arbitrary third party in
/// one call.
#[test]
fn test_claim_expired_refund_by_arbitrary_third_party_after_deadline() {
    let (env, cid, admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);

    env.ledger().with_mut(|l| l.timestamp = 1_000);
    let (pid, hospital, _payee, _token) = make_escrow(&env, &client, &admin, 1, 1_000);

    env.ledger()
        .with_mut(|l| l.timestamp = 1_000 + DEFAULT_REFUND_DEADLINE_SECS);

    // `claim_expired_refund` takes no caller/auth parameter at all — its
    // signature itself is the permissionless-keeper guarantee. Any address
    // (including one unrelated to payer/payee, as here) can submit it.
    client.claim_expired_refund(&pid);

    let p = client.get_payment(&pid);
    assert_eq!(p.status, PaymentStatus::Refunded);
    assert_eq!(p.payer, hospital);
}

#[test]
fn test_claim_expired_refund_fails_before_deadline() {
    let (env, cid, admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);

    env.ledger().with_mut(|l| l.timestamp = 1_000);
    let (pid, ..) = make_escrow(&env, &client, &admin, 2, 1_000);

    let result = client.try_claim_expired_refund(&pid);
    assert_eq!(result, Err(Ok(Error::DeadlineNotReached)));
}

/// Boundary equality: `now == refund_after` counts as reached.
#[test]
fn test_claim_expired_refund_boundary_equality_succeeds() {
    let (env, cid, admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);

    env.ledger().with_mut(|l| l.timestamp = 1_000);
    let (pid, ..) = make_escrow(&env, &client, &admin, 3, 1_000);

    env.ledger()
        .with_mut(|l| l.timestamp = 1_000 + DEFAULT_REFUND_DEADLINE_SECS);
    client.claim_expired_refund(&pid);
    assert_eq!(client.get_payment(&pid).status, PaymentStatus::Refunded);
}

#[test]
fn test_claim_expired_refund_fails_on_non_escrow_payment() {
    let (env, cid, _admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);

    env.ledger().with_mut(|l| l.timestamp = 1_000);
    let (pid, _, _) = make_payment(&env, &client, 4, 100);

    env.ledger()
        .with_mut(|l| l.timestamp = 1_000 + DEFAULT_REFUND_DEADLINE_SECS * 10);
    let result = client.try_claim_expired_refund(&pid);
    assert_eq!(result, Err(Ok(Error::NotEscrowPayment)));
}

#[test]
fn test_claim_expired_refund_fails_on_terminal_payment() {
    let (env, cid, admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);

    env.ledger().with_mut(|l| l.timestamp = 1_000);
    let (pid, ..) = make_escrow(&env, &client, &admin, 5, 1_000);
    client.release_escrow(&admin, &pid);

    env.ledger()
        .with_mut(|l| l.timestamp = 1_000 + DEFAULT_REFUND_DEADLINE_SECS);
    let result = client.try_claim_expired_refund(&pid);
    assert_eq!(result, Err(Ok(Error::PaymentTerminal)));
}

// ── Dispute × deadline precedence ────────────────────────────────────────────

/// KEY RULE: an open dispute does NOT freeze `refund_after`'s permissionless
/// exit — same-ledger race between "dispute opened" and "deadline reached".
#[test]
fn test_open_dispute_does_not_block_permissionless_refund_after_deadline() {
    let (env, cid, admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);

    env.ledger().with_mut(|l| l.timestamp = 1_000);
    let (pid, ..) = make_escrow(&env, &client, &admin, 6, 1_000);
    client.record_dispute(
        &pid,
        &DisputeReason::Other,
        &soroban_sdk::String::from_str(&env, "case-open"),
    );

    // Dispute remains open (never resolved) all the way past refund_after.
    env.ledger()
        .with_mut(|l| l.timestamp = 1_000 + DEFAULT_REFUND_DEADLINE_SECS);
    client.claim_expired_refund(&pid);

    let p = client.get_payment(&pid);
    assert_eq!(p.status, PaymentStatus::Refunded);
}

/// An open dispute doesn't change the deadline check itself — still fails
/// before the hard deadline regardless of dispute state.
#[test]
fn test_open_dispute_before_deadline_still_fails() {
    let (env, cid, admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);

    env.ledger().with_mut(|l| l.timestamp = 1_000);
    let (pid, ..) = make_escrow(&env, &client, &admin, 7, 1_000);
    client.record_dispute(
        &pid,
        &DisputeReason::Other,
        &soroban_sdk::String::from_str(&env, "case-early"),
    );

    let result = client.try_claim_expired_refund(&pid);
    assert_eq!(result, Err(Ok(Error::DeadlineNotReached)));
}

/// Resolving a dispute re-derives `release_after`, extending it by exactly
/// the dispute's duration.
#[test]
fn test_resolve_dispute_extends_release_after_by_dispute_duration() {
    let (env, cid, admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);

    env.ledger().with_mut(|l| l.timestamp = 1_000);
    let (pid, ..) = make_escrow(&env, &client, &admin, 8, 1_000);
    let (release_before, _, _) = client.get_effective_deadlines(&pid);

    client.record_dispute(
        &pid,
        &DisputeReason::Other,
        &soroban_sdk::String::from_str(&env, "case-dur"),
    );

    let dispute_duration = 5_000u64;
    env.ledger()
        .with_mut(|l| l.timestamp = 1_000 + dispute_duration);
    client.resolve_dispute(&pid);

    let (release_after, _, _) = client.get_effective_deadlines(&pid);
    assert_eq!(release_after, release_before + dispute_duration);
}

/// `record_dispute` is rejected once `dispute_by` has passed.
#[test]
fn test_record_dispute_rejected_after_dispute_window_closes() {
    let (env, cid, admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);

    env.ledger().with_mut(|l| l.timestamp = 1_000);
    let (pid, ..) = make_escrow(&env, &client, &admin, 9, 1_000);

    env.ledger()
        .with_mut(|l| l.timestamp = 1_000 + DEFAULT_DISPUTE_WINDOW_SECS + 1);
    let result = client.try_record_dispute(
        &pid,
        &DisputeReason::Other,
        &soroban_sdk::String::from_str(&env, "too-late"),
    );
    assert_eq!(result, Err(Ok(Error::DisputeWindowClosed)));
}

// ── Pause × deadline precedence ──────────────────────────────────────────────

/// KEY RULE: a pause extends every escrow's deadline by exactly the pause's
/// duration — reaching the *original* refund_after isn't enough once the
/// contract has been paused in between.
#[test]
fn test_pause_extends_refund_after_by_pause_duration() {
    let (env, cid, admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);

    env.ledger().with_mut(|l| l.timestamp = 1_000);
    let (pid, ..) = make_escrow(&env, &client, &admin, 10, 1_000);

    let pause_duration = 10_000u64;
    client.pause(&admin);
    env.ledger()
        .with_mut(|l| l.timestamp = 1_000 + pause_duration);
    client.unpause(&admin);

    // Original refund_after has been reached, but the pause should have
    // pushed the effective deadline out by `pause_duration`.
    env.ledger()
        .with_mut(|l| l.timestamp = 1_000 + DEFAULT_REFUND_DEADLINE_SECS);
    let result = client.try_claim_expired_refund(&pid);
    assert_eq!(
        result,
        Err(Ok(Error::DeadlineNotReached)),
        "pause must extend refund_after, not just release_after"
    );

    // Once the extension has also elapsed, the refund succeeds.
    env.ledger()
        .with_mut(|l| l.timestamp = 1_000 + DEFAULT_REFUND_DEADLINE_SECS + pause_duration);
    client.claim_expired_refund(&pid);
    assert_eq!(client.get_payment(&pid).status, PaymentStatus::Refunded);
}

/// A pause that happened *before* a payment was created must not extend
/// that payment's deadlines (only pauses during its lifetime count).
#[test]
fn test_pause_before_creation_does_not_extend_deadline() {
    let (env, cid, admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);

    env.ledger().with_mut(|l| l.timestamp = 1_000);
    client.pause(&admin);
    env.ledger().with_mut(|l| l.timestamp = 1_500);
    client.unpause(&admin); // 500s pause, before the payment exists

    env.ledger().with_mut(|l| l.timestamp = 2_000);
    let (pid, ..) = make_escrow(&env, &client, &admin, 11, 1_000);

    let (_, refund_after, _) = client.get_effective_deadlines(&pid);
    assert_eq!(refund_after, 2_000 + DEFAULT_REFUND_DEADLINE_SECS);
}

/// Far-future ledger timestamps must saturate rather than overflow/panic.
#[test]
fn test_deadlines_saturate_on_far_future_timestamp() {
    let (env, cid, admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);

    env.ledger().with_mut(|l| l.timestamp = u64::MAX - 100);
    let (pid, ..) = make_escrow(&env, &client, &admin, 12, 1_000);

    let (release_after, refund_after, dispute_by) = client.get_effective_deadlines(&pid);
    assert_eq!(refund_after, u64::MAX);
    assert_eq!(release_after, u64::MAX);
    assert_eq!(dispute_by, u64::MAX);
}

// ── Campaign tranche schedules (#45) ────────────────────────────────────────

fn make_tranches(env: &Env, pairs: &[(u64, i128)]) -> Vec<(u64, i128)> {
    let mut v: Vec<(u64, i128)> = Vec::new(env);
    for pair in pairs {
        v.push_back(*pair);
    }
    v
}

/// A campaign schedule never releases before a tranche's unlock time.
#[test]
fn test_schedule_releases_nothing_before_first_unlock() {
    let (env, cid, admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);
    let funder = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let revoker = Address::generate(&env);
    let token_id = deploy_token_with_balance(&env, &admin, &funder, 300);

    env.ledger().with_mut(|l| l.timestamp = 1_000);
    let tranches = make_tranches(&env, &[(2_000, 100), (3_000, 200)]);
    let sid = client.create_schedule(&funder, &beneficiary, &token_id, &tranches, &revoker);

    let result = client.try_claim_tranches(&sid);
    assert_eq!(result, Err(Ok(Error::NothingToClaim)));
}

/// A schedule releases exactly its configured amounts, never early, and its
/// released total is reconstructable from the `schedule`/`tranche` events
/// (each claim emits one event per newly-unlocked tranche).
#[test]
fn test_schedule_releases_exact_amounts_in_order() {
    let (env, cid, admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);
    let funder = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let revoker = Address::generate(&env);
    let token_id = deploy_token_with_balance(&env, &admin, &funder, 300);

    env.ledger().with_mut(|l| l.timestamp = 1_000);
    let tranches = make_tranches(&env, &[(2_000, 100), (3_000, 200)]);
    let sid = client.create_schedule(&funder, &beneficiary, &token_id, &tranches, &revoker);

    // Only the first tranche has unlocked.
    env.ledger().with_mut(|l| l.timestamp = 2_000);
    let released = client.claim_tranches(&sid);
    assert_eq!(released, 100);
    assert_eq!(client.get_schedule(&sid).released_so_far, 100);

    // Re-cranking before the next unlock yields nothing (never early, never
    // double-paid).
    let result = client.try_claim_tranches(&sid);
    assert_eq!(result, Err(Ok(Error::NothingToClaim)));

    env.ledger().with_mut(|l| l.timestamp = 3_000);
    let released2 = client.claim_tranches(&sid);
    assert_eq!(released2, 200);
    assert_eq!(client.get_schedule(&sid).released_so_far, 300);

    let token_client = token::Client::new(&env, &token_id);
    assert_eq!(token_client.balance(&beneficiary), 300);
}

#[test]
fn test_schedule_rejects_out_of_order_tranches() {
    let (env, cid, admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);
    let funder = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let revoker = Address::generate(&env);
    let token_id = deploy_token_with_balance(&env, &admin, &funder, 300);

    let tranches = make_tranches(&env, &[(3_000, 100), (2_000, 200)]);
    let result =
        client.try_create_schedule(&funder, &beneficiary, &token_id, &tranches, &revoker);
    assert_eq!(result, Err(Ok(Error::InvalidTranches)));
}

#[test]
fn test_schedule_rejects_too_many_tranches() {
    let (env, cid, admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);
    let funder = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let revoker = Address::generate(&env);
    let token_id = deploy_token_with_balance(&env, &admin, &funder, 1_000);

    let mut tranches: Vec<(u64, i128)> = Vec::new(&env);
    for i in 0..(MAX_TRANCHES + 1) {
        tranches.push_back((1_000u64 + i as u64, 1i128));
    }
    let result =
        client.try_create_schedule(&funder, &beneficiary, &token_id, &tranches, &revoker);
    assert_eq!(result, Err(Ok(Error::TooManyTranches)));
}

#[test]
fn test_schedule_revoke_returns_unreleased_and_blocks_further_claims() {
    let (env, cid, admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);
    let funder = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let revoker = Address::generate(&env);
    let token_id = deploy_token_with_balance(&env, &admin, &funder, 300);

    env.ledger().with_mut(|l| l.timestamp = 1_000);
    let tranches = make_tranches(&env, &[(2_000, 100), (3_000, 200)]);
    let sid = client.create_schedule(&funder, &beneficiary, &token_id, &tranches, &revoker);

    env.ledger().with_mut(|l| l.timestamp = 2_000);
    client.claim_tranches(&sid);

    let returned = client.revoke_schedule(&revoker, &sid);
    assert_eq!(returned, 200);

    let token_client = token::Client::new(&env, &token_id);
    assert_eq!(token_client.balance(&revoker), 200);

    env.ledger().with_mut(|l| l.timestamp = 3_000);
    let result = client.try_claim_tranches(&sid);
    assert_eq!(result, Err(Ok(Error::ScheduleRevoked)));
}

#[test]
fn test_schedule_revoke_only_by_configured_revoker() {
    let (env, cid, admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);
    let funder = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let revoker = Address::generate(&env);
    let attacker = Address::generate(&env);
    let token_id = deploy_token_with_balance(&env, &admin, &funder, 100);

    let tranches = make_tranches(&env, &[(2_000, 100)]);
    let sid = client.create_schedule(&funder, &beneficiary, &token_id, &tranches, &revoker);

    let result = client.try_revoke_schedule(&attacker, &sid);
    assert_eq!(result, Err(Ok(Error::NotRevoker)));
}

// ── Timelocked upgradeability & versioned schema (#31) ─────────────────────────

#[test]
fn test_version_and_default_schema_version() {
    let (env, cid, _admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);
    assert_eq!(client.version(), CONTRACT_VERSION);
    assert_eq!(client.schema_version(), 1);
}

#[test]
fn test_propose_upgrade_queues_behind_timelock() {
    let (env, cid, _admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);
    env.ledger().with_mut(|l| l.timestamp = 10_000);

    let hash = soroban_sdk::BytesN::from_array(&env, &[7u8; 32]);
    let executable_at = client.propose_upgrade(&hash);
    assert_eq!(executable_at, 10_000 + UPGRADE_TIMELOCK_SECS);

    let pending = client.get_pending_upgrade().unwrap();
    assert_eq!(pending.new_wasm_hash, hash);
    assert_eq!(pending.proposed_at, 10_000);
    assert_eq!(pending.executable_at, executable_at);
}

#[test]
fn test_execute_upgrade_before_timelock_elapses_fails() {
    let (env, cid, _admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);
    env.ledger().with_mut(|l| l.timestamp = 10_000);

    let hash = soroban_sdk::BytesN::from_array(&env, &[7u8; 32]);
    let executable_at = client.propose_upgrade(&hash);

    env.ledger().with_mut(|l| l.timestamp = executable_at - 1);
    assert_eq!(
        client.try_execute_upgrade(),
        Err(Ok(Error::TimelockNotElapsed))
    );
    // Still queued — nothing was consumed by the failed attempt.
    assert!(client.get_pending_upgrade().is_some());
}

#[test]
fn test_propose_upgrade_twice_fails() {
    let (env, cid, _admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);

    let hash = soroban_sdk::BytesN::from_array(&env, &[7u8; 32]);
    client.propose_upgrade(&hash);
    assert_eq!(
        client.try_propose_upgrade(&hash),
        Err(Ok(Error::UpgradeAlreadyPending))
    );
}

#[test]
fn test_cancel_upgrade_clears_pending_proposal() {
    let (env, cid, _admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);

    let hash = soroban_sdk::BytesN::from_array(&env, &[7u8; 32]);
    client.propose_upgrade(&hash);
    client.cancel_upgrade();

    assert!(client.get_pending_upgrade().is_none());
    assert_eq!(
        client.try_execute_upgrade(),
        Err(Ok(Error::NoPendingUpgrade))
    );
}

#[test]
fn test_execute_upgrade_without_proposal_fails() {
    let (env, cid, _admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);
    assert_eq!(
        client.try_execute_upgrade(),
        Err(Ok(Error::NoPendingUpgrade))
    );
}

#[test]
fn test_migrate_refuses_to_run_twice() {
    let (env, cid, _admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);

    // Simulate storage written by an older binary (schema 0 < target).
    env.as_contract(&cid, || {
        env.storage()
            .instance()
            .set(&crate::SCHEMA_VERSION_KEY, &0u32)
    });
    assert_eq!(client.schema_version(), 0);

    assert_eq!(client.migrate(), TARGET_SCHEMA_VERSION);
    assert_eq!(client.schema_version(), TARGET_SCHEMA_VERSION);

    // Double-run guard: a second invocation is refused.
    assert_eq!(
        client.try_migrate(),
        Err(Ok(Error::MigrationAlreadyApplied))
    );
}

#[test]
fn test_migrate_refused_at_current_schema() {
    let (env, cid, _admin) = setup_with_admin();
    let client = PaymentContractClient::new(&env, &cid);
    assert_eq!(
        client.try_migrate(),
        Err(Ok(Error::MigrationAlreadyApplied))
    );
}

/// Full upgrade rehearsal against the real compiled WASM: populate an open
/// escrow, propose → timelock → execute the upgrade, then prove the
/// in-flight escrow still settles correctly on the new binary.
///
/// Requires the workspace WASMs to be built first — run via
/// `scripts/test-upgrade-rehearsal.sh`.
#[cfg(feature = "upgrade-rehearsal")]
mod upgrade_rehearsal {
    use super::*;

    const PAYMENT_WASM: &[u8] = include_bytes!(
        "../../../target/wasm32v1-none/release/payment_contract.wasm"
    );

    #[test]
    fn test_upgrade_rehearsal_inflight_escrow_settles_after_upgrade() {
        let (env, cid, admin) = setup_with_admin();
        let client = PaymentContractClient::new(&env, &cid);
        env.ledger().with_mut(|l| l.timestamp = 1_000_000);

        // Realistic in-flight state: an open escrow holding donor funds.
        let hospital = Address::generate(&env);
        let payee = Address::generate(&env);
        let token = deploy_token_with_balance(&env, &admin, &hospital, 5_000);
        let payment_id = client.create_escrow(&1u64, &hospital, &payee, &5_000i128, &token);

        // Propose → wait out the 48h timelock → execute the WASM swap.
        let wasm_hash = env.deployer().upload_contract_wasm(PAYMENT_WASM);
        let executable_at = client.propose_upgrade(&wasm_hash);
        env.ledger().with_mut(|l| l.timestamp = executable_at + 1);
        client.execute_upgrade();

        // Same contract ID, storage intact, new binary answering.
        assert_eq!(client.version(), CONTRACT_VERSION);
        let payment = client.get_payment(&payment_id);
        assert_eq!(payment.status, PaymentStatus::Locked);
        assert_eq!(payment.amount, 5_000);

        // Schema unchanged between identical binaries — migrate must refuse.
        assert_eq!(
            client.try_migrate(),
            Err(Ok(Error::MigrationAlreadyApplied))
        );

        // The in-flight escrow completes correctly post-upgrade.
        client.release_escrow(&admin, &payment_id);
        let token_client = soroban_sdk::token::Client::new(&env, &token);
        assert_eq!(token_client.balance(&payee), 5_000i128);
        assert_eq!(client.get_payment(&payment_id).status, PaymentStatus::Released);
    }
}
