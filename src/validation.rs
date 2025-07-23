use crate::config::Config; // Import Config

pub fn validate_subject(subject: &str, config: &Config) -> Option<String> {
    // Get validation rules from config, unwrapping Options to their effective default if None.
    // This uses the defaults defined in the `default_subject_*` functions if the field
    // was not set in *any* config file (local or global).
    let max_length = config.subject_max_length.unwrap_or_else(crate::config::default_subject_max_length);
    let start_lowercase = config.subject_start_lowercase.unwrap_or_else(crate::config::default_subject_start_lowercase);
    let no_ending_period = config.subject_no_ending_period.unwrap_or_else(crate::config::default_subject_no_ending_period);

    if subject.trim().is_empty() {
        return Some("Subject must not be empty.".to_string());
    }
    if subject.len() > max_length {
        return Some(format!("Subject should be {} characters or less (currently {}).", max_length, subject.len()));
    }
    if no_ending_period && subject.ends_with('.') {
        return Some("Subject should not end with a period.".to_string());
    }
    if start_lowercase && subject.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
        return Some("Subject should start with a lowercase letter.".to_string());
    }
    None
}