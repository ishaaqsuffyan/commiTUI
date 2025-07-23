mod config;
mod tui;
mod validation;
mod state;
mod git;

use config::Config;
use tui::run_tui;
use git::commit_with_message;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load config (from file or use default)
    let config = Config::load().unwrap_or_else(|e| {
        eprintln!("Warning: {}", e);
        Config::default()
    });

    // Run the TUI and get the commit message
    let commit_message = run_tui(config)?;

    // Actually perform the commit
    commit_with_message(&commit_message)?;

    Ok(())
}