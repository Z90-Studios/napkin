use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "edges")]
pub struct Edge {
    pub id: Option<String>,
    pub project: uuid::Uuid,
    pub source: uuid::Uuid,
    pub target: uuid::Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct EdgeReqObj {
    pub id: Option<String>,
    pub project: String,
    pub source: String,
    pub target: String,
}

impl Edge {
    pub fn to_update_str(&self) -> String {
        let update = "SET project = $project, source = $source, target = $target";
        let update = update.replace("$project", &self.project.to_string());
        let update = update.replace("$source", &self.source.to_string());
        let update = update.replace("$target", &self.target.to_string());

        update
    }
}