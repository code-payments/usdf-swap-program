use steel::*;
use super::AccountType;
use crate::state;

/// Pool account - stores the configuration for a stablecoin swap pool
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Pool {
    /// Authority that can sign swaps and transfer funds
    pub authority: Pubkey,

    /// Pool name (used as PDA seed)
    pub name: [u8; 32],

    /// USDF stablecoin mint
    pub usdf_mint: Pubkey,
    /// Other stablecoin mint
    pub other_mint: Pubkey,

    /// USDF vault (holds usdf_mint tokens)
    pub usdf_vault: Pubkey,
    /// Other vault (holds other_mint tokens)
    pub other_vault: Pubkey,

    /// Bump seed for the pool PDA
    pub bump: u8,
    /// Bump seed for USDF vault PDA
    pub usdf_vault_bump: u8,
    /// Bump seed for other vault PDA
    pub other_vault_bump: u8,

    /// Decimals for USDF mint
    pub usdf_decimals: u8,
    /// Decimals for other mint
    pub other_decimals: u8,

    _padding: [u8; 3],
}

state!(AccountType, Pool);
