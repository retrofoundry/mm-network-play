use anyhow::Result;
use gamecore::network::NetworkModule;
use serde_json;
use std::collections::{HashMap, VecDeque};
use std::panic;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;
use tokio::runtime::Runtime;

use crate::messages::{
    ActorSyncMessage, JoinSessionMessage, LeaveSessionMessage, NetworkMessage, RegisteredMessage,
    ServerMessage,
};
use crate::types::{ActorData, RemoteActorData};

// Global singleton instances
pub static NETWORK_PLAY: OnceLock<Arc<Mutex<NetworkSyncModule>>> = OnceLock::new();
pub static TOKIO_RUNTIME: OnceLock<Runtime> = OnceLock::new();

// Get or initialize the tokio runtime
fn get_tokio_runtime() -> &'static Runtime {
    TOKIO_RUNTIME.get_or_init(|| Runtime::new().expect("Failed to create Tokio runtime"))
}

// Get or initialize the network play module singleton
pub fn get_network_sync() -> Arc<Mutex<NetworkSyncModule>> {
    NETWORK_PLAY
        .get_or_init(|| Arc::new(Mutex::new(NetworkSyncModule::new())))
        .clone()
}

/// Minimal network play module with just what we need
pub struct NetworkSyncModule {
    network: NetworkModule,
    connected: bool,
    pub client_id: String,
    current_session_id: Option<String>,
    session_members: Vec<String>,
    pub remote_actors: HashMap<String, RemoteActorData>,
    /// Queue of (message_id, data) tuples
    pub message_queue: VecDeque<(String, Vec<u8>)>,
}

impl NetworkSyncModule {
    pub fn new() -> Self {
        Self {
            network: NetworkModule::new(),
            connected: false,
            client_id: "".to_string(),
            current_session_id: None,
            session_members: Vec::new(),
            remote_actors: HashMap::new(),
            message_queue: VecDeque::new(),
        }
    }

    pub fn connect(&mut self, url: &str) -> Result<()> {
        // Set up the message handler before connecting
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

        // Connect to the network using the tokio runtime
        let runtime = get_tokio_runtime();
        runtime.block_on(async { self.network.connect(url).await })?;

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
            event_type: "join_session".to_string(),
            session_id: session_id.to_string(),
        };

        // Send join request using the tokio runtime
        let json = serde_json::to_string(&join_msg)?;
        let runtime = get_tokio_runtime();
        runtime.block_on(async { self.network.send_message(&json).await })?;

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
            let leave_msg = LeaveSessionMessage {
                event_type: "leave_session".to_string(),
            };

            let json = serde_json::to_string(&leave_msg)?;

            // Send message using the tokio runtime
            let runtime = get_tokio_runtime();
            runtime.block_on(async { self.network.send_message(&json).await })?;

            // Update local state - will be confirmed by server response
            log::info!("Sent request to leave session: {}", session_id);
        }

        Ok(())
    }

    // Disconnect from the server
    pub fn disconnect(&mut self) -> Result<()> {
        if !self.connected {
            return Ok(());
        }

        // Disconnect using the tokio runtime
        let runtime = get_tokio_runtime();
        runtime.block_on(async { self.network.disconnect().await })?;

        self.connected = false;
        self.current_session_id = None;
        self.session_members.clear();

        Ok(())
    }

    // Sends an actor sync event
    pub fn send_actor_sync(&mut self, player_data: &ActorData) -> Result<()> {
        if !self.connected {
            return Err(anyhow::anyhow!("Not connected"));
        }

        let player_update_msg = ActorSyncMessage {
            event_type: "actor_sync".to_string(),
            sender_id: self.client_id.clone(),
            data: player_data.clone(),
        };

        let json = serde_json::to_string(&player_update_msg)?;

        // Send player data to the server
        let runtime = get_tokio_runtime();
        runtime.block_on(async { self.network.send_message(&json).await })?;

        Ok(())
    }

    // Send a message to other clients
    pub fn send_message(&mut self, message_id: &str, data: Vec<u8>) -> Result<()> {
        if !self.connected {
            return Err(anyhow::anyhow!("Not connected"));
        }

        if let Some(session_id) = &self.current_session_id {
            let msg = RegisteredMessage {
                event_type: "registered_message".to_string(),
                sender_id: self.client_id.clone(),
                message_id: message_id.to_string(),
                data,
            };

            let json = serde_json::to_string(&msg)?;

            // Send message to the server
            let runtime = get_tokio_runtime();
            runtime.block_on(async { self.network.send_message(&json).await })?;

            log::debug!("Sent message '{}' to session {}", message_id, session_id);
        }

        Ok(())
    }

    // Get the size of the next message in the queue
    pub fn get_pending_message_size(&self) -> u32 {
        if let Some((_, data)) = self.message_queue.front() {
            data.len() as u32
        } else {
            0 // No messages
        }
    }

    // Get the next message from the queue
    pub fn get_message(&mut self, buffer: &mut [u8]) -> Option<String> {
        if let Some((message_id, data)) = self.message_queue.pop_front() {
            if buffer.len() >= data.len() {
                buffer[..data.len()].copy_from_slice(&data);
                Some(message_id)
            } else {
                log::error!(
                    "Buffer too small for message: {} > {}",
                    data.len(),
                    buffer.len()
                );
                None
            }
        } else {
            None
        }
    }

    // Queue a message
    fn queue_message(&mut self, message_id: String, data: Vec<u8>) {
        self.message_queue.push_back((message_id, data));
    }
}

// Separate function to process messages that can safely access the global singleton
fn process_network_message(message: &str) -> Result<()> {
    // Check if the message is empty or just whitespace
    if message.trim().is_empty() {
        log::debug!("Received empty message, ignoring");
        return Ok(());
    }

    // Parse NetworkMessage
    let server_msg = match serde_json::from_str::<ServerMessage>(message) {
        Ok(msg) => msg,
        Err(e) => {
            log::debug!("Failed to parse message: {} (Error: {})", message, e);
            return Ok(());
        }
    };

    let network_sync = get_network_sync();
    let mut module = network_sync.lock().unwrap();

    match server_msg {
        ServerMessage::Welcome(msg) => {
            module.client_id = msg.sender_id.clone();
            log::info!("Connected as player ID: {}", module.client_id);
        }

        ServerMessage::SessionMembers(msg) => {
            if let Some(session_id) = msg.data.get("session_id").and_then(|v| v.as_str()) {
                // Update the member list
                if let Some(members) = msg.data.get("members").and_then(|v| v.as_array()) {
                    let session_members: Vec<String> = members
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();

                    // Get current members to identify disconnected players
                    let old_members =
                        std::mem::replace(&mut module.session_members, session_members.clone());

                    // Find any members that were removed (disconnected)
                    for old_member in old_members {
                        if !session_members.contains(&old_member) {
                            // This player is no longer in the session, remove them from remote_players
                            module.remote_actors.remove(&old_member);
                            log::info!("Player {} has disconnected", old_member);
                        }
                    }

                    log::info!(
                        "Session '{}' updated: {} members: {:?}",
                        session_id,
                        module.session_members.len(),
                        module.session_members
                    );
                }
            }
        }

        ServerMessage::ActorSync(msg) => {
            if msg.sender_id != module.client_id {
                // Only store data from other players, not ourself
                let remote_data = RemoteActorData {
                    id: msg.sender_id.clone(),
                    data: msg.data.clone(),
                    last_update: Instant::now(),
                };

                // Store the remote player data
                module
                    .remote_actors
                    .insert(msg.sender_id.clone(), remote_data);

                log::debug!("Received actor sync from {}", msg.sender_id);
            }
        }

        ServerMessage::RegisteredMessage(msg) => {
            if msg.sender_id != module.client_id {
                module.queue_message(msg.message_id.clone(), msg.data);
                log::debug!(
                    "Received message '{}' from {}",
                    msg.message_id,
                    msg.sender_id
                );
            }
        }
    }

    Ok(())
}
