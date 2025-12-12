use solana_program::program_pack::Pack;
use steel::*;

pub fn create_token_account<'info>(
    mint: &AccountInfo<'info>,
    target: &AccountInfo<'info>,
    seeds: &[&[u8]],
    payer: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    rent_sysvar: &AccountInfo<'info>,
) -> ProgramResult {
    // Check if the token account is already initialized
    if !target.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    // Safely create the account with system program
    steel::allocate_account_with_bump(
        target,
        system_program,
        payer,
        spl_token::state::Account::LEN,
        &spl_token::id(),
        &seeds[0..seeds.len()-1],
        seeds[seeds.len()-1][0],
    )?;

    // Initialize the PDA.
    solana_program::program::invoke(
        &spl_token::instruction::initialize_account(
            &spl_token::id(),
            target.key,
            mint.key,
            target.key,
        ).unwrap(),
        &[
            target.clone(),
            mint.clone(),
            target.clone(),
            rent_sysvar.clone(),
        ],
    )
}
