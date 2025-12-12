use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    system_instruction,
    transaction::Transaction,
};
use spl_associated_token_account::instruction::create_associated_token_account;
use spl_token::instruction as token_instruction;
use spl_token::state::Mint;
use spl_token::ID as TOKEN_PROGRAM_ID;
use solana_sdk::program_pack::Pack;
use usdf_swap_api::prelude::Pool;

pub fn get_rpc_client(cluster: &str) -> RpcClient {
    let url = match cluster {
        "l" | "localnet" => "http://127.0.0.1:8899",
        "d" | "devnet" => "https://api.devnet.solana.com",
        "t" | "testnet" => "https://api.testnet.solana.com",
        "m" | "mainnet" => "https://api.mainnet-beta.solana.com",
        url => url,
    };
    RpcClient::new(url.to_string())
}

pub fn get_pool(client: &RpcClient, pool_address: &Pubkey) -> Result<Pool> {
    let account = client.get_account(pool_address)?;
    let pool = Pool::unpack(&account.data)?;
    Ok(*pool)
}

/// Creates a new SPL token mint.
pub fn create_mint(
    client: &RpcClient,
    payer: &Keypair,
    decimals: u8,
) -> Result<(Pubkey, Signature)> {
    let mint = Keypair::new();
    let mint_pubkey = mint.pubkey();
    let payer_pk = payer.pubkey();

    const MINT_LEN: usize = 82;

    let mint_rent = client.get_minimum_balance_for_rent_exemption(MINT_LEN)?;

    let create_mint_ix = system_instruction::create_account(
        &payer_pk,
        &mint_pubkey,
        mint_rent,
        MINT_LEN as u64,
        &TOKEN_PROGRAM_ID,
    );

    let init_mint_ix = token_instruction::initialize_mint(
        &TOKEN_PROGRAM_ID,
        &mint_pubkey,
        &payer_pk,
        None,
        decimals,
    )?;

    let blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[create_mint_ix, init_mint_ix],
        Some(&payer_pk),
        &[payer, &mint],
        blockhash,
    );

    let signature = client.send_and_confirm_transaction(&tx)?;
    Ok((mint_pubkey, signature))
}

/// Creates an associated token account (ATA) for the specified mint and owner.
pub fn create_ata(
    client: &RpcClient,
    payer: &Keypair,
    mint: &Pubkey,
    owner: &Pubkey,
) -> Result<(Pubkey, Signature)> {
    let payer_pk = payer.pubkey();
    let ata = spl_associated_token_account::get_associated_token_address(owner, mint);

    // Check if ATA already exists
    if let Ok(account) = client.get_account(&ata) {
        if account.owner == TOKEN_PROGRAM_ID {
            return Ok((ata, Signature::default()));
        }
    }

    let create_ata_ix = create_associated_token_account(&payer_pk, owner, mint, &TOKEN_PROGRAM_ID);

    let blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[create_ata_ix],
        Some(&payer_pk),
        &[payer],
        blockhash,
    );

    let signature = client.send_and_confirm_transaction(&tx)?;
    Ok((ata, signature))
}

/// Mints tokens to the specified token account.
pub fn mint_to(
    client: &RpcClient,
    payer: &Keypair,
    mint: &Pubkey,
    destination: &Pubkey,
    amount: u64,
) -> Result<Signature> {
    let payer_pk = payer.pubkey();

    let mint_to_ix = token_instruction::mint_to(
        &TOKEN_PROGRAM_ID,
        mint,
        destination,
        &payer_pk,
        &[],
        amount,
    )?;

    let blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[mint_to_ix],
        Some(&payer_pk),
        &[payer],
        blockhash,
    );

    let signature = client.send_and_confirm_transaction(&tx)?;
    Ok(signature)
}

/// Gets the decimals for a mint.
pub fn get_mint_decimals(client: &RpcClient, mint: &Pubkey) -> Result<u8> {
    let account = client.get_account(mint)?;
    let mint_data = Mint::unpack(&account.data)?;
    Ok(mint_data.decimals)
}
