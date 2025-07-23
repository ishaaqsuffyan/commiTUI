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
        focus_input: false,
        chosen_scope: None,

        subject: String::new(),

        body: String::new(),
        body_lines: vec![],
        in_body: false,

        breaking: String::new(),

        issues: String::new(),
        focus_issues: false,
    };

    loop {
        terminal.draw(|f| {
            let size = f.size();
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
                    f.render_stateful_widget(list, size, &mut list_state);
                }
                Step::Scope => {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(2)
                        .constraints([
                            Constraint::Length(config.scopes.len() as u16 + 2),
                            Constraint::Length(3),
                        ])
                        .split(size);

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
                            .title("Or type a custom scope (Tab to switch, Enter to confirm, q/Esc/Ctrl+C to quit)")
                            .borders(Borders::ALL)
                    };
                    let paragraph = Paragraph::new(state.custom_scope.as_str())
                        .block(input_block)
                        .style(Style::default().fg(Color::Yellow));
                    f.render_widget(paragraph, chunks[1]);
                }
                Step::Subject => {
                    let block = Block::default()
                        .title("Enter Subject (short commit message, Enter to confirm, Esc/Ctrl+C to quit)")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Green));
                    let paragraph = Paragraph::new(state.subject.as_str())
                        .block(block)
                        .style(Style::default().fg(Color::Yellow));
                    f.render_widget(paragraph, size);

                    let validation_msg = validate_subject(&state.subject);
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
                    f.render_widget(paragraph, size);
                }
                Step::Breaking => {
                    let block = Block::default()
                        .title("Enter Breaking Changes (optional, Enter to confirm, Esc/Ctrl+C to quit)")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Red));
                    let paragraph = Paragraph::new(state.breaking.as_str())
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
                            if !state.body_lines.is_empty() {
                                full_preview.push('\n');
                            }
                            full_preview.push_str(&state.body);
                        }
                    }
                    if !state.breaking.trim().is_empty() {
                        full_preview.push_str("\n\nBREAKING CHANGE: ");
                        full_preview.push_str(&state.breaking.trim());
                    }
                    if !state.issues.trim().is_empty() {
                        full_preview.push_str("\n\n");
                        full_preview.push_str(&state.issues.trim());
                    }

                    let paragraph = Paragraph::new(full_preview)
                        .block(Block::default()
                            .title("Preview Commit Message (Tab to edit issues, y/Enter to confirm, b to go back, Esc/Ctrl+C to quit)")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Green)))
                        .style(Style::default().fg(Color::Yellow))
                        .wrap(Wrap { trim: false });
                    f.render_widget(paragraph, chunks[0]);

                    let input_block = if state.focus_issues {
                        Block::default()
                            .title("Issue References (e.g., Closes #123, Fixes #456)")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Green))
                    } else {
                        Block::default()
                            .title("Issue References (Tab to edit, Enter to confirm)")
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
                    match state.step {
                        Step::Type => {
                            if (key.code == KeyCode::Char('q') && key.modifiers.is_empty())
                                || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
                                || key.code == KeyCode::Esc
                            {
                                break;
                            }
                            match key.code {
                                KeyCode::Down => {
                                    let mut idx = state.selected_type;
                                    if idx < config.types.len() - 1 {
                                        idx += 1;
                                    }
                                    state.selected_type = idx;
                                }
                                KeyCode::Up => {
                                    let mut idx = state.selected_type;
                                    if idx > 0 {
                                        idx -= 1;
                                    }
                                    state.selected_type = idx;
                                }
                                KeyCode::Enter => {
                                    state.chosen_type = Some(config.types[state.selected_type].clone());
                                    state.step = Step::Scope;
                                }
                                _ => {}
                            }
                        }
                        Step::Scope => {
                            if state.focus_input {
                                if (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
                                    || key.code == KeyCode::Esc
                                {
                                    break;
                                }
                                match key.code {
                                    KeyCode::Tab => {
                                        state.focus_input = false;
                                    }
                                    KeyCode::Enter => {
                                        if !state.custom_scope.trim().is_empty() {
                                            state.chosen_scope = Some(state.custom_scope.trim().to_string());
                                            state.step = Step::Subject;
                                        }
                                    }
                                    KeyCode::Char(c) => {
                                        state.custom_scope.push(c);
                                    }
                                    KeyCode::Backspace => {
                                        state.custom_scope.pop();
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
                                        state.focus_input = true;
                                    }
                                    KeyCode::Down => {
                                        let idx = next_selectable_scope(&config.scopes, state.selected_scope, 1);
                                        state.selected_scope = idx;
                                    }
                                    KeyCode::Up => {
                                        let idx = next_selectable_scope(&config.scopes, state.selected_scope, -1);
                                        state.selected_scope = idx;
                                    }
                                    KeyCode::Enter => {
                                        if is_scope_selectable(&config.scopes, state.selected_scope) {
                                            if state.selected_scope == 0 {
                                                state.chosen_scope = None;
                                            } else {
                                                state.chosen_scope = Some(config.scopes[state.selected_scope].clone());
                                            }
                                            state.step = Step::Subject;
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
                            let validation_msg = validate_subject(&state.subject);
                            match key.code {
                                KeyCode::Enter => {
                                    if validation_msg.is_none() {
                                        state.step = Step::Body;
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
                        }
                        Step::Body => {
                            if (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
                                || key.code == KeyCode::Esc
                            {
                                break;
                            }
                            match key.code {
                                KeyCode::Enter => {
                                    if state.body.is_empty() {
                                        state.step = Step::Breaking;
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
                        }
                        Step::Breaking => {
                            if (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
                                || key.code == KeyCode::Esc
                            {
                                break;
                            }
                            match key.code {
                                KeyCode::Enter => {
                                    state.step = Step::Preview;
                                }
                                KeyCode::Char(c) => {
                                    state.breaking.push(c);
                                }
                                KeyCode::Backspace => {
                                    state.breaking.pop();
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
                            if state.focus_issues {
                                match key.code {
                                    KeyCode::Tab => {
                                        state.focus_issues = false;
                                    }
                                    KeyCode::Enter => {
                                        break;
                                    }
                                    KeyCode::Char(c) => {
                                        state.issues.push(c);
                                    }
                                    KeyCode::Backspace => {
                                        state.issues.pop();
                                    }
                                    KeyCode::Char('b') => {
                                        state.focus_issues = false;
                                        state.step = Step::Breaking;
                                    }
                                    _ => {}
                                }
                            } else {
                                match key.code {
                                    KeyCode::Tab => {
                                        state.focus_issues = true;
                                    }
                                    KeyCode::Char('y') | KeyCode::Enter => {
                                        break;
                                    }
                                    KeyCode::Char('b') => {
                                        state.step = Step::Breaking;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
        if matches!(state.step, Step::Body) && !state.in_body {
            state.body.clear();
            state.in_body = true;
        }
        if !matches!(state.step, Step::Body) {
            state.in_body = false;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Build the commit message string:
    let mut result = String::new();
    if let Some(ty) = state.chosen_type {
        if state.chosen_scope.is_none() || state.chosen_scope.as_deref().unwrap_or("").is_empty() {
            result = format!("{}: {}", ty, state.subject);
        } else {
            result = format!("{}({}): {}", ty, state.chosen_scope.as_deref().unwrap(), state.subject);
        }
    }
    if !state.body_lines.is_empty() || !state.body.is_empty() {
        result.push('\n');
        for line in &state.body_lines {
            result.push_str(&format!("\n{}", line));
        }
        if !state.body.is_empty() {
            result.push_str(&format!("\n{}", state.body));
        }
    }
    if !state.breaking.trim().is_empty() {
        result.push_str(&format!("\n\nBREAKING CHANGE: {}", state.breaking.trim()));
    }
    if !state.issues.trim().is_empty() {
        result.push_str(&format!("\n\n{}", state.issues.trim()));
    }
    result.push('\n');

    Ok(result)
}