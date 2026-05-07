use crate::data::{EntityData, EntityDefinition, EntityDefinitionId, TypeId};
use crate::storage::{Storage, StorageError};
use std::str::FromStr;
use turso::Builder;

pub struct TursoStorage {
    conn: turso::Connection,
}

impl TursoStorage {
    async fn init_tables(&self) -> Result<(), StorageError> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS entity_definitions (
                id TEXT PRIMARY KEY,
                type_id_prefix TEXT NOT NULL
            )",
            (),
        ).await.map_err(|e| StorageError::QueryError(e.to_string()))?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS entity_data (
                id TEXT PRIMARY KEY,
                entity_definition_id TEXT NOT NULL,
                data TEXT NOT NULL
            )",
            (),
        ).await.map_err(|e| StorageError::QueryError(e.to_string()))?;

        Ok(())
    }
}

impl Storage for TursoStorage {
    async fn create_database(path: &str) -> Result<Self, StorageError> {
        let db = Builder::new_local(path).build().await.map_err(|e| StorageError::ConnectionError(e.to_string()))?;
        let conn = db.connect().map_err(|e| StorageError::ConnectionError(e.to_string()))?;
        let storage = Self { conn };
        storage.init_tables().await?;
        Ok(storage)
    }

    async fn open_database(path: &str) -> Result<Self, StorageError> {
        let db = Builder::new_local(path).build().await.map_err(|e| StorageError::ConnectionError(e.to_string()))?;
        let conn = db.connect().map_err(|e| StorageError::ConnectionError(e.to_string()))?;
        Ok(Self { conn })
    }

    async fn insert_entity_definition(&self, definition: &EntityDefinition) -> Result<(), StorageError> {
        self.conn.execute(
            "INSERT INTO entity_definitions (id, type_id_prefix) VALUES (?1, ?2)",
            (definition.id.0.to_string(), definition.type_id_prefix.clone()),
        ).await.map_err(|e| StorageError::QueryError(e.to_string()))?;
        Ok(())
    }

    async fn get_entity_definition(&self, id: &EntityDefinitionId) -> Result<Option<EntityDefinition>, StorageError> {
        let mut rows = self.conn.query(
            "SELECT id, type_id_prefix FROM entity_definitions WHERE id = ?1",
            (id.0.to_string(),),
        ).await.map_err(|e| StorageError::QueryError(e.to_string()))?;

        if let Some(row) = rows.next().await.map_err(|e| StorageError::QueryError(e.to_string()))? {
            let id_str: String = row.get(0).map_err(|e| StorageError::QueryError(e.to_string()))?;
            let type_id_prefix: String = row.get(1).map_err(|e| StorageError::QueryError(e.to_string()))?;
            
            let magic_id = TypeId::from_str(&id_str).map_err(|e| StorageError::SerializationError(e.to_string()))?;
            
            Ok(Some(EntityDefinition {
                id: EntityDefinitionId(magic_id),
                type_id_prefix,
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_all_entity_definitions(&self) -> Result<Vec<EntityDefinition>, StorageError> {
        let mut rows = self.conn.query(
            "SELECT id, type_id_prefix FROM entity_definitions",
            (),
        ).await.map_err(|e| StorageError::QueryError(e.to_string()))?;

        let mut definitions = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| StorageError::QueryError(e.to_string()))? {
            let id_str: String = row.get(0).map_err(|e| StorageError::QueryError(e.to_string()))?;
            let type_id_prefix: String = row.get(1).map_err(|e| StorageError::QueryError(e.to_string()))?;
            
            let magic_id = TypeId::from_str(&id_str).map_err(|e| StorageError::SerializationError(e.to_string()))?;
            
            definitions.push(EntityDefinition {
                id: EntityDefinitionId(magic_id),
                type_id_prefix,
            });
        }
        Ok(definitions)
    }

    async fn insert_entity_data(&self, data: &EntityData) -> Result<(), StorageError> {
        let json_data = serde_json::to_string(&data.data).map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        self.conn.execute(
            "INSERT INTO entity_data (id, entity_definition_id, data) VALUES (?1, ?2, ?3)",
            (data.id.to_string(), data.entity_definition_id.0.to_string(), json_data),
        ).await.map_err(|e| StorageError::QueryError(e.to_string()))?;
        Ok(())
    }

    async fn get_entity_data(&self, id: &TypeId) -> Result<Option<EntityData>, StorageError> {
        let mut rows = self.conn.query(
            "SELECT id, entity_definition_id, data FROM entity_data WHERE id = ?1",
            (id.to_string(),),
        ).await.map_err(|e| StorageError::QueryError(e.to_string()))?;

        if let Some(row) = rows.next().await.map_err(|e| StorageError::QueryError(e.to_string()))? {
            let id_str: String = row.get(0).map_err(|e| StorageError::QueryError(e.to_string()))?;
            let def_id_str: String = row.get(1).map_err(|e| StorageError::QueryError(e.to_string()))?;
            let data_str: String = row.get(2).map_err(|e| StorageError::QueryError(e.to_string()))?;
            
            let id = TypeId::from_str(&id_str).map_err(|e| StorageError::SerializationError(e.to_string()))?;
            let def_id = TypeId::from_str(&def_id_str).map_err(|e| StorageError::SerializationError(e.to_string()))?;
            let data: serde_json::Value = serde_json::from_str(&data_str).map_err(|e| StorageError::SerializationError(e.to_string()))?;
            
            Ok(Some(EntityData {
                id,
                entity_definition_id: EntityDefinitionId(def_id),
                data,
            }))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mti::prelude::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_turso_storage_lifecycle() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let path = db_path.to_str().unwrap();

        // Test create_database
        let storage = TursoStorage::create_database(path).await.expect("Failed to create database");
        
        // Test insert and get entity definition
        let def = EntityDefinition {
            id: EntityDefinitionId::new(),
            type_id_prefix: "usr".to_string(),
        };
        storage.insert_entity_definition(&def).await.expect("Failed to insert definition");

        let retrieved_def = storage.get_entity_definition(&def.id).await.expect("Failed to get definition").expect("Definition not found");
        assert_eq!(def.id.0.to_string(), retrieved_def.id.0.to_string());
        assert_eq!(def.type_id_prefix, retrieved_def.type_id_prefix);

        let all_defs = storage.get_all_entity_definitions().await.expect("Failed to get all definitions");
        assert_eq!(all_defs.len(), 1);

        // Test insert and get entity data
        let data = EntityData {
            id: "usr".create_type_id::<V7>(),
            entity_definition_id: def.id.clone(),
            data: json!({"name": "Test User", "age": 30}),
        };
        storage.insert_entity_data(&data).await.expect("Failed to insert data");

        let retrieved_data = storage.get_entity_data(&data.id).await.expect("Failed to get data").expect("Data not found");
        assert_eq!(data.id.to_string(), retrieved_data.id.to_string());
        assert_eq!(data.entity_definition_id.0.to_string(), retrieved_data.entity_definition_id.0.to_string());
        assert_eq!(data.data, retrieved_data.data);
    }
}
