use mti::prelude::*;

#[derive(Debug, Clone)]
pub struct EntityDefinitionId(pub MagicTypeId);

impl EntityDefinitionId {
    pub fn new() -> Self {
        Self("ent".create_type_id::<V7>())
    }
}

#[derive(Debug, Clone)]
pub struct EntityDefinition {
    pub id: EntityDefinitionId,
    pub type_id_prefix: String,
}
