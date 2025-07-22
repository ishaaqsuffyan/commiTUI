use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    style::{Style, Color},
    layout::{Layout, Constraint, Direction, Rect},
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, error::Error};

const COMMIT_TYPES: &[(&str, &str)] = &[
    ("feat", "A new feature"),
    ("fix", "A bug fix"),
    ("docs", "Documentation only changes"),
    ("style", "Code style changes (formatting, etc)"),
    ("refactor", "Code change that neither fixes a bug nor adds a feature"),
    ("perf", "Performance improvements"),
    ("test", "Adding or correcting tests"),
    ("build", "Build system or dependency changes"),
    ("ci", "CI configuration changes"),
    ("chore", "Other changes that don't modify src or test files"),
    ("revert", "Revert a previous commit"),
];

const SCOPES: &[&str] = &[
    "no scope", // <-- Add this at the top
    // Most common (docs removed)
    "core",
    "api",
    "ui",
    "auth",
    "db",
    "test",
    "build",
    "deps",
    "ci",
    // Separator
    "────────────",
    // Lesser used
    "config",
    "infra",
    "release",
    "chore",
    "perf",
    "style",
    "lint",
    "i18n",
    "analytics",
    "security",
    "logging",
    "devops",
    "deploy",
    "assets",
    "mock",
    "example",
];

enum Step {
    Type,
    Scope,
    Subject,
    Body,
    Breaking,
    Preview,
}

fn is_scope_selectable(idx: usize) -> bool {
    let s = SCOPES[idx];
    !s.starts_with('─')
}

fn next_selectable_scope(mut idx: usize, dir: i32) -> usize {
    loop {
        let new_idx = if dir > 0 {
            if idx + 1 >= SCOPES.len() { return idx; }
            idx + 1
        } else {
            if idx == 0 { return idx; }
            idx - 1
        };
        if is_scope_selectable(new_idx) {
            return new_idx;
        }
        idx = new_idx;
    }
}

fn validate_subject(subject: &str) -> Option<String> {
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

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Type selection
    let mut selected_type = 0;
    let mut chosen_type: Option<String> = None;

    // Scope selection
    let mut selected_scope = 0;
    let mut custom_scope = String::new();
    let mut focus_input = false;
    let mut chosen_scope: Option<String> = None;

    // Subject input
    let mut subject = String::new();

    // Body input
    let mut body = String::new();
    let mut body_lines: Vec<String> = vec![];
    let mut in_body = false;

    // Breaking changes input
    let mut breaking = String::new();

    // Issue references input
    let mut issues = String::new();
    let mut focus_issues = false;

    // State machine
    let mut step = Step::Type;

    loop {
        terminal.draw(|f| {
            let size = f.size();
            match step {
                Step::Type => {
                    let items: Vec<ListItem> = COMMIT_TYPES
                        .iter()
                        .map(|(ty, desc)| ListItem::new(format!("{:<8} {}", ty, desc)))
                        .collect();
                    let mut state = ratatui::widgets::ListState::default();
                    state.select(Some(selected_type));
                    let list = List::new(items)
                        .block(Block::default().title("Select Commit Type (Enter to confirm, q/Esc/Ctrl+C to quit)").borders(Borders::ALL))
                        .highlight_style(Style::default().bg(Color::Blue))
                        .highlight_symbol(">> ");
                    f.render_stateful_widget(list, size, &mut state);
                }
                Step::Scope => {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(2)
                        .constraints([
                            Constraint::Length(SCOPES.len() as u16 + 2),
                            Constraint::Length(3),
                        ])
                        .split(size);

                    // List of scopes
                    let items: Vec<ListItem> = SCOPES
                        .iter()
                        .map(|s| {
                            if s.starts_with('─') {
                                ListItem::new(*s).style(Style::default().fg(Color::DarkGray))
                            } else {
                                ListItem::new(*s)
                            }
                        })
                        .collect();
                    let mut state = ratatui::widgets::ListState::default();
                    state.select(Some(selected_scope));
                    let list = List::new(items)
                        .block(Block::default().title("Select Scope").borders(Borders::ALL))
                        .highlight_style(Style::default().bg(Color::Blue))
                        .highlight_symbol(">> ");
                    f.render_stateful_widget(list, chunks[0], &mut state);

                    // Custom input
                    let input_block = if focus_input {
                        Block::default()
                            .title("Or type a custom scope (Tab to switch, Enter to confirm, Esc/Ctrl+C to quit)")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Green))
                    } else {
                        Block::default()
                            .title("Or type a custom scope (Tab to switch, Enter to confirm, q/Esc/Ctrl+C to quit)")
                            .borders(Borders::ALL)
                    };
                    let paragraph = Paragraph::new(custom_scope.as_str())
                        .block(input_block)
                        .style(Style::default().fg(Color::Yellow));
                    f.render_widget(paragraph, chunks[1]);
                }
                Step::Subject => {
                    let block = Block::default()
                        .title("Enter Subject (short commit message, Enter to confirm, Esc/Ctrl+C to quit)")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Green));
                    let paragraph = Paragraph::new(subject.as_str())
                        .block(block)
                        .style(Style::default().fg(Color::Yellow));
                    f.render_widget(paragraph, size);

                    // Validation message
                    let validation_msg = validate_subject(&subject);
                    if let Some(ref msg) = validation_msg {
                        let warn = Paragraph::new(msg.as_str())
                            .block(Block::default().borders(Borders::ALL).title("Validation Error"))
                            .style(Style::default().fg(Color::Red));
                        let area = Rect {
                            x: size.x,
                            y: size.y + size.height.saturating_sub(3),
                            width: size.width,
                            height: 3,
                        };
                        f.render_widget(warn, area);
                    }
                }
                Step::Body => {
                    let block = Block::default()
                        .title("Enter Body (multi-line, Enter for new line, Empty line to finish, Esc/Ctrl+C to quit)")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Green));
                    let body_text = if body_lines.is_empty() && body.is_empty() {
                        String::from("<empty>")
                    } else {
                        let mut all = body_lines.join("\n");
                        if !body.is_empty() {
                            if !all.is_empty() {
                                all.push('\n');
                            }
                            all.push_str(&body);
                        }
                        all
                    };
                    let paragraph = Paragraph::new(body_text)
                        .block(block)
                        .style(Style::default().fg(Color::Yellow))
                        .wrap(Wrap { trim: false });
                    f.render_widget(paragraph, size);
                }
                Step::Breaking => {
                    let block = Block::default()
                        .title("Enter Breaking Changes (optional, Enter to confirm, Esc/Ctrl+C to quit)")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Red));
                    let paragraph = Paragraph::new(breaking.as_str())
                        .block(block)
                        .style(Style::default().fg(Color::Red));
                    f.render_widget(paragraph, size);
                }
                Step::Preview => {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(2)
                        .constraints([
                            Constraint::Min(5),
                            Constraint::Length(3),
                        ])
                        .split(size);

                    let type_str = chosen_type.as_deref().unwrap_or("");
                    let scope_str = chosen_scope.as_deref().unwrap_or("");
                    let mut preview = String::new();

                    if chosen_scope.is_none() || scope_str.is_empty() {
                        preview = format!("{}: {}", type_str, subject);
                    } else {
                        preview = format!("{}({}): {}", type_str, scope_str, subject);
                    }

                    let mut full_preview = preview.clone();
                    if !body_lines.is_empty() || !body.is_empty() {
                        full_preview.push_str("\n\n");
                        full_preview.push_str(&body_lines.join("\n"));
                        if !body.is_empty() {
                            if !body_lines.is_empty() {
                                full_preview.push('\n');
                            }
                            full_preview.push_str(&body);
                        }
                    }
                    if !breaking.trim().is_empty() {
                        full_preview.push_str("\n\nBREAKING CHANGE: ");
                        full_preview.push_str(&breaking.trim());
                    }
                    if !issues.trim().is_empty() {
                        full_preview.push_str("\n\n");
                        full_preview.push_str(&issues.trim());
                    }

                    let paragraph = Paragraph::new(full_preview)
                        .block(Block::default()
                            .title("Preview Commit Message (Tab to edit issues, y/Enter to confirm, b to go back, Esc/Ctrl+C to quit)")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Green)))
                        .style(Style::default().fg(Color::Yellow))
                        .wrap(Wrap { trim: false });
                    f.render_widget(paragraph, chunks[0]);

                    let input_block = if focus_issues {
                        Block::default()
                            .title("Issue References (e.g., Closes #123, Fixes #456)")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Green))
                    } else {
                        Block::default()
                            .title("Issue References (Tab to edit, Enter to confirm)")
                            .borders(Borders::ALL)
                    };
                    let issues_paragraph = Paragraph::new(issues.as_str())
                        .block(input_block)
                        .style(Style::default().fg(Color::Yellow));
                    f.render_widget(issues_paragraph, chunks[1]);
                }
            }
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match step {
                        Step::Type => {
                            if (key.code == KeyCode::Char('q') && key.modifiers.is_empty())
                                || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
                                || key.code == KeyCode::Esc
                            {
                                break;
                            }
                            match key.code {
                                KeyCode::Down => {
                                    let mut idx = selected_type;
                                    if idx < COMMIT_TYPES.len() - 1 {
                                        idx += 1;
                                    }
                                    selected_type = idx;
                                }
                                KeyCode::Up => {
                                    let mut idx = selected_type;
                                    if idx > 0 {
                                        idx -= 1;
                                    }
                                    selected_type = idx;
                                }
                                KeyCode::Enter => {
                                    chosen_type = Some(COMMIT_TYPES[selected_type].0.to_string());
                                    step = Step::Scope;
                                }
                                _ => {}
                            }
                        }
                        Step::Scope => {
                            if focus_input {
                                if (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
                                    || key.code == KeyCode::Esc
                                {
                                    break;
                                }
                                match key.code {
                                    KeyCode::Tab => {
                                        focus_input = false;
                                    }
                                    KeyCode::Enter => {
                                        if !custom_scope.trim().is_empty() {
                                            chosen_scope = Some(custom_scope.trim().to_string());
                                            step = Step::Subject;
                                        }
                                    }
                                    KeyCode::Char(c) => {
                                        custom_scope.push(c);
                                    }
                                    KeyCode::Backspace => {
                                        custom_scope.pop();
                                    }
                                    _ => {}
                                }
                            } else {
                                if (key.code == KeyCode::Char('q') && key.modifiers.is_empty())
                                    || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
                                    || key.code == KeyCode::Esc
                                {
                                    break;
                                }
                                match key.code {
                                    KeyCode::Tab => {
                                        focus_input = true;
                                    }
                                    KeyCode::Down => {
                                        let idx = next_selectable_scope(selected_scope, 1);
                                        selected_scope = idx;
                                    }
                                    KeyCode::Up => {
                                        let idx = next_selectable_scope(selected_scope, -1);
                                        selected_scope = idx;
                                    }
                                    KeyCode::Enter => {
                                        if is_scope_selectable(selected_scope) {
                                            if selected_scope == 0 {
                                                // "no scope" selected
                                                chosen_scope = None;
                                            } else {
                                                chosen_scope = Some(SCOPES[selected_scope].to_string());
                                            }
                                            step = Step::Subject;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Step::Subject => {
                            if (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
                                || key.code == KeyCode::Esc
                            {
                                break;
                            }
                            let validation_msg = validate_subject(&subject);
                            match key.code {
                                KeyCode::Enter => {
                                    if validation_msg.is_none() {
                                        step = Step::Body;
                                    }
                                }
                                KeyCode::Char(c) => {
                                    subject.push(c);
                                }
                                KeyCode::Backspace => {
                                    subject.pop();
                                }
                                _ => {}
                            }
                        }
                        Step::Body => {
                            if (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
                                || key.code == KeyCode::Esc
                            {
                                break;
                            }
                            match key.code {
                                KeyCode::Enter => {
                                    if body.is_empty() {
                                        // Empty line: finish body input
                                        step = Step::Breaking;
                                    } else {
                                        body_lines.push(body.clone());
                                        body.clear();
                                    }
                                }
                                KeyCode::Char(c) => {
                                    body.push(c);
                                }
                                KeyCode::Backspace => {
                                    body.pop();
                                }
                                _ => {}
                            }
                        }
                        Step::Breaking => {
                            if (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
                                || key.code == KeyCode::Esc
                            {
                                break;
                            }
                            match key.code {
                                KeyCode::Enter => {
                                    step = Step::Preview;
                                }
                                KeyCode::Char(c) => {
                                    breaking.push(c);
                                }
                                KeyCode::Backspace => {
                                    breaking.pop();
                                }
                                _ => {}
                            }
                        }
                        Step::Preview => {
                            if (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
                                || key.code == KeyCode::Esc
                            {
                                break;
                            }
                            if focus_issues {
                                match key.code {
                                    KeyCode::Tab => {
                                        focus_issues = false;
                                    }
                                    KeyCode::Enter => {
                                        break;
                                    }
                                    KeyCode::Char(c) => {
                                        issues.push(c);
                                    }
                                    KeyCode::Backspace => {
                                        issues.pop();
                                    }
                                    KeyCode::Char('b') => {
                                        focus_issues = false;
                                        step = Step::Breaking;
                                    }
                                    _ => {}
                                }
                            } else {
                                match key.code {
                                    KeyCode::Tab => {
                                        focus_issues = true;
                                    }
                                    KeyCode::Char('y') | KeyCode::Enter => {
                                        break;
                                    }
                                    KeyCode::Char('b') => {
                                        step = Step::Breaking;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
        // For body input, always start with an empty line if not already in body
        if matches!(step, Step::Body) && !in_body {
            body.clear();
            in_body = true;
        }
        if !matches!(step, Step::Body) {
            in_body = false;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Print the result
    println!("---\nResult:");
    if let Some(ty) = chosen_type {
        if chosen_scope.is_none() || chosen_scope.as_deref().unwrap_or("").is_empty() {
            print!("{}: {}", ty, subject);
        } else {
            print!("{}({}): {}", ty, chosen_scope.as_deref().unwrap(), subject);
        }
    }
    if !body_lines.is_empty() || !body.is_empty() {
        println!();
        for line in &body_lines {
            println!("{}", line);
        }
        if !body.is_empty() {
            println!("{}", body);
        }
    }
    if !breaking.trim().is_empty() {
        println!("\nBREAKING CHANGE: {}", breaking.trim());
    }
    if !issues.trim().is_empty() {
        println!("{}", issues.trim());
    }
    println!();

    Ok(())
}