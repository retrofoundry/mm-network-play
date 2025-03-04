mod network;
mod utils;

use env_logger::Builder;
use n64_recomp::RecompContext;
use network::get_network_play;
use std::panic;
use utils::{execute_safely, with_network_play, with_network_play_mut};

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

        let result = with_network_play_mut(
            |module| match module.connect(&url) {
                Ok(_) => {
                    log::info!("Successfully connected to {}", url);
                    1i32
                }
                Err(e) => {
                    log::error!("Failed to connect to {}: {}", url, e);
                    0i32
                }
            },
            0i32,
        );

        ctx.set_return(result);
    });
}

#[no_mangle]
pub extern "C" fn NetworkPlaySetPlayerId(_rdram: *mut u8, ctx: *mut RecompContext) {
    execute_safely(ctx, "NetworkPlaySetPlayerId", |ctx| {
        let player_id = ctx.a0() as u32;

        with_network_play_mut(
            |module| {
                module.set_player_id(player_id);
                log::info!("Set player ID to {}", player_id);
            },
            (),
        );
    });
}

#[no_mangle]
pub extern "C" fn NetworkPlaySetPlayerCanSpin(_rdram: *mut u8, ctx: *mut RecompContext) {
    execute_safely(ctx, "NetworkPlaySetPlayerCanSpin", |ctx| {
        let can_spin = ctx.a0() != 0;

        let result = with_network_play_mut(
            |module| match module.set_player_can_spin(can_spin) {
                Ok(_) => {
                    log::info!("Player spin ability set to {}", can_spin);
                    1i32
                }
                Err(e) => {
                    log::error!("Failed to set spin ability: {}", e);
                    0i32
                }
            },
            0i32,
        );

        ctx.set_return(result);
    });
}

#[no_mangle]
pub extern "C" fn NetworkPlayCanPlayerSpin(_rdram: *mut u8, ctx: *mut RecompContext) {
    execute_safely(ctx, "NetworkPlayCanPlayerSpin", |ctx| {
        let player_id = ctx.a0() as u32;

        let result = with_network_play(
            |module| {
                let can_spin = module.can_player_spin(player_id);
                if can_spin {
                    1i32
                } else {
                    0i32
                }
            },
            0i32,
        );

        ctx.set_return(result);
    });
}
