# USDF Swap Program
![license][license-image]
![version][version-image]

[version-image]: https://img.shields.io/badge/version-1.0.0-blue.svg?style=flat
[license-image]: https://img.shields.io/badge/license-MIT-blue.svg?style=flat


The USDF Swap Program is used by Flipcash to faciliate 1:1 swaps between USDF and another stable coin. It provides the following core features:

- **Pool Initialization:** Creates a new liquidity pool for two stablecoins, with PDA-derived vault accounts for each token.
- **1:1 Swaps:** Allows users to swap tokens between the two stablecoins at a fixed 1:1 rate.
- **Authority-Controlled Transfers:** Enables the pool authority to withdraw funds from pool vaults to designated accounts.

The program uses PDAs for accounts like pool and vaults to ensure deterministic addressing.

## CLI

The USDF Swap CLI is a command-line interface tool built in Rust for interacting with the USDF Swap Solana program. The program provides a Solana-based protocol for managing a stablecoin liquidity pool that facilitates 1:1 swaps between two stablecoins. The CLI allows users to initialize pools, retrieve pool information, perform swaps, and transfer funds.

The CLI supports various Solana clusters (localnet, mainnet, devnet, testnet, or custom RPC URLs) and requires a Solana keypair for signing transactions.


## Installation

To build and install the CLI:
1. Ensure you have Rust and Cargo installed.
2. Clone the repository.
3. Run `cargo build --release` to compile the binary.
4. The executable will be available at `target/release/usdf-swap-cli`.

You may need to install Solana CLI tools separately for keypair management.

## Options

These options are available for all commands and can be specified before the subcommand.

- `--authority-keypair <PATH>`: Path to the authority keypair file (JSON format). Default: `~/.config/solana/id.json`.
- `--cluster <VALUE>`: Solana cluster to connect to. Options:
  - `l`: Localnet (`http://127.0.0.1:8899`).
  - `m`: Mainnet (`https://api.mainnet-beta.solana.com`).
  - `d`: Devnet (`https://api.devnet.solana.com`).
  - `t`: Testnet (`https://api.testnet.solana.com`).
  - Custom RPC URL (e.g., `https://my-custom-rpc.com`).
  Default: `l` (localnet).

Example usage:
```
usdf-swap-cli --authority-keypair /path/to/keypair.json --cluster d initialize --name usdc-usdf --usdf-mint <PUBKEY> --other-mint <PUBKEY>
```

## Commands

### initialize

Initializes a new swap pool for two stablecoins. The caller becomes the pool authority.

**Usage:**
```
usdf-swap-cli initialize --name <NAME> --usdf-mint <PUBKEY> --other-mint <PUBKEY>
```

**Options:**
- `--name <NAME>`: Pool name (used as PDA seed, max 32 characters). Required.
- `--usdf-mint <PUBKEY>`: Public key of the USDF stablecoin mint. Required.
- `--other-mint <PUBKEY>`: Public key of the other stablecoin mint. Required.

**Output:**
- Prints the pool address, name, mint addresses, and transaction signature.

**Functionality in USDF Swap Program:**
- Calls the `initialize` instruction on the program.
- Creates a pool account with metadata (authority, name, mints, vaults).
- Creates PDA-derived vault accounts for each stablecoin.
- Sets the caller as the pool authority.
- The pool PDA is derived from: authority, name, usdf_mint, other_mint.

### get-pool

Retrieves information for a given pool.

**Usage:**
```
usdf-swap-cli get-pool --pool <PUBKEY>
```

**Options:**
- `--pool <PUBKEY>`: Public key of the pool account. Required.

**Output:**
- Pool Metadata: Name, Authority, USDF Mint, Other Mint, USDF Vault, Other Vault.

**Functionality in USDF Swap Program:**
- Fetches and deserializes the pool account from the blockchain.
- Displays on-chain data including authority and vault addresses.

### swap

Swaps tokens 1:1 between the two stablecoins.

**Usage:**
```
usdf-swap-cli swap --pool <PUBKEY> --user-keypair <PATH> --amount <NUMBER> --usdf-to-other <BOOL>
```

**Options:**
- `--pool <PUBKEY>`: Public key of the pool account. Required.
- `--user-keypair <PATH>`: Path to the user's keypair file. Required.
- `--amount <NUMBER>`: Amount to swap (in tokens). Automatically converted to smallest units based on mint decimals. Required.
- `--usdf-to-other <BOOL>`: Swap direction - true for USDF to other, false for other to USDF. Required.

**Output:**
- Prints the user address, swap direction, amount (in tokens and quarks), and transaction signature.
- If ATAs are created, prints the created ATA addresses.

**Functionality in USDF Swap Program:**
- Derives the user's associated token accounts (ATAs) for both mints from the pool.
- Automatically creates ATAs if they don't exist (user pays for creation).
- Calls the `swap` instruction on the program.
- Transfers tokens from the user's source token account to the pool vault.
- Transfers equivalent tokens from the pool vault to the user's destination token account.

### transfer

Transfers tokens from a pool vault to a destination account. Authority only.

**Usage:**
```
usdf-swap-cli transfer --pool <PUBKEY> --amount <NUMBER> --from-usdf-vault <BOOL> --destination <PUBKEY>
```

**Options:**
- `--pool <PUBKEY>`: Public key of the pool account. Required.
- `--amount <NUMBER>`: Amount to transfer (in tokens). Automatically converted to smallest units based on mint decimals. Required.
- `--from-usdf-vault <BOOL>`: Transfer from USDF vault (true) or other vault (false). Required.
- `--destination <PUBKEY>`: Destination token account. Required.

**Output:**
- Prints the amount (in tokens and quarks), source vault, destination, and transaction signature.

**Functionality in USDF Swap Program:**
- Calls the `transfer` instruction on the program.
- Transfers tokens from the specified pool vault to the destination account.
- Only callable by the pool authority.

## Examples

1. Initialize a new swap pool:
   ```
   usdf-swap-cli initialize --name usdc-usdf --usdf-mint <USDF_MINT_PUBKEY> --other-mint <OTHER_MINT_PUBKEY>
   ```

2. Get pool information:
   ```
   usdf-swap-cli get-pool --pool <POOL_PUBKEY>
   ```

3. Swap 1000 tokens from USDF to other:
   ```
   usdf-swap-cli swap --pool <POOL_PUBKEY> --user-keypair /path/to/user.json --amount 1000 --usdf-to-other true
   ```

4. Swap 500 tokens from other to USDF:
   ```
   usdf-swap-cli swap --pool <POOL_PUBKEY> --user-keypair /path/to/user.json --amount 500 --usdf-to-other false
   ```

5. Transfer 100 tokens from USDF vault:
   ```
   usdf-swap-cli transfer --pool <POOL_PUBKEY> --amount 100 --from-usdf-vault true --destination <DESTINATION_ATA>
   ```
