use steel::*;
use usdf_swap_api::prelude::*;

pub fn process_initialize(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let raw_args = InitializeIx::try_from_bytes(data)?;
    let args = raw_args.to_struct();

    let [
        authority_info,
        usdf_mint_info,
        other_mint_info,
        pool_info,
        usdf_vault_info,
        other_vault_info,
        token_program_info,
        system_program_info,
        rent_sysvar_info,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Validate accounts
    check_signer(authority_info)?;
    check_mut(pool_info)?;
    check_mut(usdf_vault_info)?;
    check_mut(other_vault_info)?;

    check_program(token_program_info, &spl_token::id())?;
    check_program(system_program_info, &system_program::id())?;
    check_sysvar(rent_sysvar_info, &sysvar::rent::id())?;

    // Check mint accounts are valid and get decimals
    let usdf_mint = usdf_mint_info.as_mint()?;
    let other_mint = other_mint_info.as_mint()?;
    let usdf_decimals = usdf_mint.decimals();
    let other_decimals = other_mint.decimals();

    check_condition(
        usdf_mint_info.key.ne(other_mint_info.key),
        "USDF mint and other mint must be different"
    )?;

    // Validate PDAs
    check_uninitialized_pda(
        pool_info,
        &[POOL, authority_info.key.as_ref(), args.name.as_ref(), usdf_mint_info.key.as_ref(), other_mint_info.key.as_ref()],
        &usdf_swap_api::id()
    )?;

    check_uninitialized_pda(
        usdf_vault_info,
        &[VAULT, pool_info.key.as_ref(), usdf_mint_info.key.as_ref()],
        &usdf_swap_api::id()
    )?;

    check_uninitialized_pda(
        other_vault_info,
        &[VAULT, pool_info.key.as_ref(), other_mint_info.key.as_ref()],
        &usdf_swap_api::id()
    )?;

    // Create USDF vault token account
    create_token_account(
        usdf_mint_info,
        usdf_vault_info,
        &[
            VAULT,
            pool_info.key.as_ref(),
            usdf_mint_info.key.as_ref(),
            &[args.usdf_vault_bump]
        ],
        authority_info,
        system_program_info,
        rent_sysvar_info,
    )?;

    // Create other vault token account
    create_token_account(
        other_mint_info,
        other_vault_info,
        &[
            VAULT,
            pool_info.key.as_ref(),
            other_mint_info.key.as_ref(),
            &[args.other_vault_bump]
        ],
        authority_info,
        system_program_info,
        rent_sysvar_info,
    )?;

    // Create the swap pool account
    create_program_account_with_bump::<Pool>(
        pool_info,
        system_program_info,
        authority_info,
        &usdf_swap_api::ID,
        &[
            POOL,
            authority_info.key.as_ref(),
            args.name.as_ref(),
            usdf_mint_info.key.as_ref(),
            other_mint_info.key.as_ref(),
        ],
        args.bump,
    )?;

    // Initialize the pool data
    let pool = pool_info.as_account_mut::<Pool>(&usdf_swap_api::ID)?;

    pool.authority = *authority_info.key;
    pool.name = args.name;
    pool.usdf_mint = *usdf_mint_info.key;
    pool.other_mint = *other_mint_info.key;
    pool.usdf_vault = *usdf_vault_info.key;
    pool.other_vault = *other_vault_info.key;
    pool.bump = args.bump;
    pool.usdf_vault_bump = args.usdf_vault_bump;
    pool.other_vault_bump = args.other_vault_bump;
    pool.usdf_decimals = usdf_decimals;
    pool.other_decimals = other_decimals;

    Ok(())
}
