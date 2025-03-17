mod messages;
mod network;
mod types;
mod utils;

use env_logger::Builder;
use n64_recomp::RecompContext;
use network::get_network_play;
use std::panic;
use types::PlayerData;
use utils::{execute_safely, with_network_play_mut};

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
    execute_safely(ctx, "NetworkPlayConnect", |ctx| {
        let host = unsafe { ctx.get_arg_string(rdram, 0) };

        log::info!("Connecting to server: {}", host);

        let result = with_network_play_mut(
            |module| match module.connect(&host) {
                Ok(_) => {
                    log::info!("Successfully connected to {}", host);
                    1i32
                }
                Err(e) => {
                    log::error!("Failed to connect to {}: {}", host, e);
                    0i32
                }
            },
            0i32,
        );

        ctx.set_return(result);
    });
}

#[no_mangle]
pub extern "C" fn NetworkPlayDisconnect(_rdram: *mut u8, ctx: *mut RecompContext) {
    execute_safely(ctx, "NetworkPlayDisconnect", |ctx| {
        log::info!("Disconnecting from server");

        let result = with_network_play_mut(
            |module| match module.disconnect() {
                Ok(_) => {
                    log::info!("Successfully disconnected");
                    1i32
                }
                Err(e) => {
                    log::error!("Failed to disconnect: {}", e);
                    0i32
                }
            },
            0i32,
        );

        ctx.set_return(result);
    });
}

#[no_mangle]
pub extern "C" fn NetworkPlayJoinSession(rdram: *mut u8, ctx: *mut RecompContext) {
    execute_safely(ctx, "NetworkPlayJoinSession", |ctx| {
        let session_id = unsafe { ctx.get_arg_string(rdram, 0) };

        log::info!("Joining session {}", session_id);

        let result = with_network_play_mut(
            |module| match module.join_session(&session_id) {
                Ok(_) => {
                    log::info!("Successfully joined session {}", session_id);
                    1i32
                }
                Err(e) => {
                    log::error!("Failed to join session {}: {}", session_id, e);
                    0i32
                }
            },
            0i32,
        );

        ctx.set_return(result);
    });
}

#[no_mangle]
pub extern "C" fn NetworkPlayLeaveSession(_rdram: *mut u8, ctx: *mut RecompContext) {
    execute_safely(ctx, "NetworkPlayLeaveSession", |ctx| {
        log::info!("Leaving current session");

        let result = with_network_play_mut(
            |module| match module.leave_session() {
                Ok(_) => {
                    log::info!("Successfully left session");
                    1i32
                }
                Err(e) => {
                    log::error!("Failed to leave session: {}", e);
                    0i32
                }
            },
            0i32,
        );

        ctx.set_return(result);
    });
}

#[no_mangle]
pub extern "C" fn NetworkPlaySendPlayerSync(rdram: *mut u8, ctx: *mut RecompContext) {
    execute_safely(ctx, "NetworkPlaySendPlayerSync", |ctx| {
        let player_data_ptr = unsafe { ctx.get_arg_ptr::<PlayerData>(rdram, 0) };
        let player_data = unsafe { &*player_data_ptr };

        let result = with_network_play_mut(
            |module| match module.send_player_sync(player_data) {
                Ok(_) => 1i32,
                Err(e) => {
                    log::error!("Failed to send player sync: {}", e);
                    0i32
                }
            },
            0i32,
        );

        ctx.set_return(result);
    });
}
