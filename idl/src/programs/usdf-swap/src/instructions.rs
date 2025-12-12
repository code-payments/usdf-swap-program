use anchor_lang::prelude::*;
use anchor_spl::token::Token;

use crate::state::Pool;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub usdf_mint: AccountInfo<'info>,
    pub other_mint: AccountInfo<'info>,
    #[account(mut)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub usdf_vault: AccountInfo<'info>,
    #[account(mut)]
    pub other_vault: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub usdf_vault: AccountInfo<'info>,
    #[account(mut)]
    pub other_vault: AccountInfo<'info>,
    #[account(mut)]
    pub user_usdf_token: AccountInfo<'info>,
    #[account(mut)]
    pub user_other_token: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Transfer<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub vault: AccountInfo<'info>,
    #[account(mut)]
    pub destination: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}
