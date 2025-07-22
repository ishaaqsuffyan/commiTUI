#[derive(Debug, Clone, PartialEq)]
pub enum Step {
    Type,
    Scope,
    Subject,
    Body,
    Breaking,
    Preview,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub step: Step,
    pub selected_type: usize,
    pub chosen_type: Option<String>,

    pub selected_scope: usize,
    pub custom_scope: String,
    pub focus_input: bool,
    pub chosen_scope: Option<String>,

    pub subject: String,

    pub body: String,
    pub body_lines: Vec<String>,
    pub in_body: bool,

    pub breaking: String,

    pub issues: String,
    pub focus_issues: bool,
}