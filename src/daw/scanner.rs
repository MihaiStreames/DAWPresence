//! Two-phase DAW scanner with event-driven exit detection.
//!
//! **Discovery**: polls process list via CreateToolhelp32Snapshot.
//! **Monitoring**: reads CPU/RAM/title for tracked PIDs.
//! Exit detection via RegisterWaitForSingleObject (NT threadpool, zero CPU idle).

use std::thread;

use tracing::debug;
use tracing::trace;

use super::config::DawConfig;
use super::config::NormalizedConfig;
use super::config::normalize_process_name;
use super::regex_cache::RegexCache;
use super::status::DawStatus;
use super::status::UNKNOWN_VERSION;
use super::win32::handle::OwnedHandle;
use super::win32::process;
use super::win32::version;
use super::win32::watcher;
use super::win32::watcher::ExitChannel;
use super::win32::window;

type Handle = windows_sys::Win32::Foundation::HANDLE;

struct TrackedProcess {
    pid: u32,
    handle: OwnedHandle,
    wait_handle: Handle,
    prev_kernel: u64,
    prev_user: u64,
    prev_wall: u64,
}

impl Drop for TrackedProcess {
    fn drop(&mut self) {
        watcher::unregister(self.wait_handle);
    }
}

struct AttachedDaw {
    config_index: usize,
    processes: Vec<TrackedProcess>,
    version: String,
}

/// Two-phase DAW monitor: discovery via process snapshot, then per-PID metric polling.
pub(crate) struct DawScanner {
    configs: Vec<NormalizedConfig>,
    regex_cache: RegexCache,
    attached: Option<AttachedDaw>,
    exits: ExitChannel,
    cpu_count: usize,
}

impl DawScanner {
    pub(crate) fn new(configs: Vec<DawConfig>) -> Self {
        let normalized = NormalizedConfig::from_configs(configs);
        debug!("Loaded {} DAW configs", normalized.len());

        Self {
            configs: normalized,
            regex_cache: RegexCache::new(),
            attached: None,
            exits: ExitChannel::new(),
            cpu_count: thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get),
        }
    }

    /// Returns current DAW status, or `None` if no DAW is running.
    /// Transitions between discovery and monitoring automatically.
    pub(crate) fn poll(&mut self) -> Option<DawStatus> {
        self.handle_exits();

        if self.attached.is_some() {
            self.read_metrics()
        } else {
            self.discover_and_attach()
        }
    }

    fn handle_exits(&mut self) {
        let dead = self.exits.drain();
        if dead.is_empty() {
            return;
        }

        if let Some(daw) = &mut self.attached {
            for pid in &dead {
                debug!("Process {pid} exited");
                daw.processes.retain(|p| p.pid != *pid);
            }

            if daw.processes.is_empty() {
                debug!("All DAW processes exited, returning to discovery");
                self.attached = None;
            }
        }
    }

    fn discover_and_attach(&mut self) -> Option<DawStatus> {
        let entries = process::snapshot();

        // normalize once per entry, not once per (entry * config)
        let normalized: Vec<(u32, String)> = entries
            .iter()
            .map(|e| (e.pid, normalize_process_name(&e.name)))
            .collect();

        for (i, cfg) in self.configs.iter().enumerate() {
            let pids: Vec<u32> = normalized
                .iter()
                .filter(|(_, name)| cfg.matches(name))
                .map(|(pid, _)| *pid)
                .collect();

            if pids.is_empty() {
                continue;
            }

            debug!("Found {} ({} processes)", cfg.display_text(), pids.len());

            self.attach(i, &pids);
            return self.read_metrics();
        }

        None
    }

    fn attach(&mut self, config_index: usize, pids: &[u32]) {
        let mut processes = Vec::new();
        let mut cached_version = String::new();

        for &pid in pids {
            let Some(handle) = process::open(pid) else {
                trace!("Couldn't open PID {pid}, skipping");
                continue;
            };

            if (cached_version.is_empty() || cached_version == UNKNOWN_VERSION)
                && let Some(path) = process::exe_path(handle.raw())
            {
                let v = version::exe_version(&path);

                if !v.is_empty() && v != UNKNOWN_VERSION {
                    cached_version = v;
                }
            }

            let wait_handle = self.exits.watch(pid, handle.raw());
            let (prev_kernel, prev_user) = process::cpu_times(handle.raw()).unwrap_or((0, 0));

            processes.push(TrackedProcess {
                pid,
                handle,
                wait_handle,
                prev_kernel,
                prev_user,
                prev_wall: process::wall_ticks(),
            });
        }

        if processes.is_empty() {
            return;
        }

        if cached_version.is_empty() {
            cached_version = UNKNOWN_VERSION.to_string();
        }

        self.attached = Some(AttachedDaw {
            config_index,
            processes,
            version: cached_version,
        });
    }

    fn read_metrics(&mut self) -> Option<DawStatus> {
        let daw = self.attached.as_mut()?;
        let cfg = &self.configs[daw.config_index];

        let mut total_cpu: f32 = 0.0;
        let mut total_memory: u64 = 0;
        let mut best_title = String::new();

        for p in &mut daw.processes {
            let h = p.handle.raw();

            if let Some(bytes) = process::memory_bytes(h) {
                total_memory += bytes / (1024 * 1024);
            }

            total_cpu += calculate_cpu_percent(p, self.cpu_count);

            let title = window::window_title(p.pid);
            if title.len() > best_title.len() {
                best_title = title;
            }
        }

        let project_name = self
            .regex_cache
            .extract_project_name(&best_title, cfg.title_regex());

        Some(DawStatus {
            is_running: true,
            display_name: cfg.display_text().to_string(),
            project_name,
            cpu_usage: total_cpu,
            memory_mb: total_memory,
            version: daw.version.clone(),
            client_id: cfg.client_id().to_string(),
            hide_version: cfg.hide_version(),
        })
    }
}

/// Per-process CPU usage as a percentage, updating baseline times in place.
fn calculate_cpu_percent(process: &mut TrackedProcess, cpu_count: usize) -> f32 {
    let Some((kernel, user)) = process::cpu_times(process.handle.raw()) else {
        return 0.0;
    };

    let now = process::wall_ticks();
    let cpu_delta = (kernel - process.prev_kernel) + (user - process.prev_user);
    let wall_delta = now - process.prev_wall;

    process.prev_kernel = kernel;
    process.prev_user = user;
    process.prev_wall = now;

    if wall_delta > 0 {
        (cpu_delta as f64 / wall_delta as f64 / cpu_count as f64 * 100.0) as f32
    } else {
        0.0
    }
}
