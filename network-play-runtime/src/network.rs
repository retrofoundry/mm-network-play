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

/// Minimal network play module with just what we need
pub struct NetworkPlayModule {
    network: NetworkModule,
    connected: bool,
    player_id: String,
}

impl NetworkPlayModule {
    pub fn new() -> Self {
        Self {
            network: NetworkModule::new(),
            connected: false,
            player_id: "".to_string(), // Default player ID
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

    if network_msg.event_type == "welcome" {
        let player_id = network_msg.player_id.clone();
        let network_play = get_network_play();
        let mut network_play = network_play.lock().unwrap();
        network_play.player_id = player_id;
    }

    log::debug!("Received valid network message: {:?}", network_msg);

    Ok(())
}
