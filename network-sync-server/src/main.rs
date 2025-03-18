use clap::Parser;
use env_logger::Builder;
use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::broadcast,
};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};
use uuid::Uuid;

// Command line arguments
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Port to listen on
    #[clap(short, long, default_value = "8080")]
    port: u16,
}

// Message types for the protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClientMessage {
    pub command: String,
    pub session_id: Option<String>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServerMessage {
    event_type: String,
    player_id: String,
    data: serde_json::Value,
}

// Server state
struct ServerState {
    // Map from connection ID to session ID
    connections: HashMap<String, Option<String>>,
    // Map from session ID to set of connection IDs
    sessions: HashMap<String, Vec<String>>,
}

impl ServerState {
    fn new() -> Self {
        Self {
            connections: HashMap::new(),
            sessions: HashMap::new(),
        }
    }

    fn register_connection(&mut self, id: &str) {
        info!("Registering connection: {}", id);
        self.connections.insert(id.to_string(), None);
    }

    fn remove_connection(&mut self, id: &str) {
        // If in a session, remove from that session
        if let Some(Some(session_id)) = self.connections.get(id) {
            if let Some(connections) = self.sessions.get_mut(session_id) {
                connections.retain(|cid| cid != id);
                // Clean up empty sessions
                if connections.is_empty() {
                    self.sessions.remove(session_id);
                }
            }
        }
        self.connections.remove(id);
    }

    fn join_session(&mut self, connection_id: &str, session_id: &str) -> Vec<String> {
        // Update connection's session
        self.connections
            .insert(connection_id.to_string(), Some(session_id.to_string()));

        // Add to session
        let session_members = self
            .sessions
            .entry(session_id.to_string())
            .or_insert_with(Vec::new);

        if !session_members.contains(&connection_id.to_string()) {
            session_members.push(connection_id.to_string());
        }

        session_members.clone()
    }

    fn leave_session(&mut self, connection_id: &str) -> Option<String> {
        if let Some(Some(session_id)) = self.connections.get(connection_id) {
            let session_id = session_id.clone();
            // Update connection
            self.connections.insert(connection_id.to_string(), None);

            // Remove from session
            if let Some(connections) = self.sessions.get_mut(&session_id) {
                connections.retain(|cid| cid != connection_id);
                // Clean up empty sessions
                if connections.is_empty() {
                    self.sessions.remove(&session_id);
                }
            }

            return Some(session_id);
        }
        None
    }

    fn get_session_members(&self, session_id: &str) -> Vec<String> {
        self.sessions.get(session_id).cloned().unwrap_or_default()
    }

    fn is_in_session(&self, connection_id: &str, session_id: &str) -> bool {
        match self.connections.get(connection_id) {
            Some(Some(s)) if s == session_id => true,
            _ => false,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = Builder::from_default_env();

    #[cfg(debug_assertions)]
    builder.filter_level(log::LevelFilter::Debug);
    #[cfg(not(debug_assertions))]
    builder.filter_level(log::LevelFilter::Info);

    builder.init();

    // Parse command line arguments
    let args = Args::parse();
    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));

    // Set up server
    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on: {}", addr);

    // Create shared server state
    let state = Arc::new(Mutex::new(ServerState::new()));

    // Create broadcast channel for server messages
    let (tx, _) = broadcast::channel::<(String, String)>(100);

    // Accept connections
    while let Ok((stream, addr)) = listener.accept().await {
        info!("New connection from: {}", addr);

        // Clone handles for this connection
        let tx = tx.clone();
        let state = Arc::clone(&state);

        // Generate a unique ID for this connection
        let connection_id = Uuid::new_v4().to_string();

        // Register connection
        {
            let mut state = state.lock().unwrap();
            state.register_connection(&connection_id);
        }

        // Clone state for disconnect handling
        let disconnect_state = Arc::clone(&state);
        let disconnect_id = connection_id.clone();

        // Spawn a task to handle this connection
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, connection_id.clone(), state, tx).await {
                error!("Error handling connection {}: {}", disconnect_id, e);
            }

            // On disconnect, clean up
            let mut state = disconnect_state.lock().unwrap();
            state.remove_connection(&connection_id);
            info!("Connection closed: {}", connection_id);
        });
    }

    Ok(())
}

async fn handle_connection(
    stream: TcpStream,
    connection_id: String,
    state: Arc<Mutex<ServerState>>,
    tx: broadcast::Sender<(String, String)>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Accept WebSocket connection
    let ws_stream = accept_async(stream).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Send welcome message with connection ID
    let welcome = ServerMessage {
        event_type: "welcome".to_string(),
        player_id: connection_id.clone(),
        data: serde_json::json!({}),
    };

    ws_sender
        .send(Message::Text(serde_json::to_string(&welcome)?))
        .await?;

    // Subscribe to broadcast messages
    let mut rx = tx.subscribe();

    // Create task to forward broadcasts to this connection
    let conn_id = connection_id.clone();
    let forward_task = tokio::spawn(async move {
        while let Ok((target, msg)) = rx.recv().await {
            // Send if broadcast is for all or specifically for this connection
            if target == "*" || target == conn_id {
                if let Err(e) = ws_sender.send(Message::Text(msg)).await {
                    error!("Failed to forward message: {}", e);
                    break;
                }
            }
        }
    });

    // Process incoming messages
    while let Some(result) = ws_receiver.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                error!("Error receiving message: {}", e);
                break;
            }
        };

        debug!("Received message from {}", connection_id);

        if let Message::Text(text) = msg {
            debug!("Received text message from {}: {}", connection_id, text);

            // Try to parse as client message
            match serde_json::from_str::<ClientMessage>(&text) {
                Ok(client_msg) => {
                    match client_msg.command.as_str() {
                        "join_session" => {
                            if let Some(session_id) = &client_msg.session_id {
                                let members = {
                                    let mut state = state.lock().unwrap();
                                    state.join_session(&connection_id, session_id)
                                };

                                // Notify all session members
                                let session_msg = ServerMessage {
                                    event_type: "session_members".to_string(),
                                    player_id: connection_id.clone(),
                                    data: serde_json::json!({
                                        "session_id": session_id,
                                        "members": members,
                                    }),
                                };

                                let msg_str = serde_json::to_string(&session_msg)?;

                                // Broadcast to all session members
                                for member in members {
                                    tx.send((member, msg_str.clone()))?;
                                }

                                info!("Player {} joined session {}", connection_id, session_id);
                            }
                        }

                        "leave_session" => {
                            let result = {
                                let mut state = state.lock().unwrap();
                                state.leave_session(&connection_id)
                            };

                            if let Some(session_id) = result {
                                let members = {
                                    let state = state.lock().unwrap();
                                    state.get_session_members(&session_id)
                                };

                                // Notify remaining members
                                let leave_msg = ServerMessage {
                                    event_type: "session_members".to_string(),
                                    player_id: connection_id.clone(),
                                    data: serde_json::json!({
                                        "session_id": session_id,
                                        "members": members,
                                    }),
                                };

                                let msg_str = serde_json::to_string(&leave_msg)?;

                                // Broadcast to remaining session members
                                for member in members {
                                    tx.send((member, msg_str.clone()))?;
                                }

                                info!("Player {} left session {}", connection_id, session_id);
                            }
                        }

                        "player_sync" => {
                            let sync_msg = ServerMessage {
                                event_type: "player_sync".to_string(),
                                player_id: connection_id.clone(),
                                data: client_msg.data.unwrap_or(serde_json::Value::Null),
                            };

                            let msg_str = serde_json::to_string(&sync_msg)?;

                            // Broadcast to everyone in the same session
                            let state = state.lock().unwrap();
                            if let Some(Some(session_id)) = state.connections.get(&connection_id) {
                                for member in state.get_session_members(session_id) {
                                    tx.send((member, msg_str.clone()))?;
                                }
                            }
                        }

                        _ => {
                            error!("Unknown command: {}", client_msg.command);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to parse message: {} ({})", text, e);
                }
            }
        }
    }

    // Cancel the forward task when the connection closes
    forward_task.abort();

    // Notify about disconnection
    let state_clone = Arc::clone(&state);
    let session_id_opt = {
        let state = state.lock().unwrap();
        state
            .connections
            .get(&connection_id)
            .and_then(|s| s.clone())
    };

    if let Some(session_id) = session_id_opt {
        let members = {
            let state = state_clone.lock().unwrap();
            state
                .get_session_members(&session_id)
                .into_iter()
                .filter(|id| id != &connection_id)
                .collect::<Vec<_>>()
        };

        if !members.is_empty() {
            // Create disconnection message with updated member list
            let disconnect_msg = ServerMessage {
                event_type: "session_members".to_string(),
                player_id: connection_id.clone(),
                data: serde_json::json!({
                    "session_id": session_id,
                    "members": members,
                }),
            };

            let msg_str = serde_json::to_string(&disconnect_msg)?;

            // Broadcast to remaining members
            log::info!("Broadcasting disconnection message to remaining members");
            for member in members {
                let _ = tx.send((member, msg_str.clone()));
            }
        }
    }

    Ok(())
}
