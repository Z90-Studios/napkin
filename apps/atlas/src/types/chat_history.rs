use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ChatHistory {
    /** Used to handle multiple chats */
    pub instance: u32,
    /** Visual user representation */
    pub user: String,
    /** Message text */
    pub message: String,
    /** Model used, if applicable */
    pub model: Option<String>,
    /** Timestamp of message */
    pub timestamp: String,
}