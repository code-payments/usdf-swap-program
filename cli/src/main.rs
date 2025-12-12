mod keypair;

use clap::{Parser, Subcommand};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::Signature, signer::Signer, transaction::Transaction};
use std::path::PathBuf;
use std::str::FromStr;
use anyhow::Result;
use usdf_swap_api::prelude::*;
use usdf_swap_client::{get_pool, create_mint, create_ata, mint_to, get_mint_decimals};
use keypair::{get_keypair_path, get_payer, load_keypair};

#[derive(Debug, Clone)]
pub enum Cluster {
    Localnet,
    Mainnet,
    Devnet,
    Testnet,
    Custom(String),
}

impl Cluster {
    pub fn rpc_url(&self) -> String {
        match self {
            Cluster::Localnet => "http://127.0.0.1:8899".to_string(),
            Cluster::Mainnet => "https://api.mainnet-beta.solana.com".to_string(),
            Cluster::Devnet => "https://api.devnet.solana.com".to_string(),
            Cluster::Testnet => "https://api.testnet.solana.com".to_string(),
            Cluster::Custom(url) => url.clone(),
        }
    }
}

impl FromStr for Cluster {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "l" => Ok(Cluster::Localnet),
            "m" => Ok(Cluster::Mainnet),
            "d" => Ok(Cluster::Devnet),
            "t" => Ok(Cluster::Testnet),
            s if s.starts_with("http://") || s.starts_with("https://") => Ok(Cluster::Custom(s.to_string())),
            _ => Err(format!(
                "Invalid cluster value: '{}'. Use l, m, d, t, or a valid RPC URL (http:// or https://)",
                s
            )),
        }
    }
}

#[derive(Parser)]
#[command(name = "usdf-swap-cli")]
#[command(about = "CLI for interacting with the USDF Swap Solana program")]
struct Cli {
    #[arg(long, global = true, help = "Path to authority keypair file (default: ~/.config/solana/id.json)")]
    authority_keypair: Option<PathBuf>,

    #[arg(
        long,
        global = true,
        default_value = "l",
        help = "Solana cluster (l = localnet, m = mainnet, d = devnet, t = testnet, or a custom RPC URL)"
    )]
    cluster: Cluster,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a test mint with tokens minted to authority's ATA
    CreateTestMint {
        #[arg(long, default_value_t = 6, help = "Decimals for the mint")]
        decimals: u8,
    },

    /// Initialize a new swap pool for two stablecoins
    Initialize {
        #[arg(long, help = "Pool name (used as PDA seed)")]
        name: String,

        #[arg(long, help = "USDF stablecoin mint address")]
        usdf_mint: Pubkey,

        #[arg(long, help = "Other stablecoin mint address")]
        other_mint: Pubkey,
    },

    /// Get pool information
    GetPool {
        #[arg(long, help = "Pool address")]
        pool: Pubkey,
    },

    /// Swap tokens 1:1
    Swap {
        #[arg(long, help = "Pool address")]
        pool: Pubkey,

        #[arg(long, help = "Path to user keypair file")]
        user_keypair: PathBuf,

        #[arg(long, help = "Amount to swap (in tokens)")]
        amount: f64,

        #[arg(long, help = "Swap direction: USDF to other (true) or other to USDF (false)")]
        usdf_to_other: bool,
    },

    /// Transfer tokens from pool vault (authority only)
    Transfer {
        #[arg(long, help = "Pool address")]
        pool: Pubkey,

        #[arg(long, help = "Amount to transfer (in tokens)")]
        amount: f64,

        #[arg(long, help = "Transfer from USDF vault (true) or other vault (false)")]
        from_usdf_vault: bool,

        #[arg(long, help = "Destination token account")]
        destination: Pubkey,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = RpcClient::new(cli.cluster.rpc_url());
    let authority_keypair_path = get_keypair_path(cli.authority_keypair);
    let authority = get_payer(authority_keypair_path)?;

    match cli.command {
        Commands::CreateTestMint { decimals } => {
            let authority_pk = authority.pubkey();

            // Create mint
            let (mint, mint_signature) = create_mint(&client, &authority, decimals)?;
            println!("Mint created!");
            println!("  Mint: {}", mint);
            println!("  Signature: {}", mint_signature);

            // Create ATA for authority
            let (ata, ata_signature) = create_ata(&client, &authority, &mint, &authority_pk)?;
            println!("ATA created!");
            println!("  ATA: {}", ata);
            println!("  Signature: {}", ata_signature);

            // Mint 1,000,000 tokens (adjusted for decimals)
            let amount = 1_000_000u64 * 10u64.pow(decimals as u32);
            let mint_to_signature = mint_to(&client, &authority, &mint, &ata, amount)?;
            println!("Tokens minted!");
            println!("  Amount: {} (1,000,000 tokens)", amount);
            println!("  Signature: {}", mint_to_signature);
        }

        Commands::Initialize { name, usdf_mint, other_mint } => {
            let authority_pk = authority.pubkey();
            let (pool_pda, _) = find_pool_pda(&authority_pk, &name, &usdf_mint, &other_mint);

            let ix = build_initialize_ix(authority_pk, &name, usdf_mint, other_mint);
            let blockhash = client.get_latest_blockhash()?;
            let tx = Transaction::new_signed_with_payer(&[ix], Some(&authority_pk), &[&authority], blockhash);
            let signature = client.send_and_confirm_transaction(&tx)?;

            println!("Pool initialized successfully!");
            println!("  Pool Address: {}", pool_pda);
            println!("  Name: {}", name);
            println!("  USDF Mint: {}", usdf_mint);
            println!("  Other Mint: {}", other_mint);
            println!("  Signature: {}", signature);
        }

        Commands::GetPool { pool } => {
            let pool_data = get_pool(&client, &pool)?;

            println!("Pool Information:");
            println!("  Name: {}", from_name(&pool_data.name));
            println!("  Authority: {}", pool_data.authority);
            println!("  USDF Mint: {}", pool_data.usdf_mint);
            println!("  Other Mint: {}", pool_data.other_mint);
            println!("  USDF Vault: {}", pool_data.usdf_vault);
            println!("  Other Vault: {}", pool_data.other_vault);
        }

        Commands::Swap { pool, user_keypair, amount, usdf_to_other } => {
            let user = load_keypair(&user_keypair)?;
            let user_pk = user.pubkey();
            let pool_data = get_pool(&client, &pool)?;

            // Get decimals from the source mint to convert tokens to quarks
            let source_mint = if usdf_to_other { pool_data.usdf_mint } else { pool_data.other_mint };
            let decimals = get_mint_decimals(&client, &source_mint)?;
            let amount_quarks = (amount * 10f64.powi(decimals as i32)) as u64;

            // Derive and create ATAs for the user if they don't exist
            let (user_usdf_token, usdf_ata_sig) = create_ata(&client, &user, &pool_data.usdf_mint, &user_pk)?;
            if usdf_ata_sig != Signature::default() {
                println!("Created ATA for USDF mint: {}", user_usdf_token);
            }

            let (user_other_token, other_ata_sig) = create_ata(&client, &user, &pool_data.other_mint, &user_pk)?;
            if other_ata_sig != Signature::default() {
                println!("Created ATA for other mint: {}", user_other_token);
            }

            let ix = build_swap_ix(
                user_pk,
                pool,
                pool_data.usdf_vault,
                pool_data.other_vault,
                user_usdf_token,
                user_other_token,
                amount_quarks,
                usdf_to_other,
            );
            let blockhash = client.get_latest_blockhash()?;
            let tx = Transaction::new_signed_with_payer(&[ix], Some(&user_pk), &[&user], blockhash);
            let signature = client.send_and_confirm_transaction(&tx)?;

            println!("Swap successful!");
            println!("  User: {}", user_pk);
            println!("  Direction: {}", if usdf_to_other { "USDF to Other" } else { "Other to USDF" });
            println!("  Amount: {} tokens ({} quarks)", amount, amount_quarks);
            println!("  Signature: {}", signature);
        }

        Commands::Transfer { pool, amount, from_usdf_vault, destination } => {
            let authority_pk = authority.pubkey();
            let pool_data = get_pool(&client, &pool)?;

            // Get decimals from the source mint to convert tokens to quarks
            let source_mint = if from_usdf_vault { pool_data.usdf_mint } else { pool_data.other_mint };
            let decimals = get_mint_decimals(&client, &source_mint)?;
            let amount_quarks = (amount * 10f64.powi(decimals as i32)) as u64;

            let ix = build_transfer_ix(
                authority_pk,
                pool,
                pool_data.usdf_mint,
                pool_data.other_mint,
                destination,
                amount_quarks,
                from_usdf_vault,
            );
            let blockhash = client.get_latest_blockhash()?;
            let tx = Transaction::new_signed_with_payer(&[ix], Some(&authority_pk), &[&authority], blockhash);
            let signature = client.send_and_confirm_transaction(&tx)?;

            println!("Transfer successful!");
            println!("  Amount: {} tokens ({} quarks)", amount, amount_quarks);
            println!("  From Vault: {}", if from_usdf_vault { "USDF" } else { "Other" });
            println!("  Destination: {}", destination);
            println!("  Signature: {}", signature);
        }
    }

    Ok(())
}
