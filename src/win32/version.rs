use std::os::windows::ffi::OsStrExt as _;
use std::path::Path;

use windows_sys::Win32::Foundation::FALSE;
use windows_sys::Win32::Storage::FileSystem::GetFileVersionInfoSizeW;
use windows_sys::Win32::Storage::FileSystem::GetFileVersionInfoW;
use windows_sys::Win32::Storage::FileSystem::VerQueryValueW;

use super::to_wide_null;
use crate::daw::UNKNOWN_VERSION;

const QUERY: &str = "\\VarFileInfo\\Translation";

/// Read the `ProductVersion` string from a PE file's version resource.
///
/// Returns [`UNKNOWN_VERSION`] if the version cannot be read.
pub(crate) fn exe_version(path: &Path) -> String {
    let path_wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let Some(data) = load_version_info(&path_wide) else {
        return UNKNOWN_VERSION.to_owned();
    };

    let data_range = data.as_ptr() as usize..data.as_ptr() as usize + data.len();

    let Some((lang, codepage)) = parse_translation(&data, &data_range) else {
        return UNKNOWN_VERSION.to_owned();
    };

    query_version_string(&data, &data_range, lang, codepage)
        .unwrap_or_else(|| UNKNOWN_VERSION.to_owned())
}

fn load_version_info(path_wide: &[u16]) -> Option<Vec<u8>> {
    let mut handle: u32 = 0;
    let size = unsafe { GetFileVersionInfoSizeW(path_wide.as_ptr(), &raw mut handle) };
    if size == 0 {
        return None;
    }

    let mut data = vec![0u8; size as usize];
    if unsafe { GetFileVersionInfoW(path_wide.as_ptr(), handle, size, data.as_mut_ptr().cast()) }
        == FALSE
    {
        return None;
    }

    Some(data)
}

fn parse_translation(data: &[u8], data_range: &std::ops::Range<usize>) -> Option<(u16, u16)> {
    let query = to_wide_null(QUERY);

    let mut ptr: *mut core::ffi::c_void = std::ptr::null_mut();
    let mut len: u32 = 0;

    let ok = unsafe {
        VerQueryValueW(
            data.as_ptr().cast(),
            query.as_ptr(),
            &raw mut ptr,
            &raw mut len,
        )
    };

    if ok == FALSE || ptr.is_null() || len < 4 {
        return None;
    }

    // bounds check: ptr must point within data buffer
    let addr = ptr as usize;
    if !data_range.contains(&addr) || addr + 4 > data_range.end {
        return None;
    }

    let translation = unsafe { std::slice::from_raw_parts(ptr as *const u16, 2) };
    #[allow(clippy::indexing_slicing)]
    Some((translation[0], translation[1]))
}

fn query_version_string(
    data: &[u8],
    data_range: &std::ops::Range<usize>,
    lang: u16,
    codepage: u16,
) -> Option<String> {
    let query = to_wide_null(&format!(
        "\\StringFileInfo\\{lang:04X}{codepage:04X}\\ProductVersion"
    ));

    let mut ptr: *mut core::ffi::c_void = std::ptr::null_mut();
    let mut len: u32 = 0;

    let ok = unsafe {
        VerQueryValueW(
            data.as_ptr().cast(),
            query.as_ptr(),
            &raw mut ptr,
            &raw mut len,
        )
    };

    if ok == FALSE || ptr.is_null() {
        return None;
    }

    let addr = ptr as usize;
    if !data_range.contains(&addr) {
        return None;
    }

    let len = len as usize;
    let max_len = (data_range.end - addr) / 2;
    let len = len.min(max_len).max(1);

    let wide = unsafe { std::slice::from_raw_parts(ptr as *const u16, len) };
    let null_pos = wide.iter().position(|c| *c == 0).unwrap_or(wide.len());
    #[allow(clippy::indexing_slicing)]
    let version = String::from_utf16_lossy(&wide[..null_pos])
        .trim()
        .to_owned();

    if version.is_empty() {
        None
    } else {
        Some(version)
    }
}
