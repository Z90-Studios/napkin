use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
}