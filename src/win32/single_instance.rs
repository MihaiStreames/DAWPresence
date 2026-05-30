use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::mpsc;

use tracing::trace;
use tracing::warn;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Foundation::ERROR_ALREADY_EXISTS;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::Foundation::WAIT_FAILED;
use windows_sys::Win32::System::Threading::CreateEventW;
use windows_sys::Win32::System::Threading::INFINITE;
use windows_sys::Win32::System::Threading::SetEvent;
use windows_sys::Win32::System::Threading::WaitForSingleObject;

use super::handle::OwnedHandle;
use super::to_wide_null;

const EVENT_NAME: &str = "Local\\DAWPresence-SingleInstance";

static SHOW_RECEIVER: LazyLock<Mutex<Option<mpsc::Receiver<()>>>> =
    LazyLock::new(|| Mutex::new(None));

static EVENT_HANDLE: OnceLock<OwnedHandle> = OnceLock::new();

pub(crate) fn take_receiver() -> Option<mpsc::Receiver<()>> {
    SHOW_RECEIVER.lock().ok()?.take()
}

/// Acquires the single-instance lock. If another instance is already running,
/// signals it to show its window and returns `false`. Returns `true` otherwise.
pub(crate) fn acquire() -> bool {
    let event_name = to_wide_null(EVENT_NAME);

    // auto-reset: OS atomically wakes one waiter and clears the event, no lost-wakeup race
    let raw = unsafe { CreateEventW(std::ptr::null(), 0, 0, event_name.as_ptr()) };
    if raw.is_null() {
        warn!("CreateEventW failed");
        return false;
    }

    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        unsafe { CloseHandle(raw) };
        trace!("Another instance detected, signaling it to show");
        signal_existing();
        return false;
    }

    let _ = EVENT_HANDLE.set(OwnedHandle::new(raw).expect("CreateEventW returned invalid handle"));

    let (sender, receiver) = mpsc::channel::<()>();
    *SHOW_RECEIVER.lock().expect("SHOW_RECEIVER mutex poisoned") = Some(receiver);
    start_event_listener(sender);

    true
}

fn signal_existing() {
    let event_name = to_wide_null(EVENT_NAME);

    let raw = unsafe { CreateEventW(std::ptr::null(), 0, 0, event_name.as_ptr()) };
    if raw.is_null() {
        warn!("CreateEventW failed, existing instance may not show");
        return;
    }

    unsafe {
        SetEvent(raw);
        CloseHandle(raw);
    }
}

fn start_event_listener(sender: mpsc::Sender<()>) {
    std::thread::Builder::new()
        .name("single-instance-event".into())
        .stack_size(64 * 1024)
        .spawn(move || run_event_listener(sender))
        .expect("Couldn't spawn event listener thread");
}

#[allow(clippy::needless_pass_by_value)]
fn run_event_listener(sender: mpsc::Sender<()>) {
    let Some(handle) = EVENT_HANDLE.get() else {
        warn!("Event handle not set, listener exiting");
        return;
    };

    loop {
        let result = unsafe { WaitForSingleObject(handle.raw(), INFINITE) };
        if result == WAIT_FAILED {
            warn!("WaitForSingleObject failed, single-instance listener exiting");
            return;
        }

        trace!("Second instance signaled, sending show signal");

        if sender.send(()).is_err() {
            return;
        }
    }
}
