use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct NapkinService {
    pub host: String,
    pub port: String,
}