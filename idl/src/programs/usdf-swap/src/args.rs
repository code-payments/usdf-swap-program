use anchor_lang::prelude::*;

use crate::consts::MAX_NAME_LEN;

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub struct InitializeArgs {
    pub name: [u8; MAX_NAME_LEN],
    pub bump: u8,
    pub usdf_vault_bump: u8,
    pub other_vault_bump: u8,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub struct SwapArgs {
    pub amount: u64,
    pub usdf_to_other: u8,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub struct TransferArgs {
    pub amount: u64,
    pub is_usdf: u8,
}
