use anchor_lang::prelude::*;

use crate::consts::MAX_NAME_LEN;

#[account]
#[repr(C, align(8))]
#[derive(Copy, Debug, PartialEq)]
pub struct Pool {
    pub authority: Pubkey,
    pub name: [u8; MAX_NAME_LEN],
    pub usdf_mint: Pubkey,
    pub other_mint: Pubkey,
    pub usdf_vault: Pubkey,
    pub other_vault: Pubkey,
    pub bump: u8,
    pub usdf_vault_bump: u8,
    pub other_vault_bump: u8,
    pub usdf_decimals: u8,
    pub other_decimals: u8,
    pub _padding: [u8; 3],
}
