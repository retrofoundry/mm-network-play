use serde::{Deserialize, Serialize};

use crate::types::PlayerData;

// Simple message format for our limited functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMessage {
    pub event_type: String,
    pub player_id: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinSessionMessage {
    pub command: String,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaveSessionMessage {
    pub command: String,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSyncMessage {
    pub command: String,
    pub session_id: String,
    pub player_id: String,
    pub data: PlayerData,
}
