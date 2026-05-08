use crate::data::{EntityData, EntityDefinition, EntityDefinitionId, TypeId};
use std::fmt;

#[derive(Debug)]
pub enum StorageError {
    ConnectionError(String),
    QueryError(String),
    NotFound(String),
    SerializationError(String),
    Other(String),
}

impl std::error::Error for StorageError {}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::ConnectionError(msg) => write!(f, "Connection Error: {}", msg),
            StorageError::QueryError(msg) => write!(f, "Query Error: {}", msg),
            StorageError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            StorageError::SerializationError(msg) => write!(f, "Serialization Error: {}", msg),
            StorageError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

pub mod turso;

pub trait Storage {
    // Note: Rust 1.75+ (and Edition 2024) natively support async fn in traits
    
    // Database lifecycle
    async fn create_database(path: &str) -> Result<Self, StorageError> where Self: Sized;
    async fn open_database(path: &str) -> Result<Self, StorageError> where Self: Sized;

    // EntityDefinition operations
    async fn insert_entity_definition(&self, definition: &EntityDefinition) -> Result<(), StorageError>;
    async fn get_entity_definition(&self, id: &EntityDefinitionId) -> Result<Option<EntityDefinition>, StorageError>;
    async fn get_entity_definition_by_prefix(&self, prefix: &str) -> Result<Option<EntityDefinition>, StorageError>;
    async fn get_entity_definition_by_name(&self, name: &str) -> Result<Option<EntityDefinition>, StorageError>;
    async fn get_all_entity_definitions(&self) -> Result<Vec<EntityDefinition>, StorageError>;
    
    // EntityData operations
    async fn insert_entity_data(&self, data: &EntityData) -> Result<(), StorageError>;
    async fn get_entity_data(&self, id: &TypeId) -> Result<Option<EntityData>, StorageError>;
}
