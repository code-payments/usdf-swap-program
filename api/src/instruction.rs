use steel::*;
use crate::consts::MAX_NAME_LEN;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum InstructionType {
    Unknown = 0,
    InitializeIx,
    SwapIx,
    TransferIx,
}

instruction!(InstructionType, InitializeIx);
instruction!(InstructionType, SwapIx);
instruction!(InstructionType, TransferIx);

// ============================================================================
// Initialize Instruction
// ============================================================================

#[derive(Debug)]
pub struct ParsedInitializeIx {
    pub name: [u8; MAX_NAME_LEN],
    pub bump: u8,
    pub usdf_vault_bump: u8,
    pub other_vault_bump: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct InitializeIx {
    pub name: [u8; MAX_NAME_LEN],
    pub bump: u8,
    pub usdf_vault_bump: u8,
    pub other_vault_bump: u8,
}

impl InitializeIx {
    pub fn from_struct(parsed: ParsedInitializeIx) -> Self {
        Self {
            name: parsed.name,
            bump: parsed.bump,
            usdf_vault_bump: parsed.usdf_vault_bump,
            other_vault_bump: parsed.other_vault_bump,
        }
    }

    pub fn to_struct(&self) -> ParsedInitializeIx {
        ParsedInitializeIx {
            name: self.name,
            bump: self.bump,
            usdf_vault_bump: self.usdf_vault_bump,
            other_vault_bump: self.other_vault_bump,
        }
    }
}

// ============================================================================
// Swap Instruction
// ============================================================================

#[derive(Debug)]
pub struct ParsedSwapIx {
    /// Amount of tokens to swap
    pub amount: u64,
    /// Direction: true = USDF to other, false = other to USDF
    pub usdf_to_other: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct SwapIx {
    pub amount: [u8; 8],
    pub usdf_to_other: u8,
}

impl SwapIx {
    pub fn from_struct(parsed: ParsedSwapIx) -> Self {
        Self {
            amount: parsed.amount.to_le_bytes(),
            usdf_to_other: if parsed.usdf_to_other { 1 } else { 0 },
        }
    }

    pub fn to_struct(&self) -> ParsedSwapIx {
        ParsedSwapIx {
            amount: u64::from_le_bytes(self.amount),
            usdf_to_other: self.usdf_to_other != 0,
        }
    }
}

// ============================================================================
// Transfer Instruction
// ============================================================================

#[derive(Debug)]
pub struct ParsedTransferIx {
    /// Amount of tokens to transfer
    pub amount: u64,
    /// Which mint to transfer: true = usdf_mint, false = other_mint
    pub is_usdf: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct TransferIx {
    pub amount: [u8; 8],
    pub is_usdf: u8,
}

impl TransferIx {
    pub fn from_struct(parsed: ParsedTransferIx) -> Self {
        Self {
            amount: parsed.amount.to_le_bytes(),
            is_usdf: if parsed.is_usdf { 1 } else { 0 },
        }
    }

    pub fn to_struct(&self) -> ParsedTransferIx {
        ParsedTransferIx {
            amount: u64::from_le_bytes(self.amount),
            is_usdf: self.is_usdf != 0,
        }
    }
}
