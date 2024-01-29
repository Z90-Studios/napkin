use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "edge_metadata")]
pub struct EdgeMetadata {
    pub owner_id: uuid::Uuid,
    pub name: String,
    pub value: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct EdgeMetadataReqObj {
    pub owner_id: String,
    pub name: String,
    pub value: serde_json::Value,
}

impl EdgeMetadata {
    pub fn to_update_str(&self) -> String {
        let update = "SET owner_id = $owner_id, name = $name, value = $value";
        let update = update.replace("$owner_id", &self.owner_id.to_string());
        let update = update.replace("$name", &self.name);
        let update = update.replace("$value", &self.value.to_string());

        update
    }
}