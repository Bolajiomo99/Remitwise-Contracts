#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as AddressTrait, Events, Ledger},
    testutils::storage::Instance as StorageInstance,
    token::{StellarAssetClient, TokenClient},
    Address, Env, Symbol, TryFromVal,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Register a native Stellar asset (SAC) and return (contract_id, admin).
/// The admin is the issuer; we mint `amount` to `recipient`.
fn setup_token(env: &Env, admin: &Address, recipient: &Address, amount: i128) -> Address {
    let token_id = env.register_stellar_asset_contract_v2(admin.clone()).address();
    let sac = StellarAssetClient::new(env, &token_id);
    sac.mint(recipient, &amount);
    token_id
}

/// Build a fresh AccountGroup with four distinct addresses.
fn make_accounts(env: &Env) -> AccountGroup {
    AccountGroup {
        spending: Address::generate(env),
        savings: Address::generate(env),
        bills: Address::generate(env),
        insurance: Address::generate(env),
    }
}

// ---------------------------------------------------------------------------
// initialize_split
// ---------------------------------------------------------------------------

#[test]
fn test_initialize_split_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    let success = client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    assert_eq!(success, true);

    let config = client.get_config().unwrap();
    assert_eq!(config.owner, owner);
    assert_eq!(config.spending_percent, 50);
    assert_eq!(config.savings_percent, 30);
    assert_eq!(config.bills_percent, 15);
    assert_eq!(config.insurance_percent, 5);
    assert_eq!(config.usdc_contract, token_id);
}

#[test]
fn test_initialize_split_invalid_sum() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    let result = client.try_initialize_split(&owner, &0, &token_id, &50, &50, &10, &0);
    assert_eq!(result, Err(Ok(RemittanceSplitError::PercentagesDoNotSumTo100)));
}

#[test]
fn test_initialize_split_already_initialized() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    let result = client.try_initialize_split(&owner, &1, &token_id, &50, &30, &15, &5);
    assert_eq!(result, Err(Ok(RemittanceSplitError::AlreadyInitialized)));
}

#[test]
#[should_panic]
fn test_initialize_split_requires_auth() {
    let env = Env::default();
    // No mock_all_auths — owner has not authorized
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_id = Address::generate(&env);
    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
}

// ---------------------------------------------------------------------------
// update_split
// ---------------------------------------------------------------------------

#[test]
fn test_update_split() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    let success = client.update_split(&owner, &1, &40, &40, &10, &10);
    assert_eq!(success, true);

    let config = client.get_config().unwrap();
    assert_eq!(config.spending_percent, 40);
    assert_eq!(config.savings_percent, 40);
    assert_eq!(config.bills_percent, 10);
    assert_eq!(config.insurance_percent, 10);
}

#[test]
fn test_update_split_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let other = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    let result = client.try_update_split(&other, &0, &40, &40, &10, &10);
    assert_eq!(result, Err(Ok(RemittanceSplitError::Unauthorized)));
}

#[test]
fn test_update_split_not_initialized() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let caller = Address::generate(&env);

    let result = client.try_update_split(&caller, &0, &25, &25, &25, &25);
    assert_eq!(result, Err(Ok(RemittanceSplitError::NotInitialized)));
}

#[test]
fn test_update_split_percentages_must_sum_to_100() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    let result = client.try_update_split(&owner, &1, &60, &30, &15, &5);
    assert_eq!(result, Err(Ok(RemittanceSplitError::PercentagesDoNotSumTo100)));
}

// ---------------------------------------------------------------------------
// calculate_split
// ---------------------------------------------------------------------------

#[test]
fn test_calculate_split() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    let amounts = client.calculate_split(&1000);
    assert_eq!(amounts.get(0).unwrap(), 500);
    assert_eq!(amounts.get(1).unwrap(), 300);
    assert_eq!(amounts.get(2).unwrap(), 150);
    assert_eq!(amounts.get(3).unwrap(), 50);
}

#[test]
fn test_calculate_split_zero_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    let result = client.try_calculate_split(&0);
    assert_eq!(result, Err(Ok(RemittanceSplitError::InvalidAmount)));
}

#[test]
fn test_calculate_split_rounding() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    client.initialize_split(&owner, &0, &token_id, &33, &33, &33, &1);
    let amounts = client.calculate_split(&100);
    let sum: i128 = amounts.iter().sum();
    assert_eq!(sum, 100);
}

#[test]
fn test_calculate_complex_rounding() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    client.initialize_split(&owner, &0, &token_id, &17, &19, &23, &41);
    let amounts = client.calculate_split(&1000);
    assert_eq!(amounts.get(0).unwrap(), 170);
    assert_eq!(amounts.get(1).unwrap(), 190);
    assert_eq!(amounts.get(2).unwrap(), 230);
    assert_eq!(amounts.get(3).unwrap(), 410);
}

// ---------------------------------------------------------------------------
// distribute_usdc — happy path
// ---------------------------------------------------------------------------

#[test]
fn test_distribute_usdc_success() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let total = 1_000i128;
    let token_id = setup_token(&env, &token_admin, &owner, total);

    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);

    let accounts = make_accounts(&env);
    let result = client.distribute_usdc(&token_id, &owner, &1, &accounts, &total);
    assert_eq!(result, true);

    let token = TokenClient::new(&env, &token_id);
    assert_eq!(token.balance(&accounts.spending), 500);
    assert_eq!(token.balance(&accounts.savings), 300);
    assert_eq!(token.balance(&accounts.bills), 150);
    assert_eq!(token.balance(&accounts.insurance), 50);
    assert_eq!(token.balance(&owner), 0);
}

#[test]
fn test_distribute_usdc_emits_event() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 1_000);

    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    let accounts = make_accounts(&env);
    client.distribute_usdc(&token_id, &owner, &1, &accounts, &1_000);

    let events = env.events().all();
    let last = events.last().unwrap();
    let topic0: Symbol = Symbol::try_from_val(&env, &last.1.get(0).unwrap()).unwrap();
    let topic1: SplitEvent = SplitEvent::try_from_val(&env, &last.1.get(1).unwrap()).unwrap();
    assert_eq!(topic0, symbol_short!("split"));
    assert_eq!(topic1, SplitEvent::DistributionCompleted);
}

#[test]
fn test_distribute_usdc_nonce_increments() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 2_000);

    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    // nonce after init = 1
    let accounts = make_accounts(&env);
    client.distribute_usdc(&token_id, &owner, &1, &accounts, &1_000);
    // nonce after first distribute = 2
    assert_eq!(client.get_nonce(&owner), 2);
}

// ---------------------------------------------------------------------------
// distribute_usdc — auth must be first (before amount check)
// ---------------------------------------------------------------------------

#[test]
#[should_panic]
fn test_distribute_usdc_requires_auth() {
    // Set up contract state with a mocked env first
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 1_000);
    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);

    // Now call distribute_usdc without mocking auth for `owner` — should panic
    // We create a fresh env that does NOT mock auths
    let env2 = Env::default();
    // Re-register the same contract in env2 (no mock_all_auths)
    let contract_id2 = env2.register_contract(None, RemittanceSplit);
    let client2 = RemittanceSplitClient::new(&env2, &contract_id2);
    let accounts = make_accounts(&env2);
    // This should panic because owner has not authorized in env2
    client2.distribute_usdc(&token_id, &owner, &0, &accounts, &1_000);
}

// ---------------------------------------------------------------------------
// distribute_usdc — owner-only enforcement
// ---------------------------------------------------------------------------

#[test]
fn test_distribute_usdc_non_owner_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 1_000);

    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);

    // Attacker self-authorizes but is not the config owner
    let accounts = make_accounts(&env);
    let result = client.try_distribute_usdc(&token_id, &attacker, &0, &accounts, &1_000);
    assert_eq!(result, Err(Ok(RemittanceSplitError::Unauthorized)));
}

// ---------------------------------------------------------------------------
// distribute_usdc — untrusted token contract
// ---------------------------------------------------------------------------

#[test]
fn test_distribute_usdc_untrusted_token_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 1_000);

    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);

    // Supply a different (malicious) token contract address
    let evil_token = Address::generate(&env);
    let accounts = make_accounts(&env);
    let result = client.try_distribute_usdc(&evil_token, &owner, &1, &accounts, &1_000);
    assert_eq!(result, Err(Ok(RemittanceSplitError::UntrustedTokenContract)));
}

// ---------------------------------------------------------------------------
// distribute_usdc — self-transfer guard
// ---------------------------------------------------------------------------

#[test]
fn test_distribute_usdc_self_transfer_spending_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 1_000);

    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);

    // spending destination == owner
    let accounts = AccountGroup {
        spending: owner.clone(),
        savings: Address::generate(&env),
        bills: Address::generate(&env),
        insurance: Address::generate(&env),
    };
    let result = client.try_distribute_usdc(&token_id, &owner, &1, &accounts, &1_000);
    assert_eq!(result, Err(Ok(RemittanceSplitError::SelfTransferNotAllowed)));
}

#[test]
fn test_distribute_usdc_self_transfer_savings_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 1_000);

    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);

    let accounts = AccountGroup {
        spending: Address::generate(&env),
        savings: owner.clone(),
        bills: Address::generate(&env),
        insurance: Address::generate(&env),
    };
    let result = client.try_distribute_usdc(&token_id, &owner, &1, &accounts, &1_000);
    assert_eq!(result, Err(Ok(RemittanceSplitError::SelfTransferNotAllowed)));
}

#[test]
fn test_distribute_usdc_self_transfer_bills_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 1_000);

    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);

    let accounts = AccountGroup {
        spending: Address::generate(&env),
        savings: Address::generate(&env),
        bills: owner.clone(),
        insurance: Address::generate(&env),
    };
    let result = client.try_distribute_usdc(&token_id, &owner, &1, &accounts, &1_000);
    assert_eq!(result, Err(Ok(RemittanceSplitError::SelfTransferNotAllowed)));
}

#[test]
fn test_distribute_usdc_self_transfer_insurance_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 1_000);

    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);

    let accounts = AccountGroup {
        spending: Address::generate(&env),
        savings: Address::generate(&env),
        bills: Address::generate(&env),
        insurance: owner.clone(),
    };
    let result = client.try_distribute_usdc(&token_id, &owner, &1, &accounts, &1_000);
    assert_eq!(result, Err(Ok(RemittanceSplitError::SelfTransferNotAllowed)));
}

// ---------------------------------------------------------------------------
// distribute_usdc — invalid amount
// ---------------------------------------------------------------------------

#[test]
fn test_distribute_usdc_zero_amount_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 1_000);

    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    let accounts = make_accounts(&env);
    let result = client.try_distribute_usdc(&token_id, &owner, &1, &accounts, &0);
    assert_eq!(result, Err(Ok(RemittanceSplitError::InvalidAmount)));
}

#[test]
fn test_distribute_usdc_negative_amount_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 1_000);

    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    let accounts = make_accounts(&env);
    let result = client.try_distribute_usdc(&token_id, &owner, &1, &accounts, &-1);
    assert_eq!(result, Err(Ok(RemittanceSplitError::InvalidAmount)));
}

// ---------------------------------------------------------------------------
// distribute_usdc — not initialized
// ---------------------------------------------------------------------------

#[test]
fn test_distribute_usdc_not_initialized_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_id = Address::generate(&env);

    let accounts = make_accounts(&env);
    let result = client.try_distribute_usdc(&token_id, &owner, &0, &accounts, &1_000);
    assert_eq!(result, Err(Ok(RemittanceSplitError::NotInitialized)));
}

// ---------------------------------------------------------------------------
// distribute_usdc — replay protection
// ---------------------------------------------------------------------------

#[test]
fn test_distribute_usdc_replay_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 2_000);

    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    let accounts = make_accounts(&env);
    // First call with nonce=1 succeeds
    client.distribute_usdc(&token_id, &owner, &1, &accounts, &1_000);
    // Replaying nonce=1 must fail
    let result = client.try_distribute_usdc(&token_id, &owner, &1, &accounts, &500);
    assert_eq!(result, Err(Ok(RemittanceSplitError::InvalidNonce)));
}

// ---------------------------------------------------------------------------
// distribute_usdc — paused contract
// ---------------------------------------------------------------------------

#[test]
fn test_distribute_usdc_paused_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 1_000);

    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    client.pause(&owner);

    let accounts = make_accounts(&env);
    let result = client.try_distribute_usdc(&token_id, &owner, &1, &accounts, &1_000);
    assert_eq!(result, Err(Ok(RemittanceSplitError::Unauthorized)));
}

// ---------------------------------------------------------------------------
// distribute_usdc — correct split math verified end-to-end
// ---------------------------------------------------------------------------

#[test]
fn test_distribute_usdc_split_math_25_25_25_25() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 1_000);

    client.initialize_split(&owner, &0, &token_id, &25, &25, &25, &25);
    let accounts = make_accounts(&env);
    client.distribute_usdc(&token_id, &owner, &1, &accounts, &1_000);

    let token = TokenClient::new(&env, &token_id);
    assert_eq!(token.balance(&accounts.spending), 250);
    assert_eq!(token.balance(&accounts.savings), 250);
    assert_eq!(token.balance(&accounts.bills), 250);
    assert_eq!(token.balance(&accounts.insurance), 250);
}

#[test]
fn test_distribute_usdc_split_math_100_0_0_0() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 1_000);

    client.initialize_split(&owner, &0, &token_id, &100, &0, &0, &0);
    let accounts = make_accounts(&env);
    client.distribute_usdc(&token_id, &owner, &1, &accounts, &1_000);

    let token = TokenClient::new(&env, &token_id);
    assert_eq!(token.balance(&accounts.spending), 1_000);
    assert_eq!(token.balance(&accounts.savings), 0);
    assert_eq!(token.balance(&accounts.bills), 0);
    assert_eq!(token.balance(&accounts.insurance), 0);
}

#[test]
fn test_distribute_usdc_rounding_remainder_goes_to_insurance() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    // 33/33/33/1 with amount=100: 33+33+33=99, insurance gets remainder=1
    let token_id = setup_token(&env, &token_admin, &owner, 100);

    client.initialize_split(&owner, &0, &token_id, &33, &33, &33, &1);
    let accounts = make_accounts(&env);
    client.distribute_usdc(&token_id, &owner, &1, &accounts, &100);

    let token = TokenClient::new(&env, &token_id);
    let total = token.balance(&accounts.spending)
        + token.balance(&accounts.savings)
        + token.balance(&accounts.bills)
        + token.balance(&accounts.insurance);
    assert_eq!(total, 100, "all funds must be distributed");
    assert_eq!(token.balance(&accounts.insurance), 1);
}

// ---------------------------------------------------------------------------
// distribute_usdc — multiple sequential distributions
// ---------------------------------------------------------------------------

#[test]
fn test_distribute_usdc_multiple_rounds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 3_000);

    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    let accounts = make_accounts(&env);

    client.distribute_usdc(&token_id, &owner, &1, &accounts, &1_000);
    client.distribute_usdc(&token_id, &owner, &2, &accounts, &1_000);
    client.distribute_usdc(&token_id, &owner, &3, &accounts, &1_000);

    let token = TokenClient::new(&env, &token_id);
    assert_eq!(token.balance(&accounts.spending), 1_500); // 3 * 500
    assert_eq!(token.balance(&accounts.savings), 900);    // 3 * 300
    assert_eq!(token.balance(&accounts.bills), 450);      // 3 * 150
    assert_eq!(token.balance(&accounts.insurance), 150);  // 3 * 50
    assert_eq!(token.balance(&owner), 0);
}

// ---------------------------------------------------------------------------
// Boundary tests for split percentages
// ---------------------------------------------------------------------------

#[test]
fn test_split_boundary_100_0_0_0() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    let ok = client.initialize_split(&owner, &0, &token_id, &100, &0, &0, &0);
    assert!(ok);
    let amounts = client.calculate_split(&1000);
    assert_eq!(amounts.get(0).unwrap(), 1000);
    assert_eq!(amounts.get(3).unwrap(), 0);
}

#[test]
fn test_split_boundary_0_0_0_100() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    let ok = client.initialize_split(&owner, &0, &token_id, &0, &0, &0, &100);
    assert!(ok);
    let amounts = client.calculate_split(&1000);
    assert_eq!(amounts.get(0).unwrap(), 0);
    assert_eq!(amounts.get(3).unwrap(), 1000);
}

#[test]
fn test_split_boundary_25_25_25_25() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    client.initialize_split(&owner, &0, &token_id, &25, &25, &25, &25);
    let amounts = client.calculate_split(&1000);
    assert_eq!(amounts.get(0).unwrap(), 250);
    assert_eq!(amounts.get(1).unwrap(), 250);
    assert_eq!(amounts.get(2).unwrap(), 250);
    assert_eq!(amounts.get(3).unwrap(), 250);
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[test]
fn test_initialize_split_events() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);

    let events = env.events().all();
    let last_event = events.last().unwrap();
    let topic0: Symbol = Symbol::try_from_val(&env, &last_event.1.get(0).unwrap()).unwrap();
    let topic1: SplitEvent = SplitEvent::try_from_val(&env, &last_event.1.get(1).unwrap()).unwrap();
    assert_eq!(topic0, symbol_short!("split"));
    assert_eq!(topic1, SplitEvent::Initialized);
}

#[test]
fn test_update_split_events() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    client.update_split(&owner, &1, &40, &40, &10, &10);

    let events = env.events().all();
    let last_event = events.last().unwrap();
    let topic1: SplitEvent = SplitEvent::try_from_val(&env, &last_event.1.get(1).unwrap()).unwrap();
    assert_eq!(topic1, SplitEvent::Updated);
}

// ---------------------------------------------------------------------------
// Remittance schedules
// ---------------------------------------------------------------------------

#[test]
fn test_create_remittance_schedule_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    env.ledger().set(soroban_sdk::testutils::LedgerInfo {
        protocol_version: 20,
        sequence_number: 100,
        timestamp: 1000,
        network_id: [0; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: 100_000,
    });

    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    let schedule_id = client.create_remittance_schedule(&owner, &10000, &3000, &86400);
    assert_eq!(schedule_id, 1);

    let schedule = client.get_remittance_schedule(&schedule_id).unwrap();
    assert_eq!(schedule.amount, 10000);
    assert_eq!(schedule.next_due, 3000);
    assert!(schedule.active);
}

#[test]
fn test_cancel_remittance_schedule() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    env.ledger().set(soroban_sdk::testutils::LedgerInfo {
        protocol_version: 20,
        sequence_number: 100,
        timestamp: 1000,
        network_id: [0; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: 100_000,
    });

    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    let schedule_id = client.create_remittance_schedule(&owner, &10000, &3000, &86400);
    client.cancel_remittance_schedule(&owner, &schedule_id);

    let schedule = client.get_remittance_schedule(&schedule_id).unwrap();
    assert!(!schedule.active);
}

// ---------------------------------------------------------------------------
// TTL extension
// ---------------------------------------------------------------------------

#[test]
fn test_instance_ttl_extended_on_initialize_split() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set(soroban_sdk::testutils::LedgerInfo {
        protocol_version: 20,
        sequence_number: 100,
        timestamp: 1000,
        network_id: [0; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 100,
        min_persistent_entry_ttl: 100,
        max_entry_ttl: 700_000,
    });

    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    let ttl = env.as_contract(&contract_id, || env.storage().instance().get_ttl());
    assert!(ttl >= 518_400, "TTL must be >= INSTANCE_BUMP_AMOUNT after init");
}

// ---------------------------------------------------------------------------
// Helper: set ledger timestamp
// ---------------------------------------------------------------------------

fn set_time(env: &Env, ts: u64) {
    let proto = env.ledger().protocol_version();
    env.ledger().set(soroban_sdk::testutils::LedgerInfo {
        protocol_version: proto,
        sequence_number: 100,
        timestamp: ts,
        network_id: [0; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: 100_000,
    });
}

// ---------------------------------------------------------------------------
// export_snapshot / import_snapshot — basic round-trip (no schedules)
// ---------------------------------------------------------------------------

#[test]
fn test_export_import_snapshot_no_schedules() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    // nonce = 1 after init
    let snapshot = client.export_snapshot(&owner).unwrap();
    assert_eq!(snapshot.version, 1);
    assert_eq!(snapshot.schedules.len(), 0);
    assert_eq!(snapshot.next_rsch, 0);

    // import must succeed and config must be restored
    let ok = client.import_snapshot(&owner, &1, &snapshot);
    assert!(ok);
    let config = client.get_config().unwrap();
    assert_eq!(config.spending_percent, 50);
    assert_eq!(config.savings_percent, 30);
}

// ---------------------------------------------------------------------------
// export_snapshot — includes schedules
// ---------------------------------------------------------------------------

#[test]
fn test_export_snapshot_includes_schedules() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    set_time(&env, 1000);
    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    client.create_remittance_schedule(&owner, &500, &5000, &86400);
    client.create_remittance_schedule(&owner, &1000, &10000, &0);

    let snapshot = client.export_snapshot(&owner).unwrap();
    assert_eq!(snapshot.next_rsch, 2);
    assert_eq!(snapshot.schedules.len(), 2);
    assert_eq!(snapshot.schedules.get(0).unwrap().id, 1);
    assert_eq!(snapshot.schedules.get(1).unwrap().id, 2);
}

// ---------------------------------------------------------------------------
// import_snapshot — owner index is rebuilt (core bug fix)
// ---------------------------------------------------------------------------

#[test]
fn test_import_snapshot_rebuilds_owner_index() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    set_time(&env, 1000);
    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    client.create_remittance_schedule(&owner, &500, &5000, &86400);
    client.create_remittance_schedule(&owner, &1000, &10000, &86400);

    // Export and then import on a fresh contract instance
    let snapshot = client.export_snapshot(&owner).unwrap();

    let env2 = Env::default();
    env2.mock_all_auths();
    set_time(&env2, 1000);
    let cid2 = env2.register_contract(None, RemittanceSplit);
    let c2 = RemittanceSplitClient::new(&env2, &cid2);
    c2.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);

    // nonce = 1 after initialize_split on the fresh contract
    c2.import_snapshot(&owner, &1, &snapshot);

    // Owner-scoped query must return both imported schedules
    let schedules = c2.get_remittance_schedules(&owner);
    assert_eq!(schedules.len(), 2, "owner index must be rebuilt on import");

    // Verify both ids are present
    let mut found_1 = false;
    let mut found_2 = false;
    for s in schedules.iter() {
        if s.id == 1 { found_1 = true; }
        if s.id == 2 { found_2 = true; }
    }
    assert!(found_1, "schedule id=1 must be visible after import");
    assert!(found_2, "schedule id=2 must be visible after import");
}

// ---------------------------------------------------------------------------
// import_snapshot — multi-owner schedules grouped correctly
// ---------------------------------------------------------------------------

#[test]
fn test_import_snapshot_multi_owner_index() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner1 = Address::generate(&env);
    let owner2 = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner1, 0);

    set_time(&env, 1000);
    client.initialize_split(&owner1, &0, &token_id, &50, &30, &15, &5);
    client.create_remittance_schedule(&owner1, &500, &5000, &86400);
    client.create_remittance_schedule(&owner2, &200, &6000, &86400);
    client.create_remittance_schedule(&owner1, &300, &7000, &86400);

    let snapshot = client.export_snapshot(&owner1).unwrap();

    let env2 = Env::default();
    env2.mock_all_auths();
    set_time(&env2, 1000);
    let cid2 = env2.register_contract(None, RemittanceSplit);
    let c2 = RemittanceSplitClient::new(&env2, &cid2);
    c2.initialize_split(&owner1, &0, &token_id, &50, &30, &15, &5);
    c2.import_snapshot(&owner1, &1, &snapshot);

    let s1 = c2.get_remittance_schedules(&owner1);
    let s2 = c2.get_remittance_schedules(&owner2);
    assert_eq!(s1.len(), 2, "owner1 should see exactly 2 schedules");
    assert_eq!(s2.len(), 1, "owner2 should see exactly 1 schedule");

    // Verify no cross-owner leakage
    for s in s1.iter() {
        assert_eq!(s.owner, owner1, "owner1 index must not contain owner2 schedules");
    }
    for s in s2.iter() {
        assert_eq!(s.owner, owner2, "owner2 index must not contain owner1 schedules");
    }
}

// ---------------------------------------------------------------------------
// import_snapshot — NEXT_RSCH is advanced
// ---------------------------------------------------------------------------

#[test]
fn test_import_snapshot_advances_next_rsch() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    set_time(&env, 1000);
    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    client.create_remittance_schedule(&owner, &500, &5000, &86400);
    client.create_remittance_schedule(&owner, &500, &6000, &86400);
    // next_rsch == 2 at this point

    let snapshot = client.export_snapshot(&owner).unwrap();
    assert_eq!(snapshot.next_rsch, 2);

    let env2 = Env::default();
    env2.mock_all_auths();
    set_time(&env2, 1000);
    let cid2 = env2.register_contract(None, RemittanceSplit);
    let c2 = RemittanceSplitClient::new(&env2, &cid2);
    c2.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    c2.import_snapshot(&owner, &1, &snapshot);

    // A new schedule created after import must get id 3, not 1
    let new_id = c2.create_remittance_schedule(&owner, &100, &9000, &86400);
    assert_eq!(new_id, 3, "NEXT_RSCH must have been advanced past max imported id");
}

// ---------------------------------------------------------------------------
// import_snapshot — last_executed and missed_count preserved
// ---------------------------------------------------------------------------

#[test]
fn test_import_snapshot_preserves_last_executed_and_missed_count() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    set_time(&env, 1000);
    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    client.create_remittance_schedule(&owner, &500, &2000, &1000);

    // Execute the schedule so last_executed and missed_count are set
    set_time(&env, 4500); // 2000 + 2*1000 = 4000 passed, 1 missed
    client.execute_due_remittance_schedules();

    let s = client.get_remittance_schedule(&1).unwrap();
    assert!(s.last_executed.is_some(), "last_executed must be set after execution");
    assert!(s.missed_count >= 1, "missed_count must be at least 1");

    let snapshot = client.export_snapshot(&owner).unwrap();

    let env2 = Env::default();
    env2.mock_all_auths();
    set_time(&env2, 1000);
    let cid2 = env2.register_contract(None, RemittanceSplit);
    let c2 = RemittanceSplitClient::new(&env2, &cid2);
    c2.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    c2.import_snapshot(&owner, &1, &snapshot);

    let imported = c2.get_remittance_schedule(&1).unwrap();
    assert_eq!(imported.last_executed, s.last_executed, "last_executed must be preserved");
    assert_eq!(imported.missed_count, s.missed_count, "missed_count must be preserved");
}

// ---------------------------------------------------------------------------
// import_snapshot — re-import clears stale owner index
// ---------------------------------------------------------------------------

#[test]
fn test_reimport_clears_stale_owner_index() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    set_time(&env, 1000);
    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    client.create_remittance_schedule(&owner, &500, &5000, &86400);
    client.create_remittance_schedule(&owner, &500, &6000, &86400);

    // Take snapshot1 (2 schedules), then add a 3rd
    let snap1 = client.export_snapshot(&owner).unwrap();
    client.create_remittance_schedule(&owner, &500, &7000, &86400);

    // Take snapshot2 (3 schedules) then re-import snapshot1
    let _snap2 = client.export_snapshot(&owner).unwrap();

    // nonce incremented by: init(1) + 3 creates (no nonce) + export×2 (no nonce) = 1
    // snapshot import uses nonce 1 for first import_snapshot call
    client.import_snapshot(&owner, &1, &snap1);

    // After re-importing snapshot1 the contract should have exactly 2 schedules
    let schedules = client.get_remittance_schedules(&owner);
    assert_eq!(
        schedules.len(),
        2,
        "re-import must replace stale index; should have 2 schedules not 3"
    );
}

// ---------------------------------------------------------------------------
// import_snapshot — empty schedule list
// ---------------------------------------------------------------------------

#[test]
fn test_import_snapshot_empty_schedules() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    set_time(&env, 1000);
    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    // No schedules created — snapshot has empty list
    let snapshot = client.export_snapshot(&owner).unwrap();
    assert_eq!(snapshot.schedules.len(), 0);

    client.import_snapshot(&owner, &1, &snapshot);

    let schedules = client.get_remittance_schedules(&owner);
    assert_eq!(schedules.len(), 0, "empty snapshot must result in empty schedule list");
}

// ---------------------------------------------------------------------------
// import_snapshot — checksum mismatch rejected
// ---------------------------------------------------------------------------

#[test]
fn test_import_snapshot_bad_checksum_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    set_time(&env, 1000);
    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    let snapshot = client.export_snapshot(&owner).unwrap();

    let bad_snapshot = ExportSnapshot {
        checksum: snapshot.checksum.wrapping_add(1),
        ..snapshot
    };
    let result = client.try_import_snapshot(&owner, &1, &bad_snapshot);
    assert_eq!(result, Err(Ok(RemittanceSplitError::ChecksumMismatch)));
}

// ---------------------------------------------------------------------------
// get_schedules_paginated — basic pagination
// ---------------------------------------------------------------------------

#[test]
fn test_get_schedules_paginated_first_page() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    set_time(&env, 1000);
    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    for due in &[5000u64, 6000, 7000, 8000, 9000] {
        client.create_remittance_schedule(&owner, &100, due, &86400);
    }

    let page = client.get_schedules_paginated(&owner, &0, &3);
    assert_eq!(page.count, 3);
    assert_eq!(page.items.len(), 3);
    assert_ne!(page.next_cursor, 0, "there should be a next page");
}

#[test]
fn test_get_schedules_paginated_last_page() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    set_time(&env, 1000);
    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    for due in &[5000u64, 6000, 7000, 8000, 9000] {
        client.create_remittance_schedule(&owner, &100, due, &86400);
    }

    // page 1: ids 1,2,3  → next_cursor = 4
    let page1 = client.get_schedules_paginated(&owner, &0, &3);
    assert_eq!(page1.next_cursor, 4);

    // page 2: ids 4,5 → next_cursor = 0
    let page2 = client.get_schedules_paginated(&owner, &page1.next_cursor, &3);
    assert_eq!(page2.count, 2);
    assert_eq!(page2.next_cursor, 0, "last page must have next_cursor = 0");
}

#[test]
fn test_get_schedules_paginated_owner_isolation() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner1 = Address::generate(&env);
    let owner2 = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner1, 0);

    set_time(&env, 1000);
    client.initialize_split(&owner1, &0, &token_id, &50, &30, &15, &5);
    client.create_remittance_schedule(&owner1, &100, &5000, &86400);
    client.create_remittance_schedule(&owner2, &200, &6000, &86400);
    client.create_remittance_schedule(&owner1, &300, &7000, &86400);

    let page = client.get_schedules_paginated(&owner1, &0, &10);
    assert_eq!(page.count, 2, "paginated results must be owner-isolated");
    for s in page.items.iter() {
        assert_eq!(s.owner, owner1);
    }
}

// ---------------------------------------------------------------------------
// get_schedules_paginated — after import, imported schedules are visible
// ---------------------------------------------------------------------------

#[test]
fn test_paginated_query_sees_imported_schedules() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    set_time(&env, 1000);
    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    client.create_remittance_schedule(&owner, &100, &5000, &86400);
    client.create_remittance_schedule(&owner, &200, &6000, &86400);
    client.create_remittance_schedule(&owner, &300, &7000, &86400);

    let snapshot = client.export_snapshot(&owner).unwrap();

    // fresh contract
    let env2 = Env::default();
    env2.mock_all_auths();
    set_time(&env2, 1000);
    let cid2 = env2.register_contract(None, RemittanceSplit);
    let c2 = RemittanceSplitClient::new(&env2, &cid2);
    c2.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    c2.import_snapshot(&owner, &1, &snapshot);

    // All 3 imported schedules must be visible through the paginated reader
    let page = c2.get_schedules_paginated(&owner, &0, &10);
    assert_eq!(page.count, 3, "all imported schedules must be visible via pagination");
    assert_eq!(page.next_cursor, 0, "single page must have no next cursor");
}

// ---------------------------------------------------------------------------
// execute_due_remittance_schedules
// ---------------------------------------------------------------------------

#[test]
fn test_execute_due_remittance_schedules_basic() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    set_time(&env, 1000);
    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    client.create_remittance_schedule(&owner, &500, &2000, &86400);

    // Before due date — nothing should execute
    set_time(&env, 1500);
    let executed = client.execute_due_remittance_schedules();
    assert_eq!(executed.len(), 0);

    // At/after due date
    set_time(&env, 2000);
    let executed = client.execute_due_remittance_schedules();
    assert_eq!(executed.len(), 1);
    assert_eq!(executed.get(0).unwrap(), 1);

    let s = client.get_remittance_schedule(&1).unwrap();
    assert!(s.last_executed.is_some());
    assert_eq!(s.last_executed.unwrap(), 2000);
}

#[test]
fn test_execute_due_one_shot_becomes_inactive() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    set_time(&env, 1000);
    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    // interval=0 → one-shot
    client.create_remittance_schedule(&owner, &500, &2000, &0);

    set_time(&env, 2001);
    client.execute_due_remittance_schedules();

    let s = client.get_remittance_schedule(&1).unwrap();
    assert!(!s.active, "one-shot schedule must become inactive after execution");
}

#[test]
fn test_execute_due_recurring_advances_next_due() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    set_time(&env, 1000);
    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    client.create_remittance_schedule(&owner, &500, &2000, &1000);

    set_time(&env, 2000);
    client.execute_due_remittance_schedules();

    let s = client.get_remittance_schedule(&1).unwrap();
    assert!(s.active, "recurring schedule must remain active");
    assert_eq!(s.next_due, 3000, "next_due must advance by interval");
    assert_eq!(s.missed_count, 0, "no missed cycles");
}

#[test]
fn test_execute_due_missed_cycles_counted() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    set_time(&env, 1000);
    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    // interval=1000, next_due=2000; at t=4500 → 2 intervals missed (3000, 4000)
    client.create_remittance_schedule(&owner, &500, &2000, &1000);

    set_time(&env, 4500);
    client.execute_due_remittance_schedules();

    let s = client.get_remittance_schedule(&1).unwrap();
    assert_eq!(s.missed_count, 2, "two missed cycles must be counted");
    assert_eq!(s.next_due, 5000, "next_due must be advanced past current time");
}

// ---------------------------------------------------------------------------
// execute_due — imported schedules participate in due sweep
// ---------------------------------------------------------------------------

#[test]
fn test_imported_schedules_participate_in_due_sweep() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner, 0);

    set_time(&env, 1000);
    client.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    client.create_remittance_schedule(&owner, &500, &2000, &86400);
    client.create_remittance_schedule(&owner, &300, &3000, &86400);

    let snapshot = client.export_snapshot(&owner).unwrap();

    // Import onto a fresh contract, advance time past first schedule's due date
    let env2 = Env::default();
    env2.mock_all_auths();
    set_time(&env2, 1000);
    let cid2 = env2.register_contract(None, RemittanceSplit);
    let c2 = RemittanceSplitClient::new(&env2, &cid2);
    c2.initialize_split(&owner, &0, &token_id, &50, &30, &15, &5);
    c2.import_snapshot(&owner, &1, &snapshot);

    // Advance time so that schedule id=1 (due=2000) is due but id=2 (due=3000) is not
    set_time(&env2, 2500);
    let executed = c2.execute_due_remittance_schedules();
    assert_eq!(executed.len(), 1, "exactly one schedule must be executed");
    assert_eq!(executed.get(0).unwrap(), 1, "schedule id=1 must be executed");
}

// ---------------------------------------------------------------------------
// Full export→import→query round-trip (multi-owner, multiple assertions)
// ---------------------------------------------------------------------------

#[test]
fn test_full_export_import_round_trip() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, RemittanceSplit);
    let client = RemittanceSplitClient::new(&env, &contract_id);
    let owner_a = Address::generate(&env);
    let owner_b = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = setup_token(&env, &token_admin, &owner_a, 0);

    set_time(&env, 1000);
    client.initialize_split(&owner_a, &0, &token_id, &50, &30, &15, &5);
    client.create_remittance_schedule(&owner_a, &100, &5000, &86400);
    client.create_remittance_schedule(&owner_b, &200, &6000, &86400);
    client.create_remittance_schedule(&owner_a, &300, &7000, &86400);

    let snapshot = client.export_snapshot(&owner_a).unwrap();

    // --- Import to a new contract ---
    let env2 = Env::default();
    env2.mock_all_auths();
    set_time(&env2, 1000);
    let cid2 = env2.register_contract(None, RemittanceSplit);
    let c2 = RemittanceSplitClient::new(&env2, &cid2);
    c2.initialize_split(&owner_a, &0, &token_id, &50, &30, &15, &5);
    c2.import_snapshot(&owner_a, &1, &snapshot);

    // 1. owner_a sees all her schedules via get_remittance_schedules
    let sa = c2.get_remittance_schedules(&owner_a);
    assert_eq!(sa.len(), 2);

    // 2. owner_b sees her schedule
    let sb = c2.get_remittance_schedules(&owner_b);
    assert_eq!(sb.len(), 1);

    // 3. Paginated reader returns the same results
    let page_a = c2.get_schedules_paginated(&owner_a, &0, &10);
    assert_eq!(page_a.count, 2);
    let page_b = c2.get_schedules_paginated(&owner_b, &0, &10);
    assert_eq!(page_b.count, 1);

    // 4. NEXT_RSCH is advanced past 3 so a new schedule gets id 4
    let new_id = c2.create_remittance_schedule(&owner_a, &50, &9000, &86400);
    assert_eq!(new_id, 4);

    // 5. execute_due runs and can see all schedules
    set_time(&env2, 7000);
    let executed = c2.execute_due_remittance_schedules();
    // ids 1 (due=5000) and 2 (due=6000) and 3 (due=7000) are all due
    assert_eq!(executed.len(), 3);
}
