use clap::Parser;
use client::{run, Cli};

mod client;
pub mod integrations;

#[tokio::main]
async fn main() {
    todo!("Update Client to support all circuits!");
    let cli: Cli = Cli::parse();
    run(cli).await;
}
