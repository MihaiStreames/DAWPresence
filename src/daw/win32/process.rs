//! Win32 process enumeration and metrics.

use std::mem::size_of;
use std::mem::zeroed;
use std::path::PathBuf;

use windows_sys::Win32::Foundation::FALSE;
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::Foundation::MAX_PATH;
use windows_sys::Win32::System::Diagnostics::ToolHelp::CreateToolhelp32Snapshot;
use windows_sys::Win32::System::Diagnostics::ToolHelp::PROCESSENTRY32W;
use windows_sys::Win32::System::Diagnostics::ToolHelp::Process32FirstW;
use windows_sys::Win32::System::Diagnostics::ToolHelp::Process32NextW;
use windows_sys::Win32::System::Diagnostics::ToolHelp::TH32CS_SNAPPROCESS;
use windows_sys::Win32::System::ProcessStatus::GetProcessMemoryInfo;
use windows_sys::Win32::System::ProcessStatus::PROCESS_MEMORY_COUNTERS;
use windows_sys::Win32::System::Threading::GetProcessTimes;
use windows_sys::Win32::System::Threading::OpenProcess;
use windows_sys::Win32::System::Threading::PROCESS_QUERY_LIMITED_INFORMATION;
use windows_sys::Win32::System::Threading::PROCESS_SYNCHRONIZE;
use windows_sys::Win32::System::Threading::QueryFullProcessImageNameW;

use super::handle::OwnedHandle;

/// A process from the system snapshot.
#[derive(Debug, Clone)]
pub(in crate::daw) struct ProcessEntry {
    pub(in crate::daw) pid: u32,
    pub(in crate::daw) name: String,
}

/// Enumerate all running processes.
pub(in crate::daw) fn snapshot() -> Vec<ProcessEntry> {
    // SAFETY: standard Win32 call, snapshot handle cleaned up by OwnedHandle
    let raw = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    let Some(_guard) = OwnedHandle::new(raw) else {
        return Vec::new();
    };

    let mut entry: PROCESSENTRY32W = unsafe { zeroed() };
    entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;

    // SAFETY: entry is zeroed and correctly sized
    if unsafe { Process32FirstW(raw, &mut entry) } == FALSE {
        return Vec::new();
    }

    let mut entries = Vec::new();
    loop {
        let name_len = entry
            .szExeFile
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(entry.szExeFile.len());

        entries.push(ProcessEntry {
            pid: entry.th32ProcessID,
            name: String::from_utf16_lossy(&entry.szExeFile[..name_len]),
        });

        // SAFETY: entry is valid from previous call
        if unsafe { Process32NextW(raw, &mut entry) } == FALSE {
            break;
        }
    }

    entries
}

/// Open a process handle for monitoring and synchronization.
pub(in crate::daw) fn open(pid: u32) -> Option<OwnedHandle> {
    // SAFETY: standard Win32 call, only requesting limited info + sync
    let raw = unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
            FALSE, // no inherit
            pid,
        )
    };

    OwnedHandle::new(raw)
}

/// Working set memory in bytes.
pub(in crate::daw) fn memory_bytes(handle: HANDLE) -> Option<u64> {
    let mut counters: PROCESS_MEMORY_COUNTERS = unsafe { zeroed() };
    counters.cb = size_of::<PROCESS_MEMORY_COUNTERS>() as u32;

    // SAFETY: handle is valid, counters is zeroed with correct cb
    let ok = unsafe { GetProcessMemoryInfo(handle, &mut counters, counters.cb) };
    if ok == FALSE {
        return None;
    }

    Some(counters.WorkingSetSize as u64)
}

/// Kernel + user time in 100ns ticks.
pub(in crate::daw) fn cpu_times(handle: HANDLE) -> Option<(u64, u64)> {
    let mut creation = 0u64;
    let mut exit = 0u64;
    let mut kernel = 0u64;
    let mut user = 0u64;

    // SAFETY: handle is valid, all pointers are to stack variables
    let ok = unsafe {
        GetProcessTimes(
            handle,
            &mut creation as *mut u64 as *mut _,
            &mut exit as *mut u64 as *mut _,
            &mut kernel as *mut u64 as *mut _,
            &mut user as *mut u64 as *mut _,
        )
    };

    if ok == FALSE {
        return None;
    }

    Some((kernel, user))
}

/// Full executable path for a process.
pub(in crate::daw) fn exe_path(handle: HANDLE) -> Option<PathBuf> {
    let mut buf = [0u16; MAX_PATH as usize];
    let mut len = buf.len() as u32;

    // SAFETY: handle is valid, buf is correctly sized, len is in/out
    let ok = unsafe { QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut len) };
    if ok == FALSE {
        return None;
    }

    Some(PathBuf::from(String::from_utf16_lossy(
        &buf[..len as usize],
    )))
}

/// Wall-clock time in 100ns ticks.
pub(in crate::daw) fn wall_ticks() -> u64 {
    use std::time::SystemTime;
    use std::time::UNIX_EPOCH;
    // epoch offset cancels in delta calculations
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
        / 100
}
