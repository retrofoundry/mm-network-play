use anyhow::Result;
use env_logger::Builder;
use gamecore::network::NetworkModule;
use n64_recomp::RecompContext;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::panic;
use std::sync::{Arc, Mutex, OnceLock};

// Global singleton instance of the NetworkPlay module
static NETWORK_PLAY: OnceLock<Arc<Mutex<NetworkPlayModule>>> = OnceLock::new();

// Simple message format for our limited functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
struct NetworkMessage {
    event_type: String,
    player_id: u32,
    data: serde_json::Value,
}

/// Minimal network play module with just what we need
struct NetworkPlayModule {
    network: NetworkModule,
    connected: bool,
    player_id: u32,
    // Which players can spin
    player_spin_ability: HashMap<u32, bool>,
}

impl NetworkPlayModule {
    fn new() -> Self {
        Self {
            network: NetworkModule::new(),
            connected: false,
            player_id: 1, // Default player ID
            player_spin_ability: HashMap::new(),
        }
    }

    fn connect(&mut self, url: &str) -> Result<()> {
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

        // Initialize our abilities
        self.player_spin_ability.insert(self.player_id, false);

        Ok(())
    }

    fn set_player_id(&mut self, id: u32) {
        self.player_id = id;
        self.player_spin_ability.insert(id, false);
    }

    fn set_player_can_spin(&mut self, can_spin: bool) -> Result<()> {
        // Update local state
        self.player_spin_ability.insert(self.player_id, can_spin);

        // Send network event
        let event = NetworkMessage {
            event_type: "spin_ability".to_string(),
            player_id: self.player_id,
            data: serde_json::json!({ "can_spin": can_spin }),
        };

        // Use serde_json's to_string, which should never panic on our struct
        let json = match serde_json::to_string(&event) {
            Ok(json) => json,
            Err(e) => {
                log::error!("Failed to serialize message: {}", e);
                return Err(anyhow::anyhow!("Serialization error: {}", e));
            }
        };

        self.network.send_message(&json)
    }

    fn can_player_spin(&self, player_id: u32) -> bool {
        self.player_spin_ability
            .get(&player_id)
            .copied()
            .unwrap_or(false)
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

    // Try to parse as JSON first
    let json_result = serde_json::from_str::<serde_json::Value>(message);

    if let Err(e) = json_result {
        // Not valid JSON - could be a server welcome message
        log::debug!("Received non-JSON message: {} (Error: {})", message, e);
        return Ok(()); // Return without error - just ignore non-JSON messages
    }

    let json_value = json_result.unwrap();

    // Now try to parse as NetworkMessage
    let network_msg_result = serde_json::from_value::<NetworkMessage>(json_value.clone());

    if let Err(e) = network_msg_result {
        // It's valid JSON but not our expected format
        log::debug!(
            "Message is not in NetworkMessage format: {} (Error: {})",
            message,
            e
        );
        return Ok(()); // Still return Ok - we just ignore messages we don't understand
    }

    let network_msg = network_msg_result.unwrap();
    log::debug!("Received valid network message: {:?}", network_msg);

    // Process message based on event type
    if network_msg.event_type == "spin_ability" {
        if let Some(can_spin) = network_msg.data.get("can_spin").and_then(|v| v.as_bool()) {
            // Update our state using the global singleton
            if let Some(network_play) = NETWORK_PLAY.get() {
                match network_play.lock() {
                    Ok(mut module) => {
                        module
                            .player_spin_ability
                            .insert(network_msg.player_id, can_spin);
                        log::debug!(
                            "Player {} spin ability set to {}",
                            network_msg.player_id,
                            can_spin
                        );
                    }
                    Err(e) => {
                        log::error!("Failed to lock network module in message handler: {}", e);
                    }
                }
            } else {
                log::error!("Network play module not initialized");
            }
        }
    }
    Ok(())
}

// Get or initialize the network play module singleton
fn get_network_play() -> Arc<Mutex<NetworkPlayModule>> {
    NETWORK_PLAY
        .get_or_init(|| Arc::new(Mutex::new(NetworkPlayModule::new())))
        .clone()
}

// C - API

#[no_mangle]
#[allow(non_upper_case_globals)]
pub static recomp_api_version: u32 = 1;

#[no_mangle]
pub extern "C" fn NetworkPlayInit(_rdram: *mut u8, _ctx: *mut RecompContext) {
    // Set up a panic hook that logs panics but doesn't abort
    panic::set_hook(Box::new(|panic_info| {
        log::error!("Panic in network play module: {:?}", panic_info);
    }));

    let mut builder = Builder::from_default_env();

    #[cfg(debug_assertions)]
    builder.filter_level(log::LevelFilter::Debug);
    #[cfg(not(debug_assertions))]
    builder.filter_level(log::LevelFilter::Info);

    builder.init();

    // Initialize network module at startup
    let _ = get_network_play();
    log::info!("Network play module initialized");
}

#[no_mangle]
pub extern "C" fn NetworkPlayConnect(rdram: *mut u8, ctx: *mut RecompContext) {
    // Catch any panics to prevent unwinding into C code
    let result = panic::catch_unwind(|| {
        let ctx = unsafe { &mut *ctx };
        let url = unsafe { ctx.get_arg_string(rdram, 0) };
        log::info!("Connecting to server: {}", url);

        match NETWORK_PLAY.get() {
            Some(network_play) => match network_play.lock() {
                Ok(mut module) => match module.connect(&url) {
                    Ok(_) => {
                        log::info!("Successfully connected to {}", url);
                        ctx.set_return(1i32);
                    }
                    Err(e) => {
                        log::error!("Failed to connect to {}: {}", url, e);
                        ctx.set_return(0i32);
                    }
                },
                Err(e) => {
                    log::error!("Failed to lock network play module: {}", e);
                    ctx.set_return(0i32);
                }
            },
            None => {
                log::error!("Network play module not initialized");
                ctx.set_return(0i32);
            }
        }
    });

    // If a panic occurred, handle it gracefully
    if let Err(e) = result {
        log::error!("Panic in NetworkPlayConnect: {:?}", e);
        // Set return value to false if we can access ctx safely
        if let Ok(ctx) = panic::catch_unwind(|| unsafe { &mut *(ctx as *mut RecompContext) }) {
            ctx.set_return(0i32);
        }
    }
}

#[no_mangle]
pub extern "C" fn NetworkPlaySetPlayerId(rdram: *mut u8, ctx: *mut RecompContext) {
    // Catch any panics to prevent unwinding into C code
    let result = panic::catch_unwind(|| {
        let ctx = unsafe { &mut *ctx };
        let player_id = ctx.a0() as u32;

        if let Some(network_play) = NETWORK_PLAY.get() {
            if let Ok(mut module) = network_play.lock() {
                module.set_player_id(player_id);
                log::info!("Set player ID to {}", player_id);
            } else {
                log::error!("Failed to lock network play module");
            }
        } else {
            log::error!("Network play module not initialized");
        }
    });

    // Log any panics
    if let Err(e) = result {
        log::error!("Panic in NetworkPlaySetPlayerId: {:?}", e);
    }
}

#[no_mangle]
pub extern "C" fn NetworkPlaySetPlayerCanSpin(rdram: *mut u8, ctx: *mut RecompContext) {
    // Catch any panics to prevent unwinding into C code
    let result = panic::catch_unwind(|| {
        let ctx = unsafe { &mut *ctx };
        let can_spin = ctx.a0() != 0;

        if let Some(network_play) = NETWORK_PLAY.get() {
            match network_play.lock() {
                Ok(mut module) => match module.set_player_can_spin(can_spin) {
                    Ok(_) => {
                        log::info!("Player spin ability set to {}", can_spin);
                        ctx.set_return(1i32);
                    }
                    Err(e) => {
                        log::error!("Failed to set spin ability: {}", e);
                        ctx.set_return(0i32);
                    }
                },
                Err(e) => {
                    log::error!("Failed to lock network play module: {}", e);
                    ctx.set_return(0i32);
                }
            }
        } else {
            log::error!("Network play module not initialized");
            ctx.set_return(0i32);
        }
    });

    // If a panic occurred, handle it gracefully
    if let Err(e) = result {
        log::error!("Panic in NetworkPlaySetPlayerCanSpin: {:?}", e);
        // Set return value to false if we can access ctx safely
        if let Ok(ctx) = panic::catch_unwind(|| unsafe { &mut *(ctx as *mut RecompContext) }) {
            ctx.set_return(0i32);
        }
    }
}

#[no_mangle]
pub extern "C" fn NetworkPlayCanPlayerSpin(rdram: *mut u8, ctx: *mut RecompContext) {
    // Catch any panics to prevent unwinding into C code
    let result = panic::catch_unwind(|| {
        let ctx = unsafe { &mut *ctx };
        let player_id = ctx.a0() as u32;

        if let Some(network_play) = NETWORK_PLAY.get() {
            match network_play.lock() {
                Ok(module) => {
                    let can_spin = module.can_player_spin(player_id);
                    ctx.set_return(if can_spin { 1i32 } else { 0i32 });
                }
                Err(e) => {
                    log::error!("Failed to lock network play module: {}", e);
                    ctx.set_return(0i32);
                }
            }
        } else {
            log::error!("Network play module not initialized");
            ctx.set_return(0i32);
        }
    });

    // If a panic occurred, handle it gracefully
    if let Err(e) = result {
        log::error!("Panic in NetworkPlayCanPlayerSpin: {:?}", e);
        // Set return value to false if we can access ctx safely
        if let Ok(ctx) = panic::catch_unwind(|| unsafe { &mut *(ctx as *mut RecompContext) }) {
            ctx.set_return(0i32);
        }
    }
}
