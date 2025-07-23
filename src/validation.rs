use crate::config::Config; // Import Config

pub fn validate_subject(subject: &str, config: &Config) -> Option<String> {
    if subject.trim().is_empty() {
        return Some("Subject must not be empty.".to_string());
    }
    if subject.len() > config.subject_max_length {
        return Some(format!("Subject should be {} characters or less (currently {}).", config.subject_max_length, subject.len()));
    }
    if config.subject_no_ending_period && subject.ends_with('.') {
        return Some("Subject should not end with a period.".to_string());
    }
    // Only check if subject_start_lowercase is true in config
    if config.subject_start_lowercase && subject.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
        return Some("Subject should start with a lowercase letter.".to_string());
    }
    None
}