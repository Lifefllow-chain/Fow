#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Events},
    Address, Env, IntoVal, Symbol, TryIntoVal, Val,
};

use super::{
    AnalyticsContract, AnalyticsContractClient, AnalyticsError, InitializedEvent, MetricKind,
    MetricRecordedEvent, PaymentReleasedRecordedEvent, PeriodType, ReportingPeriodChangedEvent,
};

/// Assert the most recently published event matches the standard envelope
/// `(domain, event, schema_version)` plus the given payload.
fn assert_last_event(
    env: &Env,
    contract_id: &Address,
    domain: &str,
    event: &str,
    version: u32,
    data: impl IntoVal<Env, Val>,
) {
    let all = env.events().all();
    assert!(!all.is_empty(), "no events were published");
    let actual = all.get(all.len() - 1).unwrap();
    let topics: soroban_sdk::Vec<Val> =
        (Symbol::new(env, domain), Symbol::new(env, event), version).into_val(env);
    assert_eq!(actual, (contract_id.clone(), topics, data.into_val(env)));
}

fn setup<'a>() -> (Env, Address, AnalyticsContractClient<'a>) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let inventory = Address::generate(&env);
    let requests = Address::generate(&env);
    let payments = Address::generate(&env);
    let reputation = Address::generate(&env);

    let id = env.register(AnalyticsContract, ());
    let client = AnalyticsContractClient::new(&env, &id);

    client.initialize(&admin, &inventory, &requests, &payments, &reputation);

    (env, admin, client)
}

// ── Initialization ────────────────────────────────────────────────────────────

#[test]
fn test_initialize_succeeds() {
    let (_, _, client) = setup();
    assert!(client.is_initialized());
}

#[test]
fn test_initialize_sets_config() {
    let (env, admin, client) = setup();
    let cfg = client.get_config();
    assert_eq!(cfg.admin, admin);
    assert_eq!(cfg.reporting_period.duration_secs, 86_400);
    assert_eq!(cfg.initialized_at, env.ledger().timestamp());
}

#[test]
fn test_double_initialize_fails() {
    let (_, _, client) = setup();
    let admin2 = Address::generate(&client.env);
    let dummy = Address::generate(&client.env);
    let result = client.try_initialize(&admin2, &dummy, &dummy, &dummy, &dummy);
    assert_eq!(result, Err(Ok(AnalyticsError::AlreadyInitialized)));
}

#[test]
fn test_is_initialized_false_before_init() {
    let env = Env::default();
    let id = env.register(AnalyticsContract, ());
    let client = AnalyticsContractClient::new(&env, &id);
    assert!(!client.is_initialized());
}

// ── Lifetime counters start at zero ──────────────────────────────────────────

#[test]
fn test_lifetime_totals_zero_after_init() {
    let (_, _, client) = setup();
    let totals = client.get_lifetime_totals();
    assert_eq!(totals.total_donations, 0);
    assert_eq!(totals.total_requests, 0);
    assert_eq!(totals.total_deliveries, 0);
    assert_eq!(totals.total_payments_released, 0);
    assert_eq!(totals.total_volume, 0);
}

// ── Metric recording ──────────────────────────────────────────────────────────

#[test]
fn test_record_donation_increments_counters() {
    let (_, _, client) = setup();
    client.record_donation();
    client.record_donation();

    let snap = client.get_current_snapshot();
    assert_eq!(snap.total_donations, 2);

    let totals = client.get_lifetime_totals();
    assert_eq!(totals.total_donations, 2);
}

#[test]
fn test_record_request_increments_counters() {
    let (_, _, client) = setup();
    client.record_request();

    let snap = client.get_current_snapshot();
    assert_eq!(snap.total_requests, 1);
}

#[test]
fn test_record_delivery_increments_counters() {
    let (_, _, client) = setup();
    client.record_delivery();

    let snap = client.get_current_snapshot();
    assert_eq!(snap.total_deliveries, 1);
}

#[test]
fn test_record_payment_released_increments_counters() {
    let (_, _, client) = setup();
    client.record_payment_released(&500_i128);
    client.record_payment_released(&300_i128);

    let snap = client.get_current_snapshot();
    assert_eq!(snap.total_payments_released, 2);
    assert_eq!(snap.total_volume, 800);

    let totals = client.get_lifetime_totals();
    assert_eq!(totals.total_volume, 800);
}

// ── Reporting period ──────────────────────────────────────────────────────────

#[test]
fn test_set_reporting_period_weekly() {
    let (_, _, client) = setup();
    client.set_reporting_period(&PeriodType::Weekly);
    let cfg = client.get_config();
    assert_eq!(cfg.reporting_period.duration_secs, 604_800);
}

#[test]
fn test_set_reporting_period_monthly() {
    let (_, _, client) = setup();
    client.set_reporting_period(&PeriodType::Monthly);
    let cfg = client.get_config();
    assert_eq!(cfg.reporting_period.duration_secs, 2_592_000);
}

// ── Period snapshot isolation ─────────────────────────────────────────────────

#[test]
fn test_get_snapshot_not_found_returns_error() {
    let (_, _, client) = setup();
    let result = client.try_get_snapshot(&9999u64);
    assert_eq!(result, Err(Ok(AnalyticsError::PeriodNotFound)));
}

#[test]
fn test_get_snapshot_returns_correct_period() {
    let (env, _, client) = setup();
    client.record_donation();

    let period_index = env.ledger().timestamp() / 86_400;
    let snap = client.get_snapshot(&period_index);
    assert_eq!(snap.total_donations, 1);
    assert_eq!(snap.period_index, period_index);
}

// ── Guard: not initialized ────────────────────────────────────────────────────

#[test]
fn test_record_donation_fails_when_not_initialized() {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(AnalyticsContract, ());
    let client = AnalyticsContractClient::new(&env, &id);
    let result = client.try_record_donation();
    assert_eq!(result, Err(Ok(AnalyticsError::NotInitialized)));
}

// ── Event coverage (#53) ──────────────────────────────────────────────────────
//
// One test per state-mutating entrypoint asserting the exact cataloged event
// (see EVENTS.md), plus failure-branch tests proving rejected calls emit
// nothing, plus a shared envelope-shape test.

#[test]
fn test_event_envelope_third_topic_is_u32_schema_version() {
    let (env, _, client) = setup();
    let all = env.events().all();
    let (_, topics, _) = all.get(all.len() - 1).unwrap();
    assert_eq!(topics.len(), 3, "envelope must be (domain, event, schema_version)");
    let version: u32 = topics.get(2).unwrap().try_into_val(&env).unwrap();
    assert_eq!(version, 1);
}

#[test]
fn test_initialize_emits_event() {
    let (env, admin, client) = setup();
    assert_last_event(
        &env,
        &client.address,
        "anlytcs",
        "init",
        1,
        InitializedEvent {
            admin,
            initialized_at: env.ledger().timestamp(),
        },
    );
}

#[test]
fn test_double_initialize_emits_no_event() {
    let (env, _, client) = setup();
    let before = env.events().all().len();
    let dummy = Address::generate(&env);
    let _ = client.try_initialize(&dummy, &dummy, &dummy, &dummy, &dummy);
    assert_eq!(env.events().all().len(), before, "rejected call must not emit");
}

#[test]
fn test_set_reporting_period_emits_event() {
    let (env, _, client) = setup();
    client.set_reporting_period(&PeriodType::Weekly);
    assert_last_event(
        &env,
        &client.address,
        "anlytcs",
        "per_chg",
        1,
        ReportingPeriodChangedEvent {
            old_period_type: PeriodType::Daily,
            new_period_type: PeriodType::Weekly,
            new_duration_secs: 604_800,
            changed_at: env.ledger().timestamp(),
        },
    );
}

#[test]
fn test_record_donation_emits_event() {
    let (env, _, client) = setup();
    client.record_donation();
    assert_last_event(
        &env,
        &client.address,
        "anlytcs",
        "metric",
        1,
        MetricRecordedEvent {
            period_index: env.ledger().timestamp() / 86_400,
            metric: MetricKind::Donation,
            lifetime_total: 1,
            recorded_at: env.ledger().timestamp(),
        },
    );
}

#[test]
fn test_record_request_emits_event() {
    let (env, _, client) = setup();
    client.record_request();
    assert_last_event(
        &env,
        &client.address,
        "anlytcs",
        "metric",
        1,
        MetricRecordedEvent {
            period_index: env.ledger().timestamp() / 86_400,
            metric: MetricKind::Request,
            lifetime_total: 1,
            recorded_at: env.ledger().timestamp(),
        },
    );
}

#[test]
fn test_record_delivery_emits_event() {
    let (env, _, client) = setup();
    client.record_delivery();
    assert_last_event(
        &env,
        &client.address,
        "anlytcs",
        "metric",
        1,
        MetricRecordedEvent {
            period_index: env.ledger().timestamp() / 86_400,
            metric: MetricKind::Delivery,
            lifetime_total: 1,
            recorded_at: env.ledger().timestamp(),
        },
    );
}

#[test]
fn test_record_donation_fails_when_not_initialized_emits_no_event() {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(AnalyticsContract, ());
    let client = AnalyticsContractClient::new(&env, &id);
    let _ = client.try_record_donation();
    assert!(env.events().all().is_empty());
}

#[test]
fn test_record_payment_released_emits_event() {
    let (env, _, client) = setup();
    client.record_payment_released(&500i128);
    assert_last_event(
        &env,
        &client.address,
        "anlytcs",
        "pay_rel",
        1,
        PaymentReleasedRecordedEvent {
            period_index: env.ledger().timestamp() / 86_400,
            amount: 500,
            lifetime_count: 1,
            lifetime_volume: 500,
            recorded_at: env.ledger().timestamp(),
        },
    );
}

#[test]
fn test_upgrade_fails_when_not_initialized_emits_no_event() {
    let env = Env::default();
    let id = env.register(AnalyticsContract, ());
    let client = AnalyticsContractClient::new(&env, &id);
    let hash = soroban_sdk::BytesN::from_array(&env, &[9u8; 32]);
    let result = client.try_upgrade(&hash);
    assert_eq!(result, Err(Ok(AnalyticsError::NotInitialized)));
    assert!(env.events().all().is_empty());
}

#[test]
fn test_migrate_refused_at_current_schema_emits_no_event() {
    let (env, _, client) = setup();
    let before = env.events().all().len();
    let result = client.try_migrate();
    assert_eq!(result, Err(Ok(AnalyticsError::MigrationAlreadyApplied)));
    assert_eq!(env.events().all().len(), before, "refused migration must not emit");
}
