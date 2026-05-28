use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;

/// Owned Win32 handle. Calls `CloseHandle` on drop.
pub(crate) struct OwnedHandle(HANDLE);

impl OwnedHandle {
    /// Wrap a raw handle. Returns `None` for null or `INVALID_HANDLE_VALUE`.
    pub(crate) fn new(raw: HANDLE) -> Option<Self> {
        if raw.is_null() || raw == INVALID_HANDLE_VALUE {
            None
        } else {
            Some(Self(raw))
        }
    }

    pub(crate) const fn raw(&self) -> HANDLE {
        self.0
    }
}

unsafe impl Send for OwnedHandle {}
unsafe impl Sync for OwnedHandle {}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}
