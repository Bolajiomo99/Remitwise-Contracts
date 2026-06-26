use super::*;
use soroban_sdk::{
    testutils::{Address as AddressTrait, Ledger},
    Address, Env, String, Vec,
};

macro_rules! setup {
    ($env:ident, $client:ident, $owner:ident) => {
        let $env = Env::default();
        $env.mock_all_auths();
        let contract_id = $env.register_contract(None, Insurance);
        let $client = InsuranceClient::new(&$env, &contract_id);
        let $owner = Address::generate(&$env);
        $client.init(&$owner);
    };
}

fn short_name(env: &Env) -> String {
    String::from_str(env, "P")
}

// =============================================================================
// init
// =============================================================================

#[test]
fn test_init_success() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Insurance);
    let client = InsuranceClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    client.init(&owner);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_init_cannot_reinit() {
    setup!(env, client, owner);
    client.init(&owner);
}

// =============================================================================
// create_policy — happy path
// =============================================================================

#[test]
fn test_create_policy_success() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);

    let policy_id = client.create_policy(
        &caller,
        &String::from_str(&env, "Health Policy"),
        &CoverageType::Health,
        &5_000_000i128,
        &100_000_000i128,
    );

    let policy = client.get_policy(&policy_id);
    assert_eq!(policy.owner, caller);
    assert_eq!(policy.monthly_premium, 5_000_000);
    assert_eq!(policy.coverage_amount, 100_000_000);
    assert!(policy.active);
    assert_eq!(policy.id, 1);
}

#[test]
fn test_create_policy_increments_id() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);

    let id1 = client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &10_000_000i128,
    );
    let id2 = client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &10_000_000i128,
    );
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
}

// =============================================================================
// create_policy — name validation
// =============================================================================

#[test]
#[should_panic(expected = "name cannot be empty")]
fn test_create_policy_empty_name_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    client.create_policy(
        &caller,
        &String::from_str(&env, ""),
        &CoverageType::Health,
        &5_000_000i128,
        &10_000_000i128,
    );
}

#[test]
#[should_panic(expected = "name too long")]
fn test_create_policy_name_too_long_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    let long_name = String::from_str(
        &env,
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA1",
    );
    client.create_policy(
        &caller,
        &long_name,
        &CoverageType::Health,
        &5_000_000i128,
        &10_000_000i128,
    );
}

#[test]
fn test_create_policy_name_at_max_length_succeeds() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    let max_name = String::from_str(
        &env,
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    );
    client.create_policy(
        &caller,
        &max_name,
        &CoverageType::Health,
        &5_000_000i128,
        &10_000_000i128,
    );
}

// =============================================================================
// create_policy — premium validation
// =============================================================================

#[test]
#[should_panic(expected = "monthly_premium must be positive")]
fn test_create_policy_zero_premium_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &0i128,
        &50_000_000i128,
    );
}

#[test]
#[should_panic(expected = "monthly_premium must be positive")]
fn test_create_policy_negative_premium_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &-1i128,
        &50_000_000i128,
    );
}

#[test]
#[should_panic(expected = "monthly_premium out of range for coverage type")]
fn test_create_health_premium_below_min_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &999_999i128,
        &50_000_000i128,
    );
}

#[test]
#[should_panic(expected = "monthly_premium out of range for coverage type")]
fn test_create_health_premium_above_max_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &500_000_001i128,
        &10_000_000i128,
    );
}

#[test]
fn test_health_premium_at_minimum_boundary_succeeds() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &1_000_000i128,
        &10_000_000i128,
    );
}

#[test]
fn test_health_premium_at_maximum_boundary_succeeds() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &500_000_000i128,
        &10_000_000i128,
    );
}

#[test]
#[should_panic(expected = "monthly_premium out of range for coverage type")]
fn test_create_life_premium_below_min_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Life,
        &499_999i128,
        &50_000_000i128,
    );
}

#[test]
#[should_panic(expected = "monthly_premium out of range for coverage type")]
fn test_create_property_premium_below_min_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Property,
        &1_999_999i128,
        &100_000_000i128,
    );
}

#[test]
#[should_panic(expected = "monthly_premium out of range for coverage type")]
fn test_create_auto_premium_below_min_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Auto,
        &1_499_999i128,
        &20_000_000i128,
    );
}

#[test]
#[should_panic(expected = "monthly_premium out of range for coverage type")]
fn test_create_liability_premium_below_min_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Liability,
        &799_999i128,
        &5_000_000i128,
    );
}

#[test]
#[should_panic(expected = "monthly_premium out of range for coverage type")]
fn test_create_property_premium_above_max_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Property,
        &2_000_000_001i128,
        &100_000_000i128,
    );
}

#[test]
#[should_panic(expected = "monthly_premium out of range for coverage type")]
fn test_create_auto_premium_above_max_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Auto,
        &750_000_001i128,
        &20_000_000i128,
    );
}

#[test]
#[should_panic(expected = "monthly_premium out of range for coverage type")]
fn test_create_liability_premium_above_max_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Liability,
        &400_000_001i128,
        &5_000_000i128,
    );
}

// =============================================================================
// create_policy — coverage amount validation
// =============================================================================

#[test]
#[should_panic(expected = "coverage_amount must be positive")]
fn test_create_policy_zero_coverage_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &0i128,
    );
}

#[test]
#[should_panic(expected = "coverage_amount must be positive")]
fn test_create_policy_negative_coverage_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &-1i128,
    );
}

#[test]
#[should_panic(expected = "coverage_amount out of range for coverage type")]
fn test_create_health_coverage_below_min_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &9_999_999i128,
    );
}

#[test]
#[should_panic(expected = "coverage_amount out of range for coverage type")]
fn test_create_health_coverage_above_max_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &500_000_000i128,
        &100_000_000_001i128,
    );
}

#[test]
fn test_health_coverage_at_minimum_boundary_succeeds() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &10_000_000i128,
    );
}

#[test]
fn test_health_coverage_at_maximum_boundary_succeeds() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &500_000_000i128,
        &100_000_000_000i128,
    );
}

#[test]
#[should_panic(expected = "coverage_amount out of range for coverage type")]
fn test_create_life_coverage_below_min_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Life,
        &1_000_000i128,
        &49_999_999i128,
    );
}

#[test]
#[should_panic(expected = "coverage_amount out of range for coverage type")]
fn test_create_life_coverage_above_max_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Life,
        &1_000_000_000i128,
        &500_000_000_001i128,
    );
}

#[test]
#[should_panic(expected = "coverage_amount out of range for coverage type")]
fn test_create_property_coverage_below_min_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Property,
        &5_000_000i128,
        &99_999_999i128,
    );
}

#[test]
#[should_panic(expected = "coverage_amount out of range for coverage type")]
fn test_create_auto_coverage_above_max_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Auto,
        &750_000_000i128,
        &200_000_000_001i128,
    );
}

#[test]
#[should_panic(expected = "coverage_amount out of range for coverage type")]
fn test_create_liability_coverage_above_max_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Liability,
        &400_000_000i128,
        &50_000_000_001i128,
    );
}

// =============================================================================
// create_policy — ratio guard (coverage_amount > premium * 12 * 500)
// =============================================================================

#[test]
#[should_panic(
    expected = "unsupported combination: coverage_amount too high relative to premium"
)]
fn test_create_policy_coverage_too_high_for_premium_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    // premium = 1_000_000 -> annual = 12_000_000 -> max_coverage = 6_000_000_000
    // Supply coverage = 6_000_000_001 (just over ratio, within Health's hard cap of 100B)
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &1_000_000i128,
        &6_000_000_001i128,
    );
}

#[test]
fn test_create_policy_coverage_exactly_at_ratio_limit_succeeds() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    // premium = 1_000_000 -> ratio limit = 1M * 12 * 500 = 6_000_000_000
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &1_000_000i128,
        &6_000_000_000i128,
    );
}

#[test]
#[should_panic(
    expected = "unsupported combination: coverage_amount too high relative to premium"
)]
fn test_create_policy_coverage_way_beyond_ratio_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    // premium = 5_000_000 -> ratio = 5M * 12 * 500 = 30_000_000_000
    // Supply 100_000_000_000 (within Health max_coverage = 100B) way over ratio
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &100_000_000_000i128,
    );
}

// =============================================================================
// create_policy — MAX_POLICIES cap
// =============================================================================

#[test]
#[should_panic(expected = "max policies reached")]
fn test_create_policy_at_max_policies_rejects_overflow() {
    setup!(env, client, owner);
    env.budget().reset_unlimited();
    let caller = Address::generate(&env);

    for _ in 0..1_000 {
        client.create_policy(
            &caller,
            &short_name(&env),
            &CoverageType::Health,
            &5_000_000i128,
            &10_000_000i128,
        );
    }

    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &10_000_000i128,
    );
}

// =============================================================================
// pay_premium — happy path
// =============================================================================

#[test]
fn test_pay_premium_success() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);

    let id = client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &50_000_000i128,
    );

    let result = client.pay_premium(&caller, &id);
    assert!(result);
}

/// Verify that pay_premium advances last_payment_at and next_payment_date by
/// THIRTY_DAYS_SECS (30 * 86400) from the current ledger timestamp.
#[test]
fn test_pay_premium_updates_dates_correctly() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);

    env.ledger().set_timestamp(1_000_000u64);

    let id = client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &50_000_000i128,
    );

    env.ledger().set_timestamp(2_000_000u64);
    client.pay_premium(&caller, &id);

    let policy = client.get_policy(&id);
    assert_eq!(policy.last_payment_at, 2_000_000);
    assert_eq!(policy.next_payment_date, 2_000_000 + 30 * 24 * 60 * 60);
}

#[test]
fn test_multiple_premium_payments_push_date_forward() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);

    env.ledger().set_timestamp(1_000_000u64);

    let id = client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &50_000_000i128,
    );

    env.ledger().set_timestamp(2_000_000u64);
    client.pay_premium(&caller, &id);

    let p1 = client.get_policy(&id);
    assert_eq!(p1.next_payment_date, 2_000_000 + 30 * 86400);

    env.ledger().set_timestamp(3_000_000u64);
    client.pay_premium(&caller, &id);

    let p2 = client.get_policy(&id);
    assert_eq!(p2.last_payment_at, 3_000_000);
    assert_eq!(p2.next_payment_date, 3_000_000 + 30 * 86400);
}

// =============================================================================
// pay_premium — failure cases
// =============================================================================

#[test]
#[should_panic(expected = "Only the policy owner can pay premiums")]
fn test_pay_premium_non_owner_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);

    let id = client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &50_000_000i128,
    );

    let non_owner = Address::generate(&env);
    client.pay_premium(&non_owner, &id);
}

#[test]
#[should_panic(expected = "policy inactive")]
fn test_pay_premium_on_inactive_policy_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);

    let id = client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &50_000_000i128,
    );

    client.deactivate_policy(&owner, &id);
    client.pay_premium(&caller, &id);
}

#[test]
#[should_panic(expected = "policy not found")]
fn test_pay_premium_nonexistent_policy_panics() {
    setup!(env, client, owner);
    client.pay_premium(&owner, &999u32);
}

// =============================================================================
// deactivate_policy — happy path
// =============================================================================

#[test]
fn test_deactivate_policy_success() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);

    let id = client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &50_000_000i128,
    );

    let result = client.deactivate_policy(&owner, &id);
    assert!(result);

    let policy = client.get_policy(&id);
    assert!(!policy.active);
}

#[test]
fn test_deactivate_policy_by_owner_succeeds() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);

    let id = client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &50_000_000i128,
    );

    let result = client.deactivate_policy(&caller, &id);
    assert!(result);
    assert!(!client.get_policy(&id).active);
}

// =============================================================================
// deactivate_policy — failure cases
// =============================================================================

#[test]
#[should_panic(expected = "unauthorized")]
fn test_deactivate_policy_non_owner_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);

    let id = client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &50_000_000i128,
    );

    let stranger = Address::generate(&env);
    client.deactivate_policy(&stranger, &id);
}

#[test]
#[should_panic(expected = "policy not found")]
fn test_deactivate_policy_nonexistent_panics() {
    setup!(env, client, owner);
    client.deactivate_policy(&owner, &999u32);
}

#[test]
#[should_panic(expected = "already inactive")]
fn test_deactivate_policy_already_inactive_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);

    let id = client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &50_000_000i128,
    );

    client.deactivate_policy(&owner, &id);
    client.deactivate_policy(&owner, &id);
}

// =============================================================================
// deactivate_policy — removal from ActivePolicies
// =============================================================================

#[test]
fn test_deactivate_policy_removes_from_active_list() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);

    let id = client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &50_000_000i128,
    );

    {
        let active = client.get_active_policies(&caller, &0, &50);
        assert_eq!(active.count, 1);
    }

    client.deactivate_policy(&owner, &id);

    {
        let active = client.get_active_policies(&caller, &0, &50);
        assert_eq!(active.count, 0);
    }
}

#[test]
fn test_deactivate_policy_preserves_policy_data() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);

    let id = client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &50_000_000i128,
    );

    client.deactivate_policy(&owner, &id);

    let policy = client.get_policy(&id);
    assert!(!policy.active);
    assert_eq!(policy.owner, caller);
    assert_eq!(policy.monthly_premium, 5_000_000);
}

// =============================================================================
// get_active_policies — pagination & ordering
// =============================================================================

#[test]
fn test_get_active_policies_empty_initially() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    let page = client.get_active_policies(&caller, &0, &50);
    assert_eq!(page.count, 0);
    assert_eq!(page.items.len(), 0);
}

#[test]
fn test_get_active_policies_returns_all_active() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);

    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &10_000_000i128,
    );
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Life,
        &1_000_000i128,
        &60_000_000i128,
    );

    let page = client.get_active_policies(&caller, &0, &50);
    assert_eq!(page.count, 2);
    assert_eq!(page.items.len(), 2);
}

#[test]
fn test_get_active_policies_excludes_deactivated() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);

    let id1 = client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &10_000_000i128,
    );
    let id2 = client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Life,
        &1_000_000i128,
        &60_000_000i128,
    );

    client.deactivate_policy(&caller, &id1);

    let page = client.get_active_policies(&caller, &0, &50);
    assert_eq!(page.count, 1);
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items.get(0).unwrap(), id2);
}

#[test]
fn test_get_active_policies_owner_isolation() {
    setup!(env, client, owner);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.create_policy(
        &alice,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &10_000_000i128,
    );
    client.create_policy(
        &alice,
        &short_name(&env),
        &CoverageType::Life,
        &1_000_000i128,
        &60_000_000i128,
    );

    client.create_policy(
        &bob,
        &short_name(&env),
        &CoverageType::Auto,
        &2_000_000i128,
        &30_000_000i128,
    );

    let alice_page = client.get_active_policies(&alice, &0, &50);
    assert_eq!(alice_page.count, 2);

    let bob_page = client.get_active_policies(&bob, &0, &50);
    assert_eq!(bob_page.count, 1);
}

#[test]
fn test_get_active_policies_cursor_pagination() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);

    for _ in 0..5 {
        client.create_policy(
            &caller,
            &short_name(&env),
            &CoverageType::Health,
            &5_000_000i128,
            &10_000_000i128,
        );
    }

    let page1 = client.get_active_policies(&caller, &0, &2);
    assert_eq!(page1.count, 2);
    assert_eq!(page1.items.len(), 2);
    assert_eq!(page1.items.get(0).unwrap(), 1u32);
    assert_eq!(page1.items.get(1).unwrap(), 2u32);
    assert_eq!(page1.next_cursor, 2u32);

    let page2 = client.get_active_policies(&caller, &page1.next_cursor, &2);
    assert_eq!(page2.count, 2);
    assert_eq!(page2.items.get(0).unwrap(), 3u32);
    assert_eq!(page2.items.get(1).unwrap(), 4u32);
    assert_eq!(page2.next_cursor, 4u32);

    let page3 = client.get_active_policies(&caller, &page2.next_cursor, &2);
    assert_eq!(page3.count, 1);
    assert_eq!(page3.items.get(0).unwrap(), 5u32);
    assert_eq!(page3.next_cursor, 0u32);
}

// =============================================================================
// get_total_monthly_premium
// =============================================================================

#[test]
fn test_get_total_monthly_premium_zero_when_no_policies() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    assert_eq!(client.get_total_monthly_premium(&caller), 0);
}

#[test]
fn test_get_total_monthly_premium_sums_active_only() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);

    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &10_000_000i128,
    );
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Life,
        &1_000_000i128,
        &60_000_000i128,
    );

    assert_eq!(client.get_total_monthly_premium(&caller), 6_000_000);
}

#[test]
fn test_get_total_monthly_premium_excludes_deactivated() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);

    let id1 = client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &10_000_000i128,
    );
    client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Life,
        &1_000_000i128,
        &60_000_000i128,
    );

    assert_eq!(client.get_total_monthly_premium(&caller), 6_000_000);

    client.deactivate_policy(&caller, &id1);

    assert_eq!(client.get_total_monthly_premium(&caller), 1_000_000);
}

#[test]
fn test_get_total_monthly_premium_owner_isolation() {
    setup!(env, client, owner);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.create_policy(
        &alice,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &10_000_000i128,
    );
    client.create_policy(
        &bob,
        &short_name(&env),
        &CoverageType::Life,
        &2_000_000i128,
        &60_000_000i128,
    );

    assert_eq!(client.get_total_monthly_premium(&alice), 5_000_000);
    assert_eq!(client.get_total_monthly_premium(&bob), 2_000_000);
}

// =============================================================================
// get_policy
// =============================================================================

#[test]
fn test_get_policy_returns_correct_fields() {
    setup!(env, client, owner);

    env.ledger().set_timestamp(1_700_000_000u64);

    let id = client.create_policy(
        &owner,
        &String::from_str(&env, "My Health Plan"),
        &CoverageType::Health,
        &10_000_000i128,
        &100_000_000i128,
    );

    let policy = client.get_policy(&id);
    assert_eq!(policy.id, 1);
    assert_eq!(policy.owner, owner);
    assert_eq!(policy.name, String::from_str(&env, "My Health Plan"));
    assert_eq!(policy.coverage_type, CoverageType::Health);
    assert_eq!(policy.monthly_premium, 10_000_000);
    assert_eq!(policy.coverage_amount, 100_000_000);
    assert!(policy.active);
    assert_eq!(policy.created_at, 1_700_000_000);
    assert_eq!(policy.last_payment_at, 0);
    assert_eq!(policy.next_payment_date, 1_700_000_000 + 30 * 86400);
    assert!(policy.external_ref.is_none());
}

#[test]
#[should_panic(expected = "policy not found")]
fn test_get_policy_nonexistent_panics() {
    setup!(env, client, owner);
    client.get_policy(&999u32);
}

// =============================================================================
// batch_pay_premiums
// =============================================================================

#[test]
fn test_batch_pay_premiums_all_owned_active() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);

    let id1 = client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &10_000_000i128,
    );
    let id2 = client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Life,
        &1_000_000i128,
        &60_000_000i128,
    );

    let ids = Vec::from_array(&env, [id1, id2]);
    let count = client.batch_pay_premiums(&caller, &ids);
    assert_eq!(count, 2);
}

#[test]
fn test_batch_pay_premiums_skips_unowned_and_inactive() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);
    let other = Address::generate(&env);

    let owned_id = client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &10_000_000i128,
    );
    let other_id = client.create_policy(
        &other,
        &short_name(&env),
        &CoverageType::Auto,
        &2_000_000i128,
        &30_000_000i128,
    );

    let inactive_id = client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Life,
        &1_000_000i128,
        &60_000_000i128,
    );
    client.deactivate_policy(&caller, &inactive_id);

    env.ledger().set_timestamp(1_000_000u64);
    let ids = Vec::from_array(&env, [owned_id, other_id, inactive_id]);
    let count = client.batch_pay_premiums(&caller, &ids);
    assert_eq!(count, 1);

    let policy = client.get_policy(&owned_id);
    assert_eq!(policy.last_payment_at, 1_000_000);
}

#[test]
#[should_panic(expected = "batch too large")]
fn test_batch_pay_premiums_exceeds_max_batch_size() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);

    let mut arr = [0u32; 51];
    for i in 0..51 {
        arr[i] = i as u32;
    }
    let ids = Vec::from_array(&env, arr);
    client.batch_pay_premiums(&caller, &ids);
}

// =============================================================================
// set_external_ref
// =============================================================================

#[test]
fn test_set_external_ref_success() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);

    let id = client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &50_000_000i128,
    );

    let new_ref = String::from_str(&env, "EXT-REF-001");
    client.set_external_ref(&owner, &id, &Some(new_ref));

    let policy = client.get_policy(&id);
    assert!(policy.external_ref.is_some());
}

#[test]
fn test_set_external_ref_clear() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);

    let id = client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &50_000_000i128,
    );

    let new_ref = String::from_str(&env, "EXT-REF-001");
    client.set_external_ref(&owner, &id, &Some(new_ref));

    client.set_external_ref(&owner, &id, &None);

    let policy = client.get_policy(&id);
    assert!(policy.external_ref.is_none());
}

#[test]
#[should_panic(expected = "unauthorized")]
fn test_set_external_ref_non_owner_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);

    let id = client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &50_000_000i128,
    );

    let stranger = Address::generate(&env);
    client.set_external_ref(&stranger, &id, &Some(String::from_str(&env, "BAD")));
}

#[test]
#[should_panic(expected = "external_ref length out of range")]
fn test_set_external_ref_too_long_panics() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);

    let id = client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &50_000_000i128,
    );

    let long_ref = String::from_str(
        &env,
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA1",
    );
    client.set_external_ref(&owner, &id, &Some(long_ref));
}

#[test]
fn test_set_external_ref_at_max_length_succeeds() {
    setup!(env, client, owner);
    let caller = Address::generate(&env);

    let id = client.create_policy(
        &caller,
        &short_name(&env),
        &CoverageType::Health,
        &5_000_000i128,
        &50_000_000i128,
    );

    let max_ref = String::from_str(
        &env,
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    );
    client.set_external_ref(&owner, &id, &Some(max_ref));

    let policy = client.get_policy(&id);
    assert!(policy.external_ref.is_some());
}

// =============================================================================
// Uninitialized contract guard
// =============================================================================

#[test]
#[should_panic(expected = "not initialized")]
fn test_create_policy_without_init_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Insurance);
    let client = InsuranceClient::new(&env, &contract_id);
    let caller = Address::generate(&env);
    client.create_policy(
        &caller,
        &String::from_str(&env, "Test"),
        &CoverageType::Health,
        &5_000_000i128,
        &50_000_000i128,
    );
}

#[test]
#[should_panic(expected = "not initialized")]
fn test_get_active_policies_without_init_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Insurance);
    let client = InsuranceClient::new(&env, &contract_id);
    let caller = Address::generate(&env);
    client.get_active_policies(&caller, &0, &50);
}

#[test]
#[should_panic(expected = "not initialized")]
fn test_get_policy_without_init_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Insurance);
    let client = InsuranceClient::new(&env, &contract_id);
    client.get_policy(&1u32);
}

#[test]
#[should_panic(expected = "not initialized")]
fn test_get_total_monthly_premium_without_init_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Insurance);
    let client = InsuranceClient::new(&env, &contract_id);
    let caller = Address::generate(&env);
    client.get_total_monthly_premium(&caller);
}

#[test]
#[should_panic(expected = "not initialized")]
fn test_pay_premium_without_init_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Insurance);
    let client = InsuranceClient::new(&env, &contract_id);
    let caller = Address::generate(&env);
    client.pay_premium(&caller, &1u32);
}

#[test]
#[should_panic(expected = "not initialized")]
fn test_deactivate_policy_without_init_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, Insurance);
    let client = InsuranceClient::new(&env, &contract_id);
    let caller = Address::generate(&env);
    client.deactivate_policy(&caller, &1u32);
}
