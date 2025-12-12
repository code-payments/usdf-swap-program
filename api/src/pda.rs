use steel::*;
use crate::consts::*;
use crate::utils::to_name;

/// Find the pool PDA for the given authority, name, and mint pair
pub fn find_pool_pda(authority: &Pubkey, name: &str, usdf_mint: &Pubkey, other_mint: &Pubkey) -> (Pubkey, u8) {
    let name = to_name(name);
    Pubkey::find_program_address(
        &[POOL, authority.as_ref(), name.as_ref(), usdf_mint.as_ref(), other_mint.as_ref()],
        &crate::id(),
    )
}

/// Find the vault PDA for a given pool and mint
pub fn find_vault_pda(pool: &Pubkey, mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[VAULT, pool.as_ref(), mint.as_ref()],
        &crate::id(),
    )
}
