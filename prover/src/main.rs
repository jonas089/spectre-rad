use clap::Parser;
use client::{run, Cli};

mod client;
pub mod integrations;

#[tokio::main]
async fn main() {
    todo!("Update Client to support all circuits!");
    /*let cli: Cli = Cli::parse();
    run(cli).await;*/
}

// Scenario A: Finality step update => only requires step circuit (with uncompressed keys)
// Scenario B: Committee update => requires both the committee update circuit and step circuit (with compressed and uncompressed keys)
