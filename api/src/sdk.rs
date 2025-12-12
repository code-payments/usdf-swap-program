use steel::*;
use crate::prelude::*;

pub fn build_initialize_ix(
    authority: Pubkey,
    name: &str,
    usdf_mint: Pubkey,
    other_mint: Pubkey,
) -> Instruction {
    let name_bytes = to_name(name);
    let (pool_pda, pool_bump) = find_pool_pda(&authority, name, &usdf_mint, &other_mint);
    let (usdf_vault_pda, usdf_vault_bump) = find_vault_pda(&pool_pda, &usdf_mint);
    let (other_vault_pda, other_vault_bump) = find_vault_pda(&pool_pda, &other_mint);

    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(authority, true),
            AccountMeta::new_readonly(usdf_mint, false),
            AccountMeta::new_readonly(other_mint, false),
            AccountMeta::new(pool_pda, false),
            AccountMeta::new(usdf_vault_pda, false),
            AccountMeta::new(other_vault_pda, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: InitializeIx::from_struct(
            ParsedInitializeIx {
                name: name_bytes,
                bump: pool_bump,
                usdf_vault_bump,
                other_vault_bump,
            }
        ).to_bytes(),
    }
}

pub fn build_swap_ix(
    user: Pubkey,
    pool: Pubkey,
    usdf_vault: Pubkey,
    other_vault: Pubkey,
    user_usdf_token: Pubkey,
    user_other_token: Pubkey,
    amount: u64,
    usdf_to_other: bool,
) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(user, true),
            AccountMeta::new_readonly(pool, false),
            AccountMeta::new(usdf_vault, false),
            AccountMeta::new(other_vault, false),
            AccountMeta::new(user_usdf_token, false),
            AccountMeta::new(user_other_token, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: SwapIx::from_struct(ParsedSwapIx {
            amount,
            usdf_to_other,
        }).to_bytes(),
    }
}

pub fn build_transfer_ix(
    authority: Pubkey,
    pool: Pubkey,
    usdf_mint: Pubkey,
    other_mint: Pubkey,
    destination: Pubkey,
    amount: u64,
    is_usdf: bool,
) -> Instruction {
    let (usdf_vault_pda, _) = find_vault_pda(&pool, &usdf_mint);
    let (other_vault_pda, _) = find_vault_pda(&pool, &other_mint);

    let vault = if is_usdf { usdf_vault_pda } else { other_vault_pda };

    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(authority, true),
            AccountMeta::new_readonly(pool, false),
            AccountMeta::new(vault, false),
            AccountMeta::new(destination, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: TransferIx::from_struct(ParsedTransferIx {
            amount,
            is_usdf,
        }).to_bytes(),
    }
}
