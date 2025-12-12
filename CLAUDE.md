# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

USDF Swap Program - A Solana program that acts as a liquidity pool for two stablecoins, one of which is USDF. The program enables 1:1 swaps between stablecoins.

## Repository

- **GitHub:** code-payments/usdf-swap-program
- **Main Branch:** main
- **Program ID:** usdfcP2V1bh1Lz7Y87pxR4zJd3wnVtssJ6GeSHFeZeu

## Architecture

The project follows a workspace structure with 4 crates:

- **api/** - Core types, state definitions, instructions, and PDA functions
- **program/** - On-chain Solana program implementation
- **client/** - Off-chain Rust client library
- **cli/** - Command-line interface tool

## Instructions

### Initialize
Creates a new liquidity pool for two stablecoins. The caller becomes the pool authority.

**Accounts:**
- authority (signer, mut) - Pool authority
- usdf_mint - USDF stablecoin mint
- other_mint - Other stablecoin mint
- pool (PDA, mut) - Pool account
- usdf_vault (PDA, mut) - Vault for usdf_mint
- other_vault (PDA, mut) - Vault for other_mint
- token_program
- system_program
- rent

### Swap
Swaps tokens 1:1 between the two stablecoins.

**Accounts:**
- user (signer, mut) - User performing the swap
- pool - Pool account
- usdf_vault, other_vault (mut) - Pool vaults
- user_usdf_token, user_other_token (mut) - User's token accounts
- token_program

### Transfer
Allows the authority to withdraw funds from the pool vaults.

**Accounts:**
- authority (signer, mut) - Pool authority
- pool - Pool account
- vault (mut) - Source vault
- destination (mut) - Destination token account
- token_program

## Build Commands

```bash
# Build the program
make build

# Run tests
make test

# Start local validator with program
make local

# Generate documentation
make docs
```

## Testing

Tests use litesvm for fast local Solana VM testing without requiring a validator:

```bash
cd program && cargo test-sbf
```

## Key Files

- `api/src/state/pool.rs` - Pool account structure
- `api/src/instruction.rs` - Instruction definitions
- `api/src/pda.rs` - PDA derivation functions
- `api/src/sdk.rs` - Instruction builder functions
- `program/src/instruction/` - Instruction handlers
- `program/tests/integration.rs` - Integration tests
