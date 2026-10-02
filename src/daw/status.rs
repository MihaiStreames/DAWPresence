pub(crate) const UNKNOWN_VERSION: &str = "0.0.0";
pub(crate) const UNKNOWN_PROJECT: &str = "None";
pub(crate) const UNTITLED_PROJECT: &str = "Untitled";

/// Current state of a detected DAW
#[derive(Debug, Clone, Default)]
pub(crate) struct DawStatus {
    pub(crate) is_running: bool,
    pub(crate) display_name: String,
    pub(crate) project_name: String,
    pub(crate) cpu_usage: f32,
    pub(crate) memory_mb: u64,
    pub(crate) version: String,
    pub(crate) client_id: String,
    pub(crate) hide_version: bool,
}

impl DawStatus {
    pub(crate) fn cpu_usage_str(&self) -> String {
        if self.is_running {
            format!("{:.2}%", self.cpu_usage)
        } else {
            "Undefined".to_owned()
        }
    }

    pub(crate) fn ram_usage_str(&self) -> String {
        if self.is_running {
            if self.memory_mb >= 1024 {
                format!("{:.2}GB", self.memory_mb as f64 / 1024.0)
            } else {
                format!("{}MB", self.memory_mb)
            }
        } else {
            "Undefined".to_owned()
        }
    }
}

#[cfg(test)]
#[path = "tests/status.rs"]
mod tests;
