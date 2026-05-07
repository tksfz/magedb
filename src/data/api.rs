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
