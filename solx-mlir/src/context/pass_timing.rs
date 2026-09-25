//!
//! MLIR pass timing.
//!

use std::ffi::c_void;
use std::time::Duration;

/// One entry of MLIR's pass timing report.
#[derive(Debug)]
pub struct PassTiming {
    /// The name MLIR reports.
    pub name: String,
    /// The nesting level, 0 for a pipeline pass and a summary row.
    pub depth: u32,
    /// The wall time MLIR measured.
    pub duration: Duration,
}

impl PassTiming {
    /// # Safety
    ///
    /// `user_data` must point at a live `Vec<PassTiming>` that nothing else borrows, and `name`
    /// must be valid for `name.length` bytes.
    pub unsafe extern "C" fn push(
        name: mlir_sys::MlirStringRef,
        depth: u32,
        wall_seconds: f64,
        user_data: *mut c_void,
    ) {
        let pass_timings = unsafe { &mut *user_data.cast::<Vec<Self>>() };
        let name = unsafe { std::slice::from_raw_parts(name.data.cast::<u8>(), name.length) };
        pass_timings.push(Self {
            name: String::from_utf8_lossy(name).into_owned(),
            depth,
            // `Rest` is the total minus the passes in floating point, which can come out negative.
            duration: Duration::from_secs_f64(wall_seconds.max(0.0)),
        });
    }
}
