use anyhow::Result;
use gamecore::network::NetworkModule;
use serde::{Deserialize, Serialize};
use serde_json;
use std::panic;
use std::sync::{Arc, Mutex, OnceLock};

// Global singleton instance of the NetworkPlay module
pub static NETWORK_PLAY: OnceLock<Arc<Mutex<NetworkPlayModule>>> = OnceLock::new();

// Get or initialize the network play module singleton
pub fn get_network_play() -> Arc<Mutex<NetworkPlayModule>> {
    NETWORK_PLAY
        .get_or_init(|| Arc::new(Mutex::new(NetworkPlayModule::new())))
        .clone()
}

// Simple message format for our limited functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
struct NetworkMessage {
    event_type: String,
    player_id: String,
    data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JoinSessionMessage {
    command: String,
    session_id: String,
}

/// Minimal network play module with just what we need
pub struct NetworkPlayModule {
    network: NetworkModule,
    connected: bool,
    player_id: String,
    current_session_id: Option<String>,
    session_members: Vec<String>,
}

impl NetworkPlayModule {
    pub fn new() -> Self {
        Self {
            network: NetworkModule::new(),
            connected: false,
            player_id: "".to_string(),
            current_session_id: None,
            session_members: Vec::new(),
        }
    }

    pub fn connect(&mut self, url: &str) -> Result<()> {
        // Connect to the network
        self.network.connect(url)?;

        // Set up the message handler with a panic-safe wrapper
        self.network.on_message(move |message| {
            // Use catch_unwind to prevent thread panics
            if let Err(e) = panic::catch_unwind(|| {
                // Process the message
                match process_network_message(&message) {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("Error processing message: {}", e);
                    }
                }
            }) {
                // Handle any panics that might occur
                log::error!("Panic in message handler: {:?}", e);
            }
        });

        self.connected = true;

        Ok(())
    }

    // Join a specific game session
    pub fn join_session(&mut self, session_id: &str) -> Result<()> {
        if !self.connected {
            return Err(anyhow::anyhow!("Not connected to server"));
        }

        // Create join session message
        let join_msg = JoinSessionMessage {
            command: "join_session".to_string(),
            session_id: session_id.to_string(),
        };

        // Send join request
        let json = serde_json::to_string(&join_msg)?;
        self.network.send_message(&json)?;

        // Update local state - will be confirmed by server response
        self.current_session_id = Some(session_id.to_string());
        log::info!("Sent join request for session: {}", session_id);

        Ok(())
    }

    // Leave the current session
    pub fn leave_session(&mut self) -> Result<()> {
        if !self.connected {
            return Err(anyhow::anyhow!("Not connected to server"));
        }

        if let Some(session_id) = &self.current_session_id {
            let leave_msg = serde_json::json!({
                "command": "leave_session",
                "session_id": session_id
            });

            let json = serde_json::to_string(&leave_msg)?;
            self.network.send_message(&json)?;

            // Update local state - will be confirmed by server response
            log::info!("Sent request to leave session: {}", session_id);
        }

        Ok(())
    }
}

// Separate function to process messages that can safely access the global singleton
fn process_network_message(message: &str) -> Result<()> {
    log::debug!("Processing raw message: {}", message);

    // Check if the message is empty or just whitespace
    if message.trim().is_empty() {
        log::debug!("Received empty message, ignoring");
        return Ok(());
    }

    // Parse JSON
    let json_value = match serde_json::from_str::<serde_json::Value>(message) {
        Ok(value) => value,
        Err(e) => {
            log::debug!("Received non-JSON message: {} (Error: {})", message, e);
            return Ok(());
        }
    };

    // Parse NetworkMessage
    let network_msg = match serde_json::from_value::<NetworkMessage>(json_value) {
        Ok(msg) => msg,
        Err(e) => {
            log::debug!(
                "Message is not in NetworkMessage format: {} (Error: {})",
                message,
                e
            );
            return Ok(());
        }
    };

    let network_play = get_network_play();
    let mut module = network_play.lock().unwrap();

    match network_msg.event_type.as_str() {
        // Handle welcome message - just gets our player ID
        "welcome" => {
            module.player_id = network_msg.player_id.clone();
            log::info!("Connected as player ID: {}", module.player_id);
        }

        // Handle session_members event - updates who's in our session
        "session_members" => {
            if let Some(session_id) = network_msg.data.get("session_id").and_then(|v| v.as_str()) {
                // Update our current session ID
                module.current_session_id = Some(session_id.to_string());

                // Update the member list
                if let Some(members) = network_msg.data.get("members").and_then(|v| v.as_array()) {
                    let session_members: Vec<String> = members
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();

                    module.session_members = session_members;

                    log::info!(
                        "Session '{}' updated: {} members: {:?}",
                        session_id,
                        module.session_members.len(),
                        module.session_members
                    );
                }
            }
        }
        _ => {
            log::debug!("Unhandled message type: {}", network_msg.event_type);
        }
    }

    log::debug!("Received valid network message: {:?}", network_msg);

    Ok(())
}
