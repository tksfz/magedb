use crate::storage::Storage;
use crate::data::{EntityData, EntityDefinitionId, TypeId};
use serde_json::Value;

pub struct Api<S: Storage> {
    storage: S,
}

impl<S: Storage> Api<S> {
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    pub async fn put_entity_data(&self, blob: &str) -> anyhow::Result<()> {
        let value: Value = serde_json::from_str(blob)?;
        
        let id_str = value.get("id").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'id' field in JSON payload"))?;
        let type_id = id_str.parse::<TypeId>()?;
        
        let entity_def_id_str = value.get("entity_definition_id").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'entity_definition_id' field in JSON payload"))?;
        let entity_def_type_id = entity_def_id_str.parse::<mti::prelude::MagicTypeId>()?;
        let entity_definition_id = EntityDefinitionId(entity_def_type_id);
        
        let data = EntityData {
            id: type_id,
            entity_definition_id,
            data: value,
        };
        
        self.storage.insert_entity_data(&data).await?;
        Ok(())
    }

    pub async fn get_entity_data(&self, id: &str) -> anyhow::Result<Option<Value>> {
        let type_id = id.parse::<TypeId>()?;
        match self.storage.get_entity_data(&type_id).await? {
            Some(data) => {
                let json = serde_json::json!({
                    "id": data.id.to_string(),
                    "entity_definition_id": data.entity_definition_id.0.to_string(),
                    "data": data.data
                });
                Ok(Some(json))
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::StorageError;
    use crate::data::EntityDefinition;
    use std::sync::{Arc, Mutex};
    use std::collections::HashMap;
    use mti::prelude::*;

    #[derive(Clone)]
    struct MockStorage {
        data: Arc<Mutex<HashMap<TypeId, EntityData>>>,
    }

    impl MockStorage {
        fn new() -> Self {
            Self {
                data: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    impl Storage for MockStorage {
        async fn create_database(_path: &str) -> Result<Self, StorageError> {
            Ok(Self::new())
        }

        async fn open_database(_path: &str) -> Result<Self, StorageError> {
            Ok(Self::new())
        }

        async fn insert_entity_definition(&self, _definition: &EntityDefinition) -> Result<(), StorageError> {
            Ok(())
        }

        async fn get_entity_definition(&self, _id: &EntityDefinitionId) -> Result<Option<EntityDefinition>, StorageError> {
            Ok(None)
        }

        async fn get_all_entity_definitions(&self) -> Result<Vec<EntityDefinition>, StorageError> {
            Ok(vec![])
        }

        async fn insert_entity_data(&self, data: &EntityData) -> Result<(), StorageError> {
            self.data.lock().unwrap().insert(data.id.clone(), data.clone());
            Ok(())
        }

        async fn get_entity_data(&self, id: &TypeId) -> Result<Option<EntityData>, StorageError> {
            Ok(self.data.lock().unwrap().get(id).cloned())
        }
    }

    #[tokio::test]
    async fn test_put_and_get_entity_data() {
        let storage = MockStorage::new();
        let api = Api::new(storage);
        
        let type_id = "usr".create_type_id::<V7>();
        let ent_def_id = "ent".create_type_id::<V7>();
        
        let payload = format!(r#"{{
            "id": "{}",
            "entity_definition_id": "{}",
            "name": "Thom",
            "level": 42
        }}"#, type_id, ent_def_id);
        
        let result = api.put_entity_data(&payload).await;
        assert!(result.is_ok(), "Failed to put entity data: {:?}", result.err());
        
        let result = api.get_entity_data(&type_id.to_string()).await;
        assert!(result.is_ok());
        let json_opt = result.unwrap();
        assert!(json_opt.is_some());
        
        let json = json_opt.unwrap();
        assert_eq!(json["id"], type_id.to_string());
        assert_eq!(json["entity_definition_id"], ent_def_id.to_string());
        assert_eq!(json["data"]["name"], "Thom");
        assert_eq!(json["data"]["level"], 42);
    }
    
    #[tokio::test]
    async fn test_put_missing_id() {
        let storage = MockStorage::new();
        let api = Api::new(storage);
        
        let payload = r#"{
            "entity_definition_id": "ent_123",
            "name": "Thom"
        }"#;
        
        let result = api.put_entity_data(payload).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing 'id' field"));
    }
    
    #[tokio::test]
    async fn test_get_not_found() {
        let storage = MockStorage::new();
        let api = Api::new(storage);
        
        let type_id = "usr".create_type_id::<V7>();
        
        let result = api.get_entity_data(&type_id.to_string()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
