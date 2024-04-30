use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "nodes")]
pub struct Node {
    pub id: Option<String>,
    pub project: uuid::Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct NodeReqObj {
    pub id: Option<String>,
    pub project: String,
}

impl Node {
    pub fn to_update_str(&self) -> String {
        let update = "SET project = $project";
        let update = update.replace("$project", &self.project.to_string());

        update
    }
}
