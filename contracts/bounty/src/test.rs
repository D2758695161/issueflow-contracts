#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    Address, Env, IntoVal, Symbol,
};

#[test]
fn test_create_bounty_with_deadline() {
    let env = Env::default();
    let contract_id = env.register_contract(None, BountyContract);
    let client = BountyContractClient::new(&env, &contract_id);

    let maintainer = Address::generate(&env);
    let token = Address::generate(&env);
    let amount = 1000i128;
    let deadline = 9999999999u64;

    env.mock_all_auths();

    let bounty_id = client.create_bounty(&maintainer, &amount, &token, &deadline);

    assert_eq!(bounty_id, 1);

    let bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.id, 1);
    assert_eq!(bounty.maintainer, maintainer);
    assert_eq!(bounty.amount, amount);
    assert_eq!(bounty.token, token);
    assert_eq!(bounty.is_claimed, false);
    assert_eq!(bounty.is_cancelled, false);
    assert_eq!(bounty.deadline, deadline);
}

#[test]
fn test_approve_then_claim() {
    let env = Env::default();
    let contract_id = env.register_contract(None, BountyContract);
    let client = BountyContractClient::new(&env, &contract_id);

    let maintainer = Address::generate(&env);
    let claimant = Address::generate(&env);
    let token = Address::generate(&env);
    let amount = 1000i128;
    let deadline = 9999999999u64;

    env.mock_all_auths();

    let bounty_id = client.create_bounty(&maintainer, &amount, &token, &deadline);

    // Maintainer approves the claimant (Issue #6)
    client.approve_claimant(&bounty_id, &claimant);

    // Claimant can now claim
    client.claim_bounty(&bounty_id, &claimant);

    let bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.is_claimed, true);
    assert_eq!(bounty.approved_claimant, Some(claimant.clone()));
}

#[test]
#[should_panic(expected = "Claimant not approved by maintainer")]
fn test_cannot_claim_without_approval() {
    let env = Env::default();
    let contract_id = env.register_contract(None, BountyContract);
    let client = BountyContractClient::new(&env, &contract_id);

    let maintainer = Address::generate(&env);
    let claimant = Address::generate(&env);
    let token = Address::generate(&env);
    let amount = 1000i128;
    let deadline = 9999999999u64;

    env.mock_all_auths();

    let bounty_id = client.create_bounty(&maintainer, &amount, &token, &deadline);

    // Try to claim without approval — should panic
    client.claim_bounty(&bounty_id, &claimant);
}

#[test]
#[should_panic(expected = "Claimant not approved by maintainer")]
fn test_cannot_claim_wrong_claimant() {
    let env = Env::default();
    let contract_id = env.register_contract(None, BountyContract);
    let client = BountyContractClient::new(&env, &contract_id);

    let maintainer = Address::generate(&env);
    let approved_claimant = Address::generate(&env);
    let wrong_claimant = Address::generate(&env);
    let token = Address::generate(&env);
    let amount = 1000i128;
    let deadline = 9999999999u64;

    env.mock_all_auths();

    let bounty_id = client.create_bounty(&maintainer, &amount, &token, &deadline);

    // Maintainer approves one claimant
    client.approve_claimant(&bounty_id, &approved_claimant);

    // Different claimant tries to claim — should panic
    client.claim_bounty(&bounty_id, &wrong_claimant);
}

#[test]
fn test_cancel_bounty() {
    let env = Env::default();
    let contract_id = env.register_contract(None, BountyContract);
    let client = BountyContractClient::new(&env, &contract_id);

    let maintainer = Address::generate(&env);
    let token = Address::generate(&env);
    let amount = 1000i128;
    let deadline = 9999999999u64;

    env.mock_all_auths();

    let bounty_id = client.create_bounty(&maintainer, &amount, &token, &deadline);
    client.cancel_bounty(&bounty_id);

    let bounty = client.get_bounty(&bounty_id);
    assert_eq!(bounty.is_claimed, false);
    assert_eq!(bounty.is_cancelled, true);
}

#[test]
#[should_panic(expected = "Cannot cancel claimed bounty")]
fn test_cannot_cancel_claimed_bounty() {
    let env = Env::default();
    let contract_id = env.register_contract(None, BountyContract);
    let client = BountyContractClient::new(&env, &contract_id);

    let maintainer = Address::generate(&env);
    let claimant = Address::generate(&env);
    let token = Address::generate(&env);
    let amount = 1000i128;
    let deadline = 9999999999u64;

    env.mock_all_auths();

    let bounty_id = client.create_bounty(&maintainer, &amount, &token, &deadline);
    client.approve_claimant(&bounty_id, &claimant);
    client.claim_bounty(&bounty_id, &claimant);
    client.cancel_bounty(&bounty_id);
}

#[test]
#[should_panic(expected = "Bounty already cancelled")]
fn test_cannot_cancel_twice() {
    let env = Env::default();
    let contract_id = env.register_contract(None, BountyContract);
    let client = BountyContractClient::new(&env, &contract_id);

    let maintainer = Address::generate(&env);
    let token = Address::generate(&env);
    let amount = 1000i128;
    let deadline = 9999999999u64;

    env.mock_all_auths();

    let bounty_id = client.create_bounty(&maintainer, &amount, &token, &deadline);
    client.cancel_bounty(&bounty_id);
    client.cancel_bounty(&bounty_id);
}

#[test]
#[should_panic(expected = "Bounty is cancelled")]
fn test_cannot_claim_cancelled_bounty() {
    let env = Env::default();
    let contract_id = env.register_contract(None, BountyContract);
    let client = BountyContractClient::new(&env, &contract_id);

    let maintainer = Address::generate(&env);
    let claimant = Address::generate(&env);
    let token = Address::generate(&env);
    let amount = 1000i128;
    let deadline = 9999999999u64;

    env.mock_all_auths();

    let bounty_id = client.create_bounty(&maintainer, &amount, &token, &deadline);
    client.cancel_bounty(&bounty_id);
    client.claim_bounty(&bounty_id, &claimant);
}
