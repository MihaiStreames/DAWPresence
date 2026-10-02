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
        Self { cache: HashMap::new() }
    }

    /// Check if a title matches the pattern at all (used to filter false-positive windows)
    pub(super) fn title_matches(&mut self, title: &str, pattern: &str) -> bool {
        let re = self
            .cache
            .entry(pattern.to_owned())
            .or_insert_with(|| match Regex::new(pattern) {
                Ok(re) => Some(re),
                Err(e) => {
                    warn!("Invalid regex pattern: {pattern}: {e}");
                    None
                }
            });

        re.as_ref().and_then(|re| re.is_match(title).ok()).unwrap_or(false)
    }

    /// Extract project name from a window title using a cached compiled regex
    pub(super) fn extract_project_name(&mut self, title: &str, pattern: &str) -> String {
        if title.is_empty() {
            return UNKNOWN_PROJECT.to_owned();
        }

        let re = self
            .cache
            .entry(pattern.to_owned())
            .or_insert_with(|| match Regex::new(pattern) {
                Ok(re) => Some(re),
                Err(e) => {
                    warn!("Invalid regex pattern: {pattern}: {e}");
                    None
                }
            });

        let Some(re) = re else {
            return UNKNOWN_PROJECT.to_owned();
        };

        let Ok(Some(captures)) = re.captures(title) else {
            return UNKNOWN_PROJECT.to_owned();
        };

        // prefer capture group 1 (named match) over group 0 (full match)
        // for patterns without groups

        captures
            .get(1)
            .or_else(|| captures.get(0))
            .map(|m| m.as_str().trim())
            .map(|s| s.trim_end_matches('*').trim())
            .map(|s| if s.is_empty() { UNTITLED_PROJECT } else { s })
            .map_or_else(|| UNKNOWN_PROJECT.to_owned(), String::from)
    }
}

#[cfg(test)]
#[path = "tests/regex_cache.rs"]
mod tests;
