use serde::{Deserialize, Serialize};
use super::napkin_service::NapkinService;

#[derive(Clone, Serialize, Deserialize)]
pub struct NapkinSettings {
    pub model: String,
    pub service: NapkinService,
}

impl NapkinSettings {
  pub fn default() -> Self {
      Self {
          model: "mistral".to_owned(),
          service: NapkinService {
              host: "localhost".to_owned(),
              port: "11434".to_owned(),
          },
      }
  }
}