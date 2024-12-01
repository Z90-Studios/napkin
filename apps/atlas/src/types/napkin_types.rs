use std::fmt;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct NapkinProject {
    pub id: String,
    pub scope: String,
    pub name: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct NapkinNode {
    pub project: String,
    pub id: String,
}

impl fmt::Display for NapkinNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "NapkinNode {{ project: {}, id: {} }}",
            self.project, self.id
        )
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct NapkinEdge {
    pub project: String,
    pub id: String,
    pub source: String,
    pub target: String,
}

impl fmt::Display for NapkinEdge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "NapkinEdge {{ project: {}, id: {}, source: {}, target: {} }}",
            self.project, self.id, self.source, self.target
        )
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct NapkinNodeMetadata {
    pub owner_id: String,
    pub name: String,
    pub value: serde_json::Value,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct NapkinEdgeMetadata {
    pub owner_id: String,
    pub name: String,
    pub value: serde_json::Value,
}
