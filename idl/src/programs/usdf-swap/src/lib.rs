use anchor_lang::prelude::*;

mod args;
mod consts;
mod instructions;
mod state;

use args::*;
use instructions::*;

declare_id!("usdfcP2V1bh1Lz7Y87pxR4zJd3wnVtssJ6GeSHFeZeu");

#[program]
pub mod usdf_swap {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>, _data: InitializeArgs) -> Result<()> {
        Ok(())
    }

    pub fn swap(_ctx: Context<Swap>, _data: SwapArgs) -> Result<()> {
        Ok(())
    }

    pub fn transfer(_ctx: Context<Transfer>, _data: TransferArgs) -> Result<()> {
        Ok(())
    }
}
