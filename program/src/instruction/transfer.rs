use steel::*;
use usdf_swap_api::prelude::*;

pub fn process_transfer(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let raw_args = TransferIx::try_from_bytes(data)?;
    let args = raw_args.to_struct();

    let [
        authority_info,
        pool_info,
        vault_info,
        destination_info,
        token_program_info,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Validate accounts
    check_signer(authority_info)?;
    check_mut(vault_info)?;
    check_mut(destination_info)?;

    check_program(token_program_info, &spl_token::id())?;

    let pool = pool_info.as_account::<Pool>(&usdf_swap_api::ID)?;

    // Verify authority matches pool authority
    check_condition(
        pool.authority.eq(authority_info.key),
        "Authority does not match pool authority"
    )?;

    // Determine which vault and mint we're transferring from
    let (expected_vault, mint, vault_bump) = if args.is_usdf {
        (pool.usdf_vault, pool.usdf_mint, pool.usdf_vault_bump)
    } else {
        (pool.other_vault, pool.other_mint, pool.other_vault_bump)
    };

    // Verify vault matches
    check_condition(
        expected_vault.eq(vault_info.key),
        "Vault does not match expected vault for mint"
    )?;

    // Verify destination token account is for the correct mint
    let destination = destination_info.as_token_account()?;
    destination.assert(|t| t.mint().eq(&mint))?;

    check_condition(args.amount > 0, "Transfer amount must be greater than 0")?;

    // Transfer from vault to destination (signed by vault PDA)
    transfer_signed_with_bump(
        vault_info,
        vault_info,
        destination_info,
        token_program_info,
        args.amount,
        &[
            VAULT,
            pool_info.key.as_ref(),
            mint.as_ref(),
        ],
        vault_bump,
    )?;

    Ok(())
}
