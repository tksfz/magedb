pub mod data;
pub mod storage;
pub mod cli;

use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = cli::Cli::parse();
    cli::execute(app).await
}
