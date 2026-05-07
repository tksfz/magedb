pub mod data;
pub mod storage;
use clap::{Parser, Subcommand};
use crate::storage::{Storage, turso::TursoStorage};

#[derive(Parser)]
#[command(name = "mage")]
#[command(about = "Magedb CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Schema management
    Schema {
        #[command(subcommand)]
        command: SchemaCommands,
    },
}

#[derive(Subcommand)]
enum SchemaCommands {
    /// Create a new database schema
    Create {
        /// Database file path
        #[arg(default_value = "mage.db")]
        file: String,
    },
    /// Open an existing database schema
    Open {
        /// Database file path
        #[arg(default_value = "mage.db")]
        file: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Schema { command } => match command {
            SchemaCommands::Create { file } => {
                println!("Creating database at {}...", file);
                let _storage = TursoStorage::create_database(file).await?;
                println!("Database successfully created!");
            }
            SchemaCommands::Open { file } => {
                println!("Opening database at {}...", file);
                let _storage = TursoStorage::open_database(file).await?;
                println!("Database successfully opened!");
            }
        },
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }
}
