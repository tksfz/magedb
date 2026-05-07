pub mod data;
pub mod storage;
use clap::{Parser, Subcommand};
use crate::storage::{Storage, turso::TursoStorage};
use crate::data::Api;

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
    /// Entity data management
    EntityData {
        #[command(subcommand)]
        command: EntityDataCommands,
    },
}

#[derive(Subcommand)]
enum EntityDataCommands {
    /// Put entity data from a JSON blob
    Put {
        /// JSON blob containing EntityData
        blob: String,
        /// Database file path
        #[arg(default_value = "mage.db")]
        file: String,
    },
    /// Get entity data by ID
    Get {
        /// Entity ID
        id: String,
        /// Database file path
        #[arg(default_value = "mage.db")]
        file: String,
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
        Commands::EntityData { command } => match command {
            EntityDataCommands::Put { blob, file } => {
                println!("Opening database at {}...", file);
                let storage = TursoStorage::open_database(file).await?;
                let api = Api::new(storage);
                println!("Putting entity data...");
                api.put_entity_data(blob).await?;
                println!("Entity data successfully stored!");
            }
            EntityDataCommands::Get { id, file } => {
                println!("Opening database at {}...", file);
                let storage = TursoStorage::open_database(file).await?;
                let api = Api::new(storage);
                match api.get_entity_data(id).await? {
                    Some(json) => {
                        println!("{}", serde_json::to_string_pretty(&json)?);
                    }
                    None => {
                        println!("Entity data not found for id: {}", id);
                    }
                }
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
