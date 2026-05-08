use mti::prelude::*;

#[derive(Debug, Clone)]
pub struct EntityDefinitionId(pub MagicTypeId);

impl EntityDefinitionId {
    pub fn new() -> Self {
        Self("mage_entity".create_type_id::<V7>())
    }
}

#[derive(Debug, Clone)]
pub struct EntityDefinition {
    pub id: EntityDefinitionId,
    pub name: String,
    pub description: Option<String>,
    pub type_id_prefix: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_definition_id_new() {
        let id = EntityDefinitionId::new();
        let id_str = id.0.to_string();
        assert!(id_str.starts_with("mage_entity_"));
    }

}
