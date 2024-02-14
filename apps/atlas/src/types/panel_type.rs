use serde::{Deserialize, Serialize};

use super::chat_history::ChatHistory;

#[derive(Serialize, Deserialize)]
pub enum PanelType {
    Text,
    Chat {
        history: Vec<ChatHistory>,
        row_sizes: Vec<f32>,
    },
    Graph,
    Settings,
}