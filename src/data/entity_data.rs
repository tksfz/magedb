use serde_json::Value;

use super::{EntityDefinitionId, TypeId};

#[derive(Debug, Clone)]
pub struct EntityData {
    pub id: TypeId,
    pub entity_definition_id: EntityDefinitionId,
    pub data: Value,
}
