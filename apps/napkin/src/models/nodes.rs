use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "nodes")]
pub struct Node {
    pub id: Option<String>,
    pub project: uuid::Uuid,
    pub title: String,
    pub data: serde_json::Value,
    // pub embedding: pgvector::Vector,
}

#[derive(Serialize, Deserialize)]
pub struct NodeReqObj {
    pub id: Option<String>,
    pub project: String,
    pub title: String,
    pub data: String,
    // pub embedding: Vec<f32>,
}

impl Node {
    pub fn to_update_str(&self) -> String {
        let update = "SET project = $project, title = $title, data = $data";
        let update = update.replace("$project", &self.project.to_string());
        let update = update.replace("$title", &self.title);
        let update = update.replace("$data", &self.data.to_string());

        // Convert Vector to slice then to string
        // let embedding_as_str = self
        //     .embedding
        //     .as_slice()
        //     .iter()
        //     .map(|f| f.to_string())
        //     .collect::<Vec<String>>()
        //     .join(",");
        // let update = update.replace("$embedding", &embedding_as_str);

        update
    }
}
