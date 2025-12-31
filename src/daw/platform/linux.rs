use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use regex::Regex;
use sysinfo::Pid;
use tracing::debug;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{Atom, AtomEnum, ConnectionExt, Window};

/// Try to infer a version string for a Linux executable
pub fn get_process_version(exe_path: Option<&Path>) -> String {
    let Some(path) = exe_path else {
        return "0.0.0".to_string();
    };

    if let Some(version) = get_process_version_from_command(path) {
        return version;
    }

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let full_path = path.to_string_lossy();

    let Ok(re) = Regex::new(r"\d+(?:\.\d+)+") else {
        return "0.0.0".to_string();
    };

    re.find(file_name)
        .or_else(|| re.find(&full_path))
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| "0.0.0".to_string())
}

/// Run common version flags and parse the first version-like token
fn get_process_version_from_command(path: &Path) -> Option<String> {
    let re = Regex::new(r"\d+(?:\.\d+)+").ok()?;
    let flags = ["--version", "-v", "-V"];

    for flag in flags {
        if let Some(version) = run_version_command(path, flag)
            .and_then(|output| find_version_in_text(&re, &output))
        {
            return Some(version);
        }
    }

    None
}

/// Execute a binary with a version flag and return combined stdout/stderr
fn run_version_command(path: &Path, flag: &str) -> Option<String> {
    let mut child = Command::new(path)
        .arg(flag)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;

    let timeout = Duration::from_millis(500);
    let start = Instant::now();

    loop {
        if let Some(_status) = child.try_wait().ok()? {
            let mut stdout = String::new();
            let mut stderr = String::new();

            if let Some(mut out) = child.stdout.take() {
                let _ = out.read_to_string(&mut stdout);
            }
            if let Some(mut err) = child.stderr.take() {
                let _ = err.read_to_string(&mut stderr);
            }

            let combined = format!("{} {}", stdout, stderr);
            return Some(combined);
        }

        if start.elapsed() >= timeout {
            let _ = child.kill();
            return None;
        }

        thread::sleep(Duration::from_millis(25));
    }
}

/// Extract the first version-like token from text
fn find_version_in_text(re: &Regex, text: &str) -> Option<String> {
    re.find(text).map(|m| m.as_str().to_string())
}

/// Fetch the best-effort window title for a PID on Linux
pub fn get_window_title(pid: Pid) -> String {
    if std::env::var_os("DISPLAY").is_some() {
        if let Some(title) = get_window_title_x11(pid.as_u32()) {
            return title;
        }
    } else if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        debug!("wayland detected, window title lookup not available");
    }

    String::new()
}

/// Walk X11 windows and return the first title matching the PID
fn get_window_title_x11(target_pid: u32) -> Option<String> {
    let (conn, screen_num) = x11rb::connect(None).ok()?;
    let root = conn.setup().roots.get(screen_num)?.root;

    let atom_pid = intern_atom(&conn, b"_NET_WM_PID")?;
    let atom_net_wm_name = intern_atom(&conn, b"_NET_WM_NAME")?;
    let atom_utf8 = intern_atom(&conn, b"UTF8_STRING")?;
    let atom_wm_name: Atom = AtomEnum::WM_NAME.into();
    let atom_string: Atom = AtomEnum::STRING.into();

    let mut stack = vec![root];

    while let Some(window) = stack.pop() {
        if let Ok(cookie) = conn.query_tree(window) {
            if let Ok(reply) = cookie.reply() {
                stack.extend(reply.children);
            }
        }

        if !window_pid_matches(&conn, window, atom_pid, target_pid) {
            continue;
        }

        if let Some(title) = window_title_for_atom(&conn, window, atom_net_wm_name, atom_utf8) {
            if !title.trim().is_empty() {
                return Some(title);
            }
        }

        if let Some(title) = window_title_for_atom(&conn, window, atom_wm_name, atom_string) {
            if !title.trim().is_empty() {
                return Some(title);
            }
        }
    }

    None
}

/// Resolve an X11 atom by name
fn intern_atom<C: Connection>(conn: &C, name: &[u8]) -> Option<Atom> {
    conn.intern_atom(false, name).ok()?.reply().ok().map(|r| r.atom)
}

/// Check whether a window's PID matches the target process
fn window_pid_matches<C: Connection>(
    conn: &C,
    window: Window,
    atom_pid: Atom,
    target_pid: u32,
) -> bool {
    let Ok(cookie) = conn.get_property(false, window, atom_pid, AtomEnum::CARDINAL, 0, 1) else {
        return false;
    };

    let Ok(reply) = cookie.reply() else {
        return false;
    };

    if reply.format != 32 {
        return false;
    }

    reply
        .value32()
        .and_then(|mut iter| iter.next())
        .map(|pid| pid == target_pid)
        .unwrap_or(false)
}

/// Read a window title from a specific property and type
fn window_title_for_atom<C: Connection>(
    conn: &C,
    window: Window,
    property: Atom,
    atom_type: Atom,
) -> Option<String> {
    let Ok(cookie) = conn.get_property(false, window, property, atom_type, 0, 1024) else {
        return None;
    };

    let Ok(reply) = cookie.reply() else {
        return None;
    };

    if reply.format != 8 || reply.value.is_empty() {
        return None;
    }

    let title = String::from_utf8_lossy(&reply.value)
        .trim_end_matches('\0')
        .trim()
        .to_string();
    if title.is_empty() {
        None
    } else {
        Some(title)
    }
}
