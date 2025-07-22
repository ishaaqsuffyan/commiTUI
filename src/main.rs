use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    style::{Style, Color},
    layout::{Layout, Constraint, Direction},
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, error::Error};

const SCOPES: &[&str] = &["auth", "api", "ui", "db", "docs"];

enum Step {
    Scope,
    Subject,
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // State for scope step
    let mut selected_scope = 0;
    let mut custom_scope = String::new();
    let mut focus_input = false;
    let mut chosen_scope: Option<String> = None;

    // State for subject step
    let mut subject = String::new();

    // State machine
    let mut step = Step::Scope;

    loop {
        terminal.draw(|f| {
            let size = f.size();
            match step {
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
                        .map(|s| ListItem::new(*s))
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
                }
            }
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match step {
                        Step::Scope => {
                            if focus_input {
                                // Only Esc and Ctrl+C quit from input box
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
                                        // If empty, do nothing
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
                                // In list: q, Esc, or Ctrl+C quit
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
                                        if selected_scope < SCOPES.len() - 1 {
                                            selected_scope += 1;
                                        }
                                    }
                                    KeyCode::Up => {
                                        if selected_scope > 0 {
                                            selected_scope -= 1;
                                        }
                                    }
                                    KeyCode::Enter => {
                                        chosen_scope = Some(SCOPES[selected_scope].to_string());
                                        step = Step::Subject;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Step::Subject => {
                            // Esc or Ctrl+C to quit
                            if (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
                                || key.code == KeyCode::Esc
                            {
                                break;
                            }
                            match key.code {
                                KeyCode::Enter => {
                                    // Confirm subject and exit (or go to next step)
                                    break;
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
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    println!("---\nResult:");
    if let Some(scope) = chosen_scope {
        println!("Scope: {}", scope);
    } else {
        println!("No scope selected.");
    }
    println!("Subject: {}", subject);

    Ok(())
}