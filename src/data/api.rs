use crate::storage::Storage;
use crate::data::{EntityData, EntityDefinition, EntityDefinitionId, TypeId};
use serde_json::Value;

pub struct Api<S: Storage> {
    storage: S,
}

impl<S: Storage> Api<S> {
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    pub async fn add_entity_definition(&self, blob: &str) -> anyhow::Result<()> {
        let value: Value = serde_json::from_str(blob)?;
        
        let name = value.get("name").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'name' field in JSON payload"))?;
        
        let prefix = value.get("prefix").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'prefix' field in JSON payload"))?;
            
        let description = value.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
        
        let definition = EntityDefinition {
            id: EntityDefinitionId::new(),
            name: name.to_string(),
            description,
            type_id_prefix: prefix.to_string(),
        };
        
        self.storage.insert_entity_definition(&definition).await?;
        Ok(())
    }

    pub async fn list_entity_definitions(&self) -> anyhow::Result<Vec<Value>> {
        let defs = self.storage.get_all_entity_definitions().await?;
        let mut result = Vec::new();
        for def in defs {
            let mut obj = serde_json::Map::new();
            obj.insert("id".to_string(), Value::String(def.id.0.to_string()));
            obj.insert("name".to_string(), Value::String(def.name.clone()));
            if let Some(desc) = &def.description {
                obj.insert("description".to_string(), Value::String(desc.clone()));
            }
            obj.insert("prefix".to_string(), Value::String(def.type_id_prefix.clone()));
            result.push(Value::Object(obj));
        }
        Ok(result)
    }

    pub async fn put_entity_data(&self, blob: &str) -> anyhow::Result<()> {
        let value: Value = serde_json::from_str(blob)?;
        
        let id_str = value.get("id").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'id' field in JSON payload"))?;
        let type_id = id_str.parse::<TypeId>()?;
        
        // Extract prefix using the built-in prefix method from mti crate
        let prefix = type_id.prefix();
        
        let entity_definition = self.storage.get_entity_definition_by_prefix(prefix).await?
            .ok_or_else(|| anyhow::anyhow!("Unknown entity type prefix: {}", prefix))?;
        
        let data = EntityData {
            id: type_id,
            entity_definition_id: entity_definition.id,
            data: value,
        };
        
        self.storage.insert_entity_data(&data).await?;
        Ok(())
    }

    pub async fn get_entity_data(&self, id: &str) -> anyhow::Result<Option<Value>> {
        let type_id = id.parse::<TypeId>()?;
        match self.storage.get_entity_data(&type_id).await? {
            Some(data) => Ok(Some(data.data)),
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
        definitions: Arc<Mutex<Vec<EntityDefinition>>>,
    }

    impl MockStorage {
        fn new() -> Self {
            Self {
                data: Arc::new(Mutex::new(HashMap::new())),
                definitions: Arc::new(Mutex::new(Vec::new())),
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

        async fn insert_entity_definition(&self, definition: &EntityDefinition) -> Result<(), StorageError> {
            self.definitions.lock().unwrap().push(definition.clone());
            Ok(())
        }

        async fn get_entity_definition(&self, id: &EntityDefinitionId) -> Result<Option<EntityDefinition>, StorageError> {
            let defs = self.definitions.lock().unwrap();
            Ok(defs.iter().find(|d| d.id.0 == id.0).cloned())
        }

        async fn get_entity_definition_by_prefix(&self, prefix: &str) -> Result<Option<EntityDefinition>, StorageError> {
            let defs = self.definitions.lock().unwrap();
            Ok(defs.iter().find(|d| d.type_id_prefix == prefix).cloned())
        }

        async fn get_all_entity_definitions(&self) -> Result<Vec<EntityDefinition>, StorageError> {
            Ok(self.definitions.lock().unwrap().clone())
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
    async fn test_add_entity_definition() {
        let storage = MockStorage::new();
        let api = Api::new(storage.clone());
        
        let payload = r#"{
            "name": "Product",
            "description": "An item for sale",
            "prefix": "prd"
        }"#;
        
        let result = api.add_entity_definition(payload).await;
        assert!(result.is_ok(), "Failed to add entity definition: {:?}", result.err());
        
        // Verify it was saved to storage
        let def = storage.get_entity_definition_by_prefix("prd").await.unwrap();
        assert!(def.is_some());
        let def = def.unwrap();
        assert_eq!(def.name, "Product");
        assert_eq!(def.description, Some("An item for sale".to_string()));
        assert_eq!(def.type_id_prefix, "prd");
    }

    #[tokio::test]
    async fn test_list_entity_definitions() {
        let storage = MockStorage::new();
        let api = Api::new(storage.clone());
        
        let def_payload1 = r#"{
            "name": "Product",
            "description": "An item for sale",
            "prefix": "prd"
        }"#;
        let def_payload2 = r#"{
            "name": "User",
            "prefix": "usr"
        }"#;
        
        api.add_entity_definition(def_payload1).await.unwrap();
        api.add_entity_definition(def_payload2).await.unwrap();
        
        let defs = api.list_entity_definitions().await.unwrap();
        assert_eq!(defs.len(), 2);
        
        let mut prefixes: Vec<String> = defs.iter()
            .map(|v| v["prefix"].as_str().unwrap().to_string())
            .collect();
        prefixes.sort();
        
        assert_eq!(prefixes, vec!["prd".to_string(), "usr".to_string()]);
    }

    #[tokio::test]
    async fn test_put_and_get_entity_data() {
        let storage = MockStorage::new();
        let api = Api::new(storage.clone());
        
        let def_payload = r#"{
            "name": "User",
            "description": "System user",
            "prefix": "usr"
        }"#;
        
        api.add_entity_definition(def_payload).await.unwrap();
        
        let type_id = "usr".create_type_id::<V7>();
        
        let payload = format!(r#"{{
            "id": "{}",
            "name": "Thom",
            "level": 42
        }}"#, type_id);
        
        let result = api.put_entity_data(&payload).await;
        assert!(result.is_ok(), "Failed to put entity data: {:?}", result.err());
        
        let result = api.get_entity_data(&type_id.to_string()).await;
        assert!(result.is_ok());
        let json_opt = result.unwrap();
        assert!(json_opt.is_some());
        
        let json = json_opt.unwrap();
        assert_eq!(json["id"], type_id.to_string());
        assert_eq!(json["name"], "Thom");
        assert_eq!(json["level"], 42);
    }
    
    #[tokio::test]
    async fn test_put_missing_id() {
        let storage = MockStorage::new();
        let api = Api::new(storage);
        
        let payload = r#"{
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
    #[tokio::test]
    async fn test_put_unrecognized_prefix() {
        let storage = MockStorage::new();
        let api = Api::new(storage);
        
        let type_id = "unk".create_type_id::<V7>();
        
        let payload = format!(r#"{{
            "id": "{}",
            "name": "Thom"
        }}"#, type_id);
        
        let result = api.put_entity_data(&payload).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown entity type prefix: unk"));
    }
}
