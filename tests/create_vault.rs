#![cfg(test)]

extern crate std;

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    Address, BytesN, Env,
};

use disciplr_vault::{
    DisciplrVault, DisciplrVaultClient, VaultStatus, MAX_AMOUNT, MAX_VAULT_DURATION, MIN_AMOUNT,
};

fn setup() -> (
    Env,
    DisciplrVaultClient<'static>,
    Address,
    StellarAssetClient<'static>,
) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(DisciplrVault, ());
    let client = DisciplrVaultClient::new(&env, &contract_id);

    let usdc_admin = Address::generate(&env);
    let usdc_token = env.register_stellar_asset_contract_v2(usdc_admin.clone());
    let usdc_addr = usdc_token.address();
    let usdc_asset = StellarAssetClient::new(&env, &usdc_addr);

    (env, client, usdc_addr, usdc_asset)
}

#[test]
fn test_create_vault_valid_boundary_values() {
    let (env, client, usdc, usdc_asset) = setup();

    let creator = Address::generate(&env);
    let now = 1_725_000_000u64;
    env.ledger().set_timestamp(now);

    usdc_asset.mint(&creator, &MIN_AMOUNT);

    let success = Address::generate(&env);
    let failure = Address::generate(&env);
    let milestone = BytesN::from_array(&env, &[0u8; 32]);

    let vault_id = client.create_vault(
        &usdc,
        &creator,
        &MIN_AMOUNT,
        &now,
        &(now + MAX_VAULT_DURATION),
        &milestone,
        &None,
        &success,
        &failure,
    );

    assert_eq!(vault_id, 0u32);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_amount_below_minimum() {
    let (env, client, usdc, _usdc_asset) = setup();

    let creator = Address::generate(&env);
    let now = env.ledger().timestamp();

    client.create_vault(
        &usdc,
        &creator,
        &(MIN_AMOUNT - 1),
        &now,
        &(now + 86_400),
        &BytesN::from_array(&env, &[0u8; 32]),
        &None,
        &Address::generate(&env),
        &Address::generate(&env),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_amount_above_maximum() {
    let (env, client, usdc, _usdc_asset) = setup();

    let creator = Address::generate(&env);
    let now = env.ledger().timestamp();

    client.create_vault(
        &usdc,
        &creator,
        &(MAX_AMOUNT + 1),
        &now,
        &(now + 86_400),
        &BytesN::from_array(&env, &[0u8; 32]),
        &None,
        &Address::generate(&env),
        &Address::generate(&env),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_duration_exceeds_max() {
    let (env, client, usdc, _usdc_asset) = setup();

    let creator = Address::generate(&env);
    let now = env.ledger().timestamp();

    client.create_vault(
        &usdc,
        &creator,
        &MIN_AMOUNT,
        &now,
        &(now + MAX_VAULT_DURATION + 1),
        &BytesN::from_array(&env, &[0u8; 32]),
        &None,
        &Address::generate(&env),
        &Address::generate(&env),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_start_timestamp_in_past() {
    let (env, client, usdc, _usdc_asset) = setup();

    let creator = Address::generate(&env);
    let now = 1_725_000_000u64;
    env.ledger().set_timestamp(now);

    client.create_vault(
        &usdc,
        &creator,
        &MIN_AMOUNT,
        &(now - 3_600),
        &(now + 86_400),
        &BytesN::from_array(&env, &[0u8; 32]),
        &None,
        &Address::generate(&env),
        &Address::generate(&env),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_end_before_or_equal_start() {
    let (env, client, usdc, _usdc_asset) = setup();

    let creator = Address::generate(&env);
    let now = 1_725_000_000u64;
    env.ledger().set_timestamp(now);

    client.create_vault(
        &usdc,
        &creator,
        &MIN_AMOUNT,
        &(now + 200),
        &(now + 100),
        &BytesN::from_array(&env, &[0u8; 32]),
        &None,
        &Address::generate(&env),
        &Address::generate(&env),
    );
}

#[test]
fn test_amount_exactly_max_allowed() {
    let (env, client, usdc, usdc_asset) = setup();

    let creator = Address::generate(&env);
    let now = 1_725_000_000u64;
    env.ledger().set_timestamp(now);

    usdc_asset.mint(&creator, &MAX_AMOUNT);

    let vault_id = client.create_vault(
        &usdc,
        &creator,
        &MAX_AMOUNT,
        &now,
        &(now + 86_400),
        &BytesN::from_array(&env, &[0u8; 32]),
        &None,
        &Address::generate(&env),
        &Address::generate(&env),
    );

    assert_eq!(vault_id, 0u32);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_amount_zero() {
    let (env, client, usdc, _usdc_asset) = setup();

    let creator = Address::generate(&env);
    let now = env.ledger().timestamp();

    client.create_vault(
        &usdc,
        &creator,
        &0_i128,
        &now,
        &(now + 86_400),
        &BytesN::from_array(&env, &[0u8; 32]),
        &None,
        &Address::generate(&env),
        &Address::generate(&env),
    );
}

#[test]
fn test_minimum_valid_duration() {
    let (env, client, usdc, usdc_asset) = setup();

    let creator = Address::generate(&env);
    let now = 1_725_000_000u64;
    env.ledger().set_timestamp(now);

    usdc_asset.mint(&creator, &MIN_AMOUNT);

    let vault_id = client.create_vault(
        &usdc,
        &creator,
        &MIN_AMOUNT,
        &now,
        &(now + 1),
        &BytesN::from_array(&env, &[0u8; 32]),
        &None,
        &Address::generate(&env),
        &Address::generate(&env),
    );

    assert_eq!(vault_id, 0u32);
}

#[test]
fn test_valid_zero_verifier_and_normal_duration() {
    let (env, client, usdc, usdc_asset) = setup();

    let creator = Address::generate(&env);
    let now = env.ledger().timestamp();

    usdc_asset.mint(&creator, &5_000_000_000_i128);

    client.create_vault(
        &usdc,
        &creator,
        &5_000_000_000_i128,
        &now,
        &(now + 7 * 24 * 60 * 60),
        &BytesN::from_array(&env, &[1u8; 32]),
        &None,
        &Address::generate(&env),
        &Address::generate(&env),
    );
}

#[test]
fn test_get_vault_state_never_created_id_returns_none() {
    let (_env, client, _usdc, _usdc_asset) = setup();

    assert_eq!(client.vault_count(), 0u32);
    assert!(client.get_vault_state(&0u32).is_none());
    assert!(client.get_vault_state(&42u32).is_none());
}

#[test]
fn test_get_vault_state_cancelled_vault_remains_readable() {
    let (env, client, usdc, usdc_asset) = setup();

    let creator = Address::generate(&env);
    let now = 1_725_000_000u64;
    env.ledger().set_timestamp(now);
    usdc_asset.mint(&creator, &(MIN_AMOUNT * 2));

    let vault_id = client.create_vault(
        &usdc,
        &creator,
        &MIN_AMOUNT,
        &now,
        &(now + 86_400),
        &BytesN::from_array(&env, &[9u8; 32]),
        &None,
        &Address::generate(&env),
        &Address::generate(&env),
    );

    assert_eq!(client.vault_count(), 1u32);
    client.cancel_vault(&vault_id, &usdc);

    let vault = client.get_vault_state(&vault_id).unwrap();
    assert_eq!(vault.status, VaultStatus::Cancelled);
    assert!(client.get_vault_state(&1u32).is_none());
}

// ---------------------------------------------------------------------------
// Issue #145 – Concurrent vault isolation
// ---------------------------------------------------------------------------

/// Vault A and Vault B are completely independent: creating, cancelling,
/// or completing one does not alter the other's state or balances.
#[test]
fn test_two_vaults_are_independent() {
    let (env, client, usdc, usdc_asset) = setup();

    let creator_a = Address::generate(&env);
    let creator_b = Address::generate(&env);
    let success_a = Address::generate(&env);
    let success_b = Address::generate(&env);
    let failure   = Address::generate(&env);
    let now = 1_725_000_000u64;
    env.ledger().set_timestamp(now);

    usdc_asset.mint(&creator_a, &(MIN_AMOUNT * 2));
    usdc_asset.mint(&creator_b, &(MIN_AMOUNT * 2));

    let vault_a = client.create_vault(
        &usdc, &creator_a, &MIN_AMOUNT,
        &now, &(now + 86_400),
        &BytesN::from_array(&env, &[0xAAu8; 32]),
        &None, &success_a, &failure,
    );
    let vault_b = client.create_vault(
        &usdc, &creator_b, &MIN_AMOUNT,
        &now, &(now + 86_400),
        &BytesN::from_array(&env, &[0xBBu8; 32]),
        &None, &success_b, &failure,
    );

    assert_ne!(vault_a, vault_b, "vaults must have distinct IDs");

    // Cancel vault A — vault B must remain Active with its original amount.
    client.cancel_vault(&vault_a, &usdc);

    let state_a = client.get_vault_state(&vault_a).unwrap();
    let state_b = client.get_vault_state(&vault_b).unwrap();

    assert_eq!(state_a.status, VaultStatus::Cancelled);
    assert_eq!(state_b.status, VaultStatus::Active,
               "cancelling vault A must not affect vault B");
    assert_eq!(state_b.amount, MIN_AMOUNT,
               "vault B amount must be unchanged after vault A cancelled");
}

/// Operations on vault B do not change the creator balance of vault A.
#[test]
fn test_vault_b_cancel_does_not_affect_vault_a_balance() {
    let (env, client, usdc, usdc_asset) = setup();

    let creator_a = Address::generate(&env);
    let creator_b = Address::generate(&env);
    let dest = Address::generate(&env);
    let now = 1_725_000_000u64;
    env.ledger().set_timestamp(now);

    usdc_asset.mint(&creator_a, &(MIN_AMOUNT * 3));
    usdc_asset.mint(&creator_b, &(MIN_AMOUNT * 3));

    let vault_a = client.create_vault(
        &usdc, &creator_a, &(MIN_AMOUNT * 2),
        &now, &(now + 86_400),
        &BytesN::from_array(&env, &[1u8; 32]),
        &None, &dest, &dest,
    );
    let vault_b = client.create_vault(
        &usdc, &creator_b, &MIN_AMOUNT,
        &now, &(now + 86_400),
        &BytesN::from_array(&env, &[2u8; 32]),
        &None, &dest, &dest,
    );

    use soroban_sdk::token::Client as TokenClient;
    let token_client = TokenClient::new(&env, &usdc);

    let balance_a_before = token_client.balance(&creator_a);

    // Cancel vault B.
    client.cancel_vault(&vault_b, &usdc);

    let balance_a_after = token_client.balance(&creator_a);
    assert_eq!(balance_a_before, balance_a_after,
               "creator_a balance must be unaffected by vault_b cancellation");

    // Vault A is still active.
    let state_a = client.get_vault_state(&vault_a).unwrap();
    assert_eq!(state_a.status, VaultStatus::Active);
    let _ = vault_b; // silence unused warning
}

/// Storage isolation: each vault ID maps to its own record; no slot aliasing.
#[test]
fn test_vault_storage_slots_are_isolated() {
    let (env, client, usdc, usdc_asset) = setup();

    let creator = Address::generate(&env);
    let dest_a  = Address::generate(&env);
    let dest_b  = Address::generate(&env);
    let now = 1_725_000_000u64;
    env.ledger().set_timestamp(now);

    usdc_asset.mint(&creator, &(MIN_AMOUNT * 10));

    let hash_a = BytesN::from_array(&env, &[0xCAu8; 32]);
    let hash_b = BytesN::from_array(&env, &[0xFEu8; 32]);

    let vault_a = client.create_vault(
        &usdc, &creator, &(MIN_AMOUNT * 3),
        &now, &(now + 3_600),
        &hash_a, &None, &dest_a, &dest_a,
    );
    let vault_b = client.create_vault(
        &usdc, &creator, &(MIN_AMOUNT * 5),
        &now, &(now + 7_200),
        &hash_b, &None, &dest_b, &dest_b,
    );

    let state_a = client.get_vault_state(&vault_a).unwrap();
    let state_b = client.get_vault_state(&vault_b).unwrap();

    // Amounts must not bleed across storage slots.
    assert_eq!(state_a.amount, MIN_AMOUNT * 3,
               "vault A amount must not be affected by vault B creation");
    assert_eq!(state_b.amount, MIN_AMOUNT * 5,
               "vault B amount must not be affected by vault A creation");

    // Milestone hash must be per-vault.
    assert_eq!(state_a.milestone_hash, hash_a);
    assert_eq!(state_b.milestone_hash, hash_b);

    // Success destinations must be independent.
    assert_eq!(state_a.success_destination, dest_a);
    assert_eq!(state_b.success_destination, dest_b);
}

/// validate_milestone on vault A does not mark vault B as validated.
#[test]
fn test_milestone_validation_isolated_across_vaults() {
    let (env, client, usdc, usdc_asset) = setup();

    let creator = Address::generate(&env);
    let dest    = Address::generate(&env);
    let now = 1_725_000_000u64;
    env.ledger().set_timestamp(now);

    usdc_asset.mint(&creator, &(MIN_AMOUNT * 4));

    let vault_a = client.create_vault(
        &usdc, &creator, &MIN_AMOUNT,
        &now, &(now + 86_400),
        &BytesN::from_array(&env, &[0x11u8; 32]),
        &None, &dest, &dest,
    );
    let vault_b = client.create_vault(
        &usdc, &creator, &MIN_AMOUNT,
        &now, &(now + 86_400),
        &BytesN::from_array(&env, &[0x22u8; 32]),
        &None, &dest, &dest,
    );

    // Validate only vault A.
    client.validate_milestone(&vault_a);

    let state_a = client.get_vault_state(&vault_a).unwrap();
    let state_b = client.get_vault_state(&vault_b).unwrap();

    assert!(state_a.milestone_validated,
            "vault A must be marked validated");
    assert!(!state_b.milestone_validated,
            "vault B must NOT be marked validated — isolation broken");
    let _ = vault_b;
}
