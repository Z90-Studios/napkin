use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Project {
  pub id: Option<uuid::Uuid>,
  pub project: uuid::Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct Node {
  pub id: Option<uuid::Uuid>,
  pub project: uuid::Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct Edge {
  pub id: Option<uuid::Uuid>,
  pub project: uuid::Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct NodeMetadata {
  pub owner_id: uuid::Uuid,
  pub name: String,
  pub value: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct EdgeMetadata {
  pub owner_id: uuid::Uuid,
  pub name: String,
  pub value: serde_json::Value,
}