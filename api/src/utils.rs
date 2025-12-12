use steel::*;
use solana_program::msg;
use crate::consts::MAX_NAME_LEN;

pub fn check_condition(condition: bool, message: &str) -> ProgramResult {
    if !condition {
        msg!("Failed condition: {}", message);
        return Err(ProgramError::InvalidArgument);
    }
    Ok(())
}

pub fn check_signer(account: &AccountInfo) -> ProgramResult {
    account.is_signer()?.is_writable()?;
    Ok(())
}

pub fn check_signer_readonly(account: &AccountInfo) -> ProgramResult {
    account.is_signer()?;
    Ok(())
}

pub fn check_mut(account: &AccountInfo) -> ProgramResult {
    account.is_writable()?;
    Ok(())
}

pub fn check_uninitialized_pda(account: &AccountInfo, seeds: &[&[u8]], program_id: &Pubkey) -> ProgramResult {
    if !account.owner.eq(&system_program::ID) {
        return Err(ProgramError::InvalidAccountData);
    }

    account.is_empty()?.is_writable()?.has_seeds(seeds, program_id)?;
    Ok(())
}

pub fn check_seeds(account: &AccountInfo, seeds: &[&[u8]], program_id: &Pubkey) -> ProgramResult {
    account.has_seeds(seeds, program_id)?;
    Ok(())
}

pub fn check_program(account: &AccountInfo, program_id: &Pubkey) -> ProgramResult {
    account.is_program(program_id)?;
    Ok(())
}

pub fn check_sysvar(account: &AccountInfo, sysvar_id: &Pubkey) -> ProgramResult {
    account.is_sysvar(sysvar_id)?;
    Ok(())
}

pub fn to_name(val: &str) -> [u8; MAX_NAME_LEN] {
    assert!(val.len() <= MAX_NAME_LEN, "name too long");

    let mut name_bytes = [0u8; MAX_NAME_LEN];
    name_bytes[..val.as_bytes().len()].copy_from_slice(val.as_bytes());
    name_bytes
}

pub fn from_name(val: &[u8]) -> String {
    let mut name_bytes = val.to_vec();
    name_bytes.retain(|&x| x != 0);
    String::from_utf8(name_bytes).unwrap()
}
