mod network;
mod utils;

use env_logger::Builder;
use n64_recomp::RecompContext;
use network::{get_network_play, NETWORK_PLAY};
use std::panic;
use utils::execute_safely;

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
}

#[no_mangle]
pub extern "C" fn NetworkPlaySetPlayerId(_rdram: *mut u8, ctx: *mut RecompContext) {
    execute_safely(ctx, "NetworkPlaySetPlayerId", |ctx| {
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
}

#[no_mangle]
pub extern "C" fn NetworkPlaySetPlayerCanSpin(_rdram: *mut u8, ctx: *mut RecompContext) {
    execute_safely(ctx, "NetworkPlaySetPlayerCanSpin", |ctx| {
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
}

#[no_mangle]
pub extern "C" fn NetworkPlayCanPlayerSpin(_rdram: *mut u8, ctx: *mut RecompContext) {
    execute_safely(ctx, "NetworkPlayCanPlayerSpin", |ctx| {
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
}
