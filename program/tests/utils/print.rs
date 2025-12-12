#![cfg(test)]
use litesvm::types::TransactionMetadata;
use solana_sdk::transaction::Transaction;

pub fn print_tx(meta: TransactionMetadata, tx: Transaction) {
    println!("\n========================================");
    println!("Transaction");
    println!("========================================");
    println!("Signatures: {:?}", tx.signatures);
    println!("Compute units: {}", meta.compute_units_consumed);
    for log in meta.logs {
        println!("  {}", log);
    }
    println!("========================================\n");
}
