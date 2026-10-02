use super::*;

const FL_STUDIO_REGEX: &str = "^(.*?)(?= - FL Studio)";

#[test]
fn title_matches_invalid_regex_is_false() {
    let mut cache = RegexCache::new();
    assert!(!cache.title_matches("title", "[invalid"));
}

#[test]
fn extract_strips_unsaved_marker() {
    let mut cache = RegexCache::new();
    assert_eq!(
        cache.extract_project_name("My Song* - FL Studio", FL_STUDIO_REGEX),
        "My Song"
    );
}

#[test]
fn extract_unsaved_marker_only_is_untitled() {
    let mut cache = RegexCache::new();
    assert_eq!(
        cache.extract_project_name("* - FL Studio", FL_STUDIO_REGEX),
        UNTITLED_PROJECT
    );
}

#[test]
fn extract_lookbehind_without_group_uses_full_match() {
    let mut cache = RegexCache::new();
    assert_eq!(
        cache.extract_project_name("Bitwig Studio - My Project", "(?<=Bitwig Studio - ).*"),
        "My Project"
    );
}

#[test]
fn extract_empty_title_is_unknown() {
    let mut cache = RegexCache::new();
    assert_eq!(cache.extract_project_name("", ".*"), UNKNOWN_PROJECT);
}

#[test]
fn extract_no_match_is_unknown() {
    let mut cache = RegexCache::new();
    assert_eq!(
        cache.extract_project_name("Random Window Title", FL_STUDIO_REGEX),
        UNKNOWN_PROJECT
    );
}

#[test]
fn extract_invalid_regex_is_unknown() {
    let mut cache = RegexCache::new();
    assert_eq!(cache.extract_project_name("title", "[invalid"), UNKNOWN_PROJECT);
}
