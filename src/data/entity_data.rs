use mti::prelude::*;
use serde_json::Value;

use super::{EntityDefinition, EntityDefinitionId, TypeId};

#[derive(Debug, Clone)]
pub struct EntityData {
    pub id: TypeId,
    pub entity_definition_id: EntityDefinitionId,
    pub data: Value,
}

impl EntityData {
    pub fn new(definition: &EntityDefinition, data: Value) -> Self {
        Self {
            id: definition.type_id_prefix.as_str().create_type_id::<V7>(),
            entity_definition_id: definition.id.clone(),
            data,
        }
    }
}
