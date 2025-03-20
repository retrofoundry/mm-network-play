mod messages;
mod network;
mod types;
mod utils;

use env_logger::Builder;
use n64_recomp::{mem_bu, mem_bu_write, N64MemoryIO, RecompContext};
use network::get_network_sync;
use std::panic;
use types::ActorData;
use utils::{execute_safely, with_network_sync, with_network_sync_mut};

// C - API

#[no_mangle]
#[allow(non_upper_case_globals)]
pub static recomp_api_version: u32 = 1;

#[no_mangle]
pub extern "C" fn NetworkSyncInit(_rdram: *mut u8, _ctx: *mut RecompContext) {
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
    let _ = get_network_sync();
    log::info!("Network play module initialized");
}

#[no_mangle]
pub extern "C" fn NetworkSyncConnect(rdram: *mut u8, ctx: *mut RecompContext) {
    execute_safely(ctx, "NetworkSyncConnect", |ctx| {
        let host = unsafe { ctx.get_arg_string(rdram, 0) };
        log::info!("Connecting to server: {}", host);

        let result = with_network_sync_mut(
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
pub extern "C" fn NetworkSyncDisconnect(_rdram: *mut u8, ctx: *mut RecompContext) {
    execute_safely(ctx, "NetworkSyncDisconnect", |ctx| {
        log::info!("Disconnecting from server");

        let result = with_network_sync_mut(
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
pub extern "C" fn NetworkSyncGetClientId(rdram: *mut u8, ctx: *mut RecompContext) {
    execute_safely(ctx, "NetworkSyncGetClientId", |ctx| {
        let actor_id_buf = ctx.get_arg_u64(0);
        let max_len = ctx.get_arg_u32(1) as usize;

        let success = with_network_sync(
            |module| {
                if !module.client_id.is_empty() {
                    unsafe {
                        let _ = ctx.write_string_to_mem(
                            rdram,
                            actor_id_buf,
                            &module.client_id,
                            max_len,
                        );
                    }
                    1i32
                } else {
                    0i32
                }
            },
            0i32,
        );

        ctx.set_return(success);
    });
}

#[no_mangle]
pub extern "C" fn NetworkSyncJoinSession(rdram: *mut u8, ctx: *mut RecompContext) {
    execute_safely(ctx, "NetworkSyncJoinSession", |ctx| {
        let session_id = unsafe { ctx.get_arg_string(rdram, 0) };

        log::info!("Joining session {}", session_id);

        let result = with_network_sync_mut(
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
pub extern "C" fn NetworkSyncLeaveSession(_rdram: *mut u8, ctx: *mut RecompContext) {
    execute_safely(ctx, "NetworkSyncLeaveSession", |ctx| {
        log::info!("Leaving current session");

        let result = with_network_sync_mut(
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
pub extern "C" fn NetworkSyncEmitActorData(rdram: *mut u8, ctx: *mut RecompContext) {
    execute_safely(ctx, "NetworkSyncEmitActorData", |ctx| {
        let addr = ctx.get_arg_u64(0);
        let player_data = unsafe { ActorData::read_from_mem(ctx, rdram, addr) };

        let result = with_network_sync_mut(
            |module| match module.send_actor_sync(&player_data) {
                Ok(_) => 1i32,
                Err(e) => {
                    log::error!("Failed to send actor sync: {}", e);
                    0i32
                }
            },
            0i32,
        );

        ctx.set_return(result);
    });
}

#[no_mangle]
pub extern "C" fn NetworkSyncGetRemoteActorIDs(rdram: *mut u8, ctx: *mut RecompContext) {
    execute_safely(ctx, "NetworkSyncGetRemoteActorIDs", |ctx| {
        let max_players = ctx.get_arg_u32(0);
        let ids_buffer = ctx.get_arg_u64(1); // Get the virtual address
        let id_buffer_size = ctx.get_arg_u32(2);

        let count = with_network_sync(
            |module| {
                let mut count = 0;

                if max_players > 0 {
                    let actor_ids: Vec<&String> = module.remote_actors.keys().collect();
                    let str_refs: Vec<&str> = actor_ids.iter().map(|s| s.as_str()).collect();

                    unsafe {
                        count = ctx.write_string_array_to_mem(
                            rdram,
                            ids_buffer,
                            &str_refs,
                            id_buffer_size as usize,
                            max_players as usize,
                        ) as i32;
                    }
                }

                count
            },
            0i32,
        );

        ctx.set_return(count);
    });
}

#[no_mangle]
pub extern "C" fn NetworkSyncGetRemoteActorData(rdram: *mut u8, ctx: *mut RecompContext) {
    execute_safely(ctx, "NetworkSyncGetRemoteActorData", |ctx| {
        let actor_id = unsafe { ctx.get_arg_string(rdram, 0) };
        let data_buffer_addr = ctx.get_arg_u64(1);

        let success = with_network_sync(
            |module| {
                if let Some(remote_player) = module.remote_actors.get(&actor_id) {
                    unsafe {
                        remote_player
                            .data
                            .write_to_mem(ctx, rdram, data_buffer_addr);
                    }
                    1i32
                } else {
                    0i32
                }
            },
            0i32,
        );

        ctx.set_return(success);
    });
}

#[no_mangle]
pub extern "C" fn NetworkSyncEmitMessage(rdram: *mut u8, ctx: *mut RecompContext) {
    execute_safely(ctx, "NetworkSyncEmitMessage", |ctx| {
        let message_id = unsafe { ctx.get_arg_string(rdram, 0) };
        let data_size = ctx.get_arg_u32(1) as usize;
        let data_ptr = ctx.get_arg_u64(2);

        // Read the data from the provided pointer
        let mut data = Vec::with_capacity(data_size);
        unsafe {
            for i in 0..data_size {
                data.push(mem_bu(rdram, data_ptr + i as u64));
            }
        }

        let result = with_network_sync_mut(
            |module| match module.send_message(&message_id, data) {
                Ok(_) => 1i32,
                Err(e) => {
                    log::error!("Failed to send message {}: {}", message_id, e);
                    0i32
                }
            },
            0i32,
        );

        ctx.set_return(result);
    });
}

#[no_mangle]
pub extern "C" fn NetworkSyncGetPendingMessageSize(rdram: *mut u8, ctx: *mut RecompContext) {
    execute_safely(ctx, "NetworkSyncGetPendingMessageSize", |ctx| {
        let size = with_network_sync(|module| module.get_pending_message_size() as i32, 0i32);
        ctx.set_return(size);
    });
}

#[no_mangle]
pub extern "C" fn NetworkSyncGetMessage(rdram: *mut u8, ctx: *mut RecompContext) {
    execute_safely(ctx, "NetworkSyncGetMessage", |ctx| {
        let buffer_ptr = ctx.get_arg_u64(0);
        let buffer_size = ctx.get_arg_u32(1) as usize;
        let message_id_buffer = ctx.get_arg_u64(2);

        // Create a buffer to hold the message data
        let mut buffer = vec![0u8; buffer_size];

        let message_id = with_network_sync_mut(
            |module| {
                if let Some(id) = module.get_message(&mut buffer) {
                    // Copy the data to the guest memory
                    unsafe {
                        for (i, &byte) in buffer.iter().enumerate() {
                            mem_bu_write(rdram, buffer_ptr + i as u64, byte);
                        }
                    }
                    Some(id)
                } else {
                    None
                }
            },
            None,
        );

        // Write the message ID to the return value or empty string if None
        if let Some(id) = message_id.clone() {
            let id_len = id.len().min(63); // Max 63 chars + null terminator
            unsafe {
                ctx.write_string_to_mem(rdram, message_id_buffer, &id, id_len + 1);
            }
        } else {
            unsafe {
                mem_bu_write(rdram, message_id_buffer, 0);
            }
        }

        ctx.set_return(if message_id.is_some() { 1i32 } else { 0i32 });
    });
}
