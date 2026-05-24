use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::mpsc;

use tracing::trace;
use tracing::warn;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::Storage::FileSystem::PIPE_ACCESS_DUPLEX;
use windows_sys::Win32::System::Pipes::CallNamedPipeW;
use windows_sys::Win32::System::Pipes::ConnectNamedPipe;
use windows_sys::Win32::System::Pipes::CreateNamedPipeW;
use windows_sys::Win32::System::Pipes::DisconnectNamedPipe;
use windows_sys::Win32::System::Pipes::PIPE_READMODE_BYTE;
use windows_sys::Win32::System::Pipes::PIPE_TYPE_BYTE;
use windows_sys::Win32::System::Pipes::PIPE_UNLIMITED_INSTANCES;
use windows_sys::Win32::System::Pipes::PIPE_WAIT;
use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::System::Threading::MUTEX_ALL_ACCESS;
use windows_sys::Win32::System::Threading::OpenMutexW;

use super::handle::OwnedHandle;
use super::to_wide_null;

static SHOW_RECEIVER: LazyLock<Mutex<Option<mpsc::Receiver<()>>>> =
    LazyLock::new(|| Mutex::new(None));

static MUTEX_HANDLE: OnceLock<OwnedHandle> = OnceLock::new();

pub(crate) fn take_receiver() -> Option<mpsc::Receiver<()>> {
    SHOW_RECEIVER.lock().ok()?.take()
}

const MUTEX_NAME: &str = "Local\\DAWPresence-SingleInstance";
const PIPE_NAME: &str = "\\\\.\\pipe\\DAWPresence-SingleInstance";
const PIPE_TIMEOUT_MS: u32 = 1000;

/// Acquires the single-instance lock. If another instance is already running,
/// signals it to show its window and returns `false`. Returns `true` otherwise.
pub(crate) fn acquire() -> bool {
    let mutex_name = to_wide_null(MUTEX_NAME);

    // succeeds only if another instance created it
    let existing = unsafe { OpenMutexW(MUTEX_ALL_ACCESS, 0, mutex_name.as_ptr()) };
    if !existing.is_null() {
        unsafe { CloseHandle(existing) };
        trace!("Another instance detected, signaling it to show");
        signal_existing();
        return false;
    }

    // we're first, create mutex and start pipe server
    let raw = unsafe { CreateMutexW(std::ptr::null(), 0, mutex_name.as_ptr()) };
    if raw.is_null() {
        warn!("CreateMutexW failed");
        return false;
    }

    let _ = MUTEX_HANDLE.set(OwnedHandle::new(raw).unwrap());

    let (sender, receiver) = mpsc::channel::<()>();
    *SHOW_RECEIVER.lock().unwrap() = Some(receiver);
    start_pipe_listener(sender);

    true
}

fn signal_existing() {
    let pipe = to_wide_null(PIPE_NAME);
    let mut buf: u8 = 1;
    let mut bytes_read: u32 = 0;

    let ok = unsafe {
        CallNamedPipeW(
            pipe.as_ptr(),
            std::ptr::addr_of_mut!(buf).cast(),
            1,
            std::ptr::addr_of_mut!(buf).cast(),
            1,
            std::ptr::addr_of_mut!(bytes_read),
            PIPE_TIMEOUT_MS,
        )
    };

    if ok == 0 {
        warn!("CallNamedPipeW failed, existing instance may not show");
    }
}

fn create_pipe(pipe_name: &[u16]) -> Option<OwnedHandle> {
    let raw = unsafe {
        CreateNamedPipeW(
            pipe_name.as_ptr(),
            PIPE_ACCESS_DUPLEX,
            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
            PIPE_UNLIMITED_INSTANCES,
            0,
            1,
            PIPE_TIMEOUT_MS,
            std::ptr::null(),
        )
    };
    OwnedHandle::new(raw)
}

fn start_pipe_listener(sender: mpsc::Sender<()>) {
    std::thread::Builder::new()
        .name("single-instance-pipe".into())
        .stack_size(64 * 1024)
        .spawn(move || run_pipe_listener(sender))
        .expect("Couldn't spawn pipe listener thread");
}

#[allow(clippy::needless_pass_by_value)]
fn run_pipe_listener(sender: mpsc::Sender<()>) {
    let pipe_name = to_wide_null(PIPE_NAME);

    let Some(mut pipe) = create_pipe(&pipe_name) else {
        warn!("CreateNamedPipeW failed, single-instance pipe unavailable");
        return;
    };

    loop {
        // blocks until second instance connects
        unsafe { ConnectNamedPipe(pipe.raw(), std::ptr::null_mut()) };
        trace!("Second instance connected, sending show signal");

        let next = create_pipe(&pipe_name); // avoids teardown race

        unsafe { DisconnectNamedPipe(pipe.raw()) };
        drop(pipe);

        if sender.send(()).is_err() {
            return;
        }

        let Some(next_pipe) = next else {
            warn!("CreateNamedPipeW failed, single-instance pipe unavailable");
            return;
        };

        pipe = next_pipe;
    }
}
