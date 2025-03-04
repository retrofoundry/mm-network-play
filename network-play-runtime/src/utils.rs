use n64_recomp::RecompContext;
use std::panic;

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
