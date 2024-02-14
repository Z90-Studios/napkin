use serde::{Deserialize, Serialize};
use super::panel_type::PanelType;

pub type Title = String;

#[derive(Serialize, Deserialize)]
pub struct PanelTab {
    pub title: Title,
    pub panel_type: PanelType,
    pub text: Option<String>,
    // More options to come
}

impl Default for PanelTab {
    fn default() -> Self {
        Self {
            title: "".to_owned(),
            panel_type: PanelType::Text,
            text: None,
        }
    }
}