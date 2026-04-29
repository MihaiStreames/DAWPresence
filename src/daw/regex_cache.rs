use std::collections::HashMap;

use fancy_regex::Regex;
use tracing::warn;

use super::status::UNKNOWN_PROJECT;
use super::status::UNTITLED_PROJECT;

pub(super) struct RegexCache {
    cache: HashMap<String, Option<Regex>>,
}

impl RegexCache {
    pub(super) fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Extract project name from a window title using a cached compiled regex.
    pub(super) fn extract_project_name(&mut self, title: &str, pattern: &str) -> String {
        if title.is_empty() {
            return UNKNOWN_PROJECT.to_string();
        }

        let re =
            self.cache
                .entry(pattern.to_owned())
                .or_insert_with(|| match Regex::new(pattern) {
                    Ok(re) => Some(re),
                    Err(e) => {
                        warn!("Invalid regex pattern: {pattern}: {e}");
                        None
                    }
                });

        let Some(re) = re else {
            return UNKNOWN_PROJECT.to_string();
        };

        let Ok(Some(captures)) = re.captures(title) else {
            return UNKNOWN_PROJECT.to_string();
        };

        captures
            .get(1)
            .or_else(|| captures.get(0))
            .map(|m| m.as_str().trim())
            .map(|s| s.trim_end_matches('*').trim())
            .map(|s| if s.is_empty() { UNTITLED_PROJECT } else { s })
            .map_or_else(|| UNKNOWN_PROJECT.to_string(), String::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_fl_studio_project() {
        let mut cache = RegexCache::new();
        let regex = r"^(.*?)(?= - FL Studio)";
        assert_eq!(
            cache.extract_project_name("My Song - FL Studio", regex),
            "My Song"
        );
    }

    #[test]
    fn extract_fl_studio_dirty() {
        let mut cache = RegexCache::new();
        let regex = r"^(.*?)(?= - FL Studio)";
        assert_eq!(
            cache.extract_project_name("My Song* - FL Studio", regex),
            "My Song"
        );
    }

    #[test]
    fn extract_ableton_project() {
        let mut cache = RegexCache::new();
        let regex = r"^(.*?)(?= - Ableton Live 12 Suite)";
        assert_eq!(
            cache.extract_project_name("Demo Track - Ableton Live 12 Suite", regex),
            "Demo Track"
        );
    }

    #[test]
    fn extract_reaper_project() {
        let mut cache = RegexCache::new();
        let regex = r"^(.*?)(?= - REAPER v)";
        assert_eq!(
            cache.extract_project_name("Mix Session - REAPER v7.0", regex),
            "Mix Session"
        );
    }

    #[test]
    fn extract_bitwig_project() {
        let mut cache = RegexCache::new();
        let regex = r"(?<=Bitwig Studio - ).*";
        assert_eq!(
            cache.extract_project_name("Bitwig Studio - My Project", regex),
            "My Project"
        );
    }

    #[test]
    fn extract_studio_one_project() {
        let mut cache = RegexCache::new();
        let regex = r"(?<=Studio One - ).*";
        assert_eq!(
            cache.extract_project_name("Studio One - Song.song", regex),
            "Song.song"
        );
    }

    #[test]
    fn extract_lmms_project() {
        let mut cache = RegexCache::new();
        let regex = r"^(.*?)(?= - LMMS)";
        assert_eq!(cache.extract_project_name("Beat - LMMS", regex), "Beat");
    }

    #[test]
    fn extract_cubase_project() {
        let mut cache = RegexCache::new();
        let regex = r"(?<=Cubase Pro Project - ).*";
        assert_eq!(
            cache.extract_project_name("Cubase Pro Project - Film Score", regex),
            "Film Score"
        );
    }

    #[test]
    fn extract_empty_title() {
        let mut cache = RegexCache::new();
        assert_eq!(cache.extract_project_name("", r".*"), "None");
    }

    #[test]
    fn extract_no_match() {
        let mut cache = RegexCache::new();
        let regex = r"^(.*?)(?= - FL Studio)";
        assert_eq!(
            cache.extract_project_name("Random Window Title", regex),
            "None"
        );
    }

    #[test]
    fn extract_invalid_regex() {
        let mut cache = RegexCache::new();
        assert_eq!(cache.extract_project_name("title", r"[invalid"), "None");
    }

    #[test]
    fn caches_compiled_regex() {
        let mut cache = RegexCache::new();
        let regex = r"^(.*?)(?= - FL Studio)";
        cache.extract_project_name("Song1 - FL Studio", regex);
        cache.extract_project_name("Song2 - FL Studio", regex);
        // Should only have one entry
        assert_eq!(cache.cache.len(), 1);
    }
}
