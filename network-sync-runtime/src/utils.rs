use n64_recomp::RecompContext;
use std::panic;

use crate::network::{NetworkSyncModule, NETWORK_PLAY};

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

pub fn with_network_sync_mut<F, R>(f: F, default: R) -> R
where
    F: FnOnce(&mut NetworkSyncModule) -> R,
{
    match NETWORK_PLAY.get() {
        Some(network_sync) => match network_sync.lock() {
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

pub fn with_network_sync<F, R>(f: F, default: R) -> R
where
    F: FnOnce(&NetworkSyncModule) -> R,
{
    match NETWORK_PLAY.get() {
        Some(network_sync) => match network_sync.lock() {
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
