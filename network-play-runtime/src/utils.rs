use n64_recomp::RecompContext;
use std::panic;

use crate::network::{NetworkPlayModule, NETWORK_PLAY};

/// Helper function to safely execute code that might panic
pub fn execute_safely<F>(ctx: *mut RecompContext, func_name: &str, f: F)
where
    F: FnOnce(&mut RecompContext) + panic::UnwindSafe,
{
    let result = panic::catch_unwind(|| {
        let ctx = unsafe { &mut *ctx };
        f(ctx)
    });

    if let Err(e) = result {
        log::error!("Panic in {}: {:?}", func_name, e);
        // Try to set return value to 0 if we can access ctx safely
        if let Ok(ctx) = panic::catch_unwind(|| unsafe { &mut *ctx }) {
            ctx.set_return(0i32);
        }
    }
}

pub fn with_network_play_mut<F, R>(f: F, default: R) -> R
where
    F: FnOnce(&mut NetworkPlayModule) -> R,
{
    match NETWORK_PLAY.get() {
        Some(network_play) => match network_play.lock() {
            Ok(mut module) => f(&mut module),
            Err(e) => {
                log::error!("Failed to lock network play module: {}", e);
                default
            }
        },
        None => {
            log::error!("Network play module not initialized");
            default
        }
    }
}

pub fn with_network_play<F, R>(f: F, default: R) -> R
where
    F: FnOnce(&NetworkPlayModule) -> R,
{
    match NETWORK_PLAY.get() {
        Some(network_play) => match network_play.lock() {
            Ok(module) => f(&module),
            Err(e) => {
                log::error!("Failed to lock network play module: {}", e);
                default
            }
        },
        None => {
            log::error!("Network play module not initialized");
            default
        }
    }
}
