/// Tests for the blood matching algorithm.
///
/// These tests exercise the pure matching logic directly (no cross-contract
/// calls) so they run fast and deterministically without a full Soroban
/// environment. Contract-level integration tests follow at the bottom.
#[cfg(test)]
mod pure_matching {
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::Env;

    use crate::matching::{
        compatible_donor_types, is_compatible, score_unit, select_units, sort_by_expiration,
    };
    use crate::types::{BloodStatus, BloodType, BloodUnit, MatchKind, Urgency};

    // ── Helpers ──────────────────────────────────────────────────────────────

    fn env() -> Env {
        Env::default()
    }

    fn make_unit(
        env: &Env,
        id: u64,
        blood_type: BloodType,
        quantity_ml: u32,
        expiration_timestamp: u64,
    ) -> BloodUnit {
        BloodUnit {
            id,
            blood_type,
            quantity_ml,
            bank_id: soroban_sdk::Address::generate(env),
            donor_id: None,
            donation_timestamp: 0,
            expiration_timestamp,
            status: BloodStatus::Available,
            metadata: soroban_sdk::Map::new(env),
        }
    }

    fn make_unit_with_status(
        env: &Env,
        id: u64,
        blood_type: BloodType,
        quantity_ml: u32,
        expiration_timestamp: u64,
        status: BloodStatus,
    ) -> BloodUnit {
        BloodUnit {
            id,
            blood_type,
            quantity_ml,
            bank_id: soroban_sdk::Address::generate(env),
            donor_id: None,
            donation_timestamp: 0,
            expiration_timestamp,
            status,
            metadata: soroban_sdk::Map::new(env),
        }
    }

    // ── ABO / Rh compatibility matrix ────────────────────────────────────────

    #[test]
    fn o_negative_is_universal_donor() {
        use BloodType::*;
        let all = [
            APositive, ANegative, BPositive, BNegative,
            ABPositive, ABNegative, OPositive, ONegative,
        ];
        for recipient in all {
            assert!(
                is_compatible(ONegative, recipient),
                "O- should donate to {:?}",
                recipient
            );
        }
    }

    #[test]
    fn ab_positive_is_universal_recipient() {
        use BloodType::*;
        let all = [
            APositive, ANegative, BPositive, BNegative,
            ABPositive, ABNegative, OPositive, ONegative,
        ];
        for donor in all {
            assert!(
                is_compatible(donor, ABPositive),
                "{:?} should donate to AB+",
                donor
            );
        }
    }

    #[test]
    fn rh_positive_cannot_donate_to_rh_negative() {
        use BloodType::*;
        // Rh+ donors cannot give to Rh- recipients (except O- universal donor)
        assert!(!is_compatible(OPositive, ONegative));
        assert!(!is_compatible(APositive, ANegative));
        assert!(!is_compatible(BPositive, BNegative));
        assert!(!is_compatible(ABPositive, ABNegative));
    }

    #[test]
    fn rh_negative_can_donate_to_same_rh_positive() {
        use BloodType::*;
        assert!(is_compatible(ANegative, APositive));
        assert!(is_compatible(BNegative, BPositive));
        assert!(is_compatible(ABNegative, ABPositive));
    }

    #[test]
    fn incompatible_abo_groups_rejected() {
        use BloodType::*;
        assert!(!is_compatible(APositive, BPositive));
        assert!(!is_compatible(BPositive, APositive));
        assert!(!is_compatible(APositive, OPositive));
        assert!(!is_compatible(BPositive, OPositive));
    }

    #[test]
    fn compatible_donor_types_o_negative_only_receives_o_negative() {
        let env = env();
        let types = compatible_donor_types(&env, BloodType::ONegative);
        assert_eq!(types.len(), 1);
        assert_eq!(types.get(0).unwrap(), BloodType::ONegative);
    }

    #[test]
    fn compatible_donor_types_ab_positive_receives_all_eight() {
        let env = env();
        let types = compatible_donor_types(&env, BloodType::ABPositive);
        assert_eq!(types.len(), 8);
    }

    #[test]
    fn compatible_donor_types_a_positive_receives_four() {
        let env = env();
        let types = compatible_donor_types(&env, BloodType::APositive);
        // A+, A-, O+, O-
        assert_eq!(types.len(), 4);
        assert_eq!(types.get(0).unwrap(), BloodType::APositive); // exact first
    }

    #[test]
    fn compatible_donor_types_first_element_is_always_exact_match() {
        let env = env();
        use BloodType::*;
        let all = [
            APositive, ANegative, BPositive, BNegative,
            ABPositive, ABNegative, OPositive, ONegative,
        ];
        for bt in all {
            let types = compatible_donor_types(&env, bt);
            assert_eq!(
                types.get(0).unwrap(),
                bt,
                "First compatible type for {:?} should be itself",
                bt
            );
        }
    }

    // ── FIFO sort ────────────────────────────────────────────────────────────

    #[test]
    fn sort_by_expiration_orders_oldest_first() {
        let env = env();
        let mut units = soroban_sdk::Vec::new(&env);
        units.push_back(make_unit(&env, 1, BloodType::APositive, 450, 3000));
        units.push_back(make_unit(&env, 2, BloodType::APositive, 450, 1000));
        units.push_back(make_unit(&env, 3, BloodType::APositive, 450, 2000));

        sort_by_expiration(&mut units);

        assert_eq!(units.get(0).unwrap().id, 2); // expires at 1000
        assert_eq!(units.get(1).unwrap().id, 3); // expires at 2000
        assert_eq!(units.get(2).unwrap().id, 1); // expires at 3000
    }

    #[test]
    fn sort_by_expiration_stable_on_equal_timestamps() {
        let env = env();
        let mut units = soroban_sdk::Vec::new(&env);
        units.push_back(make_unit(&env, 1, BloodType::OPositive, 450, 1000));
        units.push_back(make_unit(&env, 2, BloodType::OPositive, 450, 1000));

        sort_by_expiration(&mut units);

        // Both have same expiry — order preserved (insertion sort is stable)
        assert_eq!(units.get(0).unwrap().id, 1);
        assert_eq!(units.get(1).unwrap().id, 2);
    }

    #[test]
    fn sort_by_expiration_single_element_unchanged() {
        let env = env();
        let mut units = soroban_sdk::Vec::new(&env);
        units.push_back(make_unit(&env, 42, BloodType::BNegative, 300, 9999));
        sort_by_expiration(&mut units);
        assert_eq!(units.get(0).unwrap().id, 42);
    }

    // ── Scoring ──────────────────────────────────────────────────────────────

    #[test]
    fn exact_match_scores_higher_than_compatible() {
        let env = env();
        let exact_unit = make_unit(&env, 1, BloodType::APositive, 450, 86_400 * 5); // 5 days
        let compat_unit = make_unit(&env, 2, BloodType::ONegative, 450, 86_400 * 5);

        let exact_score = score_unit(&exact_unit, BloodType::APositive, Urgency::Routine, None, 0);
        let compat_score = score_unit(&compat_unit, BloodType::APositive, Urgency::Routine, None, 0);

        assert!(
            exact_score > compat_score,
            "exact={} should beat compatible={}",
            exact_score,
            compat_score
        );
    }

    #[test]
    fn expiring_soon_scores_higher_than_fresh() {
        let env = env();
        let expiring = make_unit(&env, 1, BloodType::OPositive, 450, 86_400 * 2); // 2 days
        let fresh    = make_unit(&env, 2, BloodType::OPositive, 450, 86_400 * 60); // 60 days

        let s_expiring = score_unit(&expiring, BloodType::OPositive, Urgency::Routine, None, 0);
        let s_fresh    = score_unit(&fresh,    BloodType::OPositive, Urgency::Routine, None, 0);

        assert!(s_expiring > s_fresh);
    }

    #[test]
    fn critical_urgency_scores_higher_than_scheduled() {
        let env = env();
        let unit = make_unit(&env, 1, BloodType::BPositive, 450, 86_400 * 10);

        let s_critical  = score_unit(&unit, BloodType::BPositive, Urgency::Critical,  None, 0);
        let s_scheduled = score_unit(&unit, BloodType::BPositive, Urgency::Scheduled, None, 0);

        assert!(s_critical > s_scheduled);
    }

    // ── select_units ─────────────────────────────────────────────────────────

    #[test]
    fn exact_match_preferred_over_compatible() {
        let env = env();
        let mut candidates = soroban_sdk::Vec::new(&env);
        // Compatible unit expires sooner (would win on FIFO alone)
        candidates.push_back(make_unit(&env, 1, BloodType::ONegative, 450, 1000));
        // Exact match expires later
        candidates.push_back(make_unit(&env, 2, BloodType::APositive, 450, 9000));

        let result = select_units(
            &env,
            candidates,
            BloodType::APositive,
            Urgency::Routine,
            450,
            None,
            0,
        );

        assert_eq!(result.len(), 1);
        assert_eq!(result.get(0).unwrap().unit_id, 2); // exact match wins
        assert_eq!(result.get(0).unwrap().match_kind, MatchKind::Exact);
    }

    #[test]
    fn fifo_within_exact_tier() {
        let env = env();
        let mut candidates = soroban_sdk::Vec::new(&env);
        candidates.push_back(make_unit(&env, 1, BloodType::APositive, 450, 5000)); // newer
        candidates.push_back(make_unit(&env, 2, BloodType::APositive, 450, 1000)); // older

        let result = select_units(
            &env,
            candidates,
            BloodType::APositive,
            Urgency::Routine,
            450,
            None,
            0,
        );

        assert_eq!(result.len(), 1);
        assert_eq!(result.get(0).unwrap().unit_id, 2); // oldest expiry first
    }

    #[test]
    fn fifo_within_compatible_tier() {
        let env = env();
        let mut candidates = soroban_sdk::Vec::new(&env);
        candidates.push_back(make_unit(&env, 1, BloodType::ONegative, 450, 8000));
        candidates.push_back(make_unit(&env, 2, BloodType::ONegative, 450, 2000));

        let result = select_units(
            &env,
            candidates,
            BloodType::APositive, // O- is compatible with A+
            Urgency::Routine,
            450,
            None,
            0,
        );

        assert_eq!(result.len(), 1);
        assert_eq!(result.get(0).unwrap().unit_id, 2); // oldest first
    }

    #[test]
    fn partial_matching_returns_available_volume() {
        let env = env();
        let mut candidates = soroban_sdk::Vec::new(&env);
        candidates.push_back(make_unit(&env, 1, BloodType::BPositive, 300, 1000));

        // Request 600 ml but only 300 available
        let result = select_units(
            &env,
            candidates,
            BloodType::BPositive,
            Urgency::Urgent,
            600,
            None,
            0,
        );

        assert_eq!(result.len(), 1);
        assert_eq!(result.get(0).unwrap().quantity_ml, 300);
    }

    #[test]
    fn partial_matching_across_multiple_units() {
        let env = env();
        let mut candidates = soroban_sdk::Vec::new(&env);
        candidates.push_back(make_unit(&env, 1, BloodType::OPositive, 200, 1000));
        candidates.push_back(make_unit(&env, 2, BloodType::OPositive, 200, 2000));
        candidates.push_back(make_unit(&env, 3, BloodType::OPositive, 200, 3000));

        // Request 500 ml — needs 3 units but last one only partially used
        let result = select_units(
            &env,
            candidates,
            BloodType::OPositive,
            Urgency::Critical,
            500,
            None,
            0,
        );

        assert_eq!(result.len(), 3);
        let total: u32 = (0..result.len()).map(|i| result.get(i).unwrap().quantity_ml).sum();
        assert_eq!(total, 500);
        assert_eq!(result.get(2).unwrap().quantity_ml, 100); // last unit partially used
    }

    #[test]
    fn non_available_units_are_excluded() {
        let env = env();
        let mut candidates = soroban_sdk::Vec::new(&env);
        candidates.push_back(make_unit_with_status(
            &env, 1, BloodType::ABNegative, 450, 1000, BloodStatus::Reserved,
        ));
        candidates.push_back(make_unit_with_status(
            &env, 2, BloodType::ABNegative, 450, 2000, BloodStatus::Expired,
        ));
        candidates.push_back(make_unit(
            &env, 3, BloodType::ABNegative, 450, 3000, // Available
        ));

        let result = select_units(
            &env,
            candidates,
            BloodType::ABNegative,
            Urgency::Routine,
            450,
            None,
            0,
        );

        assert_eq!(result.len(), 1);
        assert_eq!(result.get(0).unwrap().unit_id, 3);
    }

    #[test]
    fn no_candidates_returns_empty() {
        let env = env();
        let candidates = soroban_sdk::Vec::new(&env);

        let result = select_units(
            &env,
            candidates,
            BloodType::ABPositive,
            Urgency::Critical,
            900,
            None,
            0,
        );

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn exact_match_exhausted_falls_back_to_compatible() {
        let env = env();
        let mut candidates = soroban_sdk::Vec::new(&env);
        // 200 ml exact
        candidates.push_back(make_unit(&env, 1, BloodType::APositive, 200, 1000));
        // 300 ml compatible
        candidates.push_back(make_unit(&env, 2, BloodType::ONegative, 300, 2000));

        let result = select_units(
            &env,
            candidates,
            BloodType::APositive,
            Urgency::Urgent,
            500,
            None,
            0,
        );

        assert_eq!(result.len(), 2);
        assert_eq!(result.get(0).unwrap().match_kind, MatchKind::Exact);
        assert_eq!(result.get(1).unwrap().match_kind, MatchKind::Compatible);
        let total: u32 = (0..result.len()).map(|i| result.get(i).unwrap().quantity_ml).sum();
        assert_eq!(total, 500);
    }

    #[test]
    fn request_fully_satisfied_stops_early() {
        let env = env();
        let mut candidates = soroban_sdk::Vec::new(&env);
        candidates.push_back(make_unit(&env, 1, BloodType::BNegative, 450, 1000));
        candidates.push_back(make_unit(&env, 2, BloodType::BNegative, 450, 2000));
        candidates.push_back(make_unit(&env, 3, BloodType::BNegative, 450, 3000));

        // Only need 450 ml
        let result = select_units(
            &env,
            candidates,
            BloodType::BNegative,
            Urgency::Routine,
            450,
            None,
            0,
        );

        assert_eq!(result.len(), 1);
        assert_eq!(result.get(0).unwrap().unit_id, 1); // oldest first
    }

    // ── Multi-request urgency ordering ───────────────────────────────────────

    #[test]
    fn urgency_priority_values_are_ordered() {
        assert!(Urgency::Critical.priority() > Urgency::Urgent.priority());
        assert!(Urgency::Urgent.priority() > Urgency::Routine.priority());
        assert!(Urgency::Routine.priority() > Urgency::Scheduled.priority());
    }
}

// ---------------------------------------------------------------------------
// Contract-level integration tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod contract_tests {
    use soroban_sdk::{
        testutils::Address as _,
        Address, Env,
    };

    use crate::{BloodType, MatchingContract, MatchingContractClient};

    fn setup<'a>() -> (Env, MatchingContractClient<'a>, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(MatchingContract, ());
        let client = MatchingContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let inventory = Address::generate(&env);
        let requests = Address::generate(&env);

        client.initialize(&admin, &inventory, &requests);

        (env, client, admin, inventory, requests)
    }

    #[test]
    fn initialize_sets_state() {
        let (_env, client, admin, _inv, _req) = setup();
        assert!(client.is_initialized());
        assert_eq!(client.get_admin(), admin);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #600)")]
    fn double_initialize_panics() {
        let (_env, client, admin, inv, req) = setup();
        client.initialize(&admin, &inv, &req);
    }

    #[test]
    fn get_compatible_types_o_negative() {
        let (_env, client, ..) = setup();
        let types = client.get_compatible_types(&BloodType::ONegative);
        assert_eq!(types.len(), 1);
        assert_eq!(types.get(0).unwrap(), BloodType::ONegative);
    }

    #[test]
    fn get_compatible_types_ab_positive_all_eight() {
        let (_env, client, ..) = setup();
        let types = client.get_compatible_types(&BloodType::ABPositive);
        assert_eq!(types.len(), 8);
    }

    #[test]
    fn check_compatibility_o_neg_to_all() {
        let (_env, client, ..) = setup();
        use BloodType::*;
        for recipient in [APositive, ANegative, BPositive, BNegative,
                          ABPositive, ABNegative, OPositive, ONegative] {
            assert!(client.check_compatibility(&ONegative, &recipient));
        }
    }

    #[test]
    fn check_compatibility_a_pos_cannot_donate_to_b_pos() {
        let (_env, client, ..) = setup();
        assert!(!client.check_compatibility(&BloodType::APositive, &BloodType::BPositive));
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #601)")]
    fn match_request_before_init_panics() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(MatchingContract, ());
        let client = MatchingContractClient::new(&env, &contract_id);
        client.match_request(&1);
    }
}

#[cfg(test)]
mod circuit_breaker_tests {
    use soroban_sdk::{testutils::Address as _, Address, Env};

    use crate::{MatchingContract, MatchingContractClient};

    fn setup<'a>() -> (Env, MatchingContractClient<'a>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(MatchingContract, ());
        let client = MatchingContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let inventory = Address::generate(&env);
        let requests = Address::generate(&env);
        client.initialize(&admin, &inventory, &requests);
        (env, client, admin)
    }

    #[test]
    fn test_pause_and_unpause() {
        let (_env, client, admin) = setup();
        assert!(!client.is_paused());
        client.pause(&admin);
        assert!(client.is_paused());
        client.unpause(&admin);
        assert!(!client.is_paused());
    }

    #[test]
    fn test_pause_blocks_match_request() {
        let (_env, client, admin) = setup();
        client.pause(&admin);
        let result = client.try_match_request(&1u64);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_initialized_readable_while_paused() {
        let (_env, client, admin) = setup();
        client.pause(&admin);
        assert!(client.is_initialized());
    }

    #[test]
    #[should_panic]
    fn test_non_admin_cannot_pause() {
        let (env, client, _admin) = setup();
        let attacker = Address::generate(&env);
        client.pause(&attacker);
    }
}

// ── Event coverage (#53) ─────────────────────────────────────────────────────
//
// One test per state-mutating entrypoint asserting the exact cataloged event
// (see EVENTS.md), plus failure-branch tests proving rejected calls emit
// nothing, plus a shared envelope-shape test. `match_request` additionally
// needs minimal mock Inventory/Requests contracts to exercise its success path.
#[cfg(test)]
mod event_coverage {
    use soroban_sdk::{
        contract, contractimpl,
        testutils::{Address as _, Events},
        Address, Env, IntoVal, Map, Symbol, TryIntoVal, Val, Vec,
    };

    use crate::{
        BloodRequest, BloodStatus, BloodType, BloodUnit, InitializedEvent, MatchComputedEvent,
        MatchingContract, MatchingContractClient, MatchingError, MigratedEvent, PauseChangedEvent,
        RequestStatus, UpgradedEvent, Urgency,
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
        let topics: Vec<Val> =
            (Symbol::new(env, domain), Symbol::new(env, event), version).into_val(env);
        assert_eq!(actual, (contract_id.clone(), topics, data.into_val(env)));
    }

    fn setup<'a>() -> (Env, MatchingContractClient<'a>, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(MatchingContract, ());
        let client = MatchingContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let inventory = Address::generate(&env);
        let requests = Address::generate(&env);
        client.initialize(&admin, &inventory, &requests);
        (env, client, admin, inventory, requests)
    }

    #[test]
    fn test_event_envelope_third_topic_is_u32_schema_version() {
        let (env, _client, ..) = setup();
        let all = env.events().all();
        let (_, topics, _) = all.get(all.len() - 1).unwrap();
        assert_eq!(topics.len(), 3, "envelope must be (domain, event, schema_version)");
        let version: u32 = topics.get(2).unwrap().try_into_val(&env).unwrap();
        assert_eq!(version, 1);
    }

    #[test]
    fn test_initialize_emits_event() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(MatchingContract, ());
        let client = MatchingContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let inventory = Address::generate(&env);
        let requests = Address::generate(&env);
        client.initialize(&admin, &inventory, &requests);

        assert_last_event(
            &env,
            &client.address,
            "match",
            "init",
            1,
            InitializedEvent {
                admin,
                inventory_contract: inventory,
                requests_contract: requests,
                initialized_at: env.ledger().timestamp(),
            },
        );
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #600)")]
    fn test_double_initialize_panics_before_emitting() {
        let (_env, client, admin, inv, req) = setup();
        // AlreadyInitialized panics (unwrapped client), so no second event.
        client.initialize(&admin, &inv, &req);
    }

    #[test]
    fn test_pause_emits_event() {
        let (env, client, admin, ..) = setup();
        client.pause(&admin);
        assert_last_event(
            &env,
            &client.address,
            "match",
            "pause",
            1,
            PauseChangedEvent {
                admin,
                paused: true,
                changed_at: env.ledger().timestamp(),
            },
        );
    }

    #[test]
    fn test_unpause_emits_event() {
        let (env, client, admin, ..) = setup();
        client.pause(&admin);
        client.unpause(&admin);
        assert_last_event(
            &env,
            &client.address,
            "match",
            "pause",
            1,
            PauseChangedEvent {
                admin,
                paused: false,
                changed_at: env.ledger().timestamp(),
            },
        );
    }

    #[test]
    fn test_match_request_before_init_emits_no_event() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(MatchingContract, ());
        let client = MatchingContractClient::new(&env, &contract_id);
        let result = client.try_match_request(&1u64);
        assert_eq!(result, Err(Ok(MatchingError::NotInitialized)));
        assert!(env.events().all().is_empty());
    }

    #[test]
    fn test_migrate_refused_at_current_schema_emits_no_event() {
        let (env, client, ..) = setup();
        let before = env.events().all().len();
        let result = client.try_migrate();
        assert_eq!(result, Err(Ok(MatchingError::MigrationAlreadyApplied)));
        assert_eq!(env.events().all().len(), before, "refused migration must not emit");
    }

    #[test]
    fn test_upgrade_fails_when_not_initialized_emits_no_event() {
        let env = Env::default();
        let contract_id = env.register(MatchingContract, ());
        let client = MatchingContractClient::new(&env, &contract_id);
        let hash = soroban_sdk::BytesN::from_array(&env, &[9u8; 32]);
        let result = client.try_upgrade(&hash);
        assert_eq!(result, Err(Ok(MatchingError::Unauthorized)));
        assert!(env.events().all().is_empty());
    }

    // ── match_request success path: minimal mock domain contracts ────────────

    #[contract]
    struct MockRequestsContract;

    #[contractimpl]
    impl MockRequestsContract {
        pub fn seed(env: Env, request: BloodRequest) {
            env.storage().persistent().set(&request.id, &request);
        }

        pub fn get_request(env: Env, request_id: u64) -> BloodRequest {
            env.storage().persistent().get(&request_id).unwrap()
        }
    }

    #[contract]
    struct MockInventoryContract;

    #[contractimpl]
    impl MockInventoryContract {
        pub fn seed(env: Env, unit: BloodUnit) {
            env.storage().persistent().set(&(1u32, unit.id), &unit);
            let mut ids: Vec<u64> = env
                .storage()
                .persistent()
                .get(&(2u32, unit.blood_type as u32))
                .unwrap_or(Vec::new(&env));
            ids.push_back(unit.id);
            env.storage()
                .persistent()
                .set(&(2u32, unit.blood_type as u32), &ids);
        }

        pub fn get_blood_unit(env: Env, blood_unit_id: u64) -> BloodUnit {
            env.storage().persistent().get(&(1u32, blood_unit_id)).unwrap()
        }

        pub fn get_units_by_blood_type(env: Env, blood_type: BloodType) -> Vec<u64> {
            env.storage()
                .persistent()
                .get(&(2u32, blood_type as u32))
                .unwrap_or(Vec::new(&env))
        }
    }

    #[test]
    fn test_match_request_emits_event_with_matched_units() {
        let env = Env::default();
        env.mock_all_auths();

        let inv_id = env.register(MockInventoryContract, ());
        let req_id = env.register(MockRequestsContract, ());
        let inv_client = MockInventoryContractClient::new(&env, &inv_id);
        let req_client = MockRequestsContractClient::new(&env, &req_id);

        let hospital = Address::generate(&env);
        let bank = Address::generate(&env);

        let unit = BloodUnit {
            id: 1,
            blood_type: BloodType::OPositive,
            quantity_ml: 500,
            bank_id: bank,
            donor_id: None,
            donation_timestamp: 0,
            expiration_timestamp: 1_000_000,
            status: BloodStatus::Available,
            metadata: Map::new(&env),
        };
        inv_client.seed(&unit);

        let request = BloodRequest {
            id: 42,
            hospital_id: hospital,
            blood_type: BloodType::OPositive,
            component: crate::BloodComponent::WholeBlood,
            quantity_ml: 300,
            urgency: Urgency::Urgent,
            created_timestamp: 0,
            required_by_timestamp: 999_999,
            status: RequestStatus::Pending,
            assigned_units: Vec::new(&env),
            fulfilled_quantity_ml: 0,
        };
        req_client.seed(&request);

        let contract_id = env.register(MatchingContract, ());
        let client = MatchingContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin, &inv_id, &req_id);

        let result = client.match_request(&42u64);
        assert_eq!(result.total_matched_ml, 300);
        assert_eq!(result.remaining_ml, 0);
        assert!(!result.partial_fulfillment);

        assert_last_event(
            &env,
            &client.address,
            "match",
            "matched",
            1,
            MatchComputedEvent {
                request_id: 42,
                matched_unit_ids: soroban_sdk::vec![&env, 1u64],
                total_matched_ml: 300,
                remaining_ml: 0,
                partial_fulfillment: false,
                matched_at: env.ledger().timestamp(),
            },
        );
    }

    // Reference the upgrade/migrate event types so they stay covered by
    // the compiler even though their success path isn't independently
    // testable without a real compiled WASM binary (see EVENTS.md).
    #[allow(dead_code)]
    fn _reference_upgrade_and_migrate_event_types(u: UpgradedEvent, m: MigratedEvent) {
        let _ = (u, m);
    }
}
