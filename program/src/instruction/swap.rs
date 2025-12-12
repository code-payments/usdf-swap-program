use steel::*;
use usdf_swap_api::prelude::*;

/// Converts an amount from one decimal precision to another.
/// When scaling down, truncates in favor of the pool (user receives less).
/// Returns None only if the conversion would overflow.
fn convert_amount(amount: u64, from_decimals: u8, to_decimals: u8) -> Option<u64> {
    if from_decimals == to_decimals {
        return Some(amount);
    }

    if to_decimals > from_decimals {
        // Scaling up: multiply by 10^(to - from)
        let scale = 10u64.checked_pow((to_decimals - from_decimals) as u32)?;
        amount.checked_mul(scale)
    } else {
        // Scaling down: divide by 10^(from - to)
        // Truncation favors the pool (user receives less)
        let scale = 10u64.checked_pow((from_decimals - to_decimals) as u32)?;
        Some(amount / scale)
    }
}

pub fn process_swap(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let raw_args = SwapIx::try_from_bytes(data)?;
    let args = raw_args.to_struct();

    let [
        user_info,
        pool_info,
        usdf_vault_info,
        other_vault_info,
        user_usdf_token_info,
        user_other_token_info,
        token_program_info,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Validate accounts
    check_signer(user_info)?;
    check_mut(usdf_vault_info)?;
    check_mut(other_vault_info)?;
    check_mut(user_usdf_token_info)?;
    check_mut(user_other_token_info)?;

    check_program(token_program_info, &spl_token::id())?;

    let pool = pool_info.as_account::<Pool>(&usdf_swap_api::ID)?;

    // Verify vaults match pool
    check_condition(
        pool.usdf_vault.eq(usdf_vault_info.key) && pool.other_vault.eq(other_vault_info.key),
        "Vaults do not match pool"
    )?;

    // Verify user token accounts
    let user_usdf_token = user_usdf_token_info.as_token_account()?;
    let user_other_token = user_other_token_info.as_token_account()?;

    user_usdf_token
        .assert(|t| t.owner().eq(user_info.key))?
        .assert(|t| t.mint().eq(&pool.usdf_mint))?;

    user_other_token
        .assert(|t| t.owner().eq(user_info.key))?
        .assert(|t| t.mint().eq(&pool.other_mint))?;

    check_condition(args.amount > 0, "Swap amount must be greater than 0")?;

    // Check swap amount doesn't exceed the maximum limit
    let source_decimals = if args.usdf_to_other { pool.usdf_decimals } else { pool.other_decimals };
    let max_swap_amount = MAX_SWAP_DOLLARS
        .checked_mul(10u64.pow(source_decimals as u32))
        .ok_or(ProgramError::ArithmeticOverflow)?;
    check_condition(
        args.amount <= max_swap_amount,
        "Swap amount exceeds maximum limit"
    )?;

    if args.usdf_to_other {
        // User sends USDF, receives other
        // Calculate output amount considering decimal differences
        let output_amount = convert_amount(args.amount, pool.usdf_decimals, pool.other_decimals)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        // Transfer from user_usdf_token to usdf_vault
        transfer(
            user_info,
            user_usdf_token_info,
            usdf_vault_info,
            token_program_info,
            args.amount,
        )?;

        // Transfer from other_vault to user_other_token (signed by vault PDA)
        transfer_signed_with_bump(
            other_vault_info,
            other_vault_info,
            user_other_token_info,
            token_program_info,
            output_amount,
            &[
                VAULT,
                pool_info.key.as_ref(),
                pool.other_mint.as_ref(),
            ],
            pool.other_vault_bump,
        )?;
    } else {
        // User sends other, receives USDF
        // Calculate output amount considering decimal differences
        let output_amount = convert_amount(args.amount, pool.other_decimals, pool.usdf_decimals)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        // Transfer from user_other_token to other_vault
        transfer(
            user_info,
            user_other_token_info,
            other_vault_info,
            token_program_info,
            args.amount,
        )?;

        // Transfer from usdf_vault to user_usdf_token (signed by vault PDA)
        transfer_signed_with_bump(
            usdf_vault_info,
            usdf_vault_info,
            user_usdf_token_info,
            token_program_info,
            output_amount,
            &[
                VAULT,
                pool_info.key.as_ref(),
                pool.usdf_mint.as_ref(),
            ],
            pool.usdf_vault_bump,
        )?;
    }

    Ok(())
}
