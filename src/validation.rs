pub fn validate_subject(subject: &str) -> Option<String> {
    if subject.trim().is_empty() {
        return Some("Subject must not be empty.".to_string());
    }
    if subject.len() > 72 {
        return Some("Subject should be 72 characters or less.".to_string());
    }
    if subject.ends_with('.') {
        return Some("Subject should not end with a period.".to_string());
    }
    if subject.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
        return Some("Subject should start with a lowercase letter.".to_string());
    }
    None
}