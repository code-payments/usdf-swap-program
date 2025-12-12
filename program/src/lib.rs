#![allow(unexpected_cfgs)]
use steel::*;
use usdf_swap_api::prelude::*;

pub mod instruction;
use instruction::*;

mod security;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let (ix, data) = parse_instruction(&usdf_swap_api::ID, program_id, data)?;

    match ix {
        InstructionType::Unknown => return Err(ProgramError::InvalidInstructionData),
        InstructionType::InitializeIx => process_initialize(accounts, data)?,
        InstructionType::SwapIx => process_swap(accounts, data)?,
        InstructionType::TransferIx => process_transfer(accounts, data)?,
    }

    Ok(())
}

entrypoint!(process_instruction);
