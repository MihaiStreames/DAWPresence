//! DAW status data type and display formatting.

/// Placeholder when version cannot be read from a PE resource.
pub(crate) const UNKNOWN_VERSION: &str = "0.0.0";

/// Current state of a detected DAW.
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
            "Undefined".to_string()
        }
    }

    pub(crate) fn ram_usage_str(&self) -> String {
        if self.is_running {
            let memory_kb = self.memory_mb.saturating_mul(1024);

            if memory_kb >= 1024 * 1024 {
                let memory_gb = memory_kb as f64 / (1024.0 * 1024.0);
                format!("{memory_gb:.2}GB")
            } else if memory_kb >= 1024 {
                format!("{}MB", self.memory_mb)
            } else {
                format!("{memory_kb}KB")
            }
        } else {
            "Undefined".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_usage_str_running() {
        let status = DawStatus {
            is_running: true,
            cpu_usage: 12.345,
            ..Default::default()
        };
        assert_eq!(status.cpu_usage_str(), "12.35%");
    }

    #[test]
    fn cpu_usage_str_not_running() {
        let status = DawStatus::default();
        assert_eq!(status.cpu_usage_str(), "Undefined");
    }

    #[test]
    fn ram_usage_str_mb() {
        let status = DawStatus {
            is_running: true,
            memory_mb: 512,
            ..Default::default()
        };
        assert_eq!(status.ram_usage_str(), "512MB");
    }

    #[test]
    fn ram_usage_str_gb() {
        let status = DawStatus {
            is_running: true,
            memory_mb: 2048,
            ..Default::default()
        };
        assert_eq!(status.ram_usage_str(), "2.00GB");
    }

    #[test]
    fn ram_usage_str_not_running() {
        let status = DawStatus::default();
        assert_eq!(status.ram_usage_str(), "Undefined");
    }
}
