use crate::config::Config;
use crate::state::{AppState, Step};
use crate::validation::validate_subject;
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
use std::io;

fn is_scope_selectable(scopes: &[String], idx: usize) -> bool {
    let s = &scopes[idx];
    !s.starts_with('─')
}

fn next_selectable_scope(scopes: &[String], mut idx: usize, dir: i32) -> usize {
    loop {
        let new_idx = if dir > 0 {
            if idx + 1 >= scopes.len() { return idx; }
            idx + 1
        } else {
            if idx == 0 { return idx; }
            idx - 1
        };
        if is_scope_selectable(scopes, new_idx) {
            return new_idx;
        }
        idx = new_idx;
    }
}

fn step_number(step: &Step) -> usize {
    match step {
        Step::Type => 1,
        Step::Scope => 2,
        Step::Subject => 3,
        Step::Body => 4,
        Step::Breaking => 5,
        Step::Preview => 6,
    }
}

pub fn run_tui(config: Config) -> Result<String, Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState {
        step: Step::Type,
        selected_type: 0,
        chosen_type: None,

        selected_scope: 0,
        custom_scope: String::new(),
        focus_input: false, // For custom scope input
        chosen_scope: None,

        subject: String::new(),
        // For Subject, Body, Breaking, Issues, we'll use state.focus_input to mean "is this text input active?"
        // This makes focus_input a re-used flag for "is a text field currently being typed into" for the active step.

        body: String::new(),
        body_lines: vec![],
        in_body: false, // Special flag for multi-line body

        breaking: String::new(),

        issues: String::new(),
        focus_issues: false, // Specific for issues field in preview
    };

    let total_steps = 6;

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let progress = format!(
                "Step {}/{}",
                step_number(&state.step),
                total_steps
            );
            let progress_paragraph = Paragraph::new(progress)
                .style(Style::default().fg(Color::Cyan));
            let chunks_outer = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1), // For progress indicator
                    Constraint::Min(1),    // For the main content
                ])
                .split(size);
            f.render_widget(progress_paragraph, chunks_outer[0]);

            let area = chunks_outer[1]; // Main drawing area

            match state.step {
                Step::Type => {
                    let items: Vec<ListItem> = config.types
                        .iter()
                        .map(|ty| ListItem::new(ty.clone()))
                        .collect();
                    let mut list_state = ratatui::widgets::ListState::default();
                    list_state.select(Some(state.selected_type));
                    let list = List::new(items)
                        .block(Block::default().title("Select Commit Type (Enter to confirm, q/Esc/Ctrl+C to quit)").borders(Borders::ALL))
                        .highlight_style(Style::default().bg(Color::Blue))
                        .highlight_symbol(">> ");
                    f.render_stateful_widget(list, area, &mut list_state);
                }
                Step::Scope => {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(config.scopes.len() as u16 + 2),
                            Constraint::Length(3),
                        ])
                        .split(area); // Use `area` for splitting

                    let items: Vec<ListItem> = config.scopes
                        .iter()
                        .map(|s| {
                            if s.starts_with('─') {
                                ListItem::new(s.clone()).style(Style::default().fg(Color::DarkGray))
                            } else {
                                ListItem::new(s.clone())
                            }
                        })
                        .collect();
                    let mut list_state = ratatui::widgets::ListState::default();
                    list_state.select(Some(state.selected_scope));
                    let list = List::new(items)
                        .block(Block::default().title("Select Scope").borders(Borders::ALL))
                        .highlight_style(Style::default().bg(Color::Blue))
                        .highlight_symbol(">> ");
                    f.render_stateful_widget(list, chunks[0], &mut list_state);

                    let input_block = if state.focus_input {
                        Block::default()
                            .title("Or type a custom scope (Tab to switch, Enter to confirm, Esc/Ctrl+C to quit)")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Green))
                    } else {
                        Block::default()
                            .title("Or type a custom scope (Tab to switch, Enter to confirm, b/Left to go back, q/Esc/Ctrl+C to quit)")
                            .borders(Borders::ALL)
                    };
                    let paragraph = Paragraph::new(state.custom_scope.as_str())
                        .block(input_block)
                        .style(Style::default().fg(Color::Yellow));
                    f.render_widget(paragraph, chunks[1]);
                }
                Step::Subject => {
                    let block = if state.focus_input {
                        Block::default()
                            .title("Enter Subject (Tab to navigate, Enter to confirm, Esc/Ctrl+C to quit)")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Green))
                    } else {
                        Block::default()
                            .title("Subject (Tab to edit, b/Left to go back, Enter to confirm, Esc/Ctrl+C to quit)")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Green))
                    };
                    let paragraph = Paragraph::new(state.subject.as_str())
                        .block(block)
                        .style(Style::default().fg(Color::Yellow));
                    f.render_widget(paragraph, area); // Use `area` for rendering

                    let validation_msg = validate_subject(&state.subject);
                    if let Some(ref msg) = validation_msg {
                        let warn = Paragraph::new(msg.as_str())
                            .block(Block::default().borders(Borders::ALL).title("Validation Error"))
                            .style(Style::default().fg(Color::Red));
                        let warn_area = Rect {
                            x: area.x,
                            y: area.y + area.height.saturating_sub(3),
                            width: area.width,
                            height: 3,
                        };
                        f.render_widget(warn, warn_area);
                    }
                }
                Step::Body => {
                    let block = if state.focus_input {
                        Block::default()
                            .title("Enter Body (Tab to navigate, Enter for new line, Empty line to finish, Esc/Ctrl+C to quit)")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Green))
                    } else {
                        Block::default()
                            .title("Body (Tab to edit, b/Left to go back, Enter for new line, Empty line to finish, Esc/Ctrl+C to quit)")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Green))
                    };
                    let body_text = if state.body_lines.is_empty() && state.body.is_empty() {
                        String::from("<empty>")
                    } else {
                        let mut all = state.body_lines.join("\n");
                        if !state.body.is_empty() {
                            if !all.is_empty() {
                                all.push('\n');
                            }
                            all.push_str(&state.body);
                        }
                        all
                    };
                    let paragraph = Paragraph::new(body_text)
                        .block(block)
                        .style(Style::default().fg(Color::Yellow))
                        .wrap(Wrap { trim: false });
                    f.render_widget(paragraph, area);
                }
                Step::Breaking => {
                    let block = if state.focus_input {
                        Block::default()
                            .title("Enter Breaking Changes (Tab to navigate, Enter to confirm, Esc/Ctrl+C to quit)")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Red))
                    } else {
                        Block::default()
                            .title("Breaking Changes (Tab to edit, b/Left to go back, Enter to confirm, Esc/Ctrl+C to quit)")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Red))
                    };
                    let paragraph = Paragraph::new(state.breaking.as_str())
                        .block(block)
                        .style(Style::default().fg(Color::Red));
                    f.render_widget(paragraph, area);
                }
                Step::Preview => {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Min(5),
                            Constraint::Length(3),
                        ])
                        .split(area); // Use `area` for splitting

                    let type_str = state.chosen_type.as_deref().unwrap_or("");
                    let scope_str = state.chosen_scope.as_deref().unwrap_or("");
                    let mut preview = String::new();

                    if state.chosen_scope.is_none() || scope_str.is_empty() {
                        preview = format!("{}: {}", type_str, state.subject);
                    } else {
                        preview = format!("{}({}): {}", type_str, scope_str, state.subject);
                    }

                    let mut full_preview = preview.clone();
                    if !state.body_lines.is_empty() || !state.body.is_empty() {
                        full_preview.push_str("\n\n");
                        full_preview.push_str(&state.body_lines.join("\n"));
                        if !state.body.is_empty() {
                            if !full_preview.is_empty() && !full_preview.ends_with('\n') {
                                full_preview.push('\n');
                            }
                            full_preview.push_str(&state.body);
                        }
                    }
                    if !state.breaking.trim().is_empty() {
                        if !full_preview.is_empty() && !full_preview.ends_with("\n\n") {
                            full_preview.push_str("\n\n");
                        }
                        full_preview.push_str(&format!("BREAKING CHANGE: {}", state.breaking.trim()));
                    }
                    if !state.issues.trim().is_empty() {
                        if !full_preview.is_empty() && !full_preview.ends_with("\n\n") {
                            full_preview.push_str("\n\n");
                        }
                        full_preview.push_str(&state.issues.trim());
                    }

                    let paragraph = Paragraph::new(full_preview)
                        .block(Block::default()
                            .title("Preview Commit Message (Tab to edit issues, y/Enter to confirm, b/Left to go back, Esc/Ctrl+C to quit)")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Green)))
                        .style(Style::default().fg(Color::Yellow))
                        .wrap(Wrap { trim: false });
                    f.render_widget(paragraph, chunks[0]);

                    let input_block = if state.focus_issues {
                        Block::default()
                            .title("Issue References (Tab to switch, Enter to confirm)")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Green))
                    } else {
                        Block::default()
                            .title("Issue References (Tab to edit, y/Enter to confirm, b/Left to go back, Esc/Ctrl+C to quit)")
                            .borders(Borders::ALL)
                    };
                    let issues_paragraph = Paragraph::new(state.issues.as_str())
                        .block(input_block)
                        .style(Style::default().fg(Color::Yellow));
                    f.render_widget(issues_paragraph, chunks[1]);
                }
            }
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // Global quit hotkeys (Esc or Ctrl+C) always work
                    if (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
                        || key.code == KeyCode::Esc
                    {
                        break;
                    }

                    match state.step {
                        Step::Type => {
                            // Only 'q' quits here, Esc/Ctrl+C are handled globally
                            if key.code == KeyCode::Char('q') && key.modifiers.is_empty() {
                                break;
                            }
                            match key.code {
                                KeyCode::Down => {
                                    state.selected_type = (state.selected_type + 1).min(config.types.len() - 1);
                                }
                                KeyCode::Up => {
                                    state.selected_type = state.selected_type.saturating_sub(1);
                                }
                                KeyCode::Enter => {
                                    state.chosen_type = Some(config.types[state.selected_type].clone());
                                    state.step = Step::Scope;
                                    state.focus_input = false; // Start scope list focused
                                }
                                _ => {}
                            }
                        }
                        Step::Scope => {
                            if state.focus_input { // Custom scope input focused
                                match key.code {
                                    KeyCode::Tab => {
                                        state.focus_input = false; // Switch to list
                                    }
                                    KeyCode::Enter => {
                                        if !state.custom_scope.trim().is_empty() {
                                            state.chosen_scope = Some(state.custom_scope.trim().to_string());
                                        } else {
                                            state.chosen_scope = None; // If custom input is empty, clear scope
                                        }
                                        state.step = Step::Subject;
                                        state.focus_input = true; // Start subject input focused
                                    }
                                    KeyCode::Char(c) => {
                                        state.custom_scope.push(c);
                                    }
                                    KeyCode::Backspace => {
                                        state.custom_scope.pop();
                                    }
                                    _ => {}
                                }
                            } else { // Scope list focused
                                // Only 'q' quits here, Esc/Ctrl+C are handled globally
                                if key.code == KeyCode::Char('q') && key.modifiers.is_empty() {
                                    break;
                                }
                                match key.code {
                                    KeyCode::Tab => {
                                        state.focus_input = true; // Switch to custom input
                                    }
                                    KeyCode::Down => {
                                        state.selected_scope = next_selectable_scope(&config.scopes, state.selected_scope, 1);
                                    }
                                    KeyCode::Up => {
                                        state.selected_scope = next_selectable_scope(&config.scopes, state.selected_scope, -1);
                                    }
                                    KeyCode::Enter => {
                                        if is_scope_selectable(&config.scopes, state.selected_scope) {
                                            if state.selected_scope == 0 { // "no scope" selected
                                                state.chosen_scope = None;
                                            } else {
                                                state.chosen_scope = Some(config.scopes[state.selected_scope].clone());
                                            }
                                            state.step = Step::Subject;
                                            state.focus_input = true; // Start subject input focused
                                        }
                                    }
                                    KeyCode::Char('b') | KeyCode::Left => {
                                        state.step = Step::Type;
                                        state.selected_type = config.types.iter().position(|t| Some(t) == state.chosen_type.as_ref()).unwrap_or(0);
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Step::Subject => {
                            if state.focus_input { // Subject input focused
                                let validation_msg = validate_subject(&state.subject);
                                match key.code {
                                    KeyCode::Tab => {
                                        state.focus_input = false; // Switch to navigation mode for subject
                                    }
                                    KeyCode::Enter => {
                                        if validation_msg.is_none() {
                                            state.step = Step::Body;
                                            state.focus_input = true; // Start body input focused
                                            state.in_body = false; // Reset multi-line body state
                                        }
                                    }
                                    KeyCode::Char(c) => {
                                        state.subject.push(c);
                                    }
                                    KeyCode::Backspace => {
                                        state.subject.pop();
                                    }
                                    _ => {}
                                }
                            } else { // Navigation mode for subject
                                match key.code {
                                    KeyCode::Tab => {
                                        state.focus_input = true; // Switch to subject input
                                    }
                                    KeyCode::Char('b') | KeyCode::Left => {
                                        state.step = Step::Scope;
                                        // Restore state for scope
                                        state.focus_input = state.chosen_scope.is_some() && !config.scopes.contains(state.chosen_scope.as_ref().unwrap());
                                        state.selected_scope = config.scopes.iter().position(|s| Some(s) == state.chosen_scope.as_ref()).unwrap_or(0);
                                        state.custom_scope = state.chosen_scope.clone().unwrap_or_default();
                                    }
                                    KeyCode::Enter => {
                                        // If enter is pressed in nav mode, it should still move forward if valid.
                                        if validate_subject(&state.subject).is_none() {
                                            state.step = Step::Body;
                                            state.focus_input = true;
                                            state.in_body = false;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Step::Body => {
                            if state.focus_input { // Body input focused
                                match key.code {
                                    KeyCode::Tab => {
                                        state.focus_input = false; // Switch to navigation mode for body
                                    }
                                    KeyCode::Enter => {
                                        if state.body.is_empty() {
                                            state.step = Step::Breaking;
                                            state.focus_input = true; // Start breaking changes input focused
                                        } else {
                                            state.body_lines.push(state.body.clone());
                                            state.body.clear();
                                        }
                                    }
                                    KeyCode::Char(c) => {
                                        state.body.push(c);
                                    }
                                    KeyCode::Backspace => {
                                        state.body.pop();
                                    }
                                    _ => {}
                                }
                            } else { // Navigation mode for body
                                match key.code {
                                    KeyCode::Tab => {
                                        state.focus_input = true; // Switch to body input
                                    }
                                    KeyCode::Char('b') | KeyCode::Left => {
                                        state.step = Step::Subject;
                                        state.focus_input = true; // Return to subject input focus
                                    }
                                    KeyCode::Enter => {
                                        // If enter is pressed in nav mode, it should still move forward.
                                        state.step = Step::Breaking;
                                        state.focus_input = true;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Step::Breaking => {
                            if state.focus_input { // Breaking changes input focused
                                match key.code {
                                    KeyCode::Tab => {
                                        state.focus_input = false; // Switch to navigation mode for breaking
                                    }
                                    KeyCode::Enter => {
                                        state.step = Step::Preview;
                                        state.focus_issues = false; // Start preview with issues not focused
                                    }
                                    KeyCode::Char(c) => {
                                        state.breaking.push(c);
                                    }
                                    KeyCode::Backspace => {
                                        state.breaking.pop();
                                    }
                                    _ => {}
                                }
                            } else { // Navigation mode for breaking
                                match key.code {
                                    KeyCode::Tab => {
                                        state.focus_input = true; // Switch to breaking changes input
                                    }
                                    KeyCode::Char('b') | KeyCode::Left => {
                                        state.step = Step::Body;
                                        state.focus_input = true; // Return to body input focus
                                    }
                                    KeyCode::Enter => {
                                        state.step = Step::Preview;
                                        state.focus_issues = false;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Step::Preview => {
                            if state.focus_issues { // Issues input focused
                                match key.code {
                                    KeyCode::Tab => {
                                        state.focus_issues = false; // Switch to preview navigation
                                    }
                                    KeyCode::Enter => {
                                        // Confirm and exit
                                        break;
                                    }
                                    KeyCode::Char(c) => {
                                        state.issues.push(c);
                                    }
                                    KeyCode::Backspace => {
                                        state.issues.pop();
                                    }
                                    KeyCode::Char('b') | KeyCode::Left => {
                                        state.focus_issues = false; // Leave issue input
                                        state.step = Step::Breaking; // Go back
                                        state.focus_input = true; // Return to breaking input focus
                                    }
                                    _ => {}
                                }
                            } else { // Preview navigation
                                match key.code {
                                    KeyCode::Tab => {
                                        state.focus_issues = true; // Switch to issues input
                                    }
                                    KeyCode::Char('y') | KeyCode::Enter => {
                                        // Confirm and exit
                                        break;
                                    }
                                    KeyCode::Char('b') | KeyCode::Left => {
                                        state.step = Step::Breaking; // Go back
                                        state.focus_input = true; // Return to breaking input focus
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
        // Special handling for multi-line body input state
        if matches!(state.step, Step::Body) && !state.in_body {
            state.body.clear(); // Clear current line when entering body step for first time
            state.in_body = true;
            state.focus_input = true; // Ensure body input starts focused
        }
        if !matches!(state.step, Step::Body) {
            state.in_body = false;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Build the commit message string to return
    let mut result = String::new();
    if let Some(ty) = state.chosen_type {
        if state.chosen_scope.is_none() || state.chosen_scope.as_deref().unwrap_or("").is_empty() {
            result = format!("{}: {}", ty, state.subject);
        } else {
            result = format!("{}({}): {}", ty, state.chosen_scope.as_deref().unwrap(), state.subject);
        }
    }
    if !state.body_lines.is_empty() || !state.body.is_empty() {
        result.push('\n'); // Add newline after subject line
        for (i, line) in state.body_lines.iter().enumerate() {
            if i > 0 || !result.ends_with('\n') {
                result.push('\n');
            }
            result.push_str(line);
        }
        if !state.body.is_empty() {
            if !result.ends_with('\n') { // Ensure newline if body_lines had content
                result.push('\n');
            }
            result.push_str(&state.body);
        }
    }
    
    // Ensure two newlines before optional footers if there's content before them
    let mut has_previous_content = !state.body_lines.is_empty() || !state.body.is_empty();
    if !has_previous_content && (!result.is_empty() && result != "\n") { // If only subject and no body
        has_previous_content = true;
    }

    if !state.breaking.trim().is_empty() {
        if has_previous_content && !result.ends_with("\n\n") {
            result.push_str("\n\n");
        } else if !has_previous_content && !result.ends_with('\n') { // If breaking is first after subject line
            result.push('\n');
        }
        result.push_str(&format!("BREAKING CHANGE: {}", state.breaking.trim()));
        has_previous_content = true; // Mark that content was added
    }
    
    if !state.issues.trim().is_empty() {
        if has_previous_content && !result.ends_with("\n\n") {
            result.push_str("\n\n");
        } else if !has_previous_content && !result.ends_with('\n') { // If issues is first after subject line
            result.push('\n');
        }
        result.push_str(&state.issues.trim());
    }
    
    // Ensure final newline for git to pick it up correctly
    if !result.ends_with('\n') {
        result.push('\n');
    }

    Ok(result)
}