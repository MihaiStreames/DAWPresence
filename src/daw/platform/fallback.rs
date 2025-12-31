use std::path::Path;

use sysinfo::Pid;

/// Return a default version on unsupported platforms
pub fn get_process_version(_exe_path: Option<&Path>) -> String {
    "0.0.0".to_string()
}

/// Return empty window titles on unsupported platforms
pub fn get_window_title(_pid: Pid) -> String {
    String::new()
}
