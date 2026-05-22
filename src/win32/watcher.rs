use std::ffi::c_void;

use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use windows_sys::Win32::Foundation::FALSE;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
use windows_sys::Win32::System::Threading::RegisterWaitForSingleObject;
use windows_sys::Win32::System::Threading::UnregisterWaitEx;
use windows_sys::Win32::System::Threading::WT_EXECUTEONLYONCE;

/// Channel pair for receiving process exit notifications.
pub(crate) struct ExitChannel {
    tx: Sender<u32>,
    rx: Receiver<u32>,
}

impl ExitChannel {
    pub(crate) fn new() -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();
        Self { tx, rx }
    }

    /// Drain all pending exit PIDs without blocking.
    pub(crate) fn drain(&self) -> Vec<u32> {
        self.rx.try_iter().collect()
    }

    /// Register an NT threadpool wait on a process handle.
    ///
    /// Returns the wait handle for cleanup, or null on failure.
    pub(crate) fn watch(&self, pid: u32, process_handle: HANDLE) -> HANDLE {
        let ctx = Box::into_raw(Box::new((pid, self.tx.clone())));
        let mut wait_handle: HANDLE = std::ptr::null_mut();

        // SAFETY: process_handle is valid (from open()), ctx lives until callback frees it
        let ok = unsafe {
            RegisterWaitForSingleObject(
                &raw mut wait_handle,
                process_handle,
                Some(exit_callback),
                ctx.cast(),
                u32::MAX, // INFINITE
                WT_EXECUTEONLYONCE,
            )
        };

        if ok == FALSE {
            // SAFETY: registration failed, we still own ctx
            let _ = unsafe { Box::from_raw(ctx) };
            tracing::warn!("RegisterWaitForSingleObject failed for PID {pid}");
            return std::ptr::null_mut();
        }

        wait_handle
    }
}

/// Cancel a registered wait, blocking until any in-flight callback completes.
/// No-op if handle is null.
pub(crate) fn unregister(wait_handle: HANDLE) {
    if !wait_handle.is_null() {
        // SAFETY: wait_handle is from RegisterWaitForSingleObject;
        // INVALID_HANDLE_VALUE makes UnregisterWaitEx block until the callback finishes,
        // preventing use-after-free if the callback is mid-execution during drop
        unsafe {
            UnregisterWaitEx(wait_handle, INVALID_HANDLE_VALUE);
        }
    }
}

unsafe extern "system" fn exit_callback(ctx: *mut c_void, _timed_out: bool) {
    if ctx.is_null() {
        return;
    }
    // SAFETY: ctx was created by Box::into_raw in watch()
    let ctx = unsafe { Box::from_raw(ctx.cast::<(u32, Sender<u32>)>()) };
    let _ = ctx.1.send(ctx.0);
}
