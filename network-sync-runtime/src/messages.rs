use serde::{self, Deserialize, Serialize};

use crate::types::ActorData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinSessionMessage {
    pub event_type: String,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaveSessionMessage {
    pub event_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMessage {
    pub event_type: String,
    pub sender_id: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorSyncMessage {
    pub event_type: String,
    pub sender_id: String,
    pub data: ActorData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredMessage {
    pub event_type: String,
    pub sender_id: String,
    pub message_id: String,
    pub data: Vec<u8>,
}

// But implement custom deserialization for ServerMessage
#[derive(Deserialize)]
#[serde(from = "MessageHelper")]
pub enum ServerMessage {
    Welcome(NetworkMessage),
    SessionMembers(NetworkMessage),
    ActorSync(ActorSyncMessage),
    RegisteredMessage(RegisteredMessage),
}

// Helper struct for deserialization
#[derive(Deserialize)]
struct MessageHelper {
    event_type: String,
    #[serde(flatten)]
    rest: serde_json::Value,
}

// Convert from helper to actual enum
impl From<MessageHelper> for ServerMessage {
    fn from(helper: MessageHelper) -> Self {
        // Combine the event_type with the rest of the fields
        let mut json_map = serde_json::Map::new();
        json_map.insert("event_type".to_string(), helper.event_type.clone().into());

        if let serde_json::Value::Object(rest_map) = helper.rest {
            json_map.extend(rest_map);
        }

        let json = serde_json::Value::Object(json_map);

        match helper.event_type.as_str() {
            "welcome" => ServerMessage::Welcome(serde_json::from_value(json).unwrap()),
            "session_members" => {
                ServerMessage::SessionMembers(serde_json::from_value(json).unwrap())
            }
            "actor_sync" => ServerMessage::ActorSync(serde_json::from_value(json).unwrap()),
            "registered_message" => {
                ServerMessage::RegisteredMessage(serde_json::from_value(json).unwrap())
            }
            _ => panic!("Unknown message type: {}", helper.event_type),
        }
    }
}
