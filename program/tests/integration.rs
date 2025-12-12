#![cfg(test)]

pub mod utils;
use utils::*;

use usdf_swap_api::prelude::*;
use solana_sdk::{signer::Signer, transaction::Transaction};

fn as_token(val: u64, decimals: u8) -> u64 {
    val.checked_mul(10u64.pow(decimals as u32))
        .expect("Overflow in as_token")
}

#[test]
fn test_initialize_pool() {
    let mut svm = setup_svm();

    let authority = create_payer(&mut svm);
    let authority_pk = authority.pubkey();

    let usdf_decimals = 6;
    let usdc_decimals = 6;

    // Create two stablecoin mints
    let usdf = create_mint(&mut svm, &authority, &authority_pk, usdf_decimals);
    let usdc = create_mint(&mut svm, &authority, &authority_pk, usdc_decimals);

    let pool_name = "usdf-usdc";
    let (pool_pda, pool_bump) = find_pool_pda(&authority_pk, pool_name, &usdf, &usdc);
    let (usdf_vault_pda, usdf_vault_bump) = find_vault_pda(&pool_pda, &usdf);
    let (other_vault_pda, other_vault_bump) = find_vault_pda(&pool_pda, &usdc);

    // Initialize the pool
    let blockhash = svm.latest_blockhash();
    let ix = build_initialize_ix(authority_pk, pool_name, usdf, usdc);
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&authority_pk), &[&authority], blockhash);
    let res = send_tx(&mut svm, tx);
    assert!(res.is_ok());

    // Verify pool state
    let account = svm.get_account(&pool_pda).unwrap();
    let pool = Pool::unpack(&account.data).unwrap();

    assert_eq!(pool.authority, authority_pk);
    assert_eq!(from_name(&pool.name), pool_name);
    assert_eq!(pool.usdf_mint, usdf);
    assert_eq!(pool.other_mint, usdc);
    assert_eq!(pool.usdf_vault, usdf_vault_pda);
    assert_eq!(pool.other_vault, other_vault_pda);
    assert_eq!(pool.bump, pool_bump);
    assert_eq!(pool.usdf_vault_bump, usdf_vault_bump);
    assert_eq!(pool.other_vault_bump, other_vault_bump);
    assert_eq!(pool.usdf_decimals, usdf_decimals);
    assert_eq!(pool.other_decimals, usdc_decimals);

    // Verify vaults are empty
    assert_eq!(get_ata_balance(&svm, &usdf_vault_pda), 0);
    assert_eq!(get_ata_balance(&svm, &other_vault_pda), 0);
}

#[test]
fn test_swap_usdf_to_other() {
    let mut svm = setup_svm();

    let authority = create_payer(&mut svm);
    let authority_pk = authority.pubkey();

    let usdf_decimals = 6;
    let usdc_decimals = 6;

    let usdf = create_mint(&mut svm, &authority, &authority_pk, usdf_decimals);
    let usdc = create_mint(&mut svm, &authority, &authority_pk, usdc_decimals);

    let pool_name = "usdf-usdc";
    let (pool_pda, _) = find_pool_pda(&authority_pk, pool_name, &usdf, &usdc);
    let (usdf_vault_pda, _) = find_vault_pda(&pool_pda, &usdf);
    let (other_vault_pda, _) = find_vault_pda(&pool_pda, &usdc);

    // Initialize the pool
    let blockhash = svm.latest_blockhash();
    let ix = build_initialize_ix(authority_pk, pool_name, usdf, usdc);
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&authority_pk), &[&authority], blockhash);
    let res = send_tx(&mut svm, tx);
    assert!(res.is_ok());

    // Fund other_vault with USDC so swaps can happen
    let other_vault_funding = as_token(10_000, usdc_decimals);
    let res = mint_to(&mut svm, &authority, &usdc, &authority, &other_vault_pda, other_vault_funding);
    assert!(res.is_ok());

    // Create a user
    let user = create_payer(&mut svm);
    let user_pk = user.pubkey();

    // Create user token accounts
    let user_usdf_ata = create_ata(&mut svm, &authority, &usdf, &user_pk);
    let user_usdc_ata = create_ata(&mut svm, &authority, &usdc, &user_pk);

    // Mint USDF to user
    let user_usdf_amount = as_token(1000, usdf_decimals);
    let res = mint_to(&mut svm, &authority, &usdf, &authority, &user_usdf_ata, user_usdf_amount);
    assert!(res.is_ok());

    assert_eq!(get_ata_balance(&svm, &user_usdf_ata), user_usdf_amount);
    assert_eq!(get_ata_balance(&svm, &user_usdc_ata), 0);

    // Swap USDF for USDC (usdf_to_other)
    let swap_amount = as_token(500, usdf_decimals);
    let blockhash = svm.latest_blockhash();
    let ix = build_swap_ix(
        user_pk,
        pool_pda,
        usdf_vault_pda,
        other_vault_pda,
        user_usdf_ata,
        user_usdc_ata,
        swap_amount,
        true, // usdf_to_other
    );
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&user_pk), &[&user], blockhash);
    let res = send_tx(&mut svm, tx);
    assert!(res.is_ok());

    // Verify balances after swap
    assert_eq!(get_ata_balance(&svm, &user_usdf_ata), user_usdf_amount - swap_amount);
    assert_eq!(get_ata_balance(&svm, &user_usdc_ata), swap_amount); // 1:1 swap
    assert_eq!(get_ata_balance(&svm, &usdf_vault_pda), swap_amount);
    assert_eq!(get_ata_balance(&svm, &other_vault_pda), other_vault_funding - swap_amount);
}

#[test]
fn test_swap_other_to_usdf() {
    let mut svm = setup_svm();

    let authority = create_payer(&mut svm);
    let authority_pk = authority.pubkey();

    let usdf_decimals = 6;
    let usdc_decimals = 6;

    let usdf = create_mint(&mut svm, &authority, &authority_pk, usdf_decimals);
    let usdc = create_mint(&mut svm, &authority, &authority_pk, usdc_decimals);

    let pool_name = "usdf-usdc";
    let (pool_pda, _) = find_pool_pda(&authority_pk, pool_name, &usdf, &usdc);
    let (usdf_vault_pda, _) = find_vault_pda(&pool_pda, &usdf);
    let (other_vault_pda, _) = find_vault_pda(&pool_pda, &usdc);

    // Initialize the pool
    let blockhash = svm.latest_blockhash();
    let ix = build_initialize_ix(authority_pk, pool_name, usdf, usdc);
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&authority_pk), &[&authority], blockhash);
    let res = send_tx(&mut svm, tx);
    assert!(res.is_ok());

    // Fund usdf_vault with USDF so swaps can happen
    let usdf_vault_funding = as_token(10_000, usdf_decimals);
    let res = mint_to(&mut svm, &authority, &usdf, &authority, &usdf_vault_pda, usdf_vault_funding);
    assert!(res.is_ok());

    // Create a user
    let user = create_payer(&mut svm);
    let user_pk = user.pubkey();

    // Create user token accounts
    let user_usdf_ata = create_ata(&mut svm, &authority, &usdf, &user_pk);
    let user_usdc_ata = create_ata(&mut svm, &authority, &usdc, &user_pk);

    // Mint USDC to user
    let user_usdc_amount = as_token(1000, usdc_decimals);
    let res = mint_to(&mut svm, &authority, &usdc, &authority, &user_usdc_ata, user_usdc_amount);
    assert!(res.is_ok());

    assert_eq!(get_ata_balance(&svm, &user_usdf_ata), 0);
    assert_eq!(get_ata_balance(&svm, &user_usdc_ata), user_usdc_amount);

    // Swap USDC for USDF (other_to_usdf)
    let swap_amount = as_token(500, usdc_decimals);
    let blockhash = svm.latest_blockhash();
    let ix = build_swap_ix(
        user_pk,
        pool_pda,
        usdf_vault_pda,
        other_vault_pda,
        user_usdf_ata,
        user_usdc_ata,
        swap_amount,
        false, // other_to_usdf
    );
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&user_pk), &[&user], blockhash);
    let res = send_tx(&mut svm, tx);
    assert!(res.is_ok());

    // Verify balances after swap
    assert_eq!(get_ata_balance(&svm, &user_usdf_ata), swap_amount); // 1:1 swap
    assert_eq!(get_ata_balance(&svm, &user_usdc_ata), user_usdc_amount - swap_amount);
    assert_eq!(get_ata_balance(&svm, &usdf_vault_pda), usdf_vault_funding - swap_amount);
    assert_eq!(get_ata_balance(&svm, &other_vault_pda), swap_amount);
}

#[test]
fn test_swap_different_decimals_usdf_to_other() {
    // Test swapping from 9 decimals to 6 decimals
    let mut svm = setup_svm();

    let authority = create_payer(&mut svm);
    let authority_pk = authority.pubkey();

    let usdf_decimals = 9; // usdf_mint has 9 decimals
    let usdc_decimals = 6; // other_mint has 9 decimals

    let usdf = create_mint(&mut svm, &authority, &authority_pk, usdf_decimals);
    let usdc = create_mint(&mut svm, &authority, &authority_pk, usdc_decimals);

    let pool_name = "usdf-usdc";
    let (pool_pda, _) = find_pool_pda(&authority_pk, pool_name, &usdf, &usdc);
    let (usdf_vault_pda, _) = find_vault_pda(&pool_pda, &usdf);
    let (other_vault_pda, _) = find_vault_pda(&pool_pda, &usdc);

    // Initialize the pool
    let blockhash = svm.latest_blockhash();
    let ix = build_initialize_ix(authority_pk, pool_name, usdf, usdc);
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&authority_pk), &[&authority], blockhash);
    let res = send_tx(&mut svm, tx);
    assert!(res.is_ok());

    // Fund other_vault with USDC (6 decimals) so swaps can happen
    let other_vault_funding = as_token(10_000, usdc_decimals);
    let res = mint_to(&mut svm, &authority, &usdc, &authority, &other_vault_pda, other_vault_funding);
    assert!(res.is_ok());

    // Create a user
    let user = create_payer(&mut svm);
    let user_pk = user.pubkey();

    // Create user token accounts
    let user_usdf_ata = create_ata(&mut svm, &authority, &usdf, &user_pk);
    let user_usdc_ata = create_ata(&mut svm, &authority, &usdc, &user_pk);

    // Mint USDF (9 decimals) to user: 1000 USDF = 1_000_000_000_000 base units
    let user_usdf_amount = as_token(1000, usdf_decimals);
    let res = mint_to(&mut svm, &authority, &usdf, &authority, &user_usdf_ata, user_usdf_amount);
    assert!(res.is_ok());

    // Swap 500 USDF for USDC (usdf_to_other)
    // 500 USDF = 500_000_000_000 (9 decimals)
    // Expected output: 500 USDC = 500_000_000 (6 decimals)
    let swap_amount = as_token(500, usdf_decimals);
    let expected_output = as_token(500, usdc_decimals); // Same value, different decimals

    let blockhash = svm.latest_blockhash();
    let ix = build_swap_ix(
        user_pk,
        pool_pda,
        usdf_vault_pda,
        other_vault_pda,
        user_usdf_ata,
        user_usdc_ata,
        swap_amount,
        true, // usdf_to_other
    );
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&user_pk), &[&user], blockhash);
    let res = send_tx(&mut svm, tx);
    assert!(res.is_ok());

    // Verify balances after swap
    assert_eq!(get_ata_balance(&svm, &user_usdf_ata), user_usdf_amount - swap_amount);
    assert_eq!(get_ata_balance(&svm, &user_usdc_ata), expected_output);
    assert_eq!(get_ata_balance(&svm, &usdf_vault_pda), swap_amount);
    assert_eq!(get_ata_balance(&svm, &other_vault_pda), other_vault_funding - expected_output);
}

#[test]
fn test_swap_different_decimals_other_to_usdf() {
    // Test swapping from 6 decimals to 9 decimals
    let mut svm = setup_svm();

    let authority = create_payer(&mut svm);
    let authority_pk = authority.pubkey();

    let usdf_decimals = 9; // usdf_mint has 6 decimals
    let usdc_decimals = 6; // other_mint has 9 decimals

    let usdf = create_mint(&mut svm, &authority, &authority_pk, usdf_decimals);
    let usdc = create_mint(&mut svm, &authority, &authority_pk, usdc_decimals);

    let pool_name = "usdf-usdc";
    let (pool_pda, _) = find_pool_pda(&authority_pk, pool_name, &usdf, &usdc);
    let (usdf_vault_pda, _) = find_vault_pda(&pool_pda, &usdf);
    let (other_vault_pda, _) = find_vault_pda(&pool_pda, &usdc);

    // Initialize the pool
    let blockhash = svm.latest_blockhash();
    let ix = build_initialize_ix(authority_pk, pool_name, usdf, usdc);
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&authority_pk), &[&authority], blockhash);
    let res = send_tx(&mut svm, tx);
    assert!(res.is_ok());

    // Fund usdf_vault with USDF (9 decimals) so swaps can happen
    let usdf_vault_funding = as_token(10_000, usdf_decimals);
    let res = mint_to(&mut svm, &authority, &usdf, &authority, &usdf_vault_pda, usdf_vault_funding);
    assert!(res.is_ok());

    // Create a user
    let user = create_payer(&mut svm);
    let user_pk = user.pubkey();

    // Create user token accounts
    let user_usdf_ata = create_ata(&mut svm, &authority, &usdf, &user_pk);
    let user_usdc_ata = create_ata(&mut svm, &authority, &usdc, &user_pk);

    // Mint USDC (9 decimals) to user: 1000 USDC = 1_000_000_000_000 base units
    let user_usdc_amount = as_token(1000, usdc_decimals);
    let res = mint_to(&mut svm, &authority, &usdc, &authority, &user_usdc_ata, user_usdc_amount);
    assert!(res.is_ok());

    // Swap 500 USDC for USDF (other_to_usdf)
    // 500 USDC = 500_000_000 (6 decimals)
    // Expected output: 500 USDF = 500_000_000_000 (9 decimals)
    let swap_amount = as_token(500, usdc_decimals);
    let expected_output = as_token(500, usdf_decimals); // Same value, different decimals

    let blockhash = svm.latest_blockhash();
    let ix = build_swap_ix(
        user_pk,
        pool_pda,
        usdf_vault_pda,
        other_vault_pda,
        user_usdf_ata,
        user_usdc_ata,
        swap_amount,
        false, // other_to_usdf
    );
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&user_pk), &[&user], blockhash);
    let res = send_tx(&mut svm, tx);
    assert!(res.is_ok());

    // Verify balances after swap
    assert_eq!(get_ata_balance(&svm, &user_usdf_ata), expected_output);
    assert_eq!(get_ata_balance(&svm, &user_usdc_ata), user_usdc_amount - swap_amount);
    assert_eq!(get_ata_balance(&svm, &usdf_vault_pda), usdf_vault_funding - expected_output);
    assert_eq!(get_ata_balance(&svm, &other_vault_pda), swap_amount);
}

#[test]
fn test_swap_truncation_favors_pool() {
    // Test that precision loss truncates in favor of the pool
    // When going from higher decimals to lower decimals, user receives less
    let mut svm = setup_svm();

    let authority = create_payer(&mut svm);
    let authority_pk = authority.pubkey();

    let usdf_decimals = 9; // usdf_mint has 9 decimals
    let usdc_decimals = 6; // other_mint has 6 decimals

    let usdf = create_mint(&mut svm, &authority, &authority_pk, usdf_decimals);
    let usdc = create_mint(&mut svm, &authority, &authority_pk, usdc_decimals);

    let pool_name = "usdf-usdc";
    let (pool_pda, _) = find_pool_pda(&authority_pk, pool_name, &usdf, &usdc);
    let (usdf_vault_pda, _) = find_vault_pda(&pool_pda, &usdf);
    let (other_vault_pda, _) = find_vault_pda(&pool_pda, &usdc);

    // Initialize the pool
    let blockhash = svm.latest_blockhash();
    let ix = build_initialize_ix(authority_pk, pool_name, usdf, usdc);
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&authority_pk), &[&authority], blockhash);
    let res = send_tx(&mut svm, tx);
    assert!(res.is_ok());

    // Fund usdc_vault with USDC so swaps can happen
    let usdc_vault_funding = as_token(10_000, usdf_decimals);
    let res = mint_to(&mut svm, &authority, &usdc, &authority, &other_vault_pda, usdc_vault_funding);
    assert!(res.is_ok());

    // Create a user
    let user = create_payer(&mut svm);
    let user_pk = user.pubkey();

    // Create user token accounts
    let user_usdf_ata = create_ata(&mut svm, &authority, &usdf, &user_pk);
    let user_usdc_ata = create_ata(&mut svm, &authority, &usdc, &user_pk);

    // Mint USDF (9 decimals) to user with an amount that doesn't divide evenly
    // 1_000_000_999 / 1000 = 1_000_000 (truncates 999)
    let user_usdf_amount = 1_000_000_999u64;
    let res = mint_to(&mut svm, &authority, &usdf, &authority, &user_usdf_ata, user_usdf_amount);
    assert!(res.is_ok());

    // Swap an amount that will be truncated
    let swap_amount = 1_000_000_999u64; // Not divisible by 1000
    let expected_output = 1_000_000u64;  // Truncated: 1_000_000_999 / 1000 = 1_000_000

    let blockhash = svm.latest_blockhash();
    let ix = build_swap_ix(
        user_pk,
        pool_pda,
        usdf_vault_pda,
        other_vault_pda,
        user_usdf_ata,
        user_usdc_ata,
        swap_amount,
        true, // usdf_to_other
    );
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&user_pk), &[&user], blockhash);
    let res = send_tx(&mut svm, tx);
    assert!(res.is_ok());

    // Verify balances - user receives truncated amount, pool keeps the difference
    assert_eq!(get_ata_balance(&svm, &user_usdf_ata), 0);
    assert_eq!(get_ata_balance(&svm, &user_usdc_ata), expected_output);
    assert_eq!(get_ata_balance(&svm, &usdf_vault_pda), user_usdf_amount ); // Pool received full amount
    assert_eq!(get_ata_balance(&svm, &other_vault_pda), usdc_vault_funding - expected_output);
}

#[test]
fn test_swap_limit_enforced() {
    let mut svm = setup_svm();

    let authority = create_payer(&mut svm);
    let authority_pk = authority.pubkey();

    let usdf_decimals = 6;
    let usdc_decimals = 6;

    let usdf = create_mint(&mut svm, &authority, &authority_pk, usdf_decimals);
    let usdc = create_mint(&mut svm, &authority, &authority_pk, usdc_decimals);

    let pool_name = "usdf-usdc";
    let (pool_pda, _) = find_pool_pda(&authority_pk, pool_name, &usdf, &usdc);
    let (usdf_vault_pda, _) = find_vault_pda(&pool_pda, &usdf);
    let (other_vault_pda, _) = find_vault_pda(&pool_pda, &usdc);

    // Initialize the pool
    let blockhash = svm.latest_blockhash();
    let ix = build_initialize_ix(authority_pk, pool_name, usdf, usdc);
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&authority_pk), &[&authority], blockhash);
    let res = send_tx(&mut svm, tx);
    assert!(res.is_ok());

    // Fund other_vault with USDC so swaps can happen
    let other_vault_funding = as_token(100_000, usdc_decimals);
    let res = mint_to(&mut svm, &authority, &usdc, &authority, &other_vault_pda, other_vault_funding);
    assert!(res.is_ok());

    // Create a user
    let user = create_payer(&mut svm);
    let user_pk = user.pubkey();

    // Create user token accounts
    let user_usdf_ata = create_ata(&mut svm, &authority, &usdf, &user_pk);
    let user_usdc_ata = create_ata(&mut svm, &authority, &usdc, &user_pk);

    // Mint USDF to user (more than the limit)
    let user_usdf_amount = as_token(2000, usdf_decimals);
    let res = mint_to(&mut svm, &authority, &usdf, &authority, &user_usdf_ata, user_usdf_amount);
    assert!(res.is_ok());

    // Try to swap more than $1000 - should fail
    let swap_amount = as_token(1001, usdf_decimals); // $1001 exceeds the $1000 limit
    let blockhash = svm.latest_blockhash();
    let ix = build_swap_ix(
        user_pk,
        pool_pda,
        usdf_vault_pda,
        other_vault_pda,
        user_usdf_ata,
        user_usdc_ata,
        swap_amount,
        true, // usdf_to_other
    );
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&user_pk), &[&user], blockhash);
    let res = send_tx(&mut svm, tx);
    assert!(res.is_err()); // Should fail due to limit

    // Swap exactly $1000 - should succeed
    let swap_amount = as_token(1000, usdf_decimals); // Exactly $1000
    let blockhash = svm.latest_blockhash();
    let ix = build_swap_ix(
        user_pk,
        pool_pda,
        usdf_vault_pda,
        other_vault_pda,
        user_usdf_ata,
        user_usdc_ata,
        swap_amount,
        true, // usdf_to_other
    );
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&user_pk), &[&user], blockhash);
    let res = send_tx(&mut svm, tx);
    assert!(res.is_ok()); // Should succeed

    // Verify balances after successful swap
    assert_eq!(get_ata_balance(&svm, &user_usdf_ata), user_usdf_amount - swap_amount);
    assert_eq!(get_ata_balance(&svm, &user_usdc_ata), swap_amount);
    assert_eq!(get_ata_balance(&svm, &usdf_vault_pda), swap_amount);
    assert_eq!(get_ata_balance(&svm, &other_vault_pda), other_vault_funding - swap_amount);
}

#[test]
fn test_transfer() {
    let mut svm = setup_svm();

    let authority = create_payer(&mut svm);
    let authority_pk = authority.pubkey();

    let usdf_decimals = 6;
    let usdc_decimals = 6;

    let usdf = create_mint(&mut svm, &authority, &authority_pk, usdf_decimals);
    let usdc = create_mint(&mut svm, &authority, &authority_pk, usdc_decimals);

    let pool_name = "usdf-usdc";
    let (pool_pda, _) = find_pool_pda(&authority_pk, pool_name, &usdf, &usdc);
    let (usdf_vault_pda, _) = find_vault_pda(&pool_pda, &usdf);
    let (other_vault_pda, _) = find_vault_pda(&pool_pda, &usdc);

    // Initialize the pool
    let blockhash = svm.latest_blockhash();
    let ix = build_initialize_ix(authority_pk, pool_name, usdf, usdc);
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&authority_pk), &[&authority], blockhash);
    let res = send_tx(&mut svm, tx);
    assert!(res.is_ok());

    // Fund both vaults
    let usdf_vault_funding = as_token(10_000, usdf_decimals);
    let other_vault_funding = as_token(10_000, usdc_decimals);
    let _ = mint_to(&mut svm, &authority, &usdf, &authority, &usdf_vault_pda, usdf_vault_funding);
    let _ = mint_to(&mut svm, &authority, &usdc, &authority, &other_vault_pda, other_vault_funding);

    // Create authority's destination token accounts
    let authority_usdf_ata = create_ata(&mut svm, &authority, &usdf, &authority_pk);
    let authority_usdc_ata = create_ata(&mut svm, &authority, &usdc, &authority_pk);

    // Transfer USDF from usdf_vault to authority
    let transfer_amount_usdf = as_token(3000, usdf_decimals);
    let blockhash = svm.latest_blockhash();
    let ix = build_transfer_ix(
        authority_pk,
        pool_pda,
        usdf,
        usdc,
        authority_usdf_ata,
        transfer_amount_usdf,
        true, // is_usdf
    );
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&authority_pk), &[&authority], blockhash);
    let res = send_tx(&mut svm, tx);
    assert!(res.is_ok());

    assert_eq!(get_ata_balance(&svm, &usdf_vault_pda), usdf_vault_funding - transfer_amount_usdf);
    assert_eq!(get_ata_balance(&svm, &authority_usdf_ata), transfer_amount_usdf);

    // Transfer USDC from other_vault to authority
    let transfer_amount_other = as_token(5000, usdc_decimals);
    let blockhash = svm.latest_blockhash();
    let ix = build_transfer_ix(
        authority_pk,
        pool_pda,
        usdf,
        usdc,
        authority_usdc_ata,
        transfer_amount_other,
        false, // is_usdf (false = other)
    );
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&authority_pk), &[&authority], blockhash);
    let res = send_tx(&mut svm, tx);
    assert!(res.is_ok());

    assert_eq!(get_ata_balance(&svm, &other_vault_pda), other_vault_funding - transfer_amount_other);
    assert_eq!(get_ata_balance(&svm, &authority_usdc_ata), transfer_amount_other);
}
