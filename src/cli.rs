use clap::{Parser, Subcommand, CommandFactory};
use crate::storage::{Storage, turso::TursoStorage};
use crate::data::Api;

#[derive(serde::Serialize, serde::Deserialize)]
struct State {
    db_path: String,
}

fn save_state(path: &str) -> anyhow::Result<()> {
    let state = State { db_path: path.to_string() };
    std::fs::write(".mage-state", serde_json::to_string(&state)?)?;
    Ok(())
}

fn load_state() -> anyhow::Result<State> {
    let content = std::fs::read_to_string(".mage-state")
        .map_err(|_| anyhow::anyhow!("No database opened. Please run 'mage schema open <file>' first."))?;
    let state: State = serde_json::from_str(&content)?;
    Ok(state)
}

#[derive(Parser)]
#[command(name = "mage")]
#[command(about = "Magedb CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Schema management
    Schema {
        #[command(subcommand)]
        command: SchemaCommands,
    },
    /// Entity definition management
    Entities {
        #[command(subcommand)]
        command: EntitiesCommands,
    },
    /// Entity data management
    EntityData {
        #[command(subcommand)]
        command: EntityDataCommands,
    },
}

#[derive(Subcommand)]
pub enum EntitiesCommands {
    /// Add a new entity definition from a JSON blob
    Add {
        /// JSON blob containing Entity Definition (name, prefix, [description])
        blob: String,
        /// Database file path (overrides currently open database)
        #[arg(short, long)]
        dbfile: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum EntityDataCommands {
    /// Put entity data from a JSON blob
    Put {
        /// JSON blob containing EntityData
        blob: String,
        /// Database file path (overrides currently open database)
        #[arg(short, long)]
        dbfile: Option<String>,
    },
    /// Get entity data by ID
    Get {
        /// Entity ID
        id: String,
        /// Database file path (overrides currently open database)
        #[arg(short, long)]
        dbfile: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum SchemaCommands {
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

pub async fn execute(cli: Cli) -> anyhow::Result<()> {
    match &cli.command {
        Commands::Schema { command } => match command {
            SchemaCommands::Create { file } => {
                println!("Creating database at {}...", file);
                let _storage = TursoStorage::create_database(file).await?;
                save_state(file)?;
                println!("Database successfully created and set as active!");
            }
            SchemaCommands::Open { file } => {
                println!("Opening database at {}...", file);
                let _storage = TursoStorage::open_database(file).await?;
                save_state(file)?;
                println!("Database successfully opened and set as active!");
            }
        },
        Commands::Entities { command } => match command {
            EntitiesCommands::Add { blob, dbfile } => {
                let db_path = match dbfile {
                    Some(p) => p.clone(),
                    None => load_state()?.db_path,
                };
                println!("Opening database at {}...", db_path);
                let storage = TursoStorage::open_database(&db_path).await?;
                let api = Api::new(storage);
                println!("Adding entity definition...");
                api.add_entity_definition(blob).await?;
                println!("Entity definition successfully added!");
            }
        },
        Commands::EntityData { command } => match command {
            EntityDataCommands::Put { blob, dbfile } => {
                let db_path = match dbfile {
                    Some(p) => p.clone(),
                    None => load_state()?.db_path,
                };
                println!("Opening database at {}...", db_path);
                let storage = TursoStorage::open_database(&db_path).await?;
                let api = Api::new(storage);
                println!("Putting entity data...");
                api.put_entity_data(blob).await?;
                println!("Entity data successfully stored!");
            }
            EntityDataCommands::Get { id, dbfile } => {
                let db_path = match dbfile {
                    Some(p) => p.clone(),
                    None => load_state()?.db_path,
                };
                println!("Opening database at {}...", db_path);
                let storage = TursoStorage::open_database(&db_path).await?;
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

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }
}
